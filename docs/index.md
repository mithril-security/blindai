# 👋 Welcome to BlindAI!
________________________________________________________

<font size="5"><span style="font-weight: 200">
An AI privacy solution to query models, while ensuring your data remains confidential every step of the way!
</font></span>

## What is BlindAI?
________________________________________________________

**BlindAI** is an **open-source solution** to query and deploy AI models while **guaranteeing data privacy**. The querying of models is done via our **easy-to-use Python library**.

Data sent by users to the AI model is kept **confidential at all times** by hardware-enforced **Trusted Execution Environments**. We explain how they keep data and models safe in detail [here](docs/getting-started/confidential_computing.md).

**BlindAi** consists of:

- **BlindAI API**: for querying popular AI models hosted by Mithril Security (with BlindAI Core running under the hood). Its a Python library 🐍.

- **BlindAI Core**: for hosting your own BlindAI server instance in order to securely deploy your own models. It comes in two parts: a privacy-friendly **server** coded in **Rust** 🦀 using **Intel SGX** (Intel Software Guard Extensions) 🔒 and **AWS Nitro Enclaves** 🌪️ to ensure your data stays safe and an easy-to-use **Python client SDK** 🐍.

You can find our more about BlindAI and BlindAI.Core [here](docs/getting-started/blindai_structure.md).

> You can check out [the code on our GitHub](https://github.com/mithril-security/blindai/). 

We’ll update the documentation as new features come in, so dive in!

## Getting started
________________________________________________________

- Follow our [“Quick tour”](./docs/getting-started/quick-tour.ipynb) tutorial
- [Tackle](./docs/getting-started/confidential_computing.md) the technologies we use to ensure privacy
- Discover the differences between [BlindAI and BlindAI Core](./docs/getting-started/blindai_structure.md)

## Getting help
________________________________________________________

- Go to our [Discord](https://discord.com/invite/TxEHagpWd4) *#support* channel
- Report bugs by [opening an issue on our BlindAI Github](https://github.com/mithril-security/blindai/issues)
- [Book a meeting](https://calendly.com/contact-mithril-security/15mins?month=2022-11) with us

## Who made BlindAI?

BlindAI was developed by **Mithril Security**. **Mithril Security** is a startup focused on confidential machine learning based on **Confidential Computing** technology. We provide **open-source privacy solutions** to **query** and **deploy AI models** while **guaranteeing data privacy**. 
