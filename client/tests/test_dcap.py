from datetime import datetime
import unittest
from unittest.mock import MagicMock, patch
from blindai.client import UploadModelResponse
from bitstring import Bits
from copy import deepcopy
from _quote_verification import status as QuoteStatus


from blindai.dcap_attestation import (
    verify_claims,
    Policy,
    DcapClaims,
    verify_dcap_attestation,
)
from blindai.utils.errors import AttestationError
import os

policy = Policy(
    mr_enclave="83efab03b904f491c237e0469ce71ab155d40f9512d37991a3bcb6da740c5bee",
    misc_select=0x0.to_bytes(4, byteorder="little"),
    misc_mask=0xFFFFFFFF.to_bytes(4, byteorder="little"),
    attributes_flags=0x4.to_bytes(8, byteorder="little"),
    attributes_xfrm=0x3.to_bytes(8, byteorder="little"),
    attributes_mask_flags=0xFFFFFFFFFFFFFFFD.to_bytes(8, byteorder="little"),
    attributes_mask_xfrm=0xFFFFFFFFFFFFFF1B.to_bytes(8, byteorder="little"),
    allow_debug=False,
)


class TestDCAP(unittest.TestCase):
    def test_load_policy(self):
        p2 = Policy.from_file(os.path.join(os.path.dirname(__file__), "./policy.toml"))

        self.assertEqual(policy.mr_enclave, p2.mr_enclave)
        self.assertEqual(policy.misc_select, p2.misc_select)
        self.assertEqual(policy.misc_mask, p2.misc_mask)
        self.assertEqual(policy.attributes_flags, p2.attributes_flags)
        self.assertEqual(policy.attributes_xfrm, p2.attributes_xfrm)
        self.assertEqual(policy.attributes_mask_flags, p2.attributes_mask_flags)
        self.assertEqual(policy.attributes_mask_xfrm, p2.attributes_mask_xfrm)
        self.assertEqual(policy.allow_debug, p2.allow_debug)

        self.assertEqual(policy, p2)

    @patch("time.time")
    def test_verify_claims(self, time: MagicMock):
        time.return_value = time.mktime(datetime(2022, 4, 15).timetuple())

        upload_response_sample = UploadModelResponse()
        upload_response_sample.load_from_file(
            os.path.join(os.path.dirname(__file__), "exec_upload.proof")
        )

        attestation = upload_response_sample.attestation

        # should not fail
        claims = verify_dcap_attestation(
            attestation.quote, attestation.collateral, attestation.enclave_held_data
        )
        self.assertFalse(claims.sgx_is_debuggable)

        verify_claims(claims, policy)

        policy_bis = deepcopy(policy)
        claims_bis = deepcopy(claims)
        policy_bis.mr_enclave = (
            policy_bis.mr_enclave[:5] + "1" + policy_bis.mr_enclave[6:]
        )
        with self.assertRaises(AttestationError):
            verify_claims(
                claims_bis,
                policy_bis,
            )

        policy_bis = deepcopy(policy)
        claims_bis = deepcopy(claims)
        claims_bis.sgx_is_debuggable = True
        policy_bis.allow_debug = False
        with self.assertRaises(AttestationError):
            verify_claims(
                claims_bis,
                policy_bis,
            )

        policy_bis = deepcopy(policy)
        claims_bis = deepcopy(claims)
        SGX_FLAGS_INITTED = Bits("0x0100000000000000")
        claims_bis.sgx_attributes = (
            (Bits(claims_bis.sgx_attributes[:8]) & ~SGX_FLAGS_INITTED)
            + Bits(claims_bis.sgx_attributes[8:])
        ).tobytes()
        with self.assertRaises(AttestationError):
            verify_claims(
                claims_bis,
                policy_bis,
            )

        policy_bis = deepcopy(policy)
        claims_bis = deepcopy(claims)
        claims_bis.sgx_attributes = (
            claims_bis.sgx_attributes[:9] + b"5" + claims_bis.sgx_attributes[10:]
        )
        with self.assertRaises(AttestationError):
            verify_claims(
                claims_bis,
                policy_bis,
            )

    @patch("_quote_verification.Verification")
    def test_verify_dcap_attestation(self, Verification: MagicMock):
        Verification.verify = MagicMock()

        upload_response_sample = UploadModelResponse()
        upload_response_sample.load_from_file(
            os.path.join(os.path.dirname(__file__), "exec_upload.proof")
        )

        attestation = upload_response_sample.attestation

        ret = MagicMock()
        ret.ok = True
        ret.pckCertificateStatus = QuoteStatus.STATUS_OK
        ret.tcbInfoStatus = QuoteStatus.STATUS_OK
        ret.qeIdentityStatus = QuoteStatus.STATUS_OK
        ret.qveIdentityStatus = QuoteStatus.STATUS_OK
        ret.quoteStatus = QuoteStatus.STATUS_OK
        ret.reportData = b"junk data"
        ret.mrEnclave = policy.mr_enclave
        ret.attributes = 0xFFFFFFFFFFFFFF1B.to_bytes(8, byteorder="little")
        ret.miscSelect = 0xFFFFFFFFFFFFFF1B.to_bytes(8, byteorder="little")
        Verification.verify.return_value = ret
        with self.assertRaises(AttestationError):
            verify_dcap_attestation(
                attestation.quote, attestation.collateral, attestation.enclave_held_data
            )
