from hashlib import sha256
import os
import pickle
from typing import Iterator
import unittest
from unittest.mock import MagicMock, Mock, patch
import blindai.client
import cbor2
from unittest.mock import *
from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional, Tuple, Union

import time  # so we can override time.time

from blindai.pb.securedexchange_pb2 import (
    SendModelRequest,
    SendModelReply,
    Payload,
    RunModelRequest,
    RunModelReply,
    PayloadHeader,
    RunModelPayload,
    TensorInfo,
)

from blindai.client import (
    ModelDatumType,
    RunModelResponse,
    UploadModelResponse,
)

import cryptography
from cryptography.hazmat.primitives import serialization

from .covidnet import get_input, model_path, get_model


mock_time = Mock()
mock_time.return_value = time.mktime(datetime(2022, 4, 15).timetuple())


class TensorInfoMatcher:
    facts: List[Tuple]
    datum_types: List[ModelDatumType]

    def __init__(self, tensor_info: List[TensorInfo]):
        self.facts = [x.fact for x in tensor_info]
        self.datum_types = [x.datum_type for x in tensor_info]

    def __eq__(self, other):
        return self.facts == other.facts and self.datum_types == other.datum_types


class TestRequest(unittest.TestCase):
    @patch("blindai.client.AttestationStub")
    @patch("blindai.client.secure_channel")
    @patch("time.time", mock_time)
    def test_connect(self, secure_channel: MagicMock, AttestationStub: MagicMock):
        res = UploadModelResponse()
        res.load_from_file(os.path.join(os.path.dirname(__file__), "exec_upload.proof"))

        attestation = res.attestation
        AttestationStub().GetSgxQuoteWithCollateral = Mock(return_value=attestation)
        blindai.client.connect(
            "localhost",
            policy=os.path.join(os.path.dirname(__file__), "policy.toml"),
            certificate=os.path.join(os.path.dirname(__file__), "host_server.pem"),
        )

    @patch("blindai.client.ExchangeStub")
    @patch("blindai.client.AttestationStub")
    @patch("blindai.client.secure_channel")
    @patch("time.time", mock_time)
    def test_upload_model(
        self,
        _secure_channel: MagicMock,
        AttestationStub: MagicMock,
        ExchangeStub: MagicMock,
    ):
        res = UploadModelResponse()
        res.load_from_file(os.path.join(os.path.dirname(__file__), "exec_upload.proof"))
        real_response = SendModelReply(
            payload=res.payload,
            signature=res.signature,
        )

        # connect

        attestation = res.attestation
        AttestationStub().GetSgxQuoteWithCollateral = Mock(return_value=attestation)
        client = blindai.client.connect(
            "localhost",
            policy=os.path.join(os.path.dirname(__file__), "policy.toml"),
            certificate=os.path.join(os.path.dirname(__file__), "host_server.pem"),
        )

        # send_model

        datum = ModelDatumType.F32
        datum_out = ModelDatumType.F32
        shape = (1, 480, 480, 3)
        tensor_inputs = [
            TensorInfo(fact=(1, 480, 480, 3), datum_type=ModelDatumType.F32)
        ]
        tensor_outputs = [ModelDatumType.F32]

        def send_model_util(sign):
            model_bytes = get_model()

            def send_model(req: Iterator[SendModelRequest]):
                arr = b""
                reql = list(req)
                for el in reql:
                    self.assertLessEqual(len(el.data), 32 * 1024)
                    arr += el.data
                    self.assertEqual(el.sign, sign)
                    self.assertEqual(
                        TensorInfoMatcher(el.tensor_inputs),
                        TensorInfoMatcher(tensor_inputs),
                    )
                    self.assertEqual(el.tensor_outputs, tensor_outputs)
                    self.assertEqual(el.length, len(model_bytes))

                self.assertEqual(arr, model_bytes)

                return SendModelReply(
                    payload=real_response.payload,
                    signature=real_response.signature if sign else None,
                )

            ExchangeStub().SendModel = Mock(side_effect=send_model)

            response = client.upload_model(
                model_path, shape=shape, dtype=datum, dtype_out=datum_out, sign=sign
            )

            if not sign:
                self.assertFalse(response.is_signed())
            else:
                self.assertTrue(response.is_signed())
                self.assertEqual(response.payload, real_response.payload)

                client.enclave_signing_key.verify(response.signature, response.payload)
                self.assertEqual(
                    client.attestation.SerializeToString(),
                    response.attestation.SerializeToString(),
                )
                # path = os.path.join(os.path.dirname(__file__), "exec_upload.proof")
                # response.save_to_file(path)

        send_model_util(sign=False)
        send_model_util(sign=True)

    @patch("blindai.client.ExchangeStub")
    @patch("blindai.client.AttestationStub")
    @patch("blindai.client.secure_channel")
    @patch("time.time", mock_time)
    def test_run_model(
        self,
        _secure_channel: MagicMock,
        AttestationStub: MagicMock,
        ExchangeStub: MagicMock,
    ):
        res = RunModelResponse()
        res.load_from_file(os.path.join(os.path.dirname(__file__), "exec_run.proof"))
        real_response = RunModelReply(
            payload=res.payload,
            signature=res.signature,
        )

        # connect

        attestation = res.attestation
        AttestationStub().GetSgxQuoteWithCollateral = Mock(return_value=attestation)
        client = blindai.client.connect(
            "localhost",
            policy=os.path.join(os.path.dirname(__file__), "policy.toml"),
            certificate=os.path.join(os.path.dirname(__file__), "host_server.pem"),
        )

        # run_model

        input = get_input()

        def run_model_util(sign):
            def run_model(req: Iterator[RunModelRequest]):
                arr = b""
                reql = list(req)
                for el in reql:
                    self.assertLessEqual(len(el.input), 32 * 1024)
                    arr += el.input
                    self.assertEqual(el.sign, sign)

                self.assertEqual(arr, cbor2.dumps([input]))

                return RunModelReply(
                    payload=real_response.payload,
                    signature=real_response.signature if sign else None,
                )

            ExchangeStub().RunModel = Mock(side_effect=run_model)

            response = client.run_model(res.model_id, input, sign=sign)

            if not sign:
                self.assertFalse(response.is_signed())
            else:
                self.assertTrue(response.is_signed())
                self.assertEqual(response.payload, real_response.payload)

                self.assertEqual(
                    response.output,
                    cbor2.loads(
                        Payload.FromString(
                            real_response.payload
                        ).run_model_payload.output
                    ),
                )

                client.enclave_signing_key.verify(response.signature, response.payload)
                self.assertEqual(
                    client.attestation.SerializeToString(),
                    response.attestation.SerializeToString(),
                )
                # path = os.path.join(os.path.dirname(__file__), "exec_run.proof")
                # response.save_to_file(path)

        run_model_util(sign=False)
        run_model_util(sign=True)

        # close server

        self.assertFalse(client.closed)
        client.close()
        self.assertTrue(client.closed)

    @patch("blindai.client.ExchangeStub")
    @patch("blindai.client.AttestationStub")
    @patch("blindai.client.secure_channel")
    @patch("ssl.get_server_certificate")
    @patch("time.time", mock_time)
    def test_run_model_simulation(
        self,
        get_server_certificate: MagicMock,
        secure_channel: MagicMock,
        AttestationStub: MagicMock,
        ExchangeStub: MagicMock,
    ):
        res = RunModelResponse()
        res.load_from_file(os.path.join(os.path.dirname(__file__), "exec_run.proof"))
        real_response = RunModelReply(
            payload=res.payload,
            signature=res.signature,
        )

        # connect

        cert = b"-----BEGIN CERTIFICATE-----\nMIICoTCCAkagAwIBAgIICPOHq4cyW9gwCgYIKoZIzj0EAwIwITEfMB0GA1UEAwwW\ncmNnZW4gc2VsZiBzaWduZWQgY2VydDAgFw03NTAxMDEwMDAwMDBaGA80MDk2MDEw\nMTAwMDAwMFowITEfMB0GA1UEAwwWcmNnZW4gc2VsZiBzaWduZWQgY2VydDBZMBMG\nByqGSM49AgEGCCqGSM49AwEHA0IABJcBq9016gGORpbhaaJyA9fhqVh2eypiefoA\ng/C/hn+VvTSkckm6EFZSuoV8lYQ4+zVTrPBhb1hB7uPQVIggnQSjggFkMIIBYDAW\nBgNVHREEDzANggtibGluZGFpLXNydjCCARkGBSsGAQMBBIIBDjCCAQoCggEBAMn1\n2jMlbFgPAFxtzKr93ZsUEfWN7dzrC698IyXFy71F9VZPxlSFTtPLX5huC9HPRtb4\ncMXDIoFhLGahDpjN4qUarczYbFGqALqrOS0R9vod28vwq/4Wh9pif0Bj3kkR/qGK\nlZbGpr8LXEYiM1U2d4r7HwQlj//KLXcvJXv75TR6Mo3IDZmA43mlQs6rdQCJEBoU\nmodYq506xsoXZ62/HhB4IM/yK/ZAfMG/FWgL9ZW8SZLRS0WYKq8jSeDYvJGWk7YT\nRdOK4qk+HzueP5/VTErUmFWOkoFgAqidSQqL4KzTGxzSXRIn3a+YQocdnKcFZspZ\nHynF6EZmh9D2dk5PxaMCAwEAATApBgUrBgEDAgQg88lQ7Z5k8IE41l9q+T5zDELZ\nENSSG3HAXXcBwlakpKAwCgYIKoZIzj0EAwIDSQAwRgIhAIpFE0AGf/gwi4dw2onq\nmhQSC3k266hjXhwl+kEUw8K9AiEAj40q1gMJUjLSOn76W/sOVskFene71pVMN/Gl\nF1X0vsg=\n-----END CERTIFICATE-----\n"
        get_server_certificate.return_value = cert.decode("ascii")
        response = Mock()
        response.enclave_tls_certificate = cryptography.x509.load_pem_x509_certificate(
            cert
        ).public_bytes(
            encoding=serialization.Encoding.DER,
        )
        AttestationStub().GetCertificate = Mock(return_value=response)

        client = blindai.client.connect("localhost", simulation=True)

        # run_model

        input = get_input()

        def run_model(req: Iterator[RunModelRequest]):
            arr = b""
            reql = list(req)
            for el in reql:
                self.assertLessEqual(len(el.input), 32 * 1024)
                arr += el.input
                self.assertEqual(el.sign, False)

            self.assertEqual(arr, cbor2.dumps([input]))

            return RunModelReply(
                payload=real_response.payload,
            )

        ExchangeStub().RunModel = Mock(side_effect=run_model)

        response = client.run_model(res.model_id, input)

        self.assertFalse(response.is_signed())

        # close server

        self.assertFalse(client.closed)
        client.close()
        self.assertTrue(client.closed)
