# Copyright 2022 Mithril Security. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import contextlib
from functools import wraps
import getpass
import logging
import os
import socket
import ssl
import platform
from enum import IntEnum
from hashlib import sha256
from typing import Any, Dict, List, Optional, Tuple, Union

from cbor2 import dumps as cbor2_dumps
from cbor2 import loads as cbor2_loads
from cryptography.exceptions import InvalidSignature
from grpc import Channel, RpcError, secure_channel, ssl_channel_credentials
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

from blindai.dcap_attestation import (
    Policy,
    verify_claims,
    verify_dcap_attestation,
)

# These modules are generated by grpc proto compiler, from proto files in proto
import blindai.pb as _
from blindai.pb.securedexchange_pb2 import (
    DatumTypeEnum,
    Payload,
    RunModelRequest,
    SendModelRequest,
    ClientInfo,
    TensorInfo,
    DeleteModelRequest,
)
from blindai.pb.proof_files_pb2 import ResponseProof
from blindai.pb.securedexchange_pb2_grpc import ExchangeStub
from blindai.pb.untrusted_pb2 import GetCertificateRequest as certificate_request
from blindai.pb.untrusted_pb2 import GetServerInfoRequest as server_info_request
from blindai.pb.untrusted_pb2 import GetSgxQuoteWithCollateralReply
from blindai.pb.untrusted_pb2 import GetSgxQuoteWithCollateralRequest as quote_request
from blindai.pb.untrusted_pb2_grpc import AttestationStub
from blindai.utils.errors import (
    SignatureError,
    VersionError,
    check_rpc_exception,
    check_socket_exception,
)
from blindai.utils.utils import (
    create_byte_chunk,
    encode_certificate,
    get_enclave_signing_key,
    strip_https,
    supported_server_version,
)
from blindai.version import __version__ as app_version


CONNECTION_TIMEOUT = 10

ModelDatumType = IntEnum("ModelDatumType", DatumTypeEnum.items())


def _validate_quote(
    attestation: GetSgxQuoteWithCollateralReply, policy: Policy
) -> Ed25519PublicKey:
    """Returns the enclave signing key"""

    claims = verify_dcap_attestation(
        attestation.quote, attestation.collateral, attestation.enclave_held_data
    )

    verify_claims(claims, policy)
    server_cert = claims.get_server_cert()
    enclave_signing_key = get_enclave_signing_key(server_cert)

    return enclave_signing_key


def _get_input_output_tensors(
    tensor_inputs: Optional[List[List[Any]]] = None,
    tensor_outputs: Optional[ModelDatumType] = None,
    shape: Tuple = None,
    dtype: ModelDatumType = ModelDatumType.F32,
    dtype_out: ModelDatumType = ModelDatumType.F32,
) -> Tuple[List[List[Any]], List[ModelDatumType]]:
    if tensor_inputs is None or tensor_outputs is None:
        tensor_inputs = [shape, dtype]
        tensor_outputs = dtype_out

    if type(tensor_inputs[0]) != list:
        tensor_inputs = [tensor_inputs]

    if type(tensor_outputs) != list:
        tensor_outputs = [tensor_outputs]

    inputs = []
    for tensor_input in tensor_inputs:
        inputs.append(TensorInfo(fact=tensor_input[0], datum_type=tensor_input[1]))

    return (inputs, tensor_outputs)


class SignedResponse:
    payload: Optional[bytes] = None
    signature: Optional[bytes] = None
    attestation: Optional[GetSgxQuoteWithCollateralReply] = None

    def is_simulation_mode(self) -> bool:
        return self.attestation is None

    def is_signed(self) -> bool:
        return self.signature is not None

    def save_to_file(self, path: str):
        """Save the response to a file.
        The response can later be loaded with:
        ```py
        res = SignedResponse()
        res.load_from_file(path)
        ```

        Args:
            path (str): Path of the file.
        """
        with open(path, mode="wb") as file:
            file.write(self.as_bytes())

    def as_bytes(self) -> bytes:
        """Save the response as bytes.
        The response can later be loaded with:
        ```py
        res = SignedResponse()
        res.load_from_bytes(data)
        ```

        Returns:
            bytes: The data.
        """
        return ResponseProof(
            payload=self.payload,
            signature=self.signature,
            attestation=self.attestation,
        ).SerializeToString()

    def load_from_file(self, path: str):
        """Load the response from a file.

        Args:
            path (str): Path of the file.
        """
        with open(path, "rb") as file:
            self.load_from_bytes(file.read())

    def load_from_bytes(self, b: bytes):
        """Load the response from bytes.

        Args:
            b (bytes): The data.
        """
        proof = ResponseProof.FromString(b)
        self.payload = proof.payload
        self.signature = proof.signature
        self.attestation = proof.attestation
        self._load_payload()

    def _load_payload(self):
        pass


