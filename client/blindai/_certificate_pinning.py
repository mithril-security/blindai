from requests.adapters import HTTPAdapter

# from .._compat import poolmanager
from urllib3 import poolmanager


class FingerprintAdapter(HTTPAdapter):
    """
    A HTTPS Adapter for Python Requests that verifies certificate fingerprints,
    instead of certificate hostnames.
    Example usage:
    .. code-block:: python
        import requests
        import ssl
        from requests_toolbelt.adapters.fingerprint import FingerprintAdapter
        twitter_fingerprint = '...'
        s = requests.Session()
        s.mount(
            'https://twitter.com',
            FingerprintAdapter(twitter_fingerprint)
        )
    The fingerprint should be provided as a hexadecimal string, optionally
    containing colons.
    """

    __attrs__ = HTTPAdapter.__attrs__ + ['fingerprint']

    def __init__(self, fingerprint: str, **kwargs):
        self.fingerprint = fingerprint

        super(FingerprintAdapter, self).__init__(**kwargs)

    def init_poolmanager(self, connections, maxsize, block=False):
        self.poolmanager = poolmanager.PoolManager(
            num_pools=connections,
            maxsize=maxsize,
            block=block,
            assert_fingerprint=self.fingerprint)
