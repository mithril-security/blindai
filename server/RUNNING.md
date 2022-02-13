# Building & running the inference-server

This repo provides two images:

- `mithrilsecuritysas/blindai-server` (hardware mode)
  This image is secure, and uses real Intel SGX enclaves. It requires SGX+FLC-compatible hardware and drivers to run.
  [Please check this link to have more information about the drivers.](https://github.com/intel/linux-sgx-driver#build-and-install-the-intelr-sgx-driver)
- `mithrilsecuritysas/blindai-server-sim` (software mode)
  This image is **not secure**, since it simulates Intel SGX in software. It is lighter than hardware mode, and should not be used in production.

## Running the server

If you wish to build the inference-server yourself, follow the [Building from source](#building-from-source) guide below first.
Otherwise, we provide prebuilt images on dockerhub.

To run the client, you will want to get the `policy.toml` file from the server using:

```sh
# For software (simulation) mode
docker run mithrilsecuritysas/blindai-server-sim /bin/cat /root/policy.toml > policy.toml
# For hardware (production) mode
docker run mithrilsecuritysas/blindai-server /bin/cat /root/policy.toml > policy.toml
```

This file is used in the client to make sure it is talking to the right enclave.

Then, you must make sure to get your TLS certificates ready. You can generate them using:

```sh
make init
```

They will be located in `./bin/tls`.

Finally, you can run the image using:

```sh
# For software (simulation) mode
docker run -p 50051:50051 -p 50052:50052 -v $(pwd)/bin/tls:/root/tls mithrilsecuritysas/blindai-server-sim
# For hardware (production) mode
docker run --device /dev/sgx/enclave --device /dev/sgx/provision -p 50051:50051 -p 50052:50052 -v $(pwd)/bin/tls:/root/tls mithrilsecuritysas/blindai-server
```

## Building from source

### Create the enclave private key

To build the inference server, we need an enclave signing key. To generate one, you can use:

```sh
make -C inference-server/scheduler/trusted Enclave_private.pem
```

The enclave signing key is located at `./inference-server/scheduler/trusted/Enclave_private.pem`.
It will be used during compilation, and removed from the docker image afterwards.

### Build the docker image

Clone this repository, and build the docker images using one of:

```sh
# For software (simulation) mode
docker build --target software -t mithrilsecuritysas/blindai-server-sim:latest . -f ./docker/build.dockerfile
# For hardware (production) mode
docker build --target hardware -t mithrilsecuritysas/blindai-server:latest . -f ./docker/build.dockerfile
```
