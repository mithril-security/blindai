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

from datetime import datetime
import enum
import hashlib
import importlib
import os
from typing import Optional
from typing_extensions import Self
from dataclasses import dataclass
import sgx_dcap_quote_verify
from sgx_dcap_quote_verify import VerificationStatus
import toml
from pathlib import Path


class AttestationError(Exception):
    """This exception is raised when the attestation is invalid (enclave
    settings mismatching, debug mode unallowed...).

    Used as master exception for all other sub exceptions on the attestation
    validation
    """

    pass


class QuoteValidationError(Exception):
    """This exception is raised when the returned quote is invalid (TCB
    outdated, not signed by the hardware provider...).

    Used as master exception for all other sub exceptions on the quote
    validation
    """

    pass


class EnclaveHeldDataError(QuoteValidationError):
    """This exception is raised when the enclave held data expected does not
    match the one in the quote.

    The expected enclave held data in BlindAI is an SHA-256 hash of the enclave certificate to avoid man-in-the-middle attacks
    Args:
        expected_hash (str): Enclave held data hash expected
        got_hash (str): Enclave held data hash obtained from the quote's report
    """

    def __init__(self, expected: bytes, got: bytes):
        self.expected_hash = expected
        self.measured_hash = got
        super().__init__(
            f"Hash of enclave held data doesn't match with the report data from the quote. Expected {expected.hex()}, got {got.hex()} instead."
        )

    pass


class IdentityError(QuoteValidationError):
    """This exception is raised when the enclave code digest (MRENCLAVE is SGX terminology) does not match the digest provided in the manifest
    Args:
        expected_hash (str): hash from manifest
        got_hash (str): hash obtained from the quote's report
    """

    def __init__(self, expected: bytes, got: bytes):
        self.expected_hash = expected
        self.got_hash = got
        super().__init__(
            f"Error invalid MRENCLAVE. Expected {expected.hex()}, got {got.hex()} instead."
        )


