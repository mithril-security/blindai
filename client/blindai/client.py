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

import getpass
import logging
import os
import socket
import ssl
import platform
from enum import IntEnum
from hashlib import sha256
from typing import Any, Dict, List, Optional

from cbor2 import dumps as cbor2_dumps
from cbor2 import loads as cbor2_loads
from cryptography.exceptions import InvalidSignature
from grpc import Channel, RpcError, secure_channel, ssl_channel_credentials

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
    Pair,
    TensorInfo
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
) -> bytes:
    """Returns the enclave signing key"""

    claims = verify_dcap_attestation(
        attestation.quote, attestation.collateral, attestation.enclave_held_data
    )

    verify_claims(claims, policy)
    server_cert = claims.get_server_cert()
    enclave_signing_key = get_enclave_signing_key(server_cert)

    return enclave_signing_key


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


class RunModelResponse(SignedResponse):
    output: List[float]

    def validate(
        self,
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

    def _load_payload(self):
        payload = Payload.FromString(self.payload).run_model_payload
        self.output = payload.output


class BlindAiClient:
    _channel: Optional[Channel] = None
    policy: Optional[Policy] = None
    _stub: Optional[ExchangeStub] = None
    enclave_signing_key: Optional[bytes] = None
    simulation_mode: bool = False
    _disable_untrusted_server_cert_check: bool = False
    attestation: Optional[GetSgxQuoteWithCollateralReply] = None
    server_version: Optional[str] = None
    client_info: ClientInfo
    tensor_inputs: Dict
    tensor_outputs: Dict

    def __init__(self, debug_mode=False):
        if debug_mode:
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

    def is_connected(self) -> bool:
        return self._channel is not None

    def _close_channel(self):
        if self.is_connected():
            self._channel.close()
            self._channel = None

    def connect_server(
        self,
        addr: str,
        server_name: str = "blindai-srv",
        policy: Optional[str] = None,
        certificate: Optional[str] = None,
        simulation: bool = False,
        untrusted_port: int = 50052,
        attested_port: int = 50051,
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

    def upload_model(
        self,
        model: str,
        tensor_inputs: Dict[str, TensorInfo],
        tensor_outputs: Optional[Dict[str, ModelDatumType]] = {"index_0": ModelDatumType.F32},
        sign: bool = False,
    ) -> UploadModelResponse:
        """Upload an inference model to the server.
        The provided model needs to be in the Onnx format.

        Args:
            model (str): Path to Onnx model file.
            tensor_inputs (Dict[str, TensorInfo]): A dictionary describing multiple inputs of the model.
            tensor_outputs (Dict[str, ModelDatumType], optional):A dictionary describing multiple inputs of the model. Defaults to {"index_0": ModelDatumType.F32}.
            sign (bool, optional): Get signed responses from the server or not. Defaults to False.

        Raises:
            ConnectionError: Will be raised if the client is not connected.
            FileNotFoundError: Will be raised if the model file is not found.
            SignatureError: Will be raised if the response signature is invalid.

        Returns:
            UploadModelResponse: The response object.
        """

        response = None
        if not self.is_connected():
            raise ConnectionError("Not connected to the server")

        try:
            with open(model, "rb") as f:
                data = f.read()

            inputs = []
            for k, v in tensor_inputs.items():
                inputs.append(Pair(index=k, info=v))

            outputs = []
            for k, v in tensor_outputs.items():
                tensor_info = TensorInfo(fact=[], datum_type=v)
                outputs.append(Pair(index=k, info=tensor_info))

            response = self._stub.SendModel(
                iter(
                    [
                        SendModelRequest(
                            length=len(data),
                            data=chunk,
                            sign=sign,
                            client_info=self.client_info,
                            model_name=os.path.basename(model),
                            tensor_inputs=inputs,
                            tensor_outputs=outputs
                        )
                        for chunk in create_byte_chunk(data)
                    ]
                )
            )

        except RpcError as rpc_error:
            raise ConnectionError(check_rpc_exception(rpc_error))

        # Response Verification
        # payload = Payload.FromString(response.payload).send_model_payload
        ret = UploadModelResponse()

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

    def run_model(self, data_list: List[List[Any]], input_indexes: List[str], output_index: str, sign: bool = False) -> RunModelResponse:
        """Send data to the server to make a secure inference.

        The data provided must be in a list, as the tensor will be rebuilt inside the server.

        Args:
            data_list (List[Any]): The input data. It must be an array of numbers of the same type dtype specified in `upload_model`.
            tensor_index (List[str]): The key of the tensor input in the `tensor_inputs` dictionary.
            sign (bool, optional): Get signed responses from the server or not. Defaults to False.

        Raises:
            ConnectionError: Will be raised if the client is not connected.
            SignatureError: Will be raised if the response signature is invalid

        Returns:
            RunModelResponse: The response object.
        """

        if not self.is_connected():
            raise ConnectionError("Not connected to the server")

        try:
            data_list = [item for sublist in data_list for item in sublist]
            serialized_bytes = cbor2_dumps(data_list)
            response = self._stub.RunModel(
                iter(
                    [
                        RunModelRequest(
                            client_info=self.client_info,
                            input=serialized_bytes_chunk,
                            sign=sign,
                            input_indexes=input_indexes,
                            output_index=output_index
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
                data_list,
                validate_quote=False,
                enclave_signing_key=self.enclave_signing_key,
                allow_simulation_mode=self.simulation_mode,
            )

        return ret

    def close_connection(self):
        """Close the connection between the client and the inference server."""
        if self.is_connected():
            self._close_channel()
            self._channel = None
            self._stub = None
            self.policy = None
            self.server_version = None
