from blindai.client import BlindAiClient
from PIL import Image
import numpy as np
from transformers import AutoTokenizer

checkpoint = "distilbert-base-uncased"
tokenizer = AutoTokenizer.from_pretrained(checkpoint)

# Create the connection
client = BlindAiClient()
client.connect_server(
    "localhost", policy="policy.toml", certificate="host_server.pem", simulation=True
)

# Upload the model to the server
response = client.upload_model(model="./final_confidential_transformers_classifier.onnx", shape=(1, 20), datum=client.ModelDatumType.I64)

if response.ok:
    print("Model loaded")
    tokenized = tokenizer("I love potatoes", padding = "max_length", max_length = 20)["input_ids"]
    res = client.run_model(tokenized)
    print(res.output)

client.close_connection()
