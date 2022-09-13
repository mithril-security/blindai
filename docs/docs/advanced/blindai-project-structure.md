# BlindAI Project Structure

The project has several parts:

* `/client`: BlindAI client SDK, programmed with [Python programming language](https://www.python.org/).
* `/server`: the inference server, programmed using the [Rust programming language](https://www.rust-lang.org/)
  * `/server/proto`: the gRPC protobuf files, for RPC communication between the server and clients
  * `/server/blindai_app`: the host part, responsible for starting and managing the enclave
  * `/server/blindai_sgx`: the enclave part (trusted execution environment), using Intel SGX
  * `/server/blindai_sgx_attestation`: DCAP attestation library, imported from the Apache Teaclave project and modified to suit our needs
  * `/server/blindai_common`: common library used by the host and enclave
  * `/server/blindai_rpc`: common library used by the host and enclave, used for rpc communications
