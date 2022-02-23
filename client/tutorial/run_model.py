from transformers import DistilBertTokenizer, DistilBertForSequenceClassification
import torch
from blindai.client import BlindAiClient

tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")

sentence = "Hello, my dog is cute"
inputs = tokenizer(sentence, padding="max_length", max_length=8)["input_ids"]

client = BlindAiClient()
client.connect_server("localhost", simulation=True)

# Upload the model to the server
response = client.run_model(inputs)
