from blindai_preview.client import *
import numpy as np

client_v2 = connect(addr="localhost")

response = client_v2.upload_model(model="../../tests/simple/simple.onnx")

run_response = client_v2.run_model(
    model_id=response.model_id,
    input_tensors={"input": np.array(42), "sub": np.array(40)},
)

print("Run succesful, got", run_response.output[0].as_numpy())

client_v2.delete_model(model_id=response.model_id)
