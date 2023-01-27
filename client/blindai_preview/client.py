# Copyright 2022 Mithril Security. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


import pathlib
from ._dcap_attestation import validate_attestation
from .utils import *

from dataclasses import dataclass
from enum import IntEnum
from typing import Any, Dict, List, Optional, Tuple, Union

import os
import contextlib
import socket

import numpy as np
import cbor2 as cbor

from hashlib import sha256
import platform
import getpass
import logging
import tempfile
import requests
from requests.adapters import HTTPAdapter
from importlib_metadata import version

app_version = version("blindai-preview")


CONNECTION_TIMEOUT = 10


class ModelDatumType(IntEnum):
    F32 = 0
    F64 = 1
    I32 = 2
    I64 = 3
    U32 = 4
    U64 = 5
    U8 = 6
    U16 = 7
    I8 = 8
    I16 = 9
    Bool = 10


format_per_item = {
    ModelDatumType.F32: "<f4",
    ModelDatumType.F64: "<f8",
    ModelDatumType.I32: "<i4",
    ModelDatumType.I64: "<i8",
    ModelDatumType.U32: "<u4",
    ModelDatumType.U64: "<u8",
    ModelDatumType.U8: "<u1",
    ModelDatumType.U16: "<u2",
    ModelDatumType.I8: "<i1",
    ModelDatumType.I16: "<i2",
    ModelDatumType.Bool: "?",
}


def serialize_tensor(tensor: np.ndarray, type: ModelDatumType) -> bytes:
    return np.array(tensor).astype(format_per_item[type], casting="equiv").tobytes()


def deserialize_tensor(data: bytes, type: ModelDatumType) -> np.ndarray:
    return np.frombuffer(data, dtype=format_per_item[type])


class TensorInfo:
    fact: List[int]
    datum_type: ModelDatumType
    node_name: str

    def __init__(self, fact, datum_type, node_name=None):
        self.fact = fact
        self.datum_type = (
            ModelDatumType[datum_type]
            if isinstance(datum_type, str)
            else ModelDatumType(datum_type)
        )
        self.node_name = node_name


class Tensor:
    """Tensor class to convert serialized tensors into convenients objects."""

    info: Union[TensorInfo, dict]
    bytes_data: bytes

    def __init__(self, info: Union[TensorInfo, dict], bytes_data: bytes):
        self.info = info
        self.bytes_data = bytes_data

    def as_flat(self) -> list:
        """Convert the prediction calculated by the server to a flat python
        list."""
        return self.as_numpy().tolist()

    def as_numpy(self):
        """Convert the prediction calculated by the server to a numpy array."""

        arr = deserialize_tensor(self.bytes_data, self.info.datum_type)
        arr.shape = self.shape
        return arr

    def as_torch(self):
        """Convert the prediction calculated by the server to a Torch Tensor.

        As torch is heavy it's an optional dependency of the project, and is
        imported only when needed.

        Raises: ImportError if torch isn't installed
        """
        try:
            import torch
        except ImportError as e:
            raise ImportError(
                "torch not installed, please install with pip install blindai[torch]"
            ) from e
        arr = torch.tensor(self.as_numpy())
        return arr.view(self.shape)

    @property
    def shape(self) -> tuple:
        if isinstance(self.info, TensorInfo):
            return tuple(self.info.fact)
        return self.info["fact"]

    @property
    def datum_type(self) -> ModelDatumType:
        if isinstance(self.info, TensorInfo):
            return self.info.datum_type
        return self.info["datum_type"]


@dataclass
class UploadModel:
    model: List[int]
    length: int
    model_name: str
    optimize: bool

    def __init__(self, model, length, model_name="", optimize=True):
        self.model = model
        self.length = length
        self.model_name = model_name
        self.optimize = optimize


@dataclass
class RunModel:
    model_id: str
    inputs: List[Tensor]

    def __init__(self, model_id, inputs):
        self.model_id = model_id
        self.inputs = inputs


@dataclass
class DeleteModel:
    model_id: str

    def __init__(self, model_id):
        self.model_id = model_id


