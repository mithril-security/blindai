import http.client
import ssl
from transformers import DistilBertForSequenceClassification
from enum import IntEnum
from typing import Any, Dict, List, Optional, Tuple, Union
from transformers import DistilBertTokenizer
from cbor2 import dumps as cbor2_dumps
from cbor2 import loads as cbor2_loads

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

    

class uploadModel:
    model:List[int]
    input:List[TensorInfo]
    output:List[ModelDatumType]
    length:int

    def __init__(self,model,input,output,length):
        self.model = model
        self.input = input
        self.output = output
        self.length = length


class runModel:
    modelID:str
    inputs:List[int]

    def __init__(self,modelID,inputs):
        self.modelID=modelID
        self.inputs=inputs


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
        temp_tensorinfo = TensorInfo(fact=tensor_input[0], datum_type=tensor_input[1])
        inputs.append(temp_tensorinfo.__dict__)     #Required for correct cbor serialization
        #inputs.append(TensorInfo(fact=tensor_input[0], datum_type=tensor_input[1]))

    return (inputs, tensor_outputs)



tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
shape_inputs = tokenizer(sentence, padding = "max_length", max_length = 8, return_tensors="pt")["input_ids"]
shape=shape_inputs.shape
dtype=ModelDatumType.I64
dtype_out= ModelDatumType.F32

(inputs, outputs) = _get_input_output_tensors(
                None, None, shape, dtype, dtype_out
            )


conn = http.client.HTTPSConnection("localhost", 9976, context = ssl._create_unverified_context())

with open("distilbert-base-uncased.onnx","rb") as f:
    
    model = f.read()
    model=list(model)
    length = len(model)
    data = uploadModel(model = model, input = inputs, output = outputs, length = length)
    data = cbor2_dumps(data.__dict__)

    conn.request("POST","/upload",data)
    resp = conn.getresponse()
    model_id = resp.read()
    print(model_id)
    tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
    sentence = "I love AI and privacy!"
    run_inputs = tokenizer(sentence, padding = "max_length", max_length = 8)["input_ids"]
    
    if type(run_inputs[0]) != list:
            run_inputs = [run_inputs]
    
     
    run_inputs=list(cbor2_dumps(run_inputs))
    print(run_inputs)
    run_data = runModel(modelID=model_id,inputs=run_inputs)
    run_data = cbor2_dumps(run_data.__dict__)
    
    conn.request("POST","/run",run_data)
    resp = conn.getresponse()
    conn.close()