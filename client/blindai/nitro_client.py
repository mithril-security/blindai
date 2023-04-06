import contextlib
import tempfile
from .utils import cert_der_to_pem
import requests
from ._nitro_attestation import (
    validate_attestation,
    NitroAttestationError,
    NitroAttestationDocument,
)
import warnings


class NitroDebugModeWarning(Warning):
    pass


class BlindAiNitroConnection(contextlib.AbstractContextManager):
    """A class to represent a connection to a BlindAi server."""

    _conn: requests.Session

    def __init__(
        self,
        addr: str,
        debug_mode: bool,
    ):
        """Connect to a BlindAi service hosted on a Nitro enclave.

        Please refer to the connect function for documentation.

        Args:
            addr (str):
            debug_mode (bool):
        Returns:
        """

        if debug_mode:
            warnings.warn(
                (
                    "BlindAI is running in debug mode. "
                    "This mode is provided solely for testing purposes. "
                    "It MUST NOT be used in production."
                ),
                NitroDebugModeWarning,
            )

        self._addr = f"https://{addr}"
        s = requests.Session()
        # Always raise an exception when HTTP returns an error code for the unattested connection
        # Note : we might want to do the same for the attested connection ?

        # TODO: Remove verify=False for production
        s.hooks = {"response": lambda r, *args, **kwargs: r.raise_for_status()}
        attestation_doc = s.get(
            f"{self._addr }/enclave/attestation", verify=False
        ).content
        cert = s.get(f"{self._addr }/enclave/cert", verify=False).content

        # TODO: Set expected_pcr0 for production
        if debug_mode:
            expected_pcr0 = 48 * b"\x00"
        try:
            validate_attestation(
                attestation_doc, expected_pcr0=expected_pcr0, enclave_cert=cert
            )
        except NitroAttestationError:
            raise
        except Exception:
            raise NitroAttestationError("Attestation verification failed")

        # requests (http library) takes a path to a file containing the CA
        # there is no easy way to give the CA as a string/bytes directly
        # therefore a temporary file with the certificate content
        # has to be created.

        cert_file = tempfile.NamedTemporaryFile(mode="wb")
        cert_file.write(cert_der_to_pem(cert))
        cert_file.flush()

        # the file should not be close until the end of BlindAiConnection
        # so we store it in the object (else it might get garbage collected)
        self._cert_file = cert_file

        attested_conn = requests.Session()
        attested_conn.verify = cert_file.name

        # This adapter makes it possible to connect
        # to the server via a different hostname
        # that the one included in the certificate i.e. blindai-srv
        # For instance we can use it to connect to the server via the
        # domain / IP provided to connect(). See below
        from requests.adapters import HTTPAdapter

        class CustomHostNameCheckingAdapter(HTTPAdapter):
            def cert_verify(self, conn, url, verify, cert):
                conn.assert_hostname = "example.com"
                return super(CustomHostNameCheckingAdapter, self).cert_verify(
                    conn, url, verify, cert
                )

        attested_conn.mount(self._addr, CustomHostNameCheckingAdapter())
        attested_conn.hooks = {
            "response": lambda r, *args, **kwargs: r.raise_for_status()
        }
        try:
            attested_conn.get(f"{self._addr}/enclave").content
        except Exception as e:
            raise NitroAttestationError(
                "Cannot establish secure connection to the enclave"
            )

        self._conn = attested_conn

    def get_message(self):
        return self._conn.get(f"{self._addr}/enclave").content

    def close(self):
        self._conn.close()
        self._cert_file.close()

    def __enter__(self):
        """Return the BlindAiConnection upon entering the runtime context."""
        return self

    def __exit__(self, *args):
        """Close the connection to BlindAI server."""
        self.close()


def connect(addr: str, debug_mode: bool = False) -> BlindAiNitroConnection:
    """Connect to a BlindAi service hosted on a Nitro enclave.

    Args:
        addr (str): The address of the BlindAi service (such as "enclave.com:8443" or "localhost:8443").
        debug_mode (bool): Whether to run in debug mode. This mode is provided
            solely for testing purposes. It MUST NOT be used in production. Defaults to False.

    Returns:
        BlindAiNitroConnection: A connection to the BlindAi service.

    Raises:
        NitroAttestationError: If the attestation fails.
    """
    return BlindAiNitroConnection(addr, debug_mode)
