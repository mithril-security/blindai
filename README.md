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
* Support ```SGX+FLC```

## Getting started

### Software/Simulation-mode server (Docker) 🐳
This ```Docker``` image provides a version of the server that allows you to test the service without having an ```Intel SGX``` ready device. 
In order to run the server in ```software/simulation mode```, you can simply run this command: 
```bash
curl -L ... | sh
```
### Hardware mode server (Docker) 🐳
You will need to have an Intel SGX ready device (with ```SGX+FLC``` support) in order to run this ```Docker``` image.
Please make sure to have the ```SGX+FLC``` drivers installed on your system before running the ```Docker``` image. [Please check this link to have more information about the drivers.](https://github.com/intel/linux-sgx-driver#build-and-install-the-intelr-sgx-driver)
```bash
curl -L ... | sh
```
### Compile the server and run it from source

In order to compile the server, you need to have the following installed on your system:
* Rust toolchain ```nightly-2021-11-01```
* Cargo & Xargo
* Intel SGX SDK 2.15.100 + PSW

You can get a Docker image having the Intel SGX SDK pre-installed [here](https://github.com/apache/incubator-teaclave-sgx-sdk#pulling-a-pre-built-docker-container). You will still need to install Xargo with the following command: 
```bash
cargo install xargo
```
Once your development environment is set up, you can compile the project with those commands: 
```bash
git clone https://github.com/mithril-security/mithril-inference-server.git
cd mithril-inference-server
make init
make
```
If you wish to compile the project in **software mode**, please compile the project with  ```make SGX_MODE=SW``` instead of ```make```. 

### Using the server

Once the server is up and running, you can start using [the client](https://github.com/mithril-security/mithril-inference-client/#usgage) to estabilish a connection with the server, upload your model and run the inference.

The server has two entrypoints : 
* **SendModel:** accepts an ```ONNX``` model file as input ```([u8])```. Will return a ```SimpleReply``` object, containing a ```bool``` indicating if the loading was a success, and a ```string``` with an error message in case of problem.
* **SendData:** accepts an array of ```f32``` as input. Will return the classification and prediction, plus a ```SimpleReply``` object.

## Documentation

You can have a look to [the documentation](https://mithrilsecurity.gitbook.io/) to see what is under the hook.

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.
