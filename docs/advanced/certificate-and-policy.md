
# Certificate and policy

## Certificate

### What is the host_server.pem ?

The `host_server.pem` file is the https certificate for the attestation server. This is used to securely communicate with the untrusted attestation server, which is used to get the SGX quote. In production, you should generate this certificate yourself and put it in inference server. To do this, read the next sections.


### Inject your own TLS Certificate to BlindAI

The Docker image ships with a TLS certifcate by default. However, its private key is directly embedded in the public Docker hub image, therefore **it is not secure**, and should be replaced in production.

To generate a new self-signed TLS certificate, you can run

```
mkdir tls
openssl req -newkey rsa:2048 -nodes -keyout tls/host_server.key -out tls/host_server.pem -x509 -days 365
```

Once you have generated your TLS certificate, you can use it with the project using a docker volume:

=== "Hardware mode"

    ```bash
    docker run \
        -v $(pwd)/tls:/root/tls \
        -p 50051:50051 \
        -p 50052:50052 \
        --device /dev/sgx/enclave \
        --device /dev/sgx/provision \
        mithrilsecuritysas/blindai-server:latest /root/start.sh PCCS_API_KEY
    ```

=== "Hardware mode (Azure DCsv3 VMs)"

    ```bash
    docker run \
        -v $(pwd)/tls:/root/tls \
        -p 50051:50051 \
        -p 50052:50052 \
        --device /dev/sgx/enclave \
        --device /dev/sgx/provision \
        mithrilsecuritysas/blindai-server-dcsv3:latest
    ```

`-v $(pwd)/tls:/root/tls` allows you to mount your own TLS certificate to the Docker Image.&#x20;

## Policy

### What is the policy ?

The policy.toml file is generated during the enclave build and is a specification of :

- The attributes of the enclaves (authorized cpu instructions, sgx flags and options), they are introduced in [the chapter 37 of the intel documentation](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3d-part-4-manual.pdf)

- The mr_enclave : this is the enclave’s identity, a cryptographic hash of the enclave log as it goes through every step of the build and initialization process. The mr_enclave uniquely identifies any particular enclave, so different builds/versions of an enclave will result in different mr_enclave values. Thus, sealed data will not be available to different versions of the same enclave.

### Ensuring you have the good policy

When in hardware mode, the client will check if the policy you passed it is **identical** to the server's. So all that is left is to ensure that the policy you pass to the client is authentic. To do so, you should generate it by [building the server from source](server-from-sources.md) in hardware mode.
Once you pass it to the client, you will have the guarantee you are connected to an enclave running the exact same code that you compiled.
