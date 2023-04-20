import contextlib
import requests
from ._nitro_attestation import (
    validate_attestation,
    NitroAttestationError,
)
from ._certificate_pinning import FingerprintAdapter
import warnings
from typing import Optional


class NitroDebugModeWarning(Warning):
    pass


class BlindAiNitroConnection(contextlib.AbstractContextManager):
    """A class to represent a connection to a BlindAi server."""

    _conn: requests.Session

    def __init__(
        self,
        addr: str,
        expected_pcr0: Optional[bytes] = None,
        is_local: bool = False,
    ):
        """Connect to a BlindAi service hosted on a Nitro enclave.

        Please refer to the connect function for documentation.

        Args:
            addr (str):
            debug_mode (bool):
        Returns:
        """

        if expected_pcr0 is None:
            warnings.warn(
                (
                    "BlindAI is running in debug mode. "
                    "This mode is provided solely for testing purposes. "
                    "It MUST NOT be used in production."
                    "To deactivate debug mode pass the expected pcr0 of the enclave."
                ),
                NitroDebugModeWarning,
            )

        self._addr = f"https://{addr}"
        s = requests.Session()
        # Always raise an exception when HTTP returns an error code for the unattested connection
        # Note : we might want to do the same for the attested connection ?

        # TODO: Remove verify=False for production
        s.hooks = {"response": lambda r, *args, **kwargs: r.raise_for_status()}

        with warnings.catch_warnings():
            warnings.filterwarnings(
                "ignore", message="Unverified HTTPS request is being made to host"
            )
            attestation_doc = s.get(
                f"{self._addr }/enclave/attestation", verify=False
            ).content
            cert = s.get(f"{self._addr }/enclave/cert", verify=False).content

        if expected_pcr0 is None:
            expected_pcr0 = 48 * b"\x00"
        try:
            cert_hash = validate_attestation(
                attestation_doc, expected_pcr0=expected_pcr0, enclave_cert=cert
            )
        except NitroAttestationError:
            raise
        except Exception:
            raise NitroAttestationError("Attestation verification failed")

        attested_conn = requests.Session()
        attested_conn.mount(self._addr, FingerprintAdapter(cert_hash.hex()))

        if is_local:
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

    def api(self, method: str, endpoint: str, *args, **kwargs) -> str:
        _method = getattr(self._conn, method)
        return _method(f"{self._addr}{endpoint}", *args, **kwargs).text

    def close(self):
        self._conn.close()

    def __enter__(self):
        """Return the BlindAiConnection upon entering the runtime context."""
        return self

    def __exit__(self, *args):
        """Close the connection to BlindAI server."""
        self.close()


def connect(addr: str, expected_pcr0: Optional[bytes] = None, is_local: bool = False) -> BlindAiNitroConnection:
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
    return BlindAiNitroConnection(addr, expected_pcr0=expected_pcr0, is_local=is_local)
