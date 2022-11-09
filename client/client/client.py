from enum import IntEnum
from typing import Any, Dict, List, Optional, Tuple, Union
from cbor2 import dumps as cbor2_dumps
from cbor2 import loads as cbor2_loads
import os 
import http.client
import contextlib
import ssl, socket
import platform
from utils import *
from version import __version__ as app_version
from hashlib import sha256
import getpass
import logging


CONNECTION_TIMEOUT = 10

class ModelDatumType(IntEnum):
    F32 = 0
    F64 = 1
    I32 = 2
    I64 = 3
    U32 = 4
    U64 = 5

class TensorInfo:
    fact:List[int]
    datum_type:ModelDatumType

    def __init__(self, fact, datum_type):
        self.fact = fact
        self.datum_type = datum_type

    
class UploadModel:
    model:List[int]
    input:List[TensorInfo]
    output:List[ModelDatumType]
    length:int
    sign:bool
    model_name:str

    def __init__(self,model,input,output,length,sign=False,model_name=""):
        self.model = model
        self.input = input
        self.output = output
        self.length = length
        self.sign = sign
        self.model_name = model_name


class RunModel:
    model_id:str
    inputs:List[int]
    sign:bool

    def __init__(self,model_id,inputs,sign):
        self.model_id=model_id
        self.inputs=inputs
        self.sign=sign


class DeleteModel:
    model_id:str

    def __init__(self,model_id):
        self.model_id=model_id


class SendModelPayload:
    hash:List[int]
    inputfact: List[int]
    model_id:str

    def __init__(self, **entries):
        self.__dict__.update(entries)


class SendModelReply:
    payload: SendModelPayload
    signature: List[int]


class RunModelPayload:
    output:List[int]
    datum_output: int
    input_hash: List[int]
    model_id:str

    def __init__(self, **entries):
        self.__dict__.update(entries)


class RunModelReply:
    payload: RunModelPayload
    signature: List[int]

class SignedResponse:
    payload: Optional[bytes] = None
    signature: Optional[bytes] = None
    #attestation: Optional[GetSgxQuoteWithCollateralReply] = None

class UploadResponse(SignedResponse):
    model_id: str

class RunModelResponse(SignedResponse):
    output: List[float]
    model_id: str


class ClientInfo:
    uid: str
    platform_name: str
    platform_arch: str
    platform_version: str
    platform_release: str
    user_agent: str
    user_agent_version: str

    def __init__(self, uid, platform_name, platform_arch, platform_version, platform_release, user_agent, user_agent_version ):
        self.uid = uid
        self.platform_name = platform_name
        self.platform_arch = platform_arch
        self.platform_version = platform_version
        self.platform_release = platform_release
        self.user_agent = user_agent
        self.user_agent_version = user_agent_version



def _get_input_output_tensors(
    tensor_inputs: Optional[List[List[Any]]] = None,
    tensor_outputs: Optional[ModelDatumType] = None,
    shape: Tuple = None,
    dtype: ModelDatumType = ModelDatumType.F32,
    dtype_out: ModelDatumType = ModelDatumType.F32,
) -> Tuple[List[List[Any]], List[ModelDatumType]]:
    if tensor_inputs is None or tensor_outputs is None:
        tensor_inputs = [shape, dtype]
        tensor_outputs = dtype_out     #Dict may be required for correct cbor serialization

    if type(tensor_inputs[0]) != list:
        tensor_inputs = [tensor_inputs]

    if type(tensor_outputs) != list:
        tensor_outputs = [tensor_outputs]

    inputs = []
    for tensor_input in tensor_inputs:
        inputs.append(TensorInfo(fact=tensor_input[0], datum_type=tensor_input[1]).__dict__)     #Required for correct cbor serialization

    return (inputs, tensor_outputs)

def _handle_response(res):
    if res.status == 200:
        return res.read()
    if res.status == 400:
        raise ValueError("Server couldn\'t handle the request because :", res.read())
    if res.status == 500:
        raise ValueError("Server internal error")
    raise ValueError("Unknown status code in request")

