# Securely communicating with BlindAI

## Https certificate

The `host_server.pem` file is the HTTPS certificate for the attestation server. This is used to securely communicate with the untrusted attestation server, which is used to get the SGX attestation. In production, you should generate this yourself.

## Inject your own TLS Certificate to BlindAI

The Docker image ships with a TLS certificate by default. However, its private key is directly embedded in the public Docker hub image, therefore **it is not secure**, and should be replaced in production.

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

## Use a certificate to communicate with a remote BlindAI instance

You can pass the generated certificate to the client like so:

```py
blindai.connect(addr="addr", certificate="path/to/host_server.pem")
```

!!! info
    If you're not using simulation mode, you're required to pass a TLS certificate. 