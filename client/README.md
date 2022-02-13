# BlindAI Client

BlindAI Client is a python library to create client applications for BlindAI Server (Mithril-security's confidential inference server). 

## Installation

### Using pip
```bash
$ pip install blindai
```
## Usage

### Uploading a model

```python
from blindai.client import BlindAiClient

#Create the connection
client = BlindAiClient()
client.connect_server(
    "localhost",
    policy="policy.toml",
    certificate="host_server.pem",
    simulation=False
)

#Upload the model to the server
response = client.upload_model(model="./mobilenetv2-7.onnx", shape=(1, 3, 224, 224), datum=client.ModelDatumType.F32)
```
### Uploading data
```python
from blindai.client import BlindAiClient
from PIL import Image
import numpy as np

#Create the connection
client = BlindAiClient()
client.connect_server(
    "localhost",
    policy="policy.toml",
    certificate="host_server.pem",
    simulation=False
)

image = Image.open("grace_hopper.jpg").resize((224,224))
a = np.asarray(image, dtype=float)

#Send data for inference
result = client.run_model(a.flatten())
```

In order to connect to the BlindAI server, the client needs to acquire the following files from the server: 

- **policy.toml :** the enclave security policy that defines which enclave is trusted (if you are not using the simulation mode).

- **host_server.pem :** TLS certificate for the connection to the untrusted (app) part of the server.

**Simulation mode** enables to pypass the process of requesting and checking the attestation and will ignore the TLS certificate.

Usage examples can be found in [tutorial](./tutorial) folder.

Before you run an example, make sure to get `policy.toml` and `host_server.pem` (if you are not using the simulation mode) that are generated in the server side. 

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License
This project is licensed under [Apache 2.0 License.](../LICENSE)
The project uses the "Intel SGX DCAP Quote Validation Library" for attestation verification, See [Intel SGX DCAP Quote Validation Library License](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/License.txt)