class BlindAiConnection(contextlib.AbstractContextManager):
    conn: http.client.HTTPSConnection
    #policy: Optional[Policy] = None
    #_stub: Optional[ExchangeStub] = None
    enclave_signing_key: Optional[bytes] = None
    simulation_mode: bool = False
    _disable_untrusted_server_cert_check: bool = False
    #attestation: Optional[GetSgxQuoteWithCollateralReply] = None
    server_version: Optional[str] = None
    #client_info: ClientInfo
    tensor_inputs: Optional[List[List[Any]]]
    tensor_outputs: Optional[List[ModelDatumType]]
    closed: bool = False

    def __init__(
        self,
        addr: str,
        server_name: str = "blindai-srv",
        #policy: Optional[str] = None,
        certificate: Optional[str] = None,
        simulation: bool = False,
        untrusted_port: int = 9923,
        attested_port: int = 9924,
        debug_mode=False,
    ):
        #if debug_mode:  # pragma: no cover
        #    os.environ["GRPC_TRACE"] = "transport_security,tsi"
        #    os.environ["GRPC_VERBOSITY"] = "DEBUG"

        uname = platform.uname()
        
        self.client_info = ClientInfo(
            uid=sha256((socket.gethostname() + "-" + getpass.getuser()).encode("utf-8"))
            .digest()
            .hex(),
            platform_name=uname.system,
            platform_arch=uname.machine,
            platform_version=uname.version,
            platform_release=uname.release,
            user_agent="blindai_python",
            user_agent_version=app_version,
        )
        

        self.connect_server(
            addr,
            server_name,
            #policy,
            certificate,
            simulation,
            untrusted_port,
            attested_port,
        )

    def connect_server(
        self,
        addr: str,
        server_name,
        #policy,
        certificate,
        simulation,
        untrusted_port,
        attested_port,
    ):
        self.simulation_mode = simulation
        self._disable_untrusted_server_cert_check = simulation

        #addr = strip_https(addr)

        untrusted_client_to_enclave = addr + ":" + str(untrusted_port)
        attested_client_to_enclave = addr + ":" + str(attested_port)

        #if not self.simulation_mode:
        #    self.policy = Policy.from_file(policy)

        if self._disable_untrusted_server_cert_check:
            logging.warning("Untrusted server certificate check bypassed")

            #socket.setdefaulttimeout(CONNECTION_TIMEOUT)                
            context = ssl._create_unverified_context()
               
        else:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
            context.load_verify_locations("../host_server.pem")
            context.check_hostname = False
            

        try:
            
            untrusted_conn = http.client.HTTPSConnection("localhost", 9923, context = context) #ssl._create_unverified_context())
            untrusted_conn.request("GET","/")


            # The response here should be a quote/report and not just the trusted cert
            retrieved_cert = untrusted_conn.getresponse().read()
            retrieved_cert = retrieved_cert.decode('utf-8').replace('\r','')

            trusted_server_cert = ssl.get_server_certificate((addr, attested_port))

            #Stop if certs don't match
            if(not(trusted_server_cert == retrieved_cert)):
                print("Certificates do not match")
                return

            self.conn = http.client.HTTPSConnection("localhost", 9924, context = ssl._create_unverified_context()) 

        except RuntimeError:
            print("Error connecting to server")
            ###Get attestation report and validate it here




    def upload_model(
            self,
            model: str,
            tensor_inputs: Optional[List[Tuple[List[int], ModelDatumType]]] = None,
            tensor_outputs: Optional[List[ModelDatumType]] = None,
            shape: Tuple = None,
            dtype: ModelDatumType = ModelDatumType.F32,
            dtype_out: ModelDatumType = ModelDatumType.F32,
            sign: bool = False,
            model_name: Optional[str] = None, ) -> UploadResponse:
        
        if model_name is None:
            model_name = os.path.basename(model)
        
        with open(model,"rb") as f:
            model = f.read()

        model=list(model)
        length = len(model)

        (inputs, outputs) = _get_input_output_tensors(
                None, None, shape, dtype, dtype_out
            )

        data = UploadModel(model = model, input = inputs, output = outputs, length = length, sign = False, model_name = model_name)
        data = cbor2_dumps(data.__dict__)
        self.conn.request("POST","/upload",data)
        send_model_reply = _handle_response(self.conn.getresponse())
        send_model_reply = cbor2_loads(send_model_reply)
        payload = cbor2_loads(bytes(send_model_reply['payload']))
        payload = SendModelPayload(**payload)
        ret = UploadResponse()
        ret.model_id = payload.model_id
        if sign:
            ret.payload = payload
            ret.signature = send_model_reply.signature
            #ret.attestation = 

        return ret

    def run_model(
        self,
            model_id: str,
            data_list: Union[List[List[Any]], List[Any]],
            sign: bool = False,
        ) -> RunModelResponse:

        if type(data_list[0]) != list:
            data_list = [data_list]

        #Run Model Request and Response
        data_list=list(cbor2_dumps(data_list))
        run_data = RunModel(model_id=model_id,inputs=data_list,sign=False)
        run_data = cbor2_dumps(run_data.__dict__)
        self.conn.request("POST","/run",run_data)
        resp = self.conn.getresponse()
        run_model_reply = _handle_response(resp)
        run_model_reply = cbor2_loads(run_model_reply)
        payload = cbor2_loads(bytes(run_model_reply['payload']))
        payload = RunModelPayload(**payload)

        ret = RunModelResponse()
        ret.output = payload.output

        if sign:
            ret.payload = payload
            ret.signature = run_model_reply.signature
            #ret.attestation = self.attestation

        return ret


    def delete_model(self,model_id:str):
        delete_data = DeleteModel(model_id=model_id)
        delete_data = cbor2_dumps(delete_data.__dict__)
        self.conn.request("POST","/delete",delete_data)
        _handle_response(self.conn.getresponse())


    def close(self):
        """Close the connection between the client and the inference server. This method has no effect if the file is already closed."""
        if not self.closed:
            self.closed = True
            #self.policy = None
            self.server_version = None


    def __enter__(self):
        """Return the BlindAiConnection upon entering the runtime context."""
        return self

    
    def __exit__(self, *args):
        """Close the connection to BlindAI server and raise any exception triggered within the runtime context."""
        self.close()


from functools import wraps

@wraps(BlindAiConnection.__init__, assigned=("__doc__", "__annotations__"))
def connect(*args, **kwargs):
    return BlindAiConnection(*args, **kwargs)

