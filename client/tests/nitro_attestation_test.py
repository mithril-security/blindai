from datetime import datetime
from pathlib import Path
import importlib

from blindai._nitro_attestation import (
    verify_attestation_doc,
    validate_attestation,
    NitroAttestationError,
)
import pytest
import cbor2
from pycose.messages import Sign1Message
from pycose import keys


def test_verify_attestation_doc_ok():
    attestation_doc = (
        Path(__file__).parent / "nitro_attestation_document_valid.cbor"
    ).read_bytes()
    nitro_root_cert = importlib.resources.read_text(  # type: ignore
        "blindai", "aws_nitro_enclaves_rootca.pem"
    )
    verify_attestation_doc(
        attestation_doc,
        root_cert_pem=nitro_root_cert,
        time=datetime.fromisoformat("2023-03-22 14:28:27.405"),
    )


def test_verify_attestation_doc_expired():
    attestation_doc = (
        Path(__file__).parent / "nitro_attestation_document_valid.cbor"
    ).read_bytes()
    nitro_root_cert = importlib.resources.read_text(  # type: ignore
        "blindai", "aws_nitro_enclaves_rootca.pem"
    )
    with pytest.raises(NitroAttestationError):
        verify_attestation_doc(
            attestation_doc,
            root_cert_pem=nitro_root_cert,
            time=datetime.fromisoformat("2023-03-22 18:28:27.405"),
        )


def test_verify_attestation_doc_fake_sig():
    """
    Test verify on an attestation document with an fake signature
    """
    # Construct the fake signature
    attestation_doc = (
        Path(__file__).parent / "nitro_attestation_document_valid.cbor"
    ).read_bytes()
    data = cbor2.loads(attestation_doc)
    msg = Sign1Message(phdr=cbor2.loads(data[0]), uhdr=data[1], payload=data[2])
    fake_key = keys.EC2Key.generate_key(crv="P_384")
    msg.key = fake_key
    fake_attestation_doc = msg.encode(tag=False)

    nitro_root_cert = importlib.resources.read_text(  # type: ignore
        "blindai", "aws_nitro_enclaves_rootca.pem"
    )
    with pytest.raises(NitroAttestationError):
        verify_attestation_doc(
            fake_attestation_doc,
            root_cert_pem=nitro_root_cert,
            time=datetime.fromisoformat("2023-03-22 14:28:27.405"),
        )


def test_validate_attestation():
    attestation_doc = (
        Path(__file__).parent / "nitro_attestation_document_valid.cbor"
    ).read_bytes()
    validate_attestation(
        attestation_doc,
        enclave_cert=b"hello, world!",
        expected_pcr0=48 * b"\x00",
        _time=datetime.fromisoformat("2023-03-22 14:28:27.405"),
    )