@dataclass
class SendModelReply:
    hash: bytes
    model_id: str

    def __init__(self, **entries):
        self.__dict__.update(entries)


@dataclass
class RunModelReply:
    outputs: List[Any]

    def __init__(self, **entries):
        self.__dict__.update(entries)


@dataclass
class UploadResponse:
    model_id: str


@dataclass
class RunModelResponse:
    output: List[Tensor]


class _ClientInfo:
    uid: str
    platform_name: str
    platform_arch: str
    platform_version: str
    platform_release: str
    user_agent: str
    user_agent_version: str

    def __init__(
        self,
        uid,
        platform_name,
        platform_arch,
        platform_version,
        platform_release,
        user_agent,
        user_agent_version,
    ):
        self.uid = uid
        self.platform_name = platform_name
        self.platform_arch = platform_arch
        self.platform_version = platform_version
        self.platform_release = platform_release
        self.user_agent = user_agent
        self.user_agent_version = user_agent_version


def dtype_to_numpy(dtype: ModelDatumType) -> str:
    """Convert a ModelDatumType to a numpy type.

    Raises:
        ValueError: if numpy doesn't support dtype
    """
    translation_map = {
        ModelDatumType.F32: "float32",
        ModelDatumType.F64: "float64",
        ModelDatumType.I32: "int32",
        ModelDatumType.I64: "int64",
        ModelDatumType.U32: "uint32",
        ModelDatumType.U64: "uint64",
        ModelDatumType.U8: "uint8",
        ModelDatumType.U16: "uint16",
        ModelDatumType.I8: "int8",
        ModelDatumType.I16: "int16",
        ModelDatumType.Bool: "bool",
    }
    if dtype not in translation_map:
        raise ValueError(f"Numpy does not support datum type {dtype}.")
    return translation_map[dtype]


def dtype_to_torch(dtype: ModelDatumType) -> str:
    """Convert a ModelDatumType to a torch type.

    Raises:
        ValueError: if torch doesn't support dtype
    """
    # Torch does not support unsigned ints except u8.
    translation_map = {
        ModelDatumType.F32: "float32",
        ModelDatumType.F64: "float64",
        ModelDatumType.I32: "int32",
        ModelDatumType.I64: "int64",
        ModelDatumType.U8: "uint8",
        ModelDatumType.I8: "int8",
        ModelDatumType.I16: "int16",
        ModelDatumType.Bool: "bool",
    }
    if dtype not in translation_map:
        raise ValueError(f"Torch does not support datum type {dtype}.")
    return translation_map[dtype]


