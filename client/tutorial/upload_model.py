#from transformers import DistilBertTokenizer, DistilBertForSequenceClassification
#import torch
from blindai.client import BlindAiClient, ModelDatumType

"""tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
model = DistilBertForSequenceClassification.from_pretrained("distilbert-base-uncased")

sentence = "Hello, my dog is cute"
inputs = tokenizer(sentence, return_tensors="pt")["input_ids"]

torch.onnx.export(model,
                  inputs,
                  "./distilbert-base-uncased.onnx",
                  export_params=True,
                  opset_version=11,
                  do_constant_folding=True,
                  input_names = ['input'],
                  output_names = ['output'],
                  dynamic_axes={'input' : {0 : 'batch_size'},'output' : {0 : 'batch_size'}})
"""
client = BlindAiClient()
client.connect_server("localhost", certificate="policy.toml", policy="policy.toml",simulation=False)

#Upload the model to the server
response = client.upload_model(model="./distilbert-base-uncased.onnx", shape=(1, 8), dtype=ModelDatumType.I64)