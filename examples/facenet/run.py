# Script to run the facenet example.
# Start by launching the server.
# Then execute in the client folder with the command python -m examples.facenet.run

from collections import namedtuple
from client.client import *
import numpy as np

client_v2 = connect(addr="0.0.0.0", simulation=True)

inputs = dict(np.load( "./facenet.npz"))
response = client_v2.upload_model(model = "./facenet.onnx")
print(response.model_id)
run_response = client_v2.run_model(model_id = response.model_id, input_tensors = inputs, sign=False)
client_v2.delete_model(model_id = response.model_id)

