# BlindAI Client

BlindAI Client is a python library to create client applications for BlindAI Server (Mithril-security's confidential inference server). 

**If you wish to know more about BlindAI, please have a look to the project [Github repository](https://github.com/mithril-security/blindai/).**

## Installation

### Using pip
```bash
$ pip install blindai
```
## Usage

### Uploading a model

```python
from transformers import DistilBertTokenizer
from blindai.client import BlindAiClient, ModelDatumType
import torch

# Create dummy input for export
tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
inputs = tokenizer(sentence, padding = "max_length", max_length = 8, return_tensors="pt")["input_ids"]

# Export the model
torch.onnx.export(
	model, inputs, "./distilbert-base-uncased.onnx",
	export_params=True, opset_version=11,
	input_names = ['input'], output_names = ['output'],
	dynamic_axes={'input' : {0 : 'batch_size'},
	'output' : {0 : 'batch_size'}})
	
# Launch client
client = BlindAiClient()
client.connect_server(addr="localhost", policy="policy.toml", certificate="host_server.pem")
client.upload_model(model="./distilbert-base-uncased.onnx", shape=inputs.shape, dtype=ModelDatumType.I64)
```

### Uploading data
```python
from transformers import DistilBertTokenizer
from blindai.client import BlindAiClient

# Prepare the inputs
tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
inputs = tokenizer(sentence, padding = "max_length", max_length = 8)["input_ids"]

# Load the client
client = BlindAiClient()
client.connect_server(addr="localhost", policy="policy.toml", certificate="host_server.pem")

# Get prediction
response = client.run_model(inputs)
```

In order to connect to the BlindAI server, the client needs to acquire the following files from the server: 

- **policy.toml :** the enclave security policy that defines which enclave is trusted (if you are not using the simulation mode).

- **host_server.pem :** TLS certificate for the connection to the untrusted (app) part of the server.

**Simulation mode** enables to bypass the process of requesting and checking the attestation and will ignore the TLS certificate.

Before you run an example, make sure to get `policy.toml` and `host_server.pem` (if you are not using the simulation mode) that are generated in the server side.

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License
This project is licensed under [Apache 2.0 License.](https://github.com/mithril-security/blindai/blob/master/LICENSE)

The project uses the "Intel SGX DCAP Quote Validation Library" for attestation verification, See [Intel SGX DCAP Quote Validation Library License](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/License.txt)