def translate_dtype(dtype: Any) -> ModelDatumType:
    """
    Convert torch, numpy or litteral types to ModelDatumType
    Raises:
        ValueError: if dtype is erroneous or not supported
    """

    if isinstance(dtype, ModelDatumType):
        return dtype

    elif type(dtype).__module__ == "numpy" and type(dtype).__name__.startswith("dtype"):
        numpy_dtype_translation = {
            "float32": ModelDatumType.F32,
            "float64": ModelDatumType.F64,
            "int32": ModelDatumType.I32,
            "int64": ModelDatumType.I64,
            "uint32": ModelDatumType.U32,
            "uint64": ModelDatumType.U64,
            "uint8": ModelDatumType.U8,
            "uint16": ModelDatumType.U16,
            "int8": ModelDatumType.I8,
            "int16": ModelDatumType.I16,
            "bool": ModelDatumType.Bool,
        }
        if str(dtype) not in numpy_dtype_translation:
            raise ValueError(f"Numpy dtype {str(dtype)} is not supported.")
        return numpy_dtype_translation[str(dtype)]

    if type(dtype).__module__ == "torch" and type(dtype).__name__ == "dtype":
        # Torch does not support unsigned ints except u8.
        torch_dtype_translation = {
            "torch.float32": ModelDatumType.F32,
            "torch.float64": ModelDatumType.F64,
            "torch.int32": ModelDatumType.I32,
            "torch.int64": ModelDatumType.I64,
            "torch.uint8": ModelDatumType.U8,
            "torch.int8": ModelDatumType.I8,
            "torch.int16": ModelDatumType.I16,
            "torch.bool": ModelDatumType.Bool,
        }
        if str(dtype) not in torch_dtype_translation:
            raise ValueError(f"Torch dtype {str(dtype)} is not supported.")
        return torch_dtype_translation[str(dtype)]

    if isinstance(dtype, str):
        str_dtype_translation = {
            "float32": ModelDatumType.F32,
            "f32": ModelDatumType.F32,
            "float64": ModelDatumType.F64,
            "f64": ModelDatumType.F64,
            "int32": ModelDatumType.I32,
            "i32": ModelDatumType.I32,
            "int64": ModelDatumType.I64,
            "i64": ModelDatumType.I64,
            "uint32": ModelDatumType.U32,
            "u32": ModelDatumType.U32,
            "uint64": ModelDatumType.U64,
            "u64": ModelDatumType.U64,
            "uint8": ModelDatumType.U8,
            "u8": ModelDatumType.U8,
            "uint16": ModelDatumType.U16,
            "u16": ModelDatumType.U16,
            "int8": ModelDatumType.I8,
            "i8": ModelDatumType.I8,
            "int16": ModelDatumType.I16,
            "i16": ModelDatumType.I16,
            "bool": ModelDatumType.Bool,
        }
        if dtype.lower() not in str_dtype_translation:
            raise ValueError(f"Datum type {dtype} is not understood.")
        return str_dtype_translation[dtype.lower()]

    raise ValueError(
        f"DatumType instance {type(dtype).__module__}.{type(dtype).__name__} not supported"
    )


def _is_torch_tensor(tensor) -> bool:
    return type(tensor).__module__ == "torch" and type(tensor).__name__ == "Tensor"


def _is_numpy_array(tensor) -> bool:
    return type(tensor).__module__ == "numpy" and type(tensor).__name__ == "ndarray"


def translate_tensor(
    tensor: Any, or_dtype: ModelDatumType, or_shape: Tuple, name=None
) -> Tensor:
    """Put the flat/numpy/torch tensor into a Tensor object.

    Args:
        tensor: flat/numpy/torch tensor
        or_dtype: ignored if tensor isn't flat. dtype of the tensor.
        or_shape: ignored if tensor isn't flat. shape of the tensor.
    Raises:
        ValueError: if tensor format is not one of flat/numpy/torch
        ValueError: if tensor's dtype is not supported
    Returns:
        Tensor: the serialized tensor
    """
    if _is_torch_tensor(tensor):
        info = TensorInfo(tensor.shape, translate_dtype(tensor.dtype), name)
        iterable = tensor.flatten().numpy()

    elif _is_numpy_array(tensor):
        info = TensorInfo(tensor.shape, translate_dtype(tensor.dtype), name)
        iterable = tensor.flatten()

    else:
        # Input is flatten tensor.
        if not isinstance(tensor, list):
            raise ValueError(
                f"Input tensor has an unsupported type: {type(tensor).__module__}.{type(tensor).__name__}"
            )

        info = TensorInfo(or_shape, translate_dtype(or_dtype), name)
        iterable = np.array(tensor, dtype=dtype_to_numpy(or_dtype))

    if or_dtype is not None and or_dtype != info.datum_type:
        raise ValueError(
            f"Given tensor has dtype {str(tensor.dtype)}, but {or_dtype} was expected."
        )

    # todo validate tensor content, dtype and shape
    return Tensor(info.__dict__, serialize_tensor(iterable, info.datum_type))


