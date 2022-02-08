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
    tokenized = tokenizer("I love potatoes")["input_ids"]
    lst = [0] * 20
    for i in range(len(tokenized)):
        lst[i] = tokenized[i]
    print(lst)
    res = client.send_data(lst)
    print(res.output)

client.close_connection()
