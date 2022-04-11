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

from blindai.client import BlindAiClient, ModelDatumType, RunModelResponse

from covidnet import get_input

path = os.path.join(os.path.dirname(__file__), "exec_run.proof")
tmp_path = os.path.join(os.path.dirname(__file__), "tmp_exec_run.proof")


class TestProof(unittest.TestCase):
    def test_parse_run(self):
        response = RunModelResponse()
        response.load_from_file(path)

        self.assertTrue(response.is_signed())

        response2 = RunModelResponse()
        with open(path, "rb") as file:
            response2.load_from_bytes(file.read())

        self.assertEqual(response.payload, response2.payload)
        self.assertEqual(response.signature, response2.signature)
        self.assertEqual(response.attestation, response2.attestation)
        self.assertEqual(response.output, response2.output)

        response3 = RunModelResponse()
        response3.load_from_bytes(response.as_bytes())

        self.assertEqual(response.payload, response3.payload)
        self.assertEqual(response.signature, response3.signature)
        self.assertEqual(response.attestation, response3.attestation)
        self.assertEqual(response.output, response3.output)

        response3.save_to_file(tmp_path)
        response4 = RunModelResponse()
        response4.load_from_file(tmp_path)

        self.assertEqual(response.payload, response4.payload)
        self.assertEqual(response.signature, response4.signature)
        self.assertEqual(response.attestation, response4.attestation)
        self.assertEqual(response.output, response4.output)


    def test_validate_run(self):
        response = RunModelResponse()
        response.load_from_file(path)

        response.validate(
            get_input(),
            policy_file=os.path.join(os.path.dirname(__file__), "policy.toml"),
        )
