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
from cryptography.hazmat.primitives import serialization
import torch
import os


def strip_https(url: str) -> str:
    return re.sub(r"^https:\/\/", "", url)


def cert_der_to_pem(cert: bytes) -> bytes:
    return cryptography.x509.load_der_x509_certificate(cert).public_bytes(
        serialization.Encoding.PEM
    )


def fetch_whisper_tiny_20_tokens():
    # TODO: Urgent
    # Remove this implementation and actually convert the model to ONNX

    # We convert the method to ONNX
    # For the mean time, we will return a downloaded whisper model

    # Add filename to current directory

    filename = "whisper-tiny-20-tokens.onnx"
    filename = os.path.join(os.path.dirname(__file__), filename)

    # Check if the file doesn't exist, then download it
    if not os.path.exists(filename):
        os.system(f"gdown 1wqg1F0UkEdm3KB7n1BjfRLHnzKU2-G5S --output {filename}")

    # Return the absolute path of the file
    return os.path.abspath(filename)
