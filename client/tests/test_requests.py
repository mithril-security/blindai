from hashlib import sha256
import os
import pickle
from typing import Iterator
import unittest
from unittest.mock import MagicMock, Mock, patch
import blindai.client
import onnxruntime
import cv2
import numpy as np
import cbor2
from google.protobuf.timestamp_pb2 import Timestamp

from blindai.securedexchange_pb2 import (
    SendModelRequest,
    SendModelReply,
    SendModelPayload,
    Payload,
    PayloadHeader,
    RunModelRequest,
    RunModelPayload,
    RunModelReply,
)

from blindai.client import BlindAiClient, ModelDatumType

from covidnet import get_input


class TestProof(unittest.TestCase):
    @patch("blindai.client.AttestationStub")
    @patch("blindai.client.secure_channel")
    def test_connect(self, secure_channel: MagicMock, AttestationStub: MagicMock):

        client = BlindAiClient()
        with open(
            os.path.join(os.path.dirname(__file__), "quote_collateral.dat"), "rb"
        ) as file:
            attestation = pickle.load(file)
        AttestationStub().GetSgxQuoteWithCollateral = Mock(return_value=attestation)
        client.connect_server(
            "localhost",
            policy=os.path.join(os.path.dirname(__file__), "policy.toml"),
            certificate=os.path.join(os.path.dirname(__file__), "host_server.pem"),
        )

    @patch("blindai.client.ExchangeStub")
    @patch("blindai.client.AttestationStub")
    @patch("blindai.client.secure_channel")
    def test_upload_model(
        self,
        secure_channel: MagicMock,
        AttestationStub: MagicMock,
        ExchangeStub: MagicMock,
    ):
        # connect

        client = blindai.client.BlindAiClient()
        with open(
            os.path.join(os.path.dirname(__file__), "quote_collateral.dat"), "rb"
        ) as file:
            attestation = pickle.load(file)
        AttestationStub().GetSgxQuoteWithCollateral = Mock(return_value=attestation)
        client.connect_server(
            "localhost",
            policy=os.path.join(os.path.dirname(__file__), "policy.toml"),
            certificate=os.path.join(os.path.dirname(__file__), "host_server.pem"),
        )

        # send_model

        model_path = os.path.join(
            os.path.dirname(__file__), "../../tests/assets/COVID-Net-CXR-2.onnx"
        )
        datum = ModelDatumType.F32
        shape = (1, 480, 480, 3)
        with open(
            os.path.join(os.path.dirname(__file__), "upload_model.dat"), "rb"
        ) as file:
            real_response = pickle.load(file)

        def send_model_util(sign):
            with open(model_path, "rb") as file:
                model_bytes = file.read()

            def send_model(req: Iterator[SendModelRequest]):
                arr = b""
                reql = list(req)
                for el in reql:
                    self.assertLessEqual(len(el.data), 32 * 1024)
                    arr += el.data
                    self.assertEqual(el.datum, datum)
                    self.assertEqual(el.sign, sign)
                    self.assertEqual(el.input_fact, list(shape))
                    self.assertEqual(el.length, len(model_bytes))

                self.assertEqual(arr, model_bytes)

                return SendModelReply(
                    payload=real_response.payload,
                    signature=real_response.signature if sign else None,
                )

            ExchangeStub().SendModel = Mock(side_effect=send_model)

            response = client.upload_model(model_path, shape, datum, sign)

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
                # path = os.path.join(os.path.dirname(__file__), "exec_send.proof")
                # response.save_to_file(path)

        send_model_util(sign=False)
        send_model_util(sign=True)

    @patch("blindai.client.ExchangeStub")
    @patch("blindai.client.AttestationStub")
    @patch("blindai.client.secure_channel")
    def test_run_model(
        self,
        secure_channel: MagicMock,
        AttestationStub: MagicMock,
        ExchangeStub: MagicMock,
    ):
        # connect

        client = blindai.client.BlindAiClient()
        with open(
            os.path.join(os.path.dirname(__file__), "quote_collateral.dat"), "rb"
        ) as file:
            attestation = pickle.load(file)
        AttestationStub().GetSgxQuoteWithCollateral = Mock(return_value=attestation)
        client.connect_server(
            "localhost",
            policy=os.path.join(os.path.dirname(__file__), "policy.toml"),
            certificate=os.path.join(os.path.dirname(__file__), "host_server.pem"),
        )

        # run_model

        input = get_input()
        with open(
            os.path.join(os.path.dirname(__file__), "run_model.dat"), "rb"
        ) as file:
            real_response = pickle.load(file)

        def run_model_util(sign):
            def run_model(req: Iterator[RunModelRequest]):
                arr = b""
                reql = list(req)
                for el in reql:
                    self.assertLessEqual(len(el.input), 32 * 1024)
                    arr += el.input
                    self.assertEqual(el.sign, sign)

                self.assertEqual(arr, cbor2.dumps(input))

                return RunModelReply(
                    payload=real_response.payload,
                    signature=real_response.signature if sign else None,
                )

            ExchangeStub().RunModel = Mock(side_effect=run_model)

            response = client.run_model(input, sign=sign)

            if not sign:
                self.assertFalse(response.is_signed())
            else:
                self.assertTrue(response.is_signed())
                self.assertEqual(response.payload, real_response.payload)

                self.assertEqual(
                    response.output,
                    Payload.FromString(real_response.payload).run_model_payload.output,
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


if __name__ == "__main__":
    unittest.main()
