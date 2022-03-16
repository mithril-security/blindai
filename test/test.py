from transformers import DistilBertForSequenceClassification
from transformers import DistilBertTokenizer
import torch
from blindai.client import BlindAiClient, ModelDatumType

# Load the model
model = DistilBertForSequenceClassification.from_pretrained("distilbert-base-uncased")

# Create dummy input for export
tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
inputs = tokenizer(sentence, padding = "max_length", max_length = 8, return_tensors="pt")["input_ids"]

# Export the model
torch.onnx.export(
	model, inputs, "./distilbert-base-uncased.onnx",
	export_params=True, opset_version=11,
	input_names = ['input'], output_names = ['output'],
	dynamic_axes={'input' : {0 : 'batch_size'},
	'output' : {0 : 'batch_size'}})

# Launch client
client = BlindAiClient()

client.connect_server(addr="localhost", simulation=True)

client.upload_model(model="./distilbert-base-uncased.onnx", shape=inputs.shape, dtype=ModelDatumType.I64, sign=True)
"""
	TODO: 
	- The server must give two different ids if for two uploaded models
	
"""
inputs = tokenizer(sentence, padding = "max_length", max_length = 8)["input_ids"]

response = client.run_model(inputs, sign=True)
client.export_proof()

"""
	FURTHER ENHANCEMENTS:
	- The client can request from the server a list of the uploaded model ids (if it has the right to access it)
	- Add "ok" and "msg" fields to replies (And even "ok" will be replaced by error codes in the future)
	- Probably editing the format of execution_proof file
"""
"""
	QUESTIONS:
	- Should the client be able to decide at each request if the response would be signed or not
	or fix the decision when initiating the connection? 
"""
