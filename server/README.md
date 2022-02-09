<h1 align="center">BlindAI - Server</h1>

## Run the server

### Software/Simulation-mode server (Docker) 🐳
This ```Docker``` image provides a version of the server that allows you to test the service without having an ```Intel SGX``` ready device. 
In order to run the server in ```software/simulation mode```, you can simply run this command: 
```bash
sudo docker run -p 50051:50051 -p 50052:50052 blindai-server-sim:0.1.0
```
### Hardware mode server (Docker) 🐳
You will need to have an Intel SGX ready device (with ```SGX+FLC``` support) in order to run this ```Docker``` image.
Please make sure to have the ```SGX+FLC``` drivers installed on your system before running the ```Docker``` image. [Please check this link to have more information about the drivers.](https://github.com/intel/linux-sgx-driver#build-and-install-the-intelr-sgx-driver)
```bash
sudo docker run -p 50051:50051 -p 50052:50052 --device /dev/sgx/enclave --device /dev/sgx/provision blindai-server:0.1.0 API_KEY
```
### TLS certificate and policy of servers from the Docker images
If you intend to use those docker images, you will need this certificate and the policy to use the client.
```bash
curl -L ... > host_server.pem
curl -L ... > policy.toml
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
git clone https://github.com/mithril-security/blindai.git
cd blindai/server
make init
make
```
Two files will be generated after the building process:
- **policy.toml :** the enclave security policy that defines which enclave is trusted.
- **host_server.pem :** TLS certificate for the connection to the untrusted (app) part of the server.

Those two files will be needed when you will use the client. You can have a look to the documentation or the [client readme](https://github.com/mithril-security/blindai/tree/master/client#usage) for more informations.

## Documentation

You can have a look to [the documentation](https://mithrilsecurity.gitbook.io/) to see what is under the hook. 

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.
