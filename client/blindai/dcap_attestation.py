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
from typing import Any, Dict

import pybind11_module
import toml
from bitstring import Bits
from pybind11_module import status
from utils.utils import encode_certificate


def verify_dcap_attestation(
    quote: bytes, attestation_collateral: Any, enclave_held_data: bytes
) -> Dict[str, str]:
    """
    verify_dcap_attestation verifies if the enclave evidence is valid
    * validates if the quote is trustworthy (issued by an approved Intel CPU) with the
        attestation collateral using SGX Quote Verification Library
    * validates if the SHA256 hash of Enclave Held Data (EHD) matches the first 32 bytes
        of reportData field in the enclave quote. After this check
    we can be sure that the EHD bytes are endorsed by the enclave
    TODO: Handle the case where the retuned quote status is STATUS_TCB_SW_HARDENING_NEEDED
    We must do more cautious checks in this case in order to determine whether or not to accept the quote
    It returns a dictionnary of claims about the enclave like :
    {
        "sgx-ehd" : <enclave held data>
        "sgx-is-debuggable": true,
        "sgx-mrenclave": <SGX enclave MRENCLAVE hex string>
        "sgx-misc-select": <bytes MISC SELECT>
        "sgx-attributes": <bytes with SGX Attributes>
        "raw": {
            "quote": <raw binary quote>
        }
    }
    :param quote: SGX quote
    :param attestation_collateral: SGX collateral needed to assess the validity of the quote
        (collateral is signed by Intel)
    :param enclave_held_data: enclave held data
    :return: a dictionary of claims
    """

    t = pybind11_module.Verification()
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
    # Default expiration date is current time (no need to set t.expirationDate)

    ret = t.verify()

    if ret.pckCertificateStatus != status.STATUS_OK:
        raise ValueError(
            "Error : Wrong PCK Certificate Status {}", ret.pckCertificateStatus
        )

    if ret.tcbInfoStatus != status.STATUS_OK:
        raise ValueError("Error : Wrong TCB Info Status {}", ret.tcbInfoStatus)

    if ret.qeIdentityStatus != status.STATUS_OK:
        raise ValueError("Error : Wrong QE Identity Status {}", ret.qeIdentityStatus)

    if ret.quoteStatus not in [status.STATUS_OK, status.STATUS_TCB_SW_HARDENING_NEEDED]:
        raise ValueError("Error : Wrong Quote Status {}", ret.quoteStatus)

    if hashlib.sha256(enclave_held_data).digest() != bytes(ret.reportData)[:32]:
        raise ValueError(
            "Enclave Held Data hash doesn't match with the report data from the quote"
        )

    claims = {
        "raw": {"quote": quote},
        "sgx-attributes": b"".join(struct.pack("B", x) for x in ret.attributes),
        "sgx-misc-select": bytes(ctypes.c_uint32(ret.miscSelect)),
        "sgx-mrenclave": bytes(ret.mrEnclave).hex(),
        "sgx-ehd": enclave_held_data,
    }

    # https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3d-part-4-manual.pdf
    # The ATTRIBUTES data structure is comprised of bit-granular fields that are used in the SECS
    # DEBUG flag is at bit 1
    # If 1, the enclave permit debugger to read and write enclave data using EDBGRD and EDBGWR
    # Using a mask we test if this bit is set

    claims["sgx-is-debuggable"] = bool(ret.attributes[0] & (1 << 1))

    return claims


def load_policy(path: str):
    with open(path) as f:
        policy = toml.load(f)
        policy["misc_mask"] = int(policy["misc_mask_hex"], 16).to_bytes(
            4, byteorder="little"
        )
        policy["misc_select"] = int(policy["misc_select_hex"], 16).to_bytes(
            4, byteorder="little"
        )
        policy["attributes_flags"] = int(policy["attributes_flags_hex"], 16).to_bytes(
            8, byteorder="little"
        )
        policy["attributes_xfrm"] = int(policy["attributes_xfrm_hex"], 16).to_bytes(
            8, byteorder="little"
        )
        policy["attributes_mask_flags"] = int(
            policy["attributes_mask_flags_hex"], 16
        ).to_bytes(8, byteorder="little")
        policy["attributes_mask_xfrm"] = int(
            policy["attributes_mask_xfrm_hex"], 16
        ).to_bytes(8, byteorder="little")

    return policy


def verify_claims(claims, policy):
    if claims["sgx-mrenclave"] != policy["mr_enclave"]:
        raise ValueError("MRENCLAVE doesn't match with the policy")

    if claims["sgx-is-debuggable"] and not policy["allow_debug"]:
        raise ValueError("Enclave is in debug mode but the policy doesn't allow debug")

    # If this flag is set, then the enclave is initialized
    SGX_FLAGS_INITTED = Bits("0x0100000000000000")

    if (
        Bits(claims["sgx-attributes"][:8]) & Bits(policy["attributes_mask_flags"])
        != Bits(policy["attributes_flags"]) | SGX_FLAGS_INITTED
    ):
        raise ValueError("SGX attributes flags bytes do not conform to the policy")

    if Bits(claims["sgx-attributes"][8:]) & Bits(
        policy["attributes_mask_xfrm"]
    ) != Bits(policy["attributes_xfrm"]):
        raise ValueError("SGX XFRM flags bytes do not conform to the policy")

    if Bits(claims["sgx-misc-select"]) & Bits(policy["misc_mask"]) != Bits(
        policy["misc_select"]
    ):
        raise ValueError("SGX MISC SELECT bytes do not conform to the policy")


def get_server_cert(claims):
    """
    Get the server certificate from the Azure Attestation claims
    :param claims:
    :return: The PEM-encoded server certificate as a byte string
    """
    return encode_certificate(claims["sgx-ehd"])
