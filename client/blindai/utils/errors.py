import grpc


def check_rpc_exception(rpc_error):
    if rpc_error.code() == grpc.StatusCode.CANCELLED:
        return f"Cancelled GRPC call: code={rpc_error.code()} message={rpc_error.details()}"

    elif rpc_error.code() == grpc.StatusCode.UNAVAILABLE:
        return f"Failed to connect to GRPC server: code={rpc_error.code()} message={rpc_error.details()}"

    elif rpc_error.code() == grpc.StatusCode.UNIMPLEMENTED:
        return f"Incompatible client/server versions, code={rpc_error.code()} message={rpc_error.details()}"

    elif rpc_error.code() == grpc.StatusCode.FAILED_PRECONDITION:
        return f"Attestation is not available. Running in Simulation Mode, code={rpc_error.code()} message={rpc_error.details()}"
    else:
        return (
            f"Received RPC error: code={rpc_error.code()} message={rpc_error.details()}"
        )


def check_socket_exception(socket_error):
    if len(socket_error.args) >= 2:
        error_code = socket_error.args[0]
        error_message = socket_error.args[1]
        return f"Failed To connect to the server due to Socket error : code={error_code} message={error_message}"

    elif len(socket_error.args) == 1:
        error_message = socket_error.args[0]
        return f"Failed To connect to the server due to Socket error : message={error_message}"

    else:
        return "Failed To connect to the server due to Socket error "


class SignatureError(Exception):
    """This exception is raised when the signature or the returned digest is invalid"""

    pass


class AttestationError(Exception):
    """This exception is raised when the attestation is not valid (code signature mismatching, enclave settings mismatching...). Used a master exception for all other sub exceptions on the quote validation.

    Args:
        Args:
        claims (DcapClaims): The claims.
        policy (Policy): The enclave policy.
    """

    def __init__(self, message, claims, policy):
        self.claims = claims
        self.policy = policy
        self.message = message
        super().__init__(self.message)

    pass


class QuoteValidationError(Exception):
    """This exception is raised when the returned quote is invalid (TCB outdated, not signed by the hardware provider...). Used a master exception for all other sub exceptions on the quote validation"""

    pass


class NotAnEnclaveError(QuoteValidationError):
    """This exception is raised when the enclave claims are not validated by the hardware provider, meaning that the claims cannot be verified using the hardware root of trust"""

    pass


class EnclaveHeldDataError(QuoteValidationError):
    """This exception is raised when the enclave held data expected does not match the one in the quote. The expected enclave held data in BlindAI is the untrusted certificate to avoid man-in-the-middle attacks

    Args:
        expected_hash (str): Enclave held data hash expected
        measured_hash (str): Enclave held data hash calculated
    """

    def __init__(self, message, expected_hash, measured_hash):
        self.expected_hash = expected_hash
        self.measured_hash = measured_hash
        self.message = message
        super().__init__(self.message)

    pass


class IdentityError(AttestationError):
    """This exception is raised when the enclave code signature hash does not match the signature hash provided in the policy

    Args:
        expected_hash (str): Code signature hash expected
        measured_hash (str): Code signature hash calculated
    """

    def __init__(self, message, claims, policy, expected_hash, measured_hash):
        self.claims = claims
        self.policy = policy
        self.expected_hash = expected_hash
        self.measured_hash = measured_hash
        self.message = message
        super().__init__(self.message, claims, policy)

    pass


class DebugNotAllowedError(AttestationError):
    """This exception is raised when the enclave is in debug mode but the provided policy doesn't allow debug mode"""

    pass


class HardwareModeUnsupportedError(Exception):
    """This exception is raised when the server is in simulation mode but an hardware mode attestation was requested from it"""

    pass


class VersionError(Exception):
    """This exception is raised when the server version is not supported by the client"""

    pass
