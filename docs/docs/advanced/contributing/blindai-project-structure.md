<meta name="description" content="Explore BlindAI's codebase structure: client, server, docs, and more for insightful organization of secure AI solutions.">

# BlindAI Project Structure

Let us introduce you to the project directory structure for BlindAI.

Below, we outline all the folders in the root directory and some key sub folders to be aware of.

- `client`: Home of our BlindAI client Python library/ PyPi package
  - `blindai`: Source files for the BlindAI client library
- `dev-container-azure`: Configuration files for our Azure dev container (our recommended environment for contributing on an Azure VM)
- `docs`: Home of all things related to documentation
    - `docs`: Contains all our documentation files within relevant subdirectories
- `src`: Source files for the BlindAI inference server
- `runner`: The modified runner enclave, used to launch the enclave code. It also contains the operations done during remote attestation
    - `remote_attestation_sgx`: DCAP's quote generation & quote verification collateral library
- `tests`: End-to-end tests

We also have the following submodules which are libraries used by our server that we have patched, largely for compatibility with the limited range of allowed operations when using Intel SGX:

- `ring-fortanix`
- `rouille`
- `tract`
- `tar-rs-sgx`
- `tiny-http`

These submodules are used for the `x86_64-fortanix-unknown-sgx` target which compiles all the libraries we will have access to within the enclave.