def translate_tensors(tensors, dtypes, shapes) -> List[dict]:
    """Put the flat/numpy/torch tensors into a list of Tensor objects.

    Args:
        tensor: list or dict of flat/numpy/torch tensors
        dtypes: ignored if tensors aren't flat. list or dict of dtypes of the tensors.
        or_shape: ignored if tensor aren't flat. list or dict of shapes of the tensors.
    Returns:
        List[dict]: the serialized tensors as a list of dicts.
    """
    serialized_tensors = []

    # dict of tensors is the safe mean of passing inputs
    # if it's a dict of flat tensors, dtypes and shapes must be dicts as well
    #
    # list of {numpy/torch/flat} tensors are valid inputs, and are treated as multiple inputs
    # direct numpy/torch/flat tensors are valid inputs, and are treated as a single input, which
    #  will be wrapped into a 1-el list on the folowing statement
    #
    # flat list means list[int], and is the flattened tensor
    #  this means that you must specify dtype/shape for this tensor! on the other cases, it's redundant
    # (todo: accept iterables instead of flat list only)
    #
    # mental note
    # - anything not list should be wrapped into [X]
    # - list[int] should be wrapped into [X]
    # - but! list[list[int]] is should be unchanged

    if isinstance(tensors, dict):
        for name, tensor in tensors.items():
            or_dtype = dtypes[name] if dtypes is not None else None
            or_shape = shapes[name] if shapes is not None else None
            serialized_tensors.append(
                translate_tensor(tensor, or_dtype, or_shape, name).__dict__
            )
    else:
        # if arg is not a list of (list/numpy.array/torch.tensor), wrap it into a list
        if not isinstance(tensors, list) or (
            len(tensors) > 0
            and not (
                isinstance(tensors[0], list)
                or _is_torch_tensor(tensors[0])
                or _is_numpy_array(tensors[0])
            )
        ):
            tensors = [tensors]
        if dtypes is not None and not isinstance(dtypes, list):
            dtypes = [dtypes]
        if shapes is not None and not isinstance(shapes, list):
            shapes = [shapes]

        for i, tensor in enumerate(tensors):
            or_dtype = dtypes[i] if dtypes is not None and len(dtypes) > i else None
            or_shape = shapes[i] if shapes is not None and len(shapes) > i else None
            serialized_tensors.append(
                translate_tensor(tensor, or_dtype, or_shape).__dict__
            )

    return serialized_tensors


