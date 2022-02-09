import re
import grpc
import cryptography.x509
from cryptography.hazmat.primitives.serialization import Encoding



CHUNK_SIZE = 32 * 1024  # 32kb
FLOAT_NB_ELEM = 16


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


def create_float_chunk(data):
    sent = 0
    while sent < len(data):
        yield data[sent : sent + min(FLOAT_NB_ELEM, len(data) - sent)]
        sent += min(FLOAT_NB_ELEM, len(data) - sent)


def check_rpc_exception(rpc_error):
    if rpc_error.code() == grpc.StatusCode.CANCELLED:
        print(
            f"Cancelled GRPC call: code={rpc_error.code()} message={rpc_error.details()}"
        )
        pass
    elif rpc_error.code() == grpc.StatusCode.UNAVAILABLE:
        print(
            f"Failed to connect to GRPC server: code={rpc_error.code()} message={rpc_error.details()}"
        )
        pass
    else:
        print(
            f"Received RPC error: code={rpc_error.code()} message={rpc_error.details()}"
        )
        pass