---
description: 'BlindAI: Fast, accessible, and privacy-friendly AI deployment 🚀🔒'
---

# 👋 Welcome

### What is **BlindAI?**&#x20;

**BlindAI** is a **fast, easy-to-use,** and **confidential inference server**, allowing you to deploy your model on sensitive data. Thanks to the **end-to-end protection guarantees**, data owners can send private data to be analyzed by AI models, **without fearing exposing their data to anyone else**.

We reconcile _AI_ and privacy by leveraging Confidential Computing for secure inference. You can learn more about this technology here.

We currently only support _Intel SGX_, but we plan to cover _AMD SEV_ and _Nitro Enclave_ in the future. More information about our **roadmap** can be found [here](https://github.com/mithril-security/blindai/projects/1).

Our solution comes in two parts:

* A secure inference solution to serve _AI_ models with privacy guarantees.
* A _client SDK_ to securely consume the remote _AI_ models.

### Latest versions

* **BlindAI server:** _0.3.0_
* **BlindAI client:** _0.3.1_

### Features

* Simple and fast API to use the service
* Model and data protected by hardware security
* Support of Remote Attestation with TLS (DCAP library)
* Easy to install, deploy, and maintain
* Support `SGX+FLC`

#### What you can do with BlindAI

* Easily deploy state-of-the-art models with confidentiality. Run models from **BERT** for text to **ResNets** for **images**, through **WaveNet** for audio.
* Provide guarantees to third parties, for instance, clients or regulators, that you are indeed providing **data protection**, through **code attestation**.
* Explore different scenarios from confidential _Speech-to-text_, to _biometric identification_, through secure document analysis with our pool of **examples**.

#### What you cannot do with BlindAI

* Our solution aims to be modular but we have yet to incorporate tools for generic pre/post processing. Specific pipelines can be covered but will require additional handwork for now.
* We do not cover training and federated learning yet, but if this feature interests you do not hesitate to show your interest through the [roadmap](https://github.com/mithril-security/blindai/projects/1) or [Discord](https://discord.gg/rWHcHeCBWk) channel.
* The examples we provide are simple and do not take into account complex mechanisms such as secure storage of confidential data with sealing keys, an advanced scheduler for inference requests, or complex key management scenarios. If your use case involves more than what we show, do not hesitate to **contact us** for more information.

### Who made BlindAI?&#x20;

BlindAI was developed by **Mithril Security**. **Mithril Security** is a startup focused on confidential machine learning based on **Intel SGX** technology. We provide an **open-source AI inference solution**, **allowing easy and fast deployment of neural networks, with strong security properties** provided by confidential computing by performing the computation in a hardware-based **Trusted Execution Environment** (_TEE_) or simply **enclaves**.





