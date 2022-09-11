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

import ctypes
import hashlib
import pkgutil
import struct
import time
from typing_extensions import Self
from dataclasses import dataclass
from blindai.pb.untrusted_pb2 import SgxCollateral
import toml
from bitstring import Bits
import _quote_verification
from _quote_verification import status

from blindai.utils.utils import encode_certificate
from blindai.utils.errors import (
    QuoteValidationError,
    EnclaveHeldDataError,
    AttestationError,
    DebugNotAllowedError,
    IdentityError,
    NotAnEnclaveError,
)


@dataclass
class DcapClaims:
    sgx_ehd: bytes
    sgx_is_debuggable: bool
    sgx_mrenclave: str
    sgx_misc_select: bytes
    sgx_attributes: bytes
    raw_quote: bytes

    def get_server_cert(self):
        """Get the server certificate from the Attestation claims.

        Returns:
            bytes: The PEM-encoded server certificate as a byte string
        """
        return encode_certificate(self.sgx_ehd)


def verify_dcap_attestation(
    quote: bytes, attestation_collateral: SgxCollateral, enclave_held_data: bytes
) -> DcapClaims:
    """Verifies if the enclave evidence is valid.
    * Validates if the quote is trustworthy (issued by an approved Intel CPU) with the
        attestation collateral using SGX Quote Verification Library.
    * Validates if the SHA256 hash of Enclave Held Data (EHD) matches the first 32 bytes
        of reportData field in the enclave quote. After this check
        we can be sure that the EHD bytes are endorsed by the enclave.

    Args:
        quote (bytes): SGX quote
        attestation_collateral (SgxCollateral): SGX collateral needed to assess the validity of the quote
            (collateral is signed by Intel)
        enclave_held_data (bytes): Enclave held data

    Raises:
        QuoteValidationError: The quote could not be validated.
        EnclaveHeldDataError: The enclave held data expected does not match the one in the quote. The expected enclave held data in BlindAI is the untrusted certificate to avoid man-in-the-middle attacks.
        NotAnEnclaveError: The enclave claims are not validated by the hardware provider, meaning that the claims cannot be verified using the hardware root of trust.

    Returns:
        DcapClaims: The claims.
    """

    # TODO: Handle the case where the retuned quote status is STATUS_TCB_SW_HARDENING_NEEDED
    # We must do more cautious checks in this case in order to determine whether or not to accept the quote

    t = _quote_verification.Verification()
    t.trustedRootCACertificate = pkgutil.get_data(__name__, "tls/trustedRootCaCert.pem")
    t.pckCertificate = attestation_collateral.pck_certificate
    t.pckSigningChain = attestation_collateral.pck_signing_chain
    t.rootCaCrl = attestation_collateral.root_ca_crl
    t.intermediateCaCrl = attestation_collateral.pck_crl
    t.tcbInfo = attestation_collateral.tcb_info
    t.tcbSigningChain = attestation_collateral.tcb_info_issuer_chain
    t.quote = quote
    t.qeIdentity = attestation_collateral.qe_identity
    t.qveIdentity = ""
    t.expirationDate = int(time.time())

    ret = t.verify()

    if ret.pckCertificateStatus != status.STATUS_OK:
        raise NotAnEnclaveError(
            "Wrong PCK Certificate Status {}", ret.pckCertificateStatus
        )

    if ret.tcbInfoStatus != status.STATUS_OK:
        raise QuoteValidationError("Wrong TCB Info Status {}", ret.tcbInfoStatus)

    if ret.qeIdentityStatus != status.STATUS_OK:
        raise QuoteValidationError("Wrong QE Identity Status {}", ret.qeIdentityStatus)

    if ret.quoteStatus not in [status.STATUS_OK, status.STATUS_TCB_SW_HARDENING_NEEDED]:
        raise QuoteValidationError("Wrong Quote Status {}", ret.quoteStatus)

    if hashlib.sha256(enclave_held_data).digest() != bytes(ret.reportData)[:32]:
        raise EnclaveHeldDataError(
            "Enclave Held Data hash doesn't match with the report data from the quote",
            hashlib.sha256(enclave_held_data).hexdigest(),
            bytes(ret.reportData)[:32].hex(),
        )

    claims = DcapClaims(
        raw_quote=quote,
        sgx_attributes=b"".join(struct.pack("B", x) for x in ret.attributes),
        sgx_misc_select=bytes(ctypes.c_uint32(ret.miscSelect)),
        sgx_mrenclave=bytes(ret.mrEnclave).hex(),
        sgx_ehd=enclave_held_data,
        # https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3d-part-4-manual.pdf
        # The ATTRIBUTES data structure is comprised of bit-granular fields that are used in the SECS
        # DEBUG flag is at bit 1
        # If 1, the enclave permit debugger to read and write enclave data using EDBGRD and EDBGWR
        # Using a mask we test if this bit is set
        sgx_is_debuggable=bool(ret.attributes[0] & (1 << 1)),
    )

    return claims


