# BlindAIv2

## Setup

First you need to connect to mithril docker registry (use the credentials you've received by email).
```
docker login registry.mithrilsecurity.io
```

Then run the "Remote-Containers: Open Folder in Container..." command from the Command Palette or if available click on "Reopen in container" in the popup at the bottom-right of the window.

### Cargo integration with Fortanix EDP

`.cargo/config`  is already configured so that cargo can run SGX enclave. See the file for more information about the compilation options.

## Just command runner

Usual commands / recipes can be run with [just](https://just.systems/man/en/). 
```
$ just default
Available recipes:
    build *args        # Build for SGX target
    build-no-sgx *args # Build for a Linux target (no SGX)
    check *args        # Check for SGX target
    default
    run *args          # Run on SGX hardware
    run-no-sgx *args   # Run on a Linux target (no SGX)
    run-simu *args     # Run in the simulator
    valgrind *args     # Execute with valgrind instrumentation
```
Please refer to the justfile for details.

## Generate CA and certificates with OpenSSL CLI 
```
## Create a CA
# Generate an ECDSA secp256p1 private key
openssl ecparam -genkey -name prime256v1 -out ca.key
# Create a CA 
openssl req -x509 -new -nodes -subj "/C=US/O=_Development CA/CN=Development certificates" -key ca.key -sha256 -days 3650 -out ca.pem 

## Create server certificate
# Generate server private key
openssl ecparam -genkey -name prime256v1 -out server.key

# Create a certificate signing request (CSR)
openssl req -new -subj "/C=US/O=Local Development/CN=localhost" -key "server.key" -out "server.csr"
# Use the previously created CA to sign the CSR to get the server certificate
# 
openssl x509 -req \
    -in "server.csr" \
    -extfile "server.ext" \
    -CA ca.pem \
    -CAkey ca.key \
    -CAcreateserial \
    -out "server.pem" \
    -days 365 \
    -sha256
# Convert key into PCKS8 DER format 
openssl pkcs8 -topk8 -inform PEM -outform PEM -in server.key -out server2.key -nocrypt
```

## Run 

```
# Launch server in a terminal
$ cargo run
Now listening on port 9923 and 9924
-- (when running the python client)
Retrieve and send attestation report to client here
Retrieve and send attestation report to client here
/upload
Some("distilbert-base-uncased.onnx")
Successfully saved model
/run
Successfully ran model
/delete
Deleted model successfully

# In another terminal test the server with a sample client
$ cd client
# The first time you need to export the distilbert model in onnx with
$ poetry run python client/distilbert_setup.py
# Then run the client
$ poetry run python client/sample_client.py
WARNING:root:Untrusted server certificate check bypassed
b'Deleted'
``` 

## Python Client

Dependencies are managed with [Poetry](https://python-poetry.org/).
Ressources:
  * Jupyter as a dev dependency : https://hippocampus-garden.com/jupyter_poetry_pipenv/
