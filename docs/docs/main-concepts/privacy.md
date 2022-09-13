# Ensuring privacy with BlindAI

In this section we will explained how BlindAI secure its users' data and what you should do to secure your models and your data using BlindAI.

## What is confidential computing ?

Confidential computing refers to the technology that can isolate a process within a protected CPU. During its executing the program runs into a **TEE** (Trusted Execution Environment). This is because nobody, not even the machine owner, can access in any way this environment, meaning that any sensible data, the source code and the program computations are isolated.

To achieve this we are relying on Intel SGX enabled CPUs. These CPUs have the ability to start a **Secure Enclave**, which is another way to say that it can execute code inside a TEE.

## Trusting BlindAI

As a user wanting privacy guarantees, here is a step-by-step list of what you should do to securely deploy or connect to BlindAI:

- Inspect the commit of the BlindAI instance, and make sure that data is not exposed. If you don’t want to, it's ok, we will have external independent auditors do it for you.
- Build the commit's enclave, and generate its policy.toml, then pass it to the client.
- If you're deploying your own BlindAI instance you should also generate new TLS certificates.

Below are explanations about how to achieve the two last steps.

## Authenticating a blindAI enclave

How does a client know that he is communicating with an authentic enclave, and how does he know it's the right one?

### Verifying the hardware

When communicating with the client, the enclave issue its signed, hardware-backed, attestation. Using the intel public key the client can eventually assess that he is comunicating with a secure enclave powered by an up to date Intel SGX CPU. The process is in reality a lot more complex than that, and [this paper](https://eprint.iacr.org/2016/086.pdf) explains such concepts much more in depth but BlindAI exists so you don't have to worry about how such things work.

### Verifying the enclave

The enclave building process will generate a policy file that contains a hash of the compilation process and some attributes like debug mode, authorized instructions and so on. In BlindAI's case, each time our client interact with our server, the server gives out its policy so that the client can compare it against its own. This way he can attest that the secure enclave he is connected to is running the right code, with the right options.

Once again the client handles this verification process so you only have to be sure that the client get the correct policy file. To do so, you should [build the server from source](../advanced/build-from-sources/server.md) in hardware mode, and follow the instruction to extract the policy file. Once you have it, you can pass it to the client during the connection like so :

```py
blindai.connect(addr="addr", policy="path/to/policy.toml")
```

If the client connects, it means the remote enclave generation process produced an identical policy.toml.

!!! info
    If you're not using simulation mode, you're required to pass a policy.toml. [Mithril cloud](../mithril-cloud.md)'s policy.toml is directly in the python package, which is why you didn't have to specify it in the quick-start example.

### Malicious image example

TODO as the malicious image is not on the dockerhub.


## Securely communicating with blindAI

### Https certificate

The `host_server.pem` file is the https certificate for the attestation server. This is used to securely communicate with the untrusted attestation server, which is used to get the SGX attestation. In production, you should generate this certificate yourself and put it in inference server.

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

### Use a certificat to communicate with a remote BlindAI's instance

You can pass the generated certificate to the client like so:

```py
blindai.connect(addr="addr", certificate="path/to/host_server.pem")
```

!!! info
    If you're not using simulation mode, you're required to pass a tls certificate. [Mithril cloud](../mithril-cloud.md)'s certificate is directly in the python package, which is why you didn't have to specify it in the quick-start example.