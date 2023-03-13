!!! warning
    This is a preview version of BlindAI named blindai-preview. It has not yet all the features of the current BlindAI. You can find the current BlindAI [here](https://github.com/mithril-security/blindai).


**BlindAI** is a **fast, easy-to-use,** and **confidential inference server**, allowing you to easily and quickly deploy your AI models with privacy, **all in Python**. Thanks to the **end-to-end protection guarantees**, data owners can send private data to be analyzed by AI models, without fearing exposing their data to anyone else.

To interact with an AI model hosted on a remote secure enclave, we provide the `blindai_preview` Python client library. This client will:

- check that you are talking to a genuine and secure enclave with the right security features
- upload an ML model that was previously converted to ONNX
- query the model securely

## Set up

!!! info
    The following instructions assume you already have access to a configured SGX enabled machine, such as one provided by Mithril Security. If that is not the case, you can either ask for one, or read [the Documentation](docs/cloud-deployment.md).

Start by cloning the repo **with --recursive** to get the submodules.
```bash
git clone --recursive https://github.com/mithril-security/blindai-preview.git
``` 

!!! warning
    If you are on an Azure machine, you should replace the .devcontainer folder by the devcontainer-azure/.devcontainer folder.

Open the repo in vscode and `Reopen in Container`.

Then, at the root of the project:

```
cd client
poetry install
poetry shell
```

Which creates a virtual environment and install the client sdk.


## Getting Started

Now let's run a simple example to check everything is correctly set up.

At the root of the project, run
```
python tests/simple/setup.py
```
which generates the onnx file for a very minimal model which only does one thing: subbing one monovalue tensor to another.

Then, on one tab, start the server
```
just run --release # blindai_server is extremely slow in debug build
```

And on another, connect to it with a client (once it is up)
```
cd client/examples
python simple.py
```

Here is the code of simple.py
```py
from blindai_preview.client import *
import numpy as np

# For test purpose, we want to avoid setting a TLS reverse proxy on top of
# the unattested port. We pass the hazmat_http_on_unattested_port = True argument
# to allow connecting to the unattested port using plain HTTP instead of HTTPS.
# This option is hazardous therefore it starts with hazmat_
# Those options should generally not be used in production unless you
# have carefully assessed the consequences.
client_v2 = connect(addr="localhost", hazmat_http_on_unattested_port=True)

response = client_v2.upload_model(model="../../tests/simple/simple.onnx")

run_response = client_v2.run_model(
    model_id=response.model_id,
    input_tensors={"input: ": np.array(42), "sub": np.array(40)},
)

print("Ran successfully, got: ", run_response.output[0].as_numpy())

client_v2.delete_model(model_id=response.model_id)

```

## justfile overview

You can use the justfile to do recurrent developer tasks.

- ```just run [args]``` to run the server in debug mode
- ```just release [args]```to run the server in release mode
- ```just test``` to run all the tests
- ```just doc``` to build and serve the doc


## Testing

First, generates all of the necessary onnx and inputs (npz) files with:
```
bash tests/generate_all_onnx_and_npz.sh
```

Then ```just test``` run the client unittests, the end-to-end tests, and serve a pretty coverage report.