def validate_attestation(
    quote: bytes,
    collateral,
    enclave_held_data: bytes,
    manifest_path: Optional[Path] = None,
):
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
        -
    """

    # TODO: Handle the case where the retuned quote status is STATUS_TCB_SW_HARDENING_NEEDED
    # We must do more cautious checks in this case in order to determine whether or not to accept the quote

    trusted_root_ca_certificate = importlib.resources.read_text(  # type: ignore
        __package__, "Intel_SGX_Provisioning_Certification_RootCA.pem"
    )

    attestation_result = sgx_dcap_quote_verify.verify(
        trusted_root_ca_certificate=trusted_root_ca_certificate,
        pck_certificate=collateral["pck_certificate"],
        pck_signing_chain=collateral["pck_signing_chain"],
        root_ca_crl=collateral["root_ca_crl"],
        intermediate_ca_crl=collateral["pck_crl"],
        tcb_info=collateral["tcb_info"],
        tcb_signing_chain=collateral["tcb_info_issuer_chain"],
        quote=quote,
        qe_identity=collateral["qe_identity"],
        expiration_date=datetime.now(),
    )
    if attestation_result.pck_certificate_status != VerificationStatus.STATUS_OK:
        raise QuoteValidationError(
            f"Invalid PCK certificate status {attestation_result.pck_certificate_status.name}"
        )

    if attestation_result.tcb_info_status != VerificationStatus.STATUS_OK:
        raise QuoteValidationError(
            f"Invalid TCB info Status {attestation_result.tcb_info_status.name}"
        )

    if attestation_result.qe_identity_status != VerificationStatus.STATUS_OK:
        raise QuoteValidationError(
            f"Invalid QE identity status {attestation_result.qe_identity_status.name}"
        )

    if attestation_result.quote_status not in [
        VerificationStatus.STATUS_OK,
        VerificationStatus.STATUS_TCB_SW_HARDENING_NEEDED,
    ]:
        raise QuoteValidationError(
            f"Invalid quote status {attestation_result.quote_status.name}",
        )

    assert attestation_result.enclave_report is not None
    if (
        hashlib.sha256(enclave_held_data).digest()
        != attestation_result.enclave_report.report_data[:32]
    ):
        raise EnclaveHeldDataError(
            expected=hashlib.sha256(enclave_held_data).digest(),
            got=attestation_result.enclave_report.report_data[:32],
        )

    if manifest_path is None:
        manifest = EnclaveManifest.from_str(
            importlib.resources.read_text(__package__, "manifest.toml")  # type: ignore
        )
    else:
        if not isinstance(manifest_path, Path):
            raise ValueError("manifest_path should be a pathlib.Path")
        manifest = EnclaveManifest.from_str(manifest_path.read_text())

    if attestation_result.enclave_report.mr_enclave != manifest.mr_enclave:
        raise IdentityError(
            expected=manifest.mr_enclave,
            got=attestation_result.enclave_report.mr_enclave,
        )

    # Attributes data structures is made of two fields : two 64-bits bitvectors
    enclave_attributes_flags = int.from_bytes(
        attestation_result.enclave_report.attributes[0:8], byteorder="little"
    )
    enclave_attributes_xfrm = int.from_bytes(
        attestation_result.enclave_report.attributes[8:16], byteorder="little"
    )

    if (
        SgxAttributesFlags.DEBUG in SgxAttributesFlags(enclave_attributes_flags)
        and not manifest.allow_debug
    ):
        raise AttestationError(
            "Enclave is running in debug mode but the manifest forbids debug mode",
        )

    if (
        enclave_attributes_flags & manifest.attributes_mask_flags
        != manifest.attributes_flags | SgxAttributesFlags.INIT
    ):
        raise AttestationError(
            "SGX_ATTRIBUTES.FLAGS does not conform to the enclave manifest",
            SgxAttributesFlags(enclave_attributes_flags),
            SgxAttributesFlags(manifest.attributes_flags),
        )

    if (
        enclave_attributes_xfrm & manifest.attributes_mask_xfrm
        != manifest.attributes_xfrm
    ):
        raise AttestationError(
            "SGX_ATTRIBUTES.XFRM does not conform to the enclave manifest",
            enclave_attributes_xfrm,
            manifest.attributes_xfrm,
        )
    enclave_misc_select = attestation_result.enclave_report.misc_select.value
    if enclave_misc_select & manifest.misc_mask != manifest.misc_select:
        raise AttestationError(
            "SGX_MISC_SELECT does not conform to the enclave manifest",
            SgxMiscSelect(attestation_result.enclave_report.misc_select.value),
            SgxMiscSelect(manifest.misc_select),
        )


def hex_to_u64(hex_string: str) -> int:
    int_value = int(hex_string, 16)
    if int_value < 0 or int_value >= 2**64:
        raise ValueError("Not a valid unsigned 64-bit integer")
    return int_value


def hex_to_u32(hex_string: str) -> int:
    int_value = int(hex_string, 16)
    if int_value < 0 or int_value >= 2**32:
        raise ValueError("Not a valid unsigned 32-bit integer")
    return int_value


@dataclass
class EnclaveManifest:
    mr_enclave: bytes
    allow_debug: bool

    attributes_flags: int  # 64-bits bitvector
    attributes_mask_flags: int  # 64-bits bitvector

    attributes_xfrm: int  # 64-bits bitvector
    attributes_mask_xfrm: int  # 64-bits bitvector

    misc_select: int  # 32-bits bitvector
    misc_mask: int  # 32-bits bitvector

    def __post_init__(self):
        for (name, field_type) in self.__annotations__.items():
            if not isinstance(self.__dict__[name], field_type):
                current_type = type(self.__dict__[name])
                raise TypeError(
                    f"The field `{name}` was assigned by `{current_type}` instead of `{field_type}`"
                )

    @staticmethod
    def from_str(s: str) -> "EnclaveManifest":
        """Load a manifest from the content of a manifest.toml
        Args:
            s (str): The content of the manifest.
        Returns:
            manifest: The manifest.
        """
        return EnclaveManifest.from_dict(toml.loads(s))

    @staticmethod
    def from_file(path: str) -> "EnclaveManifest":
        """Load a manifest from a file.

        Args:
            path (str): The path of the file.
        Returns:
            manifest: The manifest.
        """
        return EnclaveManifest.from_dict(toml.load(path))

    @staticmethod
    def from_dict(obj: dict) -> "EnclaveManifest":
        """Load an enclave manifest from the dict obtained after manifest.toml
        decoding.

        Args:
            obj (dict): The dict.
        Returns:
            EnclaveManifest:
        """
        mr_enclave = bytes.fromhex(obj["mr_enclave"])
        if len(mr_enclave) != hashlib.sha256().digest_size:
            raise ValueError("Invalid mr_enclave in Manifest")
        return EnclaveManifest(
            mr_enclave=mr_enclave,
            allow_debug=obj["allow_debug"],
            attributes_flags=hex_to_u64(obj["attributes_flags_hex"]),
            attributes_mask_flags=hex_to_u64(obj["attributes_mask_flags_hex"]),
            attributes_xfrm=hex_to_u64(obj["attributes_xfrm_hex"]),
            attributes_mask_xfrm=hex_to_u64(obj["attributes_mask_xfrm_hex"]),
            misc_mask=hex_to_u32(obj["misc_mask_hex"]),
            misc_select=hex_to_u32(obj["misc_select_hex"]),
        )


class SgxAttributesFlags(enum.IntFlag):
    """ATTRIBUTES_FLAGS are part the SGX REPORT ATTRIBUTES_FLAGS is a 64 bits
    long bit field.

    For more information about the structure, refer to Intel Reference
    available at
    <https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3d-part-4-manual.pdf>
    """

    # This bit indicates if the enclave has been initialized by EINIT. It must be cleared when loaded as
    # part of ECREATE. For EREPORT instruction, TARGET_INFO.ATTRIBUTES[ENIT] must always be 1 to
    # match the state after EINIT has initialized the enclave
    INIT = 1 << 0
    # If set the enclave permit debugger to read and write enclave data using EDBGRD and EDBGWR.
    DEBUG = 1 << 1
    # If set, enclave runs in 64-bit mode.
    MODE64BIT = 1 << 2
    # Must be Zero.
    RESERVED = 1 << 3
    # Provisioning Key is available from EGETKEY.
    PROVISIONKEY = 1 << 4
    # EINIT token key is available from EGETKEY.
    EINITTOKENKEY = 1 << 5
    # The remaining bits 63:6 are reserved for future use


class SgxMiscSelect(enum.IntFlag):
    """The bit vector of MISCSELECT selects which extended information is to be
    saved in the MISC region of the SSA frame when an AEX is generated.

    For more information about the structuren, refer to Intel Reference
    available at
    <https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3d-part-4-manual.pdf>
    """

    # Report information about page fault and general protection exception that occurred inside an enclave.
    EXINFO = 1 << 0
    # The remaining bits 31:1 are reserved for future use and must be 0 in the meantime.
