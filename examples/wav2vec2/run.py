from collections import namedtuple
from client.client import *
import numpy as np

client_v2 = connect(addr="0.0.0.0", simulation=True)

inputs = dict(np.load( "./wav2vec2.npz"))
response = client_v2.upload_model(model = "./wav2vec2.onnx")
print(response.model_id)
run_response = client_v2.run_model(model_id = response.model_id, input_tensors = inputs, sign=False)
client_v2.delete_model(model_id = response.model_id)