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

import cryptography.x509
import grpc
from cryptography.hazmat.primitives.serialization import Encoding
from cryptography.x509 import ObjectIdentifier, load_pem_x509_certificate
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

CHUNK_SIZE = 32 * 1024  # 32kb


def strip_https(url: str):
    return re.sub(r"^https:\/\/", "", url)


def encode_certificate(cert):
    return cryptography.x509.load_der_x509_certificate(cert).public_bytes(Encoding.PEM)


def create_byte_chunk(data):
    sent_bytes = 0
    while sent_bytes < len(data):
        yield bytes(
            data[sent_bytes : sent_bytes + min(CHUNK_SIZE, len(data) - sent_bytes)]
        )
        sent_bytes += min(CHUNK_SIZE, len(data) - sent_bytes)


def check_rpc_exception(rpc_error):
    if rpc_error.code() == grpc.StatusCode.CANCELLED:
        return f"Cancelled GRPC call: code={rpc_error.code()} message={rpc_error.details()}"

    elif rpc_error.code() == grpc.StatusCode.UNAVAILABLE:
        return f"Failed to connect to GRPC server: code={rpc_error.code()} message={rpc_error.details()}"

    else:
        return (
            f"Received RPC error: code={rpc_error.code()} message={rpc_error.details()}"
        )

def check_socket_exception(socket_error):
    if len(socket_error.args) >= 2:
        error_code = socket_error.args[0]
        error_message = socket_error.args[1]
        return f"Failed To connect to the server due to Socket error : code={error_code} message={error_message}"

    elif len(socket_error.args)==1:
        error_message = socket_error.args[0]
        return f"Failed To connect to the server due to Socket error : message={error_message}"

    else:
        return f"Failed To connect to the server due to Socket error "

def get_enclave_signing_key(server_cert):
    ENCLAVE_ED25519_SIGNING_KEY_OID = ObjectIdentifier("1.3.6.1.3.2")
    enclave_ed25519_signing_key = load_pem_x509_certificate(server_cert).extensions.get_extension_for_oid(ENCLAVE_ED25519_SIGNING_KEY_OID).value.value
    enclave_signing_key = Ed25519PublicKey.from_public_bytes(enclave_ed25519_signing_key)
    return enclave_signing_key