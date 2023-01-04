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

from enum import IntEnum
from typing import Any, Dict, List, Optional, Tuple, Union
from cbor2 import dumps as cbor2_dumps
from cbor2 import loads as cbor2_loads
import os
import contextlib
import ssl, socket
import platform
from .utils import *
from hashlib import sha256
import getpass
import logging
import tempfile
import requests
from requests.adapters import HTTPAdapter
from importlib_metadata import version

app_version = version("blindaiv2-client-alpha")


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
    info: TensorInfo
    bytes_data: List[int]

    def __init__(self, info: TensorInfo, bytes_data: bytes):
        self.info = info
        self.bytes_data = bytes_data

    def as_flat(self) -> list:
        """Convert the prediction calculated by the server to a flat python list."""
        return cbor2_loads(bytes(self.bytes_data))

    def as_numpy(self):
        """Convert the prediction calculated by the server to a numpy array."""
        import numpy

        arr = numpy.array([*self.as_flat()], dtype=dtype_to_numpy(self.info.datum_type))
        arr.shape = self.shape
        return arr

    def as_torch(self):
        """Convert the prediction calculated by the server to a Torch Tensor."""
        try:
            import torch
        except ImportError as e:
            raise ImportError(
                "torch not installed, please install with pip install blindai[torch]"
            ) from e

        arr = torch.asarray(
            [*self.as_flat()],
            dtype=getattr(torch, dtype_to_torch(self.info.datum_type)),
        )
        return arr.view(self.shape)

    @property
    def shape(self) -> tuple:
        return tuple(self.info.fact)

    @property
    def datum_type(self) -> ModelDatumType:
        return self.info.datum_type


class UploadModel:
    model: List[int]
    input: List[TensorInfo]
    output: List[ModelDatumType]
    length: int
    sign: bool
    model_name: str
    optimize: bool

    def __init__(
        self, model, input, output, length, sign=False, model_name="", optimize=True
    ):
        self.model = model
        self.input = input
        self.output = output
        self.length = length
        self.sign = sign
        self.model_name = model_name
        self.optimize = optimize


class RunModel:
    model_id: str
    inputs: List[Tensor]
    sign: bool

    def __init__(self, model_id, inputs, sign):
        self.model_id = model_id
        self.inputs = inputs
        self.sign = sign


class DeleteModel:
    model_id: str

    def __init__(self, model_id):
        self.model_id = model_id


class SendModelPayload:
    hash: List[int]
    inputfact: List[int]
    model_id: str

    def __init__(self, **entries):
        self.__dict__.update(entries)


class SendModelReply:
    payload: SendModelPayload
    signature: List[int]


class RunModelPayload:
    outputs: List[Tensor]
    datum_output: List[int]
    input_hash: List[int]
    model_id: str

    def __init__(self, **entries):
        self.__dict__.update(entries)


class RunModelReply:
    payload: RunModelPayload
    signature: List[int]


class SignedResponse:
    payload: Optional[bytes] = None
    signature: Optional[bytes] = None
    # attestation: Optional[GetSgxQuoteWithCollateralReply] = None


class UploadResponse(SignedResponse):
    model_id: str


class RunModelResponse(SignedResponse):
    output: List[Tensor]
    model_id: str


class ClientInfo:
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


def _get_input_output_tensors(
    tensor_inputs: Optional[List[List[Any]]] = None,
    tensor_outputs: Optional[ModelDatumType] = None,
    shape: Tuple = None,
    dtype: ModelDatumType = ModelDatumType.F32,
    dtype_out: ModelDatumType = ModelDatumType.F32,
) -> Tuple[List[List[Any]], List[ModelDatumType]]:
    if tensor_inputs is None and (dtype is None or shape is None):
        tensor_inputs = []

    if tensor_outputs is None and dtype_out is None:
        tensor_outputs = []

    if tensor_inputs is None or tensor_outputs is None:
        tensor_inputs = [shape, dtype]
        tensor_outputs = [
            dtype_out
        ]  # Dict may be required for correct cbor serialization

    if len(tensor_inputs) > 0 and type(tensor_inputs[0]) != list:
        tensor_inputs = [tensor_inputs]

    if len(tensor_outputs) > 0 and type(tensor_outputs) != list:
        tensor_outputs = [tensor_outputs]

    inputs = []
    for tensor_input in tensor_inputs:
        inputs.append(
            TensorInfo(fact=tensor_input[0], datum_type=tensor_input[1]).__dict__
        )  # Required for correct cbor serialization

    return (inputs, tensor_outputs)