class UploadModelResponse(SignedResponse):
    model_id: str

    def validate(
        self,
        model_hash: bytes,
        policy_file: Optional[str] = None,
        policy: Optional[Policy] = None,
        validate_quote: bool = True,
        enclave_signing_key: Optional[bytes] = None,
        allow_simulation_mode: bool = False,
    ):
        """Validates whether this response is valid. This is used for responses you have saved as bytes or in a file.
        This will raise an error if the response is not signed or if it is not valid.

        Args:
            model_hash (bytes): Hash of the model to verify against.
            policy_file (Optional[str], optional): Path to the policy file. Defaults to None.
            policy (Optional[Policy], optional): Policy to use. Use `policy_file` to load from a file directly. Defaults to None.
            validate_quote (bool, optional): Whether or not the attestation should be validated too. Defaults to True.
            enclave_signing_key (Optional[bytes], optional): Enclave signing key in case the attestation should not be validated. Defaults to None.
            allow_simulation_mode (bool, optional): Whether or not simulation mode responses should be accepted. Defaults to False.

        Raises:
            AttestationError: Attestation is invalid.
            SignatureError: Signed response is invalid.
            FileNotFoundError: Will be raised if the policy file is not found.
        """
        if not self.is_signed():
            raise SignatureError("Response is not signed")

        if not allow_simulation_mode and self.is_simulation_mode():
            raise SignatureError("Response was produced using simulation mode")

        if not self.is_simulation_mode() and validate_quote and policy_file is not None:
            policy = Policy.from_file(policy_file)

        # Quote validation

        if not self.is_simulation_mode() and validate_quote:
            enclave_signing_key = _validate_quote(self.attestation, policy)

        # Payload validation

        payload = Payload.FromString(self.payload).send_model_payload
        if not self.is_simulation_mode():
            try:
                enclave_signing_key.verify(self.signature, self.payload)
            except InvalidSignature:
                raise SignatureError("Invalid signature")

        # Input validation

        if model_hash != payload.model_hash:
            raise SignatureError("Invalid returned model_hash")

    def _load_payload(self):
        payload = Payload.FromString(self.payload).send_model_payload
        self.model_id = payload.model_id


class RunModelResponse(SignedResponse):
    output: List[float]
    model_id: str

    def validate(
        self,
        model_id: str,
        data_list: List[Any],
        policy_file: Optional[str] = None,
        policy: Optional[Policy] = None,
        validate_quote: bool = True,
        enclave_signing_key: Optional[bytes] = None,
        allow_simulation_mode: bool = False,
    ):
        """Validates whether this response is valid. This is used for responses you have saved as bytes or in a file.
        This will raise an error if the response is not signed or if it is not valid.

        Args:
            model_id (str): The model id to check against.
            data_list (List[Any]): Input used to run the model, to validate against.
            policy_file (Optional[str], optional): Path to the policy file. Defaults to None.
            policy (Optional[Policy], optional): Policy to use. Use `policy_file` to load from a file directly. Defaults to None.
            validate_quote (bool, optional): Whether or not the attestation should be validated too. Defaults to True.
            enclave_signing_key (Optional[bytes], optional): Enclave signing key in case the attestation should not be validated. Defaults to None.
            allow_simulation_mode (bool, optional): Whether or not simulation mode responses should be accepted. Defaults to False.

        Raises:
            AttestationError: Attestation is invalid.
            SignatureError: Signed response is invalid.
            FileNotFoundError: Will be raised if the policy file is not found.
        """
        if not self.is_signed():
            raise SignatureError("Response is not signed")

        if not allow_simulation_mode and self.is_simulation_mode():
            raise SignatureError("Response was produced using simulation mode")

        if not self.is_simulation_mode() and validate_quote and policy_file is not None:
            policy = Policy.from_file(policy_file)

        # Quote validation

        if not self.is_simulation_mode() and validate_quote:
            enclave_signing_key = _validate_quote(self.attestation, policy)

        # Payload validation

        payload = Payload.FromString(self.payload).run_model_payload
        if not self.is_simulation_mode():
            try:
                enclave_signing_key.verify(self.signature, self.payload)
            except InvalidSignature:
                raise SignatureError("Invalid signature")

        # Input validation

        serialized_bytes = cbor2_dumps(data_list)
        if sha256(serialized_bytes).digest() != payload.input_hash:
            raise SignatureError("Invalid returned input_hash")

        if model_id != payload.model_id:
            raise SignatureError("Invalid returned model_id")

    def _load_payload(self):
        payload = Payload.FromString(self.payload).run_model_payload
        self.output = payload.output
        self.model_id = payload.model_id


