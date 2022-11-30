from client.client import *
import numpy as np
import os

client_v2 = connect(addr="0.0.0.0", simulation=True)

inputs = list(dict(np.load( "./simple.npz")).values())
for i in range(len(inputs)):
	inputs[i] = [inputs[i].tolist()]
print(inputs)
response = client_v2.upload_model(model = "./simple.onnx")
response = client_v2.run_model(model_id=response.model_id, input_tensors=inputs, dtypes=[ModelDatumType.I64, ModelDatumType.I64], shapes=[(1,), (1,)], sign=False)
print(response.output[0].as_flat(), response.output[0].info.node_name)