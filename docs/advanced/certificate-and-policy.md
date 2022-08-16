# Certificate and policy

### What are host_server.pem and policy.toml?

The `host_server.pem` file is the https certificate for the attestation server. This is used to securely communicate with the untrusted attestation server, which is used to get the SGX quote. In production, you should generate this certificate yourself and put it in inference server. To do this, read the next sections.

The `policy.toml` file is used to specify which enclave should be accepted by the client. It basically contains the hash of the server binary, and the SGX flags the server should be run with. The client uses it to validate the quote sent by the server.

### Extract Policy and default TLS Certificate from the Hardware docker image

You can extract the policy directly from the prebuilt Docker Image using:



```bash
docker run --rm mithrilsecuritysas/blindai-server:latest /bin/cat /root/policy.toml > policy.toml
```



```
docker run --rm mithrilsecuritysas/blindai-server-dcsv3:latest /bin/cat /root/policy.toml > policy.toml
```



You can also extract the default TLS certificate like this:



```
docker run --rm mithrilsecuritysas/blindai-server:latest /bin/cat /root/tls/host_server.pem > host_server.pem
```



```
docker run --rm mithrilsecuritysas/blindai-server-dcsv3:latest /bin/cat /root/tls/host_server.pem > host_server.pem
```



### Inject your own TLS Certificate to BlindAI

As you read above, the Docker image ships with a TLS certifcate by default. However, its private key is directly embedded in the public Docker hub image, therefore **it is not secure**, and should be replaced in production.

To generate a new self-signed TLS certificate, you can run

```
mkdir tls
openssl req -newkey rsa:2048 -nodes -keyout tls/host_server.key -out tls/host_server.pem -x509 -days 365
```

Once you have generated your TLS certificate, you can use it with the project using a docker volume:



```bash
docker run \
    -v $(pwd)/tls:/root/tls \
    -p 50051:50051 \
    -p 50052:50052 \
    --device /dev/sgx/enclave \
    --device /dev/sgx/provision \
    mithrilsecuritysas/blindai-server:latest /root/start.sh PCCS_API_KEY
```



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