def dtype_to_numpy(dtype: ModelDatumType) -> str:
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


def translate_dtype(dtype):
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


def is_torch_tensor(tensor):
    return type(tensor).__module__ == "torch" and type(tensor).__name__ == "Tensor"


def is_numpy_array(tensor):
    return type(tensor).__module__ == "numpy" and type(tensor).__name__ == "ndarray"


def translate_tensor(tensor, or_dtype, or_shape, name=None):
    if is_torch_tensor(tensor):
        info = TensorInfo(tensor.shape, translate_dtype(tensor.dtype), name)
        iterable = tensor.flatten().tolist()

    elif is_numpy_array(tensor):
        info = TensorInfo(tensor.shape, translate_dtype(tensor.dtype), name)
        iterable = tensor.flatten().tolist()

    else:
        # Input is flatten tensor.
        if not isinstance(tensor, list):
            raise ValueError(
                f"Input tensor has an unsupported type: {type(tensor).__module__}.{type(tensor).__name__}"
            )

        info = TensorInfo(or_shape, translate_dtype(or_dtype), name)
        iterable = tensor

    if or_dtype is not None and or_dtype != info.datum_type:
        raise ValueError(
            f"Given tensor has dtype {str(tensor.dtype)}, but {or_dtype} was expected."
        )

    # todo validate tensor content, dtype and shape
    return Tensor(info.__dict__, list(cbor2_dumps(iterable)))


