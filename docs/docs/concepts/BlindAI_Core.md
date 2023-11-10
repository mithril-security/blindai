---
description: "Discover BlindAI Core: Secure AI model serving with remote attestation, Python API, and hardware security for data protection."
---

# BlindAI Core: Overview
________________________________________

BlindAI Core is the foundation of BlindAI, allowing models to be served with assurances that users' private data will remain private. We use BlindAI.Core to serve popular AI models, but you can also use it yourself to deploy your own models with privacy guarantees for your users.

## Workflow
____________________________________________________

BlindAI.Core's workflow is as follows:

- [Launch](../tutorials/core/installation.md): Our server is deployed on a machine with secure enclave capabilities.
- [Remote attestation](../getting-started/confidential_computing.md): The remote client asks the server to provide proof that it is indeed serving a secure enclave with the right security features.
- [Prediction](../how-to-guides/covid_net_confidential.ipynb): Once remote attestation passes, the client can send data to be safely analyzed using a TLS channel that ends inside the enclave. The AI model can be uploaded and applied, then the results are securely sent back to the user.

## Features
_____________

* Simple and fast Python API to use the service
* Model and data protected by hardware security
* Support of Remote Attestation with TLS (DCAP library)

### What you can do with BlindAI.Core

* Easily deploy state-of-the-art models with confidentiality. Run models like **BERT** for text, **ResNets** for **images** or **WaveNet** for audio.
* Provide guarantees to third parties, for instance, clients or regulators, that you are indeed providing **data protection** - no one has access to user data in clear. Neither the AI service provider nor the Cloud provider (if applicable), can see the data.

### What you cannot do with BlindAI.Core

* Users can **only** currently **upload models in ONNX format** and perform inference using these models or delete them. In the future, we will look to expand the features and compatibility of BlindAI.
* Our solution aims to be modular but we have yet to incorporate tools for generic pre/post-processing. Specific pipelines can be covered but will require additional handwork for now.
* BlindAI does not cover training or federated learning, however you can check our [roadmap](https://github.com/mithril-security/blindai/projects/1) or [Discord](https://discord.gg/TxEHagpWd4) channel to know more about what we are working on and solutions.
* The **examples** we provide in our documentation are simple and do not take into account complex mechanisms such as secure storage of confidential data with sealing keys, an advanced scheduler for inference requests, or complex key management scenarios. If your use case involves more than what we show, do not hesitate to **contact us** for more information.
