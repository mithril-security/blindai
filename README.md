<p align="center">
  <img src="assets/logo.png" alt="BlindAI" width="200" height="200" />
</p>

<h1 align="center">Mithril Security - BlindAI</h1>

<h4 align="center">
  <a href="https://www.mithrilsecurity.io">Website</a> |
  <a href="https://www.linkedin.com/company/mithril-security-company">LinkedIn</a> | 
  <a href="https://blog.mithrilsecurity.io/">Blog</a> |
  <a href="https://www.twitter.com/mithrilsecurity">Twitter</a> | 
  <a href="https://docs.mithrilsecurity.io/">Documentation</a> |
  <a href="https://discord.gg/TxEHagpWd4">Discord</a>
</h4>

<h3 align="center">Fast, accessible and privacy friendly AI deployment 🚀🔒</h3>

**BlindAI** is a **fast, easy to use and confidential inference server**, allowing you to deploy your 
model on sensitive data. Thanks to the **end-to-end protection guarantees**, data owners can send private data to be analyzed by AI models, **without fearing exposing their data to anyone else**.

![Overview of BlindAI](assets/blindai-schema.png)

While most current solutions force users to trust the server where their data is sent, BlindAI leverages Confidential Computing to make sure that even when you send data to a third-party to have it analyzed by an AI, it remains always protected and inacessible.

You can learn more about Confidential Computing through our series [here](https://blog.mithrilsecurity.io/confidential-computing-explained-part-1-introduction/).

We currently only support *Intel SGX*, but we plan to cover *AMD SEV* and *Nitro Enclave* in the future. More information about our **roadmap** will be provided soon.

Our solution comes in two parts:
- A secure inference solution to serve AI models with privacy guarantees.
- A *client SDK* to securely consume the remote AI models. 

## Examples

You can see how our BlindAI can be used to deploy a variety of models, to handle use cases from analysis of medical images, to confidential document analysis, through vocal biometrics identification.

| Article name                                                   | Use case                                | Model name      | Model type   |
|----------------------------------------------------------------|-----------------------------------------|-----------------|--------------|
| [Deploy Transformers models with confidentiality](https://blog.mithrilsecurity.io/transformers-with-confidentiality/)                | Sentiment analysis                      | DistilBERT      | Transformers |
| [Confidential medical image analysis with COVID-Net and BlindAI](https://blog.mithrilsecurity.io/confidential-covidnet-with-blindai/) | Chest XRAY analysis for COVID detection | COVID-Net-CXR-2 | Deep CNN     |

## Getting started

To deploy a model on sensitive data, with end-to-end protection, we provide a *Docker* image to serve models with confidentiality, and a *client SDK* to consume this service securely.

### Note

Because the server requires specific hardware, for instance *Intel SGX* currently, we also provide a *simulation mode*. Using the *simulation mode*, any computer can serve models with our solution. However, the two key properties of secure enclaves, data in use confidentiality, and code attestation, will not be available. **Therefore this is just for testing on your local machine but is not relevant for real guarantees in production**.

Our first article [Deploy Transformers with confidentiality](https://blog.mithrilsecurity.io/transformers-with-confidentiality) covers the deployment of both simulation and hardware mode. 

### A - Deploying the server

Deploy the inference server, for instance using one of our *Docker* images. To get started quickly, you can use the image with simulation, which does not require any specific hardware. 
```bash
docker run -p 50051:50051 -p 50052:50052 mithrilsecuritysas/blindai-server-sim
```
### B - Sending data from the client to the server

Our *client SDK* is rather simple, but behind the scenes, a lot happens. If we are talking to a real *enclave* (simulation=False), the client actually verifies we are indeed talking with an *enclave* with the right security properties, such as the code loaded inside the enclave or security patches applied. Once those checks pass, data or model can be uploaded safely, with *end-to-end protection* through a *TLS* tunnel ending inside the enclave. Thanks to the data in use, protection of the *enclave* and verification of the code, everything sent remotely will not be exposed to any third party.

You can learn more about the attestation mechanism for code integrity [here](https://sgx101.gitbook.io/sgx101/sgx-bootstrap/attestation).

#### i - Upload the model

Then we need to load a model inside the secure inference server. First we will export our model from *Pytorch* to *ONNX*, then we can upload it securely to the inference server. Uploading the model through our API allows the model to be kept confidential, for instance when deploying it on foreign infrastructure, like Cloud or client on-premise. 
```python
from transformers import DistilBertTokenizer, DistilBertForSequenceClassification
import torch
from blindai.client import BlindAiClient, ModelDatumType

# Get pretrained model
model = DistilBertForSequenceClassification.from_pretrained("distilbert-base-uncased")

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
client.connect_server(addr="localhost", simulation=True)
client.upload_model(model="./distilbert-base-uncased.onnx", shape=inputs.shape, dtype=ModelDatumType.I64)
```

#### ii - Send data and run model
Upload the data securely to the inference server. 
```python
from transformers import DistilBertTokenizer
from blindai.client import BlindAiClient

tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")

sentence = "I love AI and privacy!"
inputs = tokenizer(sentence, padding = "max_length", max_length = 8)["input_ids"]

# Load the client
client = BlindAiClient()
client.connect_server("localhost", simulation=True)

# Get prediction
response = client.run_model(inputs)
```

### What you can do with BlindAI

- Easily deploy state-of-the-art models with confidentiality. Run models from **BERT** for text to **ResNets** for **images**, through **WaveNet** for audio.
- Provide guarantees to third parties, for instance clients or regulators, that you are indeed providing **data protection**, through **code attestation**.
- Explore different scenarios from confidential *Speech-to-text*, to *biometric identification*, through secure document analysis with our pool of **examples**.

### What you cannot do with BlindAI

- Our solution aims to be modular but we have yet to incorporate tools for generic pre/post processing. Specific pipelines can be covered but will require additional handwork for now.
- We do not cover training and federated learning yet, but if this feature interests you do not hesitate to show your interest through the [roadmap](https://blog.mithrilsecurity.io/our-roadmap-at-mithril-security/) or [Discord](https://discord.gg/TxEHagpWd4) channel. 
- The examples we provide are simple, and do not take into account complex mechanisms such as secure storage of confidential data with sealing keys, advanced scheduler for inference requests, or complex key management scenarios. If your use case involves more than what we show, do not hesitate to **contact us** for more information.

## Install

### A - Server

Our inference server can easily be deployed through our Docker images. You can pull it from our *Docker* repository or build it yourself. 

### B - Client

We advise you to install our *client SDK* using a *virtual environment*. You can simply install the client using *pip* with:
```bash
pip install blindai
```
You can find more details regarding the installation in our [**documentation here**](https://docs.mithrilsecurity.io/started/installation/).

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## Telemetry

BlindAI collects anonymous data regarding general usage, this allows us to understand how you are using the project. We only collect data regarding the execution mode (Hardware/Software) and the usage metrics. 

This feature can be easily disabled, by settin up the environment variable ```BLINDAI_DISABLE_TELEMETRY``` to 1.

You can find more information about the telemetry in our [**documentation**](https://docs.mithrilsecurity.io/telemetry/).

## Disclaimer
BlindAI is still being developed and is provided as is, use it at your own risk.
