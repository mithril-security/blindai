from blindai.client import BlindAiClient
from PIL import Image
import numpy as np
from transformers import DistilBertTokenizer, DistilBertForSequenceClassification

checkpoint = "distilbert-base-uncased"
tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")

# Create the connection
client = BlindAiClient()
client.connect_server("localhost", simulation=True)

# Upload the model to the server
response = client.upload_model(model="./distilbert-base-uncased.onnx", shape=(1, 8), datum=client.ModelDatumType.I64)

if response.ok:
    print("Model loaded")
    tokenized = tokenizer("Hello, my dog is cute", padding = "max_length", max_length = 8)["input_ids"]
    res = client.run_model(tokenized)
    print(res.output)

client.close_connection()
