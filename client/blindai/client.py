import os
import logging
import cryptography
import untrusted_pb2

from utils.utils import *
from cryptography.hazmat.primitives.serialization import Encoding
from securedexchange_pb2 import SimpleReply, Model, Data
from securedexchange_pb2_grpc import ExchangeStub
from untrusted_pb2_grpc import AttestationStub
from grpc import ssl_channel_credentials, secure_channel, RpcError
from dcap_attestation import (
    verify_claims,
    verify_dcap_attestation,
    get_server_cert,
    load_policy,
)

PORTS = {"untrusted_enclave": "50052", "attested_enclave": "50051"}


class BlindAiClient:
    def __init__(self, debug_mode=False):

        self.channel = None
        self.policy = None
        self.stub = None

        if debug_mode == True:
            os.environ["GRPC_TRACE"] = "transport_security,tsi"
            os.environ["GRPC_VERBOSITY"] = "DEBUG"
        self.SIMULATION_MODE = False

    def _is_connected(self):
        return self.channel is not None

    def _close_channel(self):
        if self._is_connected():
            self.channel.close()

    def connect_server(
        self,
        addr: str,
        server_name="blindai-srv",
        policy=None,
        certificate=None,
        simulation=False,
    ):

        self.SIMULATION_MODE = simulation

        self.policy = load_policy(policy)
        if self.policy is None and self.SIMULATION_MODE is False:
            logging.error("Policy not found or not valid")
            return False

        try:
            with open(certificate, "rb") as f:
                untrusted_server_creds = ssl_channel_credentials(
                    root_certificates=f.read()
                )
        except:
            logging.error("Certificate not found or not valid")
            return False

        addr = strip_https(addr)

        untrusted_client_to_enclave = addr + ":" + PORTS["untrusted_enclave"]
        attested_client_to_enclave = addr + ":" + PORTS["attested_enclave"]
        connection_options = (("grpc.ssl_target_name_override", server_name),)

        try:
            with secure_channel(
                untrusted_client_to_enclave,
                untrusted_server_creds,
                options=connection_options,
            ) as channel:
                stub = AttestationStub(channel)
                if self.SIMULATION_MODE:
                    logging.warning(
                        "Attestation process is bypassed : running without requesting and checking attestation"
                    )
                    response = stub.GetCertificate(
                        untrusted_pb2.GetCertificateRequest()
                    )
                    server_cert = cryptography.x509.load_der_x509_certificate(
                        response.enclave_tls_certificate
                    ).public_bytes(Encoding.PEM)
                else:
                    response = stub.GetSgxQuoteWithCollateral(
                        untrusted_pb2.GetSgxQuoteWithCollateralRequest()
                    )
                    claims = verify_dcap_attestation(
                        response.quote, response.collateral, response.enclave_held_data
                    )
                    verify_claims(claims, self.policy)
                    server_cert = get_server_cert(claims)

                    logging.info(f"Quote verification passed")
                    logging.info(
                        f"Certificate from attestation process\n {server_cert.decode('ascii')}"
                    )
                    logging.info(f"MREnclave\n" + claims["sgx-mrenclave"])

            server_creds = ssl_channel_credentials(root_certificates=server_cert)

            # Attested channel to the enclave
            channel = secure_channel(
                attested_client_to_enclave, server_creds, options=connection_options
            )
            self.stub = ExchangeStub(channel)
            self.channel = channel
            logging.info("Successfuly connected to the server")

        except RpcError as rpc_error:
            check_rpc_exception(rpc_error)

        return True

    def upload_model(self, model=None, shape=None):
        """Upload an inference model to the server"""

        response = SimpleReply()
        response.ok = False
        if not self._is_connected():
            response.msg = "Not connected to server"
            return response
        try:
            with open(model, "rb") as f:
                data = f.read()
            input_fact = list(shape)
            response = self.stub.SendModel(
                iter(
                    [
                        Model(length=len(data), input_fact=input_fact, data=chunk)
                        for chunk in create_byte_chunk(data)
                    ]
                )
            )
        except RpcError as rpc_error:
            check_rpc_exception(rpc_error)
            response.msg = "GRPC error"
        except FileNotFoundError:
            response.msg = "Model not found"

        return response

    def send_data(self, data_list):
        """Send data to the server to make a secure inference"""
        response = SimpleReply()
        response.ok = False
        if not self._is_connected():
            response.msg = "Not connected to server"
            return response
        try:
            response = self.stub.SendData(
                iter(
                    [
                        Data(input=data_list_chunk)
                        for data_list_chunk in create_float_chunk(data_list)
                    ]
                )
            )
            return response
        except RpcError as rpc_error:
            check_rpc_exception(rpc_error)
            response = SimpleReply()
            response.ok = False
            response.msg = "GRPC error"
        return response

    def close_connection(self):
        if self._is_connected():
            self._close_channel()
            self.channel = None
            self.stub = None
            self.policy = None