class BlindAiConnection(contextlib.AbstractContextManager):
    """A class to represent a connection to a BlindAi server."""

    _conn: requests.Session

    def __init__(
        self,
        addr: str,
        untrusted_port: int,
        attested_port: int,
        hazmat_manifest_path: Optional[pathlib.Path],
        hazmat_http_on_untrusted_port: bool,
    ):
        """Connect to a BlindAi service.

        Please refer to the connect function for documentation.

        Args:
            addr (str):
            untrusted_port (int):
            attested_port (int):
            hazmat_manifest_path (Optional[pathlib.Path]):
            hazmat_http_on_untrusted_port (bool):

        Returns:
        """
        # uname = platform.uname()

        # self.client_info = _ClientInfo(
        #     uid=sha256((socket.gethostname() + "-" + getpass.getuser()).encode("utf-8"))
        #     .digest()
        #     .hex(),
        #     platform_name=uname.system,
        #     platform_arch=uname.machine,
        #     platform_version=uname.version,
        #     platform_release=uname.release,
        #     user_agent="blindai_python",
        #     user_agent_version=app_version,
        # )

        if hazmat_http_on_untrusted_port:
            self._untrusted_url = "http://" + addr + ":" + str(untrusted_port)
        else:
            self._untrusted_url = "https://" + addr + ":" + str(untrusted_port)

        self._attested_url = "https://" + addr + ":" + str(attested_port)

        # This adapter makes it possible to connect
        # to the server via a different hostname
        # that the one included in the certificate i.e. blindai-srv
        # For instance we can use it to connect to the server via the
        # domain / IP provided to connect(). See below
        class CustomHostNameCheckingAdapter(HTTPAdapter):
            def cert_verify(self, conn, url, verify, cert):
                conn.assert_hostname = "blindai-srv"
                return super(CustomHostNameCheckingAdapter, self).cert_verify(
                    conn, url, verify, cert
                )

        s = requests.Session()
        # Always raise an exception when HTTP returns an error code for the untrusted connection
        # Note : we might want to do the same for the attested connection ?
        s.hooks = {"response": lambda r, *args, **kwargs: r.raise_for_status()}

        cert = cbor.loads(s.get(self._untrusted_url).content)
        quote = cbor.loads(s.get(f"{self._untrusted_url}/quote").content)
        collateral = cbor.loads(s.get(f"{self._untrusted_url}/collateral").content)

        validate_attestation(
            quote, collateral, cert, manifest_path=hazmat_manifest_path
        )

        # requests (http library) takes a path to a file containing the CA
        # there is no easy way to give the CA as a string/bytes directly
        # therefore a temporary file with the certificate content
        # has to be created.

        trusted_server_cert_file = tempfile.NamedTemporaryFile(mode="wb")
        trusted_server_cert_file.write(cert_der_to_pem(cert))
        trusted_server_cert_file.flush()
        # the file should not be close until the end of BlindAiConnection
        # so we store it in the object (else it might get garbage collected)
        self.trusted_cert_file = trusted_server_cert_file

        trusted_conn = requests.Session()
        trusted_conn.verify = trusted_server_cert_file.name
        trusted_conn.mount(self._attested_url, CustomHostNameCheckingAdapter())

        # finally try to connect to the enclave
        trusted_conn.get(self._attested_url)

        self._conn = trusted_conn

    def upload_model(
        self,
        model: str,
        model_name: Optional[str] = None,
        optimize: bool = True,
    ) -> UploadResponse:
        """Upload an inference model to the server.

        The provided model needs to be in the Onnx format.

        ***Security & confidentiality warnings:***
            model: The model sent on a Onnx format is encrypted in transit via TLS (as all connections).
            It may be subject to inference Attacks if an adversary is able to query the trained model
            repeatedly to determine whether or not a particular example is part of the trained dataset model.
        Args:
            model (str): Path to Onnx model file.
            model_name (Optional[str], optional): Name of the model. By default, the server will assign a random UUID.
                You can call the model with the name you specify here.
            optimize (bool): Whether tract (our inference engine) should optimize the model or not.
                Optimzing should only be turned off when tract wasn't able to optimze the model.
        Raises:
            HttpError: raised by the requests lib to relay server side errors
            ValueError: raised when inputs sanity checks fail
        Returns:
            UploadResponse: The response object.
        """
        if model_name is None:
            model_name = os.path.basename(model)

        with open(model, "rb") as f:
            model_bytes = f.read()

        length = len(model_bytes)

        data = UploadModel(
            model=list(model_bytes),
            length=length,
            model_name=model_name,
            optimize=optimize,
        )
        bytes_data = cbor.dumps(data.__dict__)
        r = self._conn.post(f"{self._attested_url}/upload", data=bytes_data)
        r.raise_for_status()
        send_model_reply = SendModelReply(**cbor.loads(r.content))
        ret = UploadResponse(model_id=send_model_reply.model_id)
        return ret

    def run_model(
        self,
        model_id: str,
        input_tensors: Optional[Union[List, Dict]] = None,
        dtypes: Optional[List[ModelDatumType]] = None,
        shapes: Optional[Union[List[List[int]], List[int]]] = None,
    ) -> RunModelResponse:
        """Send data to the server to make a secure inference.

        The data provided must be in a list, as the tensor will be rebuilt inside the
        server.

        ***Security & confidentiality warnings:***
            model_id: hash of the Onnx model uploaded. the given hash is return via gRPC through the proto files.
            It's a SHA-256 hash that is generated each time a model is uploaded.
            tensors: protected in transit and protected when running it on the secure enclave.
            In the case of a compromised OS, the data is isolated and confidential by SGX design.

        Args:
            model_id (str): If set, will run a specific model.
                input_tensors (Union[List[Any], List[List[Any]]))): The input data. It must be an array of numpy,
                tensors or flat list of the same type datum_type specified in `upload_model`.
            dtypes (Union[List[ModelDatumType], ModelDatumType], optional): The type of data
                of the data you want to upload. Only required if you are uploading flat lists, will be ignored
                if you are uploading numpy or tensors (this info will be extracted directly from the tensors/numpys).
            shapes (Union[List[List[int]], List[int]], optional): The shape of the data you want to upload.
                Only required if you are uploading flat lists, will be ignored if you are uploading numpy
                or tensors (this info will be extracted directly from the tensors/numpys).
        Raises:
            HttpError: raised by the requests lib to relay server side errors
            ValueError: raised when inputs sanity checks fail
        Returns:
            RunModelResponse: The response object.
        """
        # Run Model Request and Response
        tensors = translate_tensors(input_tensors, dtypes, shapes)
        run_data = RunModel(model_id=model_id, inputs=tensors)
        bytes_run_data = cbor.dumps(run_data.__dict__)
        r = self._conn.post(f"{self._attested_url}/run", data=bytes_run_data)
        r.raise_for_status()
        run_model_reply = RunModelReply(**cbor.loads(r.content))

        ret = RunModelResponse(
            output=[
                Tensor(TensorInfo(**output["info"]), output["bytes_data"])
                for output in run_model_reply.outputs
            ]
        )
        return ret

    def delete_model(self, model_id: str):
        """Delete a model in the inference server.

        This may be used to free up some memory. If you did not specify that you
        wanted your model to be saved on the server, please note that the model will
        only be present in memory, and will disappear when the server close.

        **Security & confidentiality warnings: **
            model_id: If you are using this on the Mithril Security Cloud, you can only delete models
            that you uploaded. Otherwise, the deletion of a model does only relies on the `model_id`.
            It doesn't relies on a session token or anything, hence if the `model_id` is known,
            it's deletion is possible.

        Args:
            model_id (str): The id of the model to remove.
        Raises:
            HttpError: raised by the requests lib to relay server side errors
            ValueError: raised when inputs sanity checks fail
        """
        delete_data = DeleteModel(model_id=model_id)
        bytes_delete_data = cbor.dumps(delete_data.__dict__)
        r = self._conn.post(f"{self._attested_url}/delete", bytes_delete_data)
        r.raise_for_status()

    def __enter__(self):
        """Return the BlindAiConnection upon entering the runtime context."""
        return self

    def __exit__(self, *args):
        """Close the connection to BlindAI server."""
        self._conn.close()


