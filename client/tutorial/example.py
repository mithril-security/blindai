from blindai.client import BlindAiClient
from PIL import Image
import numpy as np

# Create the connection
client = BlindAiClient()

client.connect_server(
    "localhost", policy="policy.toml", certificate="host_server.pem", simulation=False
)

# Upload the model to the server
response = client.upload_model(model="./mobilenetv2-7.onnx", shape=(1, 3, 224, 224))

if response.ok:
    print("Model loaded")
    image = Image.open("grace_hopper.jpg").resize((224, 224))

    # Preprocess the data
    a = np.asarray(image, dtype=float)
    mean = np.array([0.485, 0.456, 0.406])
    std = np.array([0.229, 0.224, 0.225])
    a = (a / 255.0 - mean) / std
    a = np.moveaxis(a, 2, 0)

    # Send data for inference
    result = client.send_data(a.flatten())

client.close_connection()
