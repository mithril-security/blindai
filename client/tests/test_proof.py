from copy import deepcopy
from hashlib import sha256
import os
import unittest
from google.protobuf.timestamp_pb2 import Timestamp

from blindai.pb.securedexchange_pb2 import (
    Payload,
)

from blindai.client import (
    RunModelResponse,
    UploadModelResponse,
)
from blindai.dcap_attestation import Policy
from blindai.utils.errors import SignatureError, AttestationError
from unittest.mock import *
from datetime import datetime, timedelta
import time  # so we can override time.time
import cbor2

from .covidnet import get_input, get_model

mock_time = Mock()
mock_time.return_value = time.mktime(datetime(2022, 4, 15).timetuple())


exec_run = os.path.join(os.path.dirname(__file__), "exec_run.proof")
exec_upload = os.path.join(os.path.dirname(__file__), "exec_upload.proof")
tmp_path = os.path.join(os.path.dirname(__file__), "tmp_exec.proof")
policy_file = os.path.join(os.path.dirname(__file__), "policy.toml")


class TestProof(unittest.TestCase):
    @patch("time.time", mock_time)
    def test_parse_run(self):
        response = RunModelResponse()
        response.load_from_file(exec_run)

        self.assertTrue(response.is_signed())

        response2 = RunModelResponse()
        with open(exec_run, "rb") as file:
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

    @patch("time.time", mock_time)
    def test_parse_upload(self):
        response = UploadModelResponse()
        response.load_from_file(exec_upload)

        self.assertTrue(response.is_signed())

        response2 = UploadModelResponse()
        with open(exec_upload, "rb") as file:
            response2.load_from_bytes(file.read())

        self.assertEqual(response.payload, response2.payload)
        self.assertEqual(response.signature, response2.signature)
        self.assertEqual(response.attestation, response2.attestation)

        response3 = UploadModelResponse()
        response3.load_from_bytes(response.as_bytes())

        self.assertEqual(response.payload, response3.payload)
        self.assertEqual(response.signature, response3.signature)
        self.assertEqual(response.attestation, response3.attestation)

        response3.save_to_file(tmp_path)
        response4 = UploadModelResponse()
        response4.load_from_file(tmp_path)

        self.assertEqual(response.payload, response4.payload)
        self.assertEqual(response.signature, response4.signature)
        self.assertEqual(response.attestation, response4.attestation)

    @patch("time.time", mock_time)
    def test_validate_run(self):
        response = RunModelResponse()
        response.load_from_file(exec_run)
        policy = Policy.from_file(policy_file)

        response.validate(
            response.model_id,
            get_input(),
            policy=policy,
        )

        # Not signed

        response2 = deepcopy(response)
        response2.signature = None
        response2.attestation = None
        with self.assertRaises(SignatureError):
            response2.validate(
                response.model_id,
                get_input(),
                policy=policy,
            )

        # Quote validation

        response2 = deepcopy(response)
        response2.attestation.quote += b"a"
        with self.assertRaises(AttestationError):
            response2.validate(
                response.model_id,
                get_input(),
                policy=policy,
            )

        response2 = deepcopy(response)
        response2.attestation.enclave_held_data += b"a"
        with self.assertRaises(AttestationError):
            response2.validate(
                response.model_id,
                get_input(),
                policy=policy,
            )

        # Payload validation

        response2 = deepcopy(response)
        payload = Payload.FromString(response2.payload)
        payload.run_model_payload.output = cbor2.dumps([1, 2, 3])
        response2.payload = payload.SerializeToString()
        with self.assertRaises(SignatureError):
            response2.validate(
                response.model_id,
                get_input(),
                policy=policy,
            )

        # Input validation

        response2 = deepcopy(response)
        data = deepcopy(get_input())
        data[4] += 1
        with self.assertRaises(SignatureError):
            response2.validate(
                response.model_id,
                data,
                policy=policy,
            )

        # Using file

        response.validate(
            response.model_id,
            get_input(),
            policy_file=policy_file,
        )

    @patch("time.time", mock_time)
    def test_validate_upload(self):
        response = UploadModelResponse()
        response.load_from_file(exec_upload)

        policy = Policy.from_file(policy_file)
        model_hash = sha256(get_model()).digest()

        response.validate(
            model_hash,
            policy=policy,
        )

        # Not signed

        response2 = deepcopy(response)
        response2.signature = None
        response2.attestation = None
        with self.assertRaises(SignatureError):
            response2.validate(
                model_hash,
                policy=policy,
            )

        # Quote validation

        response2 = deepcopy(response)
        response2.attestation.quote += b"a"
        with self.assertRaises(AttestationError):
            response2.validate(
                model_hash,
                policy=policy,
            )

        response2 = deepcopy(response)
        response2.attestation.enclave_held_data += b"a"
        with self.assertRaises(AttestationError):
            response2.validate(
                model_hash,
                policy=policy,
            )

        # Payload validation

        response2 = deepcopy(response)
        payload = Payload.FromString(response2.payload)
        payload.send_model_payload.model_hash = (
            b"1" + payload.send_model_payload.model_hash[1:]
        )
        response2.payload = payload.SerializeToString()
        with self.assertRaises(SignatureError):
            response2.validate(
                model_hash,
                policy=policy,
            )

        # Input validation

        response2 = deepcopy(response)
        new_hash = model_hash[:5] + b"1" + model_hash[6:]
        with self.assertRaises(SignatureError):
            response2.validate(
                new_hash,
                policy=policy,
            )

        # Using file

        response.validate(
            model_hash,
            policy_file=policy_file,
        )
