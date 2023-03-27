import platform
import subprocess
import sys
import tarfile
import urllib.request
import io
import os
from urllib.error import HTTPError
from os import path
from subprocess import Popen

from importlib_metadata import version
from pathlib import Path

app_version = version("blindai")


class MockServer:
    """Mock server process handle"""

    def __init__(self, process, _private=False):
        if not _private:
            raise NotImplementedError()
        self._process = process

    def stop(self) -> bool:
        """Stop BlindAI mock server.

        This method will kill the running server, if the provided MockServer is still running.
        Args:
            self (MockServer): Mock server process handle
        Return:
            bool, True if the server was successfully stopped.
        Raises:
            None
        """

        if self._process is not None and self._process.poll() is None:
            print("Stopping BlindAI mock server...")
            self._process.kill()
            return True
        else:
            print("BlindAI mock server already stopped")
            return False


class NotFoundError(Exception):
    """This exception is raised when there was an error opening an URL.
    Args:
        Args:
        message (str): Error message.
    """

    def __init__(self, message):
        self.message = message
        super().__init__(self.message)


def _extract_tar(data):
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tar:
        tar.extractall(Path.cwd() / "bin")


def _handle_download(path: Path, url: str, name: str, error_msg: str):
    if not path.exists():
        print(f"Downloading {name}...")
        try:
            response = urllib.request.urlopen(url)
            _extract_tar(response.read())
        except HTTPError as e:
            raise NotFoundError(
                "{}. Exact error code: {}".format(error_msg, e.code)
            ) from None
    else:
        print("{} already installed".format(name))


def _start_server(blindai_path: Path) -> MockServer:
    blindai_path.chmod(0o755)
    process = subprocess.Popen([blindai_path], env=os.environ)
    return MockServer(process, _private=True)


def start_mock_server() -> MockServer:
    """Start BlindAI mock server for testing.
    The method will download BlindAI Preview's mock server binary if needed.
    The mock server is then started allowing to run the rest of your Google Colab/Jupyter Notebook environment.
    Args:
        None
    Return:
        MockServer object, the process of the running server.
    Raises:
        NotFoundError: Will be raised if one of the URL the wheel will try to access is invalid. This might mean that either there is no available binary of BlindAI's server.
        Other exceptions might be raised by zipfile or urllib.request.
    """
    blindai_path = Path.cwd() / "bin" / "blindai_mock_server"

    arch = platform.machine()
    if arch == "AMD64":
        arch = "x86_64"

    if not (arch == "x86_64" and sys.platform == "linux"):
        raise RuntimeError(f"Unsupported system : {platform.machine()}-{sys.platform}")

    blindai_url = f"https://github.com/mithril-security/blindai/releases/download/v{app_version}/blindai_mock_server-{app_version}-x86_64-unknown-linux-gnu.tgz"

    _handle_download(
        blindai_path,
        blindai_url,
        f"BlindAI mock server (version {app_version})",
        "The release might not be available yet for the current version",
    )
    process = _start_server(blindai_path)
    return process
