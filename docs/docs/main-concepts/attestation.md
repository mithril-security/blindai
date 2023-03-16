# Attesting a BlindAI enclave

How does a client know that he is communicating with an authentic enclave, and how does he know it's the right one?

## Verifying the hardware

When communicating with the client, the enclave issues its signed, hardware-backed, attestation. Using the intel public key the client can eventually assess that he is communicating with a secure enclave powered by an up-to-date Intel SGX CPU. The process is in reality a lot more complex than that, and [this paper](https://eprint.iacr.org/2016/086.pdf) explains these concepts much more in-depth. Note however that BlindAI's goal is to abstract these complicated processes while keeping the same privacy guarantees.

## Verifying the enclave

The enclave building process will generate a manifest file that contains a hash of the compilation process and some attributes like debug mode, authorized instructions and so on. This is also referred as the MRENCLAVE and it is sufficient to safely authenticate an enclave. In BlindAI's case, each time our client interacts with our server, the server gives out its MRENCLAVE so that the client can compare it against the manifest file passed by the user. This way he can attest that the secure enclave he is connected to is running the right code, with the right options.

!!! info
    To offer good and secure defaults, each client release comes with a builtin Manifest.toml corresponding to the latest release of BlindAI, this Manifest is used by default. Most users are expected to just use the default Manifest.toml. 
    
If you want to modify the server, you will need to override the Manifest file.To do that you can use the `hazmat_manifest_path` argument of the connect function. 


The client handles this verification process so you only have to make sure that the client gets the correct manifest file. If the client connects, it means the remote enclave is compliant to the enclave description from the Manifest.


## Reproduce the enclave binary

The enclave binary and the corresponding manifest are built using this [dockerfile](https://github.com/mithril-security/blindai-preview/blob/main/.github/sgxs-release.dockerfile). You can inspect the dockerfile and run it to build the artifacts using the following commands :
```bash
docker build -f .github/sgxs-release.dockerfile --tag blindaiv2-enclave:latest .
id=$(docker create blindaiv2-enclave:latest)
docker cp "$id:/blindai-preview/target/x86_64-fortanix-unknown-sgx/release/blindai_server.sgxs" .
docker cp "$id:/blindai-preview/manifest.toml" .
docker rm -v $id
```



## Try it for yourself

If you want to test the authenticating property of the MRENCLAVE, you can do the following:

- Build the BlindAI commit of your choice
- Add a line of code anywhere in the server part (it could be a malicious log of the input data, for example.)
- Rebuild.

You will get 2 manifest.toml files with different MRENCLAVE, which means that although the two builds are similar they will create enclaves with different identities.

To go further you can [deploy one of the 2 builds](../getting-started/installation.md) and try to connect it with the client by passing the manifest.toml file of the other build, which will generate an error. This confirms that if you successfully connect to a remote BlindAI instance, its code and attributes are the ones specified in the manifest.toml.

You therefore **cannot** connect to a malicious BlindAI instance as long as you correctly generate the manifest.toml.