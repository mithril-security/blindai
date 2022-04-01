import errno
import subprocess
import os
import sys
import socket
from time import time as now, sleep
import shutil
import atexit

server_process = None
policy_file = os.path.join(os.path.dirname(__file__), "policy.toml")
certificate_file = os.path.join(os.path.dirname(__file__), "host_server.pem")

has_hardware_support = os.getenv("BLINDAI_TEST_NO_HW") is None


def launch_server():
    global server_process

    sock = None

    try:
        if server_process is None and os.getenv("BLINDAI_TEST_NO_LAUNCH_SERVER") is None:
            server_dir = os.path.join(os.path.dirname(__file__), "../server")
            bin_dir = os.path.join(server_dir, "bin")

            server_process = subprocess.Popen(
                ["./blindai_app"],
                executable=os.path.join(bin_dir, "blindai_app"),
                cwd=bin_dir,
                stdout=sys.stdout,
                stderr=sys.stderr,
                stdin=subprocess.DEVNULL,
                env={**os.environ, "BLINDAI_DISABLE_TELEMETRY": "true"},
            )

            shutil.copyfile(os.path.join(server_dir, "policy.toml"), policy_file)
            shutil.copyfile(os.path.join(server_dir, "host_server.pem"), certificate_file)

        # block until server ready (port open)
        end = now() + 30 # 30s timeout
        success = False
        while True:
            if now() > end:
                raise Exception("Server startup timed out")

            try:
                sock = socket.socket()
                sock.settimeout(end - now())
                sock.connect(("localhost", 50052))
                success = True
                sock.close()
                break
            except socket.error as err:
                if err.errno != errno.ECONNREFUSED:
                    raise
                sock.close()
                sleep(0.1)

        if not success:
            raise Exception("Server startup timed out")

        print("[TESTS] The server is running")

    except:
        if sock is not None:
            sock.close()

        if server_process is not None:
            server_process.terminate()
            server_process.wait()
        raise


def close_server():
    global server_process

    if server_process is None:
        return

    server_process.terminate()
    server_process.wait()
    server_process = None

    print("[TESTS] The server is stopped")


atexit.register(close_server)