from functools import wraps


def connect(
    addr: str,
    untrusted_port: int = 9923,
    attested_port: int = 9924,
    hazmat_manifest_path: Optional[pathlib.Path] = None,
    hazmat_http_on_untrusted_port=False,
) -> BlindAiConnection:
    """Connect to a BlindAi server.

    Args:
        addr (str): The address of BlindAI server you want to connect to.
            It can be a domain (such as "example.com" or "localhost") or an IP
        untrusted_port (int, optional): The untrusted port number. Defaults to 9923.
        attested_port (int, optional): The attested port number. Defaults to 9924.
        hazmat_manifest_path (Optional[pathlib.Path], optional):  Path to the Manifest.toml which describes
            which enclave are to be accepted.
            Defaults to the built-in Manifest.toml provided by Mithril Security as part of the Python package.
            You can override the default by providing a path to your own Manifest.toml
            Caution: Changing the manifest can impact the security of the solution.
        hazmat_http_on_untrusted_port (bool, optional): If set to True, the client will request the attestation elements of
            the server using a plain HTTP connection instead of a more secure HTTPS connection. Defaults to False.
            Caution: This parameter should never be set to True in production. Using a HTTPS connection is critical to
            get a graceful degradation in case of a failure of the Intel SGX attestation.

     Raises:
        requests.exceptions.RequestException: If a network or server error occurs
        ValueError: raised when inputs sanity checks fail
        IdentityError: raised when the enclave signature does not match the enclave signature expected in the manifest
        EnclaveHeldDataError: raised when the expected enclave held data does not match the one in the quote
        QuoteValidationError: raised when the returned quote is invalid (TCB outdated, not signed by the hardware provider...).
        AttestationError: raised when the attestation is not valid (enclave settings mismatching, debug mode unallowed...)

    Returns:
        BlindAiConnection: An object representing an active connection to a BlindAi server
    """

    return BlindAiConnection(
        addr,
        untrusted_port,
        attested_port,
        hazmat_manifest_path,
        hazmat_http_on_untrusted_port,
    )