class DeleteModelResponse:
    pass


def raise_exception_if_conn_closed(f):
    """
    Decorator which raises an exception if the BlindAiConnection is closed before calling
    the decorated method
    """

    @wraps(f)
    def wrapper(self, *args, **kwds):
        if self.closed:
            raise ValueError("Illegal operation on closed connection.")
        return f(self, *args, **kwds)

    return wrapper


class BlindAiConnection(contextlib.AbstractContextManager):
    _channel: Optional[Channel] = None
    policy: Optional[Policy] = None
    _stub: Optional[ExchangeStub] = None
    enclave_signing_key: Optional[bytes] = None
    simulation_mode: bool = False
    _disable_untrusted_server_cert_check: bool = False
    attestation: Optional[GetSgxQuoteWithCollateralReply] = None
    server_version: Optional[str] = None
    client_info: ClientInfo
    tensor_inputs: Optional[List[List[Any]]]
    tensor_outputs: Optional[List[ModelDatumType]]
    closed: bool = False

    def __init__(
        self,
        addr: str,
        server_name: str = "blindai-srv",
        policy: Optional[str] = None,
        certificate: Optional[str] = None,
        simulation: bool = False,
        untrusted_port: int = 50052,
        attested_port: int = 50051,
        debug_mode=False,
    ):
        """Connect to the server with the specified parameters.
        You will have to specify here the expected policy (server identity, configuration...)
        and the server TLS certificate, if you are using the hardware mode.

        If you're using the simulation mode, you don't need to provide a policy and certificate,
        but please keep in mind that this mode should NEVER be used in production as it doesn't
        have most of the security provided by the hardware mode.

        Args:
            addr (str): The address of BlindAI server you want to reach.
            server_name (str, optional): Contains the CN expected by the server TLS certificate. Defaults to "blindai-srv".
            policy (Optional[str], optional): Path to the toml file describing the policy of the server.
                Generated in the server side. Defaults to None.
            certificate (Optional[str], optional): Path to the public key of the untrusted inference server.
                Generated in the server side. Defaults to None.
            simulation (bool, optional): Connect to the server in simulation mode.
                If set to True, the args policy and certificate will be ignored. Defaults to False.
            untrusted_port (int, optional): Untrusted connection server port. Defaults to 50052.
            attested_port (int, optional): Attested connection server port. Defaults to 50051.

        Raises:
            AttestationError: Will be raised in case the policy doesn't match the
                server identity and configuration, or if te attestation is invalid.
            ConnectionError: will be raised if the connection with the server fails.
            VersionError: Will be raised if the version of the server is not supported by the client.
            FileNotFoundError: will be raised if the policy file, or the certificate file is not
                found (in Hardware mode).
        """
        if debug_mode:  # pragma: no cover
            os.environ["GRPC_TRACE"] = "transport_security,tsi"
            os.environ["GRPC_VERBOSITY"] = "DEBUG"

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

        self._connect_server(
            addr,
            server_name,
            policy,
            certificate,
            simulation,
            untrusted_port,
            attested_port,
        )

    def _connect_server(
        self,
        addr: str,
        server_name,
        policy,
        certificate,
        simulation,
        untrusted_port,
        attested_port,
    ):
        self.simulation_mode = simulation
        self._disable_untrusted_server_cert_check = simulation

        addr = strip_https(addr)

        untrusted_client_to_enclave = addr + ":" + str(untrusted_port)
        attested_client_to_enclave = addr + ":" + str(attested_port)

        if not self.simulation_mode:
            self.policy = Policy.from_file(policy)

        if self._disable_untrusted_server_cert_check:
            logging.warning("Untrusted server certificate check bypassed")

            try:
                socket.setdefaulttimeout(CONNECTION_TIMEOUT)
                untrusted_server_cert = ssl.get_server_certificate(
                    (addr, untrusted_port)
                )
                untrusted_server_creds = ssl_channel_credentials(
                    root_certificates=bytes(untrusted_server_cert, encoding="utf8")
                )

            except RpcError as rpc_error:
                raise ConnectionError(check_rpc_exception(rpc_error))

            except socket.error as socket_error:
                raise ConnectionError(check_socket_exception(socket_error))

        else:
            with open(certificate, "rb") as f:
                untrusted_server_creds = ssl_channel_credentials(
                    root_certificates=f.read()
                )

        connection_options = (("grpc.ssl_target_name_override", server_name),)

        try:
            channel = secure_channel(
                untrusted_client_to_enclave,
                untrusted_server_creds,
                options=connection_options,
            )
            stub = AttestationStub(channel)

            response = stub.GetServerInfo(server_info_request())
            self.server_version = response.version
            if not supported_server_version(response.version):
                raise VersionError(
                    "Incompatible client/server versions. Please use the correct client for your server."
                )

            if self.simulation_mode:
                logging.warning(
                    "Attestation process is bypassed: running without requesting and checking attestation"
                )
                response = stub.GetCertificate(certificate_request())
                server_cert = encode_certificate(response.enclave_tls_certificate)

            else:
                self.attestation = stub.GetSgxQuoteWithCollateral(quote_request())
                claims = verify_dcap_attestation(
                    self.attestation.quote,
                    self.attestation.collateral,
                    self.attestation.enclave_held_data,
                )

                verify_claims(claims, self.policy)
                server_cert = claims.get_server_cert()

                logging.info("Quote verification passed")
                logging.info(
                    f"Certificate from attestation process\n {server_cert.decode('ascii')}"
                )
                logging.info("MREnclave\n" + claims.sgx_mrenclave)

            channel.close()
            self.enclave_signing_key = get_enclave_signing_key(server_cert)
            server_creds = ssl_channel_credentials(root_certificates=server_cert)
            channel = secure_channel(
                attested_client_to_enclave, server_creds, options=connection_options
            )

            self._stub = ExchangeStub(channel)
            self._channel = channel
            logging.info("Successfuly connected to the server")

        except RpcError as rpc_error:
            channel.close()
            raise ConnectionError(check_rpc_exception(rpc_error))

    @raise_exception_if_conn_closed
    def upload_model(
        self,
        model: str,
        tensor_inputs: Optional[List[List[List[int], ModelDatumType]]] = None,
        tensor_outputs: Optional[List[ModelDatumType]] = None,
        shape: Tuple = None,
        dtype: ModelDatumType = ModelDatumType.F32,
        dtype_out: ModelDatumType = ModelDatumType.F32,
        sign: bool = False,
        model_name: Optional[str] = None,
    ) -> UploadModelResponse:
        """Upload an inference model to the server.
        The provided model needs to be in the Onnx format.

        Args:
            model (str): Path to Onnx model file.
            tensor_inputs (Union[List[List[int], ModelDatumType], List[List[List[int], ModelDatumType]]): The list of input fact and datum types for each input grouped together in lists, describing the different inputs of the model.
            tensor_outputs (Union[ModelDatumType, List[ModelDatumType]): The list of datum types describing the different output types of the model. Defaults to ModelDatumType.F32.
            shape (Tuple, optional): The shape of the model input. Defaults to None.
            dtype (ModelDatumType, optional): The type of the model input data (f32 by default). Defaults to ModelDatumType.F32.
            dtype_out (ModelDatumType, optional): The type of the model output data (f32 by default). Defaults to ModelDatumType.F32.
            sign (bool, optional): Get signed responses from the server or not. Defaults to False.
            model_name (Optional[str], optional): Name of the model.

        Raises:
            ConnectionError: Will be raised if the client is not connected.
            FileNotFoundError: Will be raised if the model file is not found.
            SignatureError: Will be raised if the response signature is invalid.
            ValueError: Will be raised if the connection is closed.

        Returns:
            UploadModelResponse: The response object.
        """

        response = None

        if model_name is None:
            model_name = os.path.basename(model)

        try:
            with open(model, "rb") as f:
                data = f.read()

            (inputs, outputs) = _get_input_output_tensors(
                tensor_inputs, tensor_outputs, shape, dtype, dtype_out
            )
            response = self._stub.SendModel(
                iter(
                    [
                        SendModelRequest(
                            length=len(data),
                            data=chunk,
                            sign=sign,
                            model_name=model_name,
                            client_info=self.client_info,
                            tensor_inputs=inputs,
                            tensor_outputs=outputs,
                        )
                        for chunk in create_byte_chunk(data)
                    ]
                )
            )

        except RpcError as rpc_error:
            raise ConnectionError(check_rpc_exception(rpc_error))

        # Response Verification
        payload = Payload.FromString(response.payload).send_model_payload
        ret = UploadModelResponse()
        ret.model_id = payload.model_id

        if sign:
            ret.payload = response.payload
            ret.signature = response.signature
            ret.attestation = self.attestation
            ret.validate(
                sha256(data).digest(),
                validate_quote=False,
                enclave_signing_key=self.enclave_signing_key,
                allow_simulation_mode=self.simulation_mode,
            )

        return ret

    @raise_exception_if_conn_closed
    def run_model(
        self,
        model_id: str,
        data_list: List[List[List[int], ModelDatumType]],
        sign: bool = False,
    ) -> RunModelResponse:
        """Send data to the server to make a secure inference.

        The data provided must be in a list, as the tensor will be rebuilt inside the server.

        Args:
            model_id (str): If set, will run a specific model.
            data_list (Union[List[Any], List[List[Any]]))): The input data. It must be an array of numbers or an array of arrays of numbers of the same type dtype specified in `upload_model`.
            sign (bool, optional): Get signed responses from the server or not. Defaults to False.

        Raises:
            ConnectionError: Will be raised if the client is not connected.
            SignatureError: Will be raised if the response signature is invalid
            ValueError: Will be raised if the connection is closed
        Returns:
            RunModelResponse: The response object.
        """

        try:
            if type(data_list[0]) != list:
                data_list = [data_list]

            serialized_bytes = cbor2_dumps(data_list)
            response = self._stub.RunModel(
                iter(
                    [
                        RunModelRequest(
                            model_id=model_id,
                            client_info=self.client_info,
                            input=serialized_bytes_chunk,
                            sign=sign,
                        )
                        for serialized_bytes_chunk in create_byte_chunk(
                            serialized_bytes
                        )
                    ]
                )
            )

        except RpcError as rpc_error:
            raise ConnectionError(check_rpc_exception(rpc_error))

        # Response Verification
        payload = Payload.FromString(response.payload).run_model_payload
        ret = RunModelResponse()
        ret.output = cbor2_loads(payload.output)

        if sign:
            ret.payload = response.payload
            ret.signature = response.signature
            ret.attestation = self.attestation
            ret.validate(
                model_id,
                data_list,
                validate_quote=False,
                enclave_signing_key=self.enclave_signing_key,
                allow_simulation_mode=self.simulation_mode,
            )

        return ret

    @raise_exception_if_conn_closed
    def delete_model(self, model_id: str) -> DeleteModelResponse:
        """Delete a model in the inference server.
        This may be used to free up some memory.
        Note that the model in currently stored in-memory, and you cannot keep it loaded across server restarts.

        Args:
            model_id (str): The id of the model to remove.

        Raises:
            ConnectionError: Will be raised if the client is not connected or if an happens.
            ValueError: Will be raised if the connection is closed
        Returns:
            DeleteModelResponse: The response object.
        """
        try:
            self._stub.DeleteModel(DeleteModelRequest(model_id=model_id))

        except RpcError as rpc_error:
            raise ConnectionError(check_rpc_exception(rpc_error))

        return DeleteModelResponse()

    def close(self):
        """Close the connection between the client and the inference server. This method has no effect if the file is already closed."""
        if not self.closed:
            self._channel.close()
            self.closed = True
            self._channel = None
            self._stub = None
            self.policy = None
            self.server_version = None

    def __enter__(self):
        """Return the BlindAiConnection upon entering the runtime context."""
        return self

    def __exit__(self, *args):
        """Close the connection to BlindAI server and raise any exception triggered within the runtime context."""
        self.close()


@wraps(BlindAiConnection.__init__, assigned=("__doc__", "__annotations__"))
def connect(*args, **kwargs):
    return BlindAiConnection(*args, **kwargs)
