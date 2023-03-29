# BlindAI Core: Overview
________________________________________

BlindAI Core is the foundation of BlindAI allowing models to be served in a Trusted Execution Environment to ensure users data remains private. You can make use of BlindAI.Core to deploy your own models with privacy guarantees for your users.

BlindAI.Core's workflow is as follows:

- [Launch](../tutorials/core/installation.md): Our server is deployed on a machine with secure enclave capabilities.
- [Remote attestation](confidential_computing.md): The remote client asks the server to provide proof that it is indeed serving a secure enclave with the right security features.
- [Prediction](../tutorials/core/quick-tour.ipynb): Once remote attestation passes, the client can send data to be safely analyzed using a TLS channel that ends inside the enclave. The AI model can be uploaded and applied, then the results are securely sent back to the user.

## Features
____________________________________________________

* Simple and fast Python API to use the service
* Model and data protected by hardware security
* Support of Remote Attestation with TLS (DCAP library)

### What you can do with BlindAI.Core

* Easily deploy state-of-the-art models with confidentiality. Run models from **BERT** for text to **ResNets** for **images**, through **WaveNet** for audio.
* Provide guarantees to third parties, for instance, clients or regulators, that you are indeed providing **data protection**- no one has access to user data in clear. Neither the AI service provider nor the Cloud provider (if applicable), can see the data.

### What you cannot do with BlindAI.Core

* Our solution aims to be modular but we have yet to incorporate tools for generic pre/post-processing. Specific pipelines can be covered but will require additional handwork for now.
* BlindAI does not cover training or federated learning, however you can check our [roadmap](https://github.com/mithril-security/blindai/projects/1) or [Discord](https://discord.gg/TxEHagpWd4) channel to know more about what we are working on and solutions.
* The **examples** we provide in our documentation are simple and do not take into account complex mechanisms such as secure storage of confidential data with sealing keys, an advanced scheduler for inference requests, or complex key management scenarios. If your use case involves more than what we show, do not hesitate to **contact us** for more information.
