# Overview
________________________________________

BlindAI workflow is simple:

- [Launch](../getting-started/installation.md): Our server is deployed on a machine with secure enclave capabilities.
- [Remote attestation](privacy.md): The remote client asks the server to provide proof that it is indeed serving a secure enclave with the right security features.
- [Prediction](../../index.md): Once remote attestation passes, the client can send data to be safely analyzed using a TLS channel that ends inside the enclave. The AI model can be uploaded and applied, then the result is sent securely.

## Features
____________________________________________________

* Simple and fast API to use the service
* Model and data protected by hardware security
* Support of Remote Attestation with TLS (DCAP library)
* Easy to install, deploy, and maintain

### What you can do with BlindAI

* Easily deploy state-of-the-art models with confidentiality. Run models from **BERT** for text to **ResNets** for **images**, through **WaveNet** for audio.
* Provide guarantees to third parties, for instance, clients or regulators, that you are indeed providing **data protection**, through **code attestation**.
* Explore different scenarios from confidential _Speech-to-text_ to _biometric identification_, through secure document analysis with our pool of **examples**.

### What you cannot do with BlindAI

* Our solution aims to be modular but we have yet to incorporate tools for generic pre/post-processing. Specific pipelines can be covered but will require additional handwork for now.
* BlindAI does not cover training or federated learning, however you can check our [roadmap](https://github.com/mithril-security/blindai/projects/1) or [Discord](https://discord.gg/TxEHagpWd4) channel to know more about what we are working on and solutions.
* The examples we provide are simple and do not take into account complex mechanisms such as secure storage of confidential data with sealing keys, an advanced scheduler for inference requests, or complex key management scenarios. If your use case involves more than what we show, do not hesitate to **contact us** for more information.
