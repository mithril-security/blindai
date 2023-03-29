## Reproduce the enclave binary
__________________________________________

The enclave binary and the corresponding manifest are built using this [dockerfile](https://github.com/mithril-security/blindai/blob/main/.github/sgxs-release.dockerfile). You can inspect the dockerfile and run it to build the artifacts using the following commands :
```bash
docker build -f .github/sgxs-release.dockerfile --tag blindaiv2-enclave:latest .
id=$(docker create blindaiv2-enclave:latest)
docker cp "$id:/blindai/target/x86_64-fortanix-unknown-sgx/release/blindai_server.sgxs" .
docker cp "$id:/blindai/manifest.toml" .
docker rm -v $id
```

## Try it for yourself
__________________________________________

If you want to test the authenticating property of the MRENCLAVE, you can do the following:

- Build the BlindAI commit of your choice
- Add a line of code anywhere in the server part (it could be a malicious log of the input data, for example.)
- Rebuild.

You will get 2 manifest.toml files with different MRENCLAVE, which means that although the two builds are similar they will create enclaves with different identities.

To go further you can [deploy one of the 2 builds](../tutorials/core/installation.md) and try to connect it with the client by passing the manifest.toml file of the other build, which will generate an error. This confirms that if you successfully connect to a remote BlindAI instance, its code and attributes are the ones specified in the manifest.toml.

You therefore **cannot** connect to a malicious BlindAI instance as long as you correctly generate the manifest.toml.
