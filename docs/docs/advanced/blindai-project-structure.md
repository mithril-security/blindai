# BlindAI Project Structure

The project has several parts:

* `/client`: BlindAI client SDK, programmed with [Python programming language](https://www.python.org/).

* `/server/`: the enclave part and the inference server (trusted execution environment), using Intel SGX and fortanix EDP. 
    * `/server/runner`: The modified runner enclave, used to launch the enclave code. It also contains the operations done for the remote attestation. 
        - `/server/runner/remote_attestation_sgx`: DCAP's quote generation & quote verification collateral library. 
    * `/server/ring-fortanix`: Crypto library that is used modified to run with the fortanix EDP. 
    * `/server/tract`: patched version of tract to be ran with Intel SGX.
    * `/server/tar-rs-sgx`: tar-rs modified for Intel SGX. 
