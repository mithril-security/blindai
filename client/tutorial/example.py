from blindai.client import BlindAiClient
from PIL import Image
import numpy as np
import torch

# Create the connection
client = BlindAiClient()

client.connect_server(
    "localhost", policy="policy.toml", certificate="host_server.pem", simulation=False
)

# Upload the model to the server
response = client.upload_model(model="./resnet18-v1-7.onnx", shape=(1, 3, 224, 224))

if response.ok:
    print("Model loaded")
    image = Image.open("kitten.jpg").resize((224, 224))

    # Preprocess the data
    a = np.asarray(image, dtype=float)

    # Send data for inference
    result = client.send_data(a.flatten())
    print(result.classification)
    print(result.prediction)

client.close_connection()
