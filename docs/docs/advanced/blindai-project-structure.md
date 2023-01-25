# BlindAI Project Structure

The project has several parts:

* `/client`: BlindAI client SDK, programmed with [Python programming language](https://www.python.org/).

* `/server`: the enclave part and the inference server (trusted execution environment), using Intel SGX and fortanix EDP. 
    * `runner`: The modified runner enclave, used to launch the enclave code. It also contains the operations done for the remote attestation. 
        * `remote_attestation_sgx`: DCAP's quote generation & quote verification collateral library. 
    * `ring-fortanix`, `tract`, `tar-rs-sgx`, `tiny-http` and `rouille` submodules are crates patched for the `x86_64-fortanix-unknown-sgx` target. 