@dataclass
class Policy:
    mr_enclave: str
    misc_select: bytes
    misc_mask: bytes
    attributes_flags: bytes
    attributes_xfrm: bytes
    attributes_mask_flags: bytes
    attributes_mask_xfrm: bytes
    allow_debug: bool

    def from_str(s: str) -> Self:
        """Load a policy from a file.

        Args:
            s (str): The content of the policy.

        Returns:
            Policy: The policy.
        """
        return Policy.from_dict(toml.loads(s))

    def from_file(path: str) -> Self:
        """Load a policy from a file.

        Args:
            path (str): The path of the file.

        Returns:
            Policy: The policy.
        """
        return Policy.from_dict(toml.load(path))

    def from_dict(obj: dict) -> Self:
        """Load a policy from a dict.

        Args:
            obj (dict): The dict.

        Returns:
            Policy: The policy.
        """
        return Policy(
            mr_enclave=obj["mr_enclave"],
            misc_mask=int(obj["misc_mask_hex"], 16).to_bytes(4, byteorder="little"),
            misc_select=int(obj["misc_select_hex"], 16).to_bytes(4, byteorder="little"),
            attributes_flags=int(obj["attributes_flags_hex"], 16).to_bytes(
                8, byteorder="little"
            ),
            attributes_xfrm=int(obj["attributes_xfrm_hex"], 16).to_bytes(
                8, byteorder="little"
            ),
            attributes_mask_flags=int(obj["attributes_mask_flags_hex"], 16).to_bytes(
                8, byteorder="little"
            ),
            attributes_mask_xfrm=int(obj["attributes_mask_xfrm_hex"], 16).to_bytes(
                8, byteorder="little"
            ),
            allow_debug=obj["allow_debug"],
        )


def verify_claims(claims: DcapClaims, policy: Policy):
    """Verify enclave claims against a policy.

    Args:
        claims (DcapClaims): The claims.
        policy (Policy): The enclave policy.

    Raises:
        AttestationError: Attestation does not match policy.
        IdentityError: The enclave code signature hash does not match the signature hash provided in the policy.
        DebugNotAllowedError: The enclave is in debug mode, but the policy does not allow it.
    """

    if claims.sgx_mrenclave != policy.mr_enclave:
        raise IdentityError(
            "Code signature mismatch (MRENCLAVE)",
            claims,
            policy,
            policy.mr_enclave,
            claims.sgx_mrenclave,
        )

    if claims.sgx_is_debuggable and not policy.allow_debug:
        raise DebugNotAllowedError(
            "Enclave is in debug mode but the policy doesn't allow debug",
            claims,
            policy,
        )

    # If this flag is set, then the enclave is initialized
    SGX_FLAGS_INITTED = Bits("0x0100000000000000")

    if (
        Bits(claims.sgx_attributes[:8]) & Bits(policy.attributes_mask_flags)
        != Bits(policy.attributes_flags) | SGX_FLAGS_INITTED
    ):
        raise AttestationError(
            "SGX attributes flags bytes do not conform to the policy", claims, policy
        )

    if Bits(claims.sgx_attributes[8:]) & Bits(policy.attributes_mask_xfrm) != Bits(
        policy.attributes_xfrm
    ):
        raise AttestationError(
            "SGX XFRM flags bytes do not conform to the policy", claims, policy
        )

    if Bits(claims.sgx_misc_select) & Bits(policy.misc_mask) != Bits(
        policy.misc_select
    ):
        raise AttestationError(
            "SGX MISC SELECT bytes do not conform to the policy", claims, policy
        )
