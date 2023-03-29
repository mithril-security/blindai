# BlindAI Core: Overview
________________________________________

BlindAI Core is the foundation of BlindAI, allowing models to be served with assurances that users' private data will remain private. We use BlindAI.Core to serve popular AI models, but you can also use it yourself to deploy your own models with privacy guarantees for your users.

## Workflow
____________________________________________________

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

### Privacy

- Check out our [page on confidential computing](confidential_computing.md) to learn more about how BlindAI.Core protects user data.
- Check out [our guide](verify_blindai.md) on how you can verify some of BlindAI's privacy features. 

### How to verify BlindAI's security features

In the infosec industry, we often see companies producing softwares that may sound great, but clients have no way to verify the security assurances made by these companies.

On top of this, dense technical explanations can exclude some clients without in-house security experts from being able to fully understand these products.

At Mithril, we want the technologies behind BlindAI to be as understandable and transparent as possible, which is why:

- Our source code is `open source`- you can inspect the code yourself on our [GitHub page](https://github.com/mithril-security/blindai).
- We aim to provide clear explanations of the technologies behind BlindAI for users from various tech backgrounds. For example, we provide [an introduction to confidential computing ](confidential_computing.md) explaining key concepts behind BlindAI, whilst also providing more advanced security explanations in our [security section](../security/remote_attestation.md).

In addition to this, we have created [a guide](../advanced/verify_blindai.md) which will walk you through how you can verify one of our security features yourself!

The feature in question is the verification that the application code has not been modified during the attestation process. We explain what this feature is and how it works [here](confidential_computing.md)