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

import re
from typing import Iterator
import cryptography.x509
from cryptography.hazmat.primitives.serialization import Encoding
from cryptography.x509 import ObjectIdentifier, load_pem_x509_certificate
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
import pkgutil
from blindai.pb.securedexchange_pb2 import DatumTypeEnum
from enum import IntEnum

CHUNK_SIZE = 32 * 1024  # 32kb

ModelDatumType = IntEnum("ModelDatumType", DatumTypeEnum.items())


def strip_https(url: str) -> str:
    return re.sub(r"^https:\/\/", "", url)


def encode_certificate(cert: bytes):
    return cryptography.x509.load_der_x509_certificate(cert).public_bytes(Encoding.PEM)


def create_byte_chunk(data: bytes) -> Iterator[bytes]:
    sent_bytes = 0
    while sent_bytes < len(data):
        yield bytes(
            data[sent_bytes : sent_bytes + min(CHUNK_SIZE, len(data) - sent_bytes)]
        )
        sent_bytes += min(CHUNK_SIZE, len(data) - sent_bytes)


def get_enclave_signing_key(server_cert: bytes) -> bytes:
    ENCLAVE_ED25519_SIGNING_KEY_OID = ObjectIdentifier("1.3.6.1.3.2")
    enclave_ed25519_signing_key = (
        load_pem_x509_certificate(server_cert)
        .extensions.get_extension_for_oid(ENCLAVE_ED25519_SIGNING_KEY_OID)
        .value.value
    )
    enclave_signing_key = Ed25519PublicKey.from_public_bytes(
        enclave_ed25519_signing_key
    )
    return enclave_signing_key


def get_supported_server_version() -> str:
    supported_versions_file = pkgutil.get_data(
        __name__, "../supported_server_versions.py"
    ).decode("utf-8")
    versions_re = r"__version__ = \"(?P<version>.+)\""
    supported_versions = re.match(versions_re, supported_versions_file).group("version")
    return supported_versions


def supported_server_version(version: str) -> bool:
    supported_versions = get_supported_server_version().split(".")
    server_version = version.split(".")
    for i in range(len(server_version)):
        # Numeric characters in supported_versions must match the ones in the server version
        # Alphabetic characters are variables
        if (supported_versions[i] == server_version[i]) or (
            not supported_versions[i].isdigit()
        ):
            continue
        else:
            return False
    return True
