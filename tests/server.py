import errno
import subprocess
import os
import sys
import socket
from time import time as now
import shutil

server_process = None
policy_file = os.path.join(os.path.dirname(__file__), "policy.toml")
certificate_file = os.path.join(os.path.dirname(__file__), "host_server.pem")

has_hardware_support = os.getenv("BLINDAI_TEST_NO_HW") is None


def launch_server():
    global server_process

    try:
        if os.getenv("BLINDAI_TEST_NO_LAUNCH_SERVER") is not None:
            return
        if server_process is not None:
            return

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
        end = now() + 5000  # 5s timeout
        while True:
            if now() > end:
                raise Exception("server startup timed out")

            try:
                s = socket.socket()
                s.settimeout(end - now())
                s.connect(("localhost", 50053))
            except socket.error as err:
                if err.errno != errno.ECONNREFUSED:
                    s.close()
                    server_process.terminate()
                    server_process.wait()
                    raise
                s.close()
            else:
                s.close()
                break

        print("Server started")

    except:
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

    print("Server stopped")
