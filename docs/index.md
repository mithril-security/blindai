# 👋 Welcome to BlindAI!
________________________________________________________

<font size="5"><span style="font-weight: 200">
An AI data privacy solution, allowing users to query popular AI models or serve their own models whilst ensuring that users' data remains private every step of the way.
</font></span>

## What is BlindAI?
________________________________________________________

BlindAI is an **open-source solution** allowing users to query popular AI models or serve their own models with **assurances that users' private data will remain private**. The querying of models is done via our **easy-to-use BlindAI Python library**.

Data sent by users to the AI model is kept **confidential at all times**. Neither the AI service provider nor the Cloud provider (if applicable), can see the data.

Confidentiality is assured by hardware-enforced **Trusted Execution Environments**. We explain how they keep data and models safe in detail [here](docs/getting-started/confidential_computing.md).

**BlindAi** consists of:

- **BlindAI API**: for querying popular AI models hosted by Mithril Security (with BlindAI.Core running under the hood).

- **BlindAI.Core**: for hosting your own BlindAI server instance in order to securely deploy your own models. It comes in two parts: a privacy-friendly **server** coded in **Rust** 🦀 using **Intel SGX** (Intel Software Guard Extensions) 🔒 to ensure your data stays safe and an easy-to-use **Python client SDK** 🐍.

You can find our more about BlindAI and BlindAI.Core [here](docs/getting-started/blindai_structure.md).

> You can check out [the code on our GitHub](https://github.com/mithril-security/blindai/). 

We’ll update the documentation as new features come in, so dive in!

## Getting started
________________________________________________________

- Follow our [“Quick tour”](./docs/getting-started/quick-tour.ipynb) tutorial
- Read about [why you should use](./docs/getting-started/why-blindai.md) BlindAI
- Discover the differences between [BlindAI vs BlindAI Core](./docs/getting-started/blindai_structure.md)
- [Tackle](./docs/getting-started/confidential_computing.md) the technologies we use to ensure privacy

## Getting help
________________________________________________________

- Go to our [Discord](https://discord.com/invite/TxEHagpWd4) *#support* channel
- Report bugs by [opening an issue on our BlindAI Github](https://github.com/mithril-security/blindai/issues)
- [Book a meeting](https://calendly.com/contact-mithril-security/15mins?month=2022-11) with us

## How is the documentation structured?
____________________________________________


- [Tutorials](./docs/tutorials/core/installation.md) take you by the hand to install and run BlindAI. We recommend you start with the **[Quick tour](./docs/getting-started/quick-tour.ipynb)** and then move on to the other tutorials!  

- [Concepts](./docs/concepts/BlindAI_Core.md) guides discuss key topics and concepts at a high level. They provide useful background information and explanations, especially on cybersecurity.

- [How-to guides](./docs/how-to-guides/covid_net_confidential.ipynb) are recipes. They guide you through the steps involved in addressing key problems and use cases. They are more advanced than tutorials and assume some knowledge of how BlindAI works.

- [API Reference](https://blindai.mithrilsecurity.io/en/latest/blindai/client.html) contains technical references for BlindAI’s API machinery. They describe how it works and how to use it but assume you have a good understanding of key concepts.

- [Security](./docs/security/remote_attestation/) guides contain technical information for security engineers. They explain the threat models and other cybersecurity topics required to audit BlindAI's security standards.

- [Advanced](./docs/advanced/build-from-sources/client/) guides are destined to developers wanting to dive deep into BlindAI and eventually collaborate with us to the open-source code. 

## Who made BlindAI?

BlindAI was developed by **Mithril Security**. **Mithril Security** is a startup focused on confidential machine learning based on **Intel SGX** technology. We provide an **open-source AI inference solution**, **allowing easy and fast deployment of neural networks**. Confidential computing provides its **strong security properties** by performing the computation in a hardware-based **Trusted Execution Environment** (_TEE_), also called **enclaves**.
