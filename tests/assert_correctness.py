import os
from blindai_preview.core import *
import onnxruntime as rt
import numpy as np
import sys
import torch

if len(sys.argv) != 3:
    print("usage: python assert_correctness.py <onnx_model> <npz_arg_tensors>")
    exit(1)

_, model_path, inputs_path = sys.argv

inputs = dict(np.load(inputs_path))


# Helper function to convert npz to blindai input tensors
def get_inputs(inputs: dict):
    if "mel" in inputs.keys():
        return torch.tensor(inputs.get("mel"))
    else:
        return inputs


# blindai code
if os.environ.get("BLINDAI_SIMULATION_MODE") == "true":
    client = connect(
        addr="localhost", hazmat_http_on_unattested_port=True, simulation_mode=True
    )
else:
    client = connect(addr="localhost", hazmat_http_on_unattested_port=True)

response = client.upload_model(model=model_path)
run_response = client.run_model(
    model_id=response.model_id, input_tensors=get_inputs(inputs)
)
client.delete_model(model_id=response.model_id)

# ort code
sess = rt.InferenceSession(model_path)
res = sess.run(None, inputs)

for x, y in zip(run_response.output, res):
    results = torch.isclose(x.as_torch(), torch.tensor(y)).tolist()
    if (
        isinstance(results, list) and not all(results)
    ) or not results:  # if tensor is singleton tolist returns scalar
        raise ValueError("Error discrepency between blindai's and onnxruntime outputs")

print(model_path, ": OK")
