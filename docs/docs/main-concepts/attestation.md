# Attesting a BlindAI enclave

How does a client know that he is communicating with an authentic enclave, and how does he know it's the right one?

## Verifying the hardware

When communicating with the client, the enclave issues its signed, hardware-backed, attestation. Using the intel public key the client can eventually assess that he is communicating with a secure enclave powered by an up-to-date Intel SGX CPU. The process is in reality a lot more complex than that, and [this paper](https://eprint.iacr.org/2016/086.pdf) explains these concepts much more in-depth. Note however that BlindAI's goal is to abstract these complicated processes while keeping the same privacy guarantees.

## Verifying the enclave

The enclave building process will generate a policy file that contains a hash of the compilation process and some attributes like debug mode, authorized instructions and so on. This is also referred as the MRENCLAVE and it is sufficient to safely authenticate an enclave. In BlindAI's case, each time our client interacts with our server, the server gives out its MRENCLAVE so that the client can compare it against the policy file passed by the user. This way he can attest that the secure enclave he is connected to is running the right code, with the right options.

Once again the client handles this verification process so you only have to be sure that the client gets the correct policy file. To do so, you should [build the server from source](../advanced/build-from-sources/server.md) in hardware mode, and follow the instruction to extract the policy file. Once you have it, you can pass it to the client during the connection like so :

```py
blindai.connect(addr="addr", policy="path/to/policy.toml")
```

If the client connects, it means the remote enclave generation process produced an identical policy.toml.

!!! info
    If you're not using simulation mode, you're required to pass a policy.toml. [Mithril cloud](../mithril-cloud.md)'s policy.toml is directly in the python package, which is why you didn't have to specify it in the quick-start example.

## Try it for yourself

If you want to test the authenticating property of the MRENCLAVE, you can do the following:

- [Build the BlindAI commit of your choice](../advanced/build-from-sources/server.md).
- Add a line of code anywhere in the server part (it could be a malicious log of the input data, for example.)
- Rebuild.

You will get 2 policy.toml files with different MRENCLAVE, which means that although the two builds are similar they will create enclaves with different identities.

To go further you can [deploy one of the 2 builds](../deploy-on-premise.md) and try to connect it with the client by passing the policy.toml file of the other build, which will generate an error. This confirms that if you successfully connect to a remote BlindAI instance, its code and attributes are the ones specified in the policy.toml.

You therefore **cannot** connect to a malicious BlindAI instance as long as you correctly generate the policy.toml.