<h1 align="center">BlindAI</h1>

<h4 align="center">
  <a href="https://www.mithrilsecurity.io">Website</a> |
  <a href="https://www.linkedin.com/company/mithril-security-company">LinkedIn</a> | 
  <a href="https://www.mithrilsecurity.io">Blog</a> |
  <a href="https://www.twitter.com/mithrilsecurity">Twitter</a> | 
  <a href="https://mithrilsecurity.gitbook.io">Documentation</a>
</h4>

<h3 align="center">Fast, accessible and privacy friendly AI deployment 🚀🔒</h3>

**BlindAI** is a **fast, easy to use and confidential inference server**, allowing you to deploy your model on sensitive data. Thanks to the **end-to-end protection guarantees**, data owners can send private data to be analyzed by AI models, **without fearing exposing their data to anyone else**.

We reconcile AI and privacy by leveraging ```Intel SGX```, you can learn more about this technology here. and having technical guarantees that your model and its data will stay secured, thanks to **Confidential Computing** with ```Intel SGX```.

## Features
* Simple and fast API to use the service
* Model and data protected by hardware security
* Support of Remote Attestation with TLS (DCAP library)
* Easy to install, deploy, and maintain

## Getting started

### Deploy a ResNet model
To get started, please start this ```Docker``` image with the following command:
```bash
curl -L ... | sh
```
Please download as well the ```certificate``` of the server built in the ```Docker``` image with this command: 
```bash
curl -L ... > host_server.pem
```
You can now connect to the server using the client, using this code: 
```python
from blindai.client import BlindAiClient

#Create the connection
client = BlindAiClient()
client.connect_server(
    "localhost",
    certificate="host_server.pem",
    simulation=True
)

#Upload the model to the server
response = client.upload_model(model="./mobilenetv2-7.onnx", shape=(1, 3, 224, 224))
```

## Documentation

You can have a look to [the documentation](https://mithrilsecurity.gitbook.io/) to see what is under the hook.

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.