def translate_tensors(tensors, dtypes, shapes):
    """
    >>> tensor1 = [1, 2, 3, 4]
    >>> o = translate_tensors(tensor1, ModelDatumType.I64, (4,))
    >>> cbor2_loads(bytes(o[0]["bytes_data"])), o[0]["info"]
    ([1, 2, 3, 4], {'fact': (4,), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': None})


    >>> import numpy
    >>> tensor2 = numpy.array([1, 2, 3, 4])
    >>> o = translate_tensors(tensor2, None, None)
    >>> cbor2_loads(bytes(o[0]["bytes_data"])), o[0]["info"]
    ([1, 2, 3, 4], {'fact': (4,), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': None})


    >>> import torch
    >>> tensor3 = torch.tensor([1, 2, 3, 4])
    >>> o = translate_tensors(tensor3, None, None)
    >>> cbor2_loads(bytes(o[0]["bytes_data"])), o[0]["info"]
    ([1, 2, 3, 4], {'fact': torch.Size([4]), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': None})


    >>> o = translate_tensors([tensor1, tensor2, tensor3], [ModelDatumType.I64, None, None], [(4,), None, None])
    >>> for t in o:
    ...    t["bytes_data"] = cbor2_loads(bytes(t["bytes_data"]))
    >>> o
    [\
{'info': {'fact': (4,), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': None}, 'bytes_data': [1, 2, 3, 4]}, \
{'info': {'fact': (4,), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': None}, 'bytes_data': [1, 2, 3, 4]}, \
{'info': {'fact': torch.Size([4]), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': None}, 'bytes_data': [1, 2, 3, 4]}]


    >>> o = translate_tensors(\
        {"tensor1": tensor1, "tensor2": tensor2, "tensor3": tensor3}, \
        {"tensor1": ModelDatumType.I64, "tensor2": None, "tensor3": None}, \
        {"tensor1": (4,), "tensor2": None, "tensor3": None}\
    )
    >>> for t in o:
    ...    t["bytes_data"] = cbor2_loads(bytes(t["bytes_data"]))
    >>> o
    [\
{'info': {'fact': (4,), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': 'tensor1'}, 'bytes_data': [1, 2, 3, 4]}, \
{'info': {'fact': (4,), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': 'tensor2'}, 'bytes_data': [1, 2, 3, 4]}, \
{'info': {'fact': torch.Size([4]), 'datum_type': <ModelDatumType.I64: 3>, 'node_name': 'tensor3'}, 'bytes_data': [1, 2, 3, 4]}]
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
                or is_torch_tensor(tensors[0])
                or is_numpy_array(tensors[0])
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
    conn: requests.Session
    # policy: Optional[Policy] = None
    # _stub: Optional[ExchangeStub] = None
    enclave_signing_key: Optional[bytes] = None
    simulation_mode: bool = False
    _disable_untrusted_server_cert_check: bool = False
    # attestation: Optional[GetSgxQuoteWithCollateralReply] = None
    server_version: Optional[str] = None
    # client_info: ClientInfo
    tensor_inputs: Optional[List[List[Any]]]
    tensor_outputs: Optional[List[ModelDatumType]]
    closed: bool = False

    def __init__(
        self,
        addr: str,
        server_name: str = "blindai-srv",
        # policy: Optional[str] = None,
        certificate: Optional[str] = None,
        simulation: bool = False,
        untrusted_port: int = 9923,
        attested_port: int = 9924,
        debug_mode=False,
    ):
        """
        certificate: path to untrusted certificate in PEM format
        """
        # if debug_mode:  # pragma: no cover
        #    os.environ["GRPC_TRACE"] = "transport_security,tsi"
        #    os.environ["GRPC_VERBOSITY"] = "DEBUG"

        uname = platform.uname()

        self.client_info = ClientInfo(
            uid=sha256((socket.gethostname() + "-" + getpass.getuser()).encode("utf-8"))
            .digest()
            .hex(),
            platform_name=uname.system,
            platform_arch=uname.machine,
            platform_version=uname.version,
            platform_release=uname.release,
            user_agent="blindai_python",
            user_agent_version=app_version,
        )

        self.connect_server(
            addr,
            server_name,
            # policy,
            certificate,
            simulation,
            untrusted_port,
            attested_port,
        )

    def connect_server(
        self,
        addr: str,
        server_name,
        # policy,
        certificate,
        simulation,
        untrusted_port,
        attested_port,
    ):
        self.simulation_mode = simulation
        self._disable_untrusted_server_cert_check = simulation

        # addr = strip_https(addr)

        self._untrusted_url = "https://" + addr + ":" + str(untrusted_port)
        self._attested_url = "https://" + addr + ":" + str(attested_port)

        # if not self.simulation_mode:
        #    self.policy = Policy.from_file(policy)

        # This adapter makes it possible to connect
        # to the server via a different hostname
        # that the one included in the certificate i.e. blindai-srv
        # For instance we can use it to connect to the server via the
        # domain / IP provided to connect(). See below

        # TODO: Document that in production one should not use that approach
        # but instead include the real domain name in the certificate
        class CustomHostNameCheckingAdapter(HTTPAdapter):
            def cert_verify(self, conn, url, verify, cert):
                conn.assert_hostname = "blindai-srv"
                return super(CustomHostNameCheckingAdapter, self).cert_verify(
                    conn, url, verify, cert
                )

        s = requests.Session()
        if self._disable_untrusted_server_cert_check:
            logging.warning("Untrusted server certificate check bypassed")
            s.verify = False
        else:
            # verify is set to a path to the certificate CA
            # It is used to pin the certificate
            # Might not be needed in production
            # Certificate pinning is a double edge sword
            # It can prevent some MITM but it also make it more difficult
            # to rotate the certificate...
            # Anyway in our case as we'll do attestation verification
            # it would make sense to use a simpler cert validation
            # maybe simply the default CA with domain validation
            # like the browser do.
            # Can't be more boring (in a good way).
            s.verify = certificate
        s.mount(self._untrusted_url, CustomHostNameCheckingAdapter())

        retrieved_cert = s.get(self._untrusted_url).text
        trusted_server_cert = ssl.get_server_certificate((addr, attested_port))

        # both certificates should match (up to the PEM encoding which might slightly differ)
        assert cryptography.x509.load_pem_x509_certificate(
            bytes(retrieved_cert, encoding="ascii")
        ) == cryptography.x509.load_pem_x509_certificate(
            bytes(trusted_server_cert, encoding="ascii")
        )

        # requests (http library) takes a path to a file containing the CA
        # there is no easy way to give the CA as a string/bytes directly
        # therefore a temporary file with the certificate content
        # has to be created.
        trusted_server_cert_file = tempfile.NamedTemporaryFile(mode="w")
        trusted_server_cert_file.write(retrieved_cert)
        trusted_server_cert_file.flush()
        # the file should not be close until the end of BlindAiConnection
        # so we store it in the object (else it might get garbage collected)
        self.trusted_cert_file = trusted_server_cert_file

        trusted_conn = requests.Session()
        trusted_conn.verify = trusted_server_cert_file.name
        trusted_conn.mount(self._attested_url, CustomHostNameCheckingAdapter())

        # finally try to connect to the enclave
        trusted_conn.get(self._attested_url)

        self.conn = trusted_conn

    def upload_model(
        self,
        model: str,
        tensor_inputs: Optional[List[Tuple[List[int], ModelDatumType]]] = None,
        tensor_outputs: Optional[List[ModelDatumType]] = None,
        shape: Tuple = None,
        dtype: ModelDatumType = None,
        dtype_out: ModelDatumType = None,
        sign: bool = False,
        model_name: Optional[str] = None,
        optimize: bool = True,
    ) -> UploadResponse:

        if model_name is None:
            model_name = os.path.basename(model)

        with open(model, "rb") as f:
            model = f.read()

        model = list(model)
        length = len(model)

        (inputs, outputs) = _get_input_output_tensors(
            tensor_inputs, tensor_outputs, shape, dtype, dtype_out
        )

        data = UploadModel(
            model=model,
            input=inputs,
            output=outputs,
            length=length,
            sign=False,
            model_name=model_name,
            optimize=optimize,
        )
        data = cbor2_dumps(data.__dict__)
        r = self.conn.post(f"{self._attested_url}/upload", data=data)
        r.raise_for_status()
        send_model_reply = cbor2_loads(r.content)
        payload = cbor2_loads(bytes(send_model_reply["payload"]))
        payload = SendModelPayload(**payload)
        ret = UploadResponse()
        ret.model_id = payload.model_id
        if sign:
            ret.payload = payload
            ret.signature = send_model_reply.signature
            # ret.attestation =

        return ret

    def run_model(
        self,
        model_id: str,
        input_tensors: Optional[Union[List[List], Dict]] = None,
        dtypes: Optional[List[ModelDatumType]] = None,
        shapes: Optional[Union[List[List[int]], List[int]]] = None,
        sign: bool = False,
    ) -> RunModelResponse:

        # Run Model Request and Response
        tensors = translate_tensors(input_tensors, dtypes, shapes)
        run_data = RunModel(model_id=model_id, inputs=tensors, sign=False)
        run_data = cbor2_dumps(run_data.__dict__)
        r = self.conn.post(f"{self._attested_url}/run", data=run_data)
        r.raise_for_status()
        run_model_reply = cbor2_loads(r.content)
        payload = cbor2_loads(bytes(run_model_reply["payload"]))
        payload = RunModelPayload(**payload)

        ret = RunModelResponse()
        ret.output = [
            Tensor(TensorInfo(**output["info"]), output["bytes_data"])
            for output in payload.outputs
        ]

        if sign:
            ret.payload = payload
            ret.signature = run_model_reply.signature
            # ret.attestation = self.attestation

        return ret

    def delete_model(self, model_id: str):
        delete_data = DeleteModel(model_id=model_id)
        delete_data = cbor2_dumps(delete_data.__dict__)
        r = self.conn.post(f"{self._attested_url}/delete", delete_data)
        r.raise_for_status()

    def close(self):
        """Close the connection between the client and the inference server. This method has no effect if the file is already closed."""
        if not self.closed:
            self.closed = True
            # self.policy = None
            self.server_version = None

    def __enter__(self):
        """Return the BlindAiConnection upon entering the runtime context."""
        return self

    def __exit__(self, *args):
        """Close the connection to BlindAI server and raise any exception triggered within the runtime context."""
        self.close()


from functools import wraps


@wraps(BlindAiConnection.__init__, assigned=("__doc__", "__annotations__"))
def connect(*args, **kwargs):
    return BlindAiConnection(*args, **kwargs)
