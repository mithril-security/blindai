from client.client import *
import onnxruntime as rt
import numpy as np
import sys
import torch

if len(sys.argv) != 3:
	print("usage: python assert_correctness.py <onnx_model> <npz_arg_tensors>")
	exit(1)

_, model_path, inputs_path = sys.argv
inputs = dict(np.load(inputs_path))

#blindai code
client_v2 = connect(addr="0.0.0.0", simulation=True)
response = client_v2.upload_model(model=model_path)
run_response = client_v2.run_model(model_id=response.model_id, input_tensors=inputs, sign=False)
client_v2.delete_model(model_id = response.model_id)

#ort code
sess = rt.InferenceSession(model_path)
res = sess.run(None, inputs)

for x, y in zip(run_response.output, res):
	results = torch.isclose(x.as_torch(), torch.tensor(y)).tolist()
	if (isinstance(results, list) and not all(results)) or not results: #if tensor is singleton tolist returns scalar
		raise ValueError("Error discrepency between blindai's and onnxruntime outputs")

print(model_path, ": OK")