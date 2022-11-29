

from transformers import DistilBertTokenizer
import torch
from transformers import DistilBertForSequenceClassification

if __name__ == "__main__":
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
