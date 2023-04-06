import hashlib
from typing import Optional, List, Dict
import cbor2
import cryptography
from datetime import datetime
from pycose.messages import Sign1Message
from pycose import keys
from pycose.keys import curves
from pycose.keys.keyparam import KpKty, KpAlg, EC2KpCurve, EC2KpX, EC2KpY
from pycose.algorithms import Es384
from pycose.keys.curves import P384
from pycose.keys.keytype import KtyEC2
from OpenSSL import crypto
from pydantic import BaseModel, StrictBytes, StrictInt, StrictStr
import importlib
import cryptography.hazmat.primitives.asymmetric.ec
from pycose.messages.signcommon import SignCommon


class NitroAttestationError(Exception):
    """This exception is raised when the attestation is invalid (enclave
    settings mismatching, debug mode unallowed...).

    Used as base exception for all other sub exceptions on the attestation
    validation
    """

    pass


class NitroAttestationDocument(BaseModel):
    module_id: StrictStr  # issuing Nitro hypervisor module ID
    timestamp: StrictInt  # UTC time when document was created, in milliseconds since UNIX epoch
    digest: StrictStr  # the digest function used for calculating the register values
    pcrs: Dict[
        StrictInt, StrictBytes
    ]  # { + index => pcr },      ; map of all locked PCRs at the moment the attestation document was generated
    certificate: StrictBytes  # the public key certificate for the public key
    # that was used to sign the attestation document
    cabundle: List[StrictBytes]  # issuing CA bundle for infrastructure certificate
    public_key: Optional[StrictBytes]  # an optional DER-encoded key the attestation
    # consumer can use to encrypt data with
    user_data: Optional[StrictBytes]  # additional signed user data, defined by protocol
    nonce: Optional[StrictBytes]  # an optional cryptographic nonce provided by the
    # attestation consumer as a proof of authenticity


def verify_attestation_doc(
    attestation_doc: bytes, root_cert_pem: str, time: Optional[datetime] = None
) -> NitroAttestationDocument:
    """
    Verify that the attestation document is genuine
    If invalid, raise an exception

    The verification is made as documented by AWS in <https://docs.aws.amazon.com/enclaves/latest/user/verify-root.html>
    """
    # Decode CBOR attestation document
    data = cbor2.loads(attestation_doc)

    # Load and decode document payload
    payload = data[2]
    att_doc = NitroAttestationDocument(**cbor2.loads(payload))

    # Load signing certificate from attestation document
    signing_cert = crypto.load_certificate(crypto.FILETYPE_ASN1, att_doc.certificate)

    # Verify the certificate's chain

    store = crypto.X509Store()

    # To make testing easier, we offer the ability to set the current time
    # used for verification
    if time is not None:
        store.set_time(time)

    # Create the CA cert object from PEM string, and store into X509Store
    _rootca_cert = crypto.load_certificate(crypto.FILETYPE_PEM, root_cert_pem)  # type: ignore
    store.add_cert(_rootca_cert)

    # Use CA bundle from attestation document to build certificate chain
    # We remove the first certificate, which is the root certificate
    # Note : we don't have to worry about AWS changing the order since they
    # explicitly guarantee the ordering of the certificates.
    chain = [
        crypto.load_certificate(crypto.FILETYPE_ASN1, _cert_binary)
        for _cert_binary in att_doc.cabundle[1:]
    ]

    store_ctx = crypto.X509StoreContext(store, signing_cert, chain=chain)

    # Verify the signing certificate
    try:
        # if the cert is invalid, it will raise a X509StoreContextError
        store_ctx.verify_certificate()
    except crypto.X509StoreContextError:
        raise NitroAttestationError("Invalid signing certificate")

    # Check that the attestation document is properly signed.

    # Get the key parameters from the signing cert public key
    cert_pubkey = signing_cert.get_pubkey().to_cryptography_key()

    if not isinstance(
        cert_pubkey, cryptography.hazmat.primitives.asymmetric.ec.EllipticCurvePublicKey
    ):
        raise NitroAttestationError("Unsupported signing algorithm")

    if not isinstance(
        cert_pubkey.curve, cryptography.hazmat.primitives.asymmetric.ec.SECP384R1
    ):
        raise NitroAttestationError("Unsupported elliptic curve used for signing")

    cert_public_numbers = cert_pubkey.public_numbers()

    assert isinstance(
        cert_public_numbers,
        cryptography.hazmat.primitives.asymmetric.ec.EllipticCurvePublicNumbers,
    )

    x = cert_public_numbers.x.to_bytes(cert_pubkey.curve.key_size // 8, "big")
    y = cert_public_numbers.y.to_bytes(cert_pubkey.curve.key_size // 8, "big")

    key_attribute_dict = {
        KpKty: KtyEC2,
        EC2KpCurve: P384,
        KpAlg: Es384,
        EC2KpX: x,
        EC2KpY: y,
    }

    key = keys.CoseKey.from_dict(key_attribute_dict)

    # Get the protected header from the COSE document
    phdr = cbor2.loads(data[0])

    # Construct the Sign1 message
    msg = Sign1Message(phdr=phdr, uhdr=data[1], payload=payload)

    # We should not use "_signature" because it is a private attribute...
    # There is an issue on pycose github to make the signature attribute settable
    # <https://github.com/TimothyClaeys/pycose/issues/105>
    # When resolved we can switch to a much nicer "msg.signature"
    msg._signature = data[3]  # type: ignore

    msg.key = key

    # Verify the signature using the EC2 key
    if not msg.verify_signature():  # type: ignore
        raise NitroAttestationError("Wrong signature on attestation document")

    return att_doc


def validate_attestation(
    raw_attestation_doc: bytes,
    enclave_cert: bytes,
    expected_pcr0: bytes,
    _root_cert_pem: Optional[str] = None,
    _time: Optional[datetime] = None,
):
    """
    Validate the attestation

    * Verify that the attestation document is genuine (issued by AWS)
    * Check if the PCR0 (hash of the enclave image file) matches
    * Check that the user data embedded in attestation document
     contains the enclave certificate

    For more information about the attestation of AWS Nitro Enclaves,
    refer to <https://docs.aws.amazon.com/enclaves/latest/user/set-up-attestation.html>
    """

    if _root_cert_pem is None:
        # AWS Nitro root CA is embedded in the python package
        # It was obtained from AWS at
        # <https://aws-nitro-enclaves.amazonaws.com/AWS_NitroEnclaves_Root-G1.zip>
        root_cert_pem = importlib.resources.read_text(  # type: ignore
            __package__, "aws_nitro_enclaves_rootca.pem"
        )
    else:
        root_cert_pem = _root_cert_pem

    attestation_doc = verify_attestation_doc(
        raw_attestation_doc, root_cert_pem=root_cert_pem, time=_time
    )
    assert attestation_doc.digest == "SHA384"
    assert attestation_doc.pcrs[0] == expected_pcr0
    cert_hash = hashlib.sha256(enclave_cert).digest()
    app_hash = 32 * b"\x00"
    expected_user_data = b"sha256:%b;sha256:%b" % (cert_hash, app_hash)
    assert attestation_doc.user_data == expected_user_data
