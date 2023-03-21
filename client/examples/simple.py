from blindai_preview.core import *
import numpy as np

# For test purpose, we want to avoid setting a TLS reverse proxy on top of
# the untrusted port. We pass the hazmat_http_on_untrusted_port = True argument
# to allow connecting to the untrusted port using plain HTTP instead of HTTPS.
# This option is hazardous therefore it starts with hazmat_
# Those options should generally not be used in production unless you
# have carefully assessed the consequences.
client_v2 = connect(addr="localhost", hazmat_http_on_unattested_port=True)

response = client_v2.upload_model(model="../../tests/simple/simple.onnx")

run_response = client_v2.run_model(
    model_id=response.model_id,
    input_tensors={"input": np.array(42), "sub": np.array(40)},
)

print("Run succesful, got", run_response.output[0].as_numpy())

client_v2.delete_model(model_id=response.model_id)
