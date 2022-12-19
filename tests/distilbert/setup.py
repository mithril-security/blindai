import torch
from transformers import DistilBertTokenizer
from transformers import DistilBertForSequenceClassification
import numpy as np
import os
from onnxsim import simplify
import onnx

path = os.path.dirname(os.path.realpath(__file__))

# Load the model
model = DistilBertForSequenceClassification.from_pretrained("distilbert-base-uncased")

# Create dummy input for export
tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
inputs = tokenizer(sentence, padding = "max_length", max_length = 8, return_tensors="pt")["input_ids"]

# Export the model
torch.onnx.export(
	model, inputs, path + "/distilbert-base-uncased.onnx",
	export_params=True, opset_version=11,
	input_names = ['input'], output_names = ['output'],
	dynamic_axes={'input' : {0 : 'batch_size'},
	'output' : {0 : 'batch_size'}})

model = onnx.load(path + "/distilbert-base-uncased.onnx")
model_simp, check = simplify(model)
assert check, "Simplified ONNX model could not be validated"
onnx.save(model_simp, path + "/distilbert-base-uncased.onnx")

# save inputs to npz format
inputs = {"input": tokenizer(sentence, padding = "max_length", max_length = 8, return_tensors="np")["input_ids"]}
np.savez(path + "/distilbert-base-uncased.npz", **inputs)