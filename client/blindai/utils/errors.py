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

from enum import IntEnum
import grpc


class Actions(IntEnum):
    READ_CERT_FILE = 0
    GET_UNTRUSTED_SERVER_CERT = 1
    READ_POLICY_FILE = 2
    LOAD_POLICY = 3
    VERIFY_CLAIMS = 4
    CONNECT_SERVER = 5
    READ_MODEL_FILE = 6


ACTION_MESSAGE = {
    0: "read certificate file",
    1: "get untrusted server certificate",
    2: "read policy file",
    3: "load policy",
    4: "verify claims",
    5: "connect to the server",
    6: "read model file",
}


def check_exception(error, action, simulation, debug):
    if simulation:
        mode = "simulation"
    else:
        mode = "hardware"

    message = f"Failed to {ACTION_MESSAGE[int(action)]} in {mode} mode.\nError details : {error}"
    if (
        action == Actions.READ_CERT_FILE
        or action == Actions.READ_POLICY_FILE
        or action == Actions.READ_MODEL_FILE
    ):
        err = IOError(message)

    elif (
        action == Actions.GET_UNTRUSTED_SERVER_CERT or action == Actions.CONNECT_SERVER
    ):
        err = ConnectionError(message)

    elif action == Actions.LOAD_POLICY or action == Actions.VERIFY_CLAIMS:
        err = ValueError(message)

    if debug:
        # This will raise both err and the original exception, getting the whole traceback
        raise err
    return err


def check_rpc_exception(error, action, simulation, debug):
    if simulation:
        mode = "simulation"
    else:
        mode = "hardware"

    message = (
        f"Failed to {ACTION_MESSAGE[int(action)]} in {mode} mode.\nError details : "
    )
    if error.code() == grpc.StatusCode.CANCELLED:
        message += f"Cancelled GRPC call: code={error.code()} message={error.details()}"

    elif error.code() == grpc.StatusCode.UNAVAILABLE:
        message += f"Failed to connect to GRPC server: code={error.code()} message={error.details()}"
    else:
        message += f"Received RPC error: code={error.code()} message={error.details()}"

    err = ConnectionError(message)
    if debug:
        # This will raise both err and the original exception
        raise err
    return err
