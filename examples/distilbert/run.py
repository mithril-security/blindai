from collections import namedtuple
from client.client import *
from transformers import DistilBertTokenizer
from transformers import DistilBertForSequenceClassification

client_v2 = connect(addr="0.0.0.0", simulation=True)

tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
shape_inputs = tokenizer(sentence, padding = "max_length", max_length = 8, return_tensors="pt")["input_ids"]
shape=shape_inputs.shape
dtype=ModelDatumType.I64
dtype_out= ModelDatumType.F32
response = client_v2.upload_model(model = "./distilbert-base-uncased.onnx", shape = shape, dtype = dtype, dtype_out = dtype_out)


tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
run_inputs = tokenizer(sentence, padding = "max_length", max_length = 8)["input_ids"]

print(response.model_id)

run_response = client_v2.run_model(model_id = response.model_id, input_tensors = run_inputs, shapes = (len(run_inputs),), dtypes = ModelDatumType.I64)

print(run_response.output[0].as_flat())

client_v2.delete_model(model_id = response.model_id)
