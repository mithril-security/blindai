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
import grpc
import cryptography.x509
from cryptography.hazmat.primitives.serialization import Encoding

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

