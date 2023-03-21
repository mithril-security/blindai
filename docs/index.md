# 👋 Welcome to BlindAI!
________________________________________________________

<font size="5"><span style="font-weight: 200">
An AI model deployment solution which ensures users' data remains private every step of the way.</font></span>

## What is BlindAI?
________________________________________________________

**BlindAI** is an **AI inference server** with an added **privacy layer**, protecting the data sent to models.

BlindAI facilitates  **privacy-friendly AI model deployment** by letting AI engineers upload and delete models to their secure server instance using our **Python API**. Clients can then connect to the server, upload their data and run models on it without compromising on privacy. 

Data sent by users to the AI model is kept **confidential at all times**. Neither the AI service provider nor the Cloud provider (if applicable), can see the data. 
Confidentiality is assured by hardware-enforced [**Trusted Execution Environments**](). We explain how they keep data and models safe in detail [here]().

**BlindAi is an open-source project** consisting of:

- A privacy-friendly **server** coded in **Rust** 🦀 using **Intel SGX** (Intel Software Guard Extensions) 🔒 to ensure your data stays safe.
- An easy-to-use **Python client SDK** 🐍.

> You can check out [the code on our GitHub](https://github.com/mithril-security/blindai-preview/). 

We’ll update the documentation as new features come in, so dive in!

## Getting started
________________________________________________________

- Follow our [“Quick tour”](./docs/getting-started/quick-tour.ipynb) tutorial
- Read about [why you should use](./docs/getting-started/why-blindai.md) BlindAI
- Explore our [installation guide](./docs/getting-started/installation.md)
- [Tackle](./docs/advanced/security/remote_attestation.md) the technologies we use to ensure privacy

## Getting help
________________________________________________________

- Go to our [Discord](https://discord.com/invite/TxEHagpWd4) *#support* channel
- Report bugs by [opening an issue on our BlindAI Github](https://github.com/mithril-security/blindai-preview/issues)
- [Book a meeting](https://calendly.com/contact-mithril-security/15mins?month=2022-11) with us

## How is the documentation structured?
____________________________________________
<!-- 
- [Tutorials](link) take you by the hand to install and run BlindAI. We recommend you start with the **[Quick tour](./docs/docs/getting-started/quick-tour.ipynb)** and then move on to the other tutorials!  

- [How-to guides](link) are recipes. They guide you through the steps involved in addressing key problems and use cases. They are more advanced than tutorials and assume some knowledge of how BlindAI works.

- [Concepts](link) guides discuss key topics and concepts at a high level. They provide useful background information and explanations, especially on cybersecurity.
-->
- [Getting Started](./docs/getting-started/why-blindai.md) take you by the hand to install and run BlindAI. We recommend you start with the **[Quick tour](./docs/getting-started/quick-tour.ipynb)** and then move on to [installation](./docs/getting-started/installation.md)! 

- [API Reference](https://blindai-preview.mithrilsecurity.io/en/latest/blindai_preview/client.html) contains technical references for BlindAI’s API machinery. They describe how it works and how to use it but assume you have a good understanding of key concepts.

- [Security](./docs/advanced/security/remote_attestation/) guides contain technical information for security engineers. They explain the threat models and other cybersecurity topics required to audit BlindAI's security standards.

- [Advanced](./docs/advanced/build-from-sources/client/) guides are destined to developpers wanting to dive deep into BlindAI and eventually collaborate with us to the open-source code. 

## Who made BlindAI?

BlindAI was developed by **Mithril Security**. **Mithril Security** is a startup focused on confidential machine learning based on **Intel SGX** technology. We provide an **open-source AI inference solution**, **allowing easy and fast deployment of neural networks**. Confidential computing provides its **strong security properties** by performing the computation in a hardware-based **Trusted Execution Environment** (_TEE_), also called **enclaves**.
