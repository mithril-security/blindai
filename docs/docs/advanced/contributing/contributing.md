# Contributing to BlindAI 
_________________________________

Welcome and thank you for taking the time to contribute to BlindAI! 🎉🎉

The following guide is a set of guidelines to help you contribute to the [BlindAI](https://github.com/mithril-security/blindai) project. These are mostly advice, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## 📝 Code of conduct
____________________________

This project and everyone participating in it is governed by the [Mithril Security Code Of Conduct](code_of_conduct.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [contact@mithrilsecurity.io](mailto:contact@mithrilsecurity.io).

## 🚀 What should I know before I get started?
____________________________

### ❓ How to ask a question?
If you have a question to ask or if you want to open a discussion about BlindAI or privacy in data science in general, we have a dedicated [Discord Community](https://discord.gg/TxEHagpWd4) in which all these kind of exchanges are more than welcome!

### ⚙️ The BlindAI project

BlindAI is a simple privacy framework for data science collaboration.

It acts like an **access control solution**, for data owners to protect the privacy of their datasets, and **stands as a guard**, to enforce that only privacy-friendly operations are allowed on the data and anonymized outputs are shown to the data scientist.

- Data owners can let external or internal data scientists explore and extract values from their datasets, according to a strict privacy policy they'll define in BlindAI.

- Data scientists can remotely run queries on data frames without seeing the original data or intermediary results.

### 📁 BlindAI project structure.
```sh
BlindAI Project ⚙️🔒/
├─ Python Client/
│  ├─ src/
│  │  ├─ BlindAI/
│  │  │  ├─ Polars/
│  │  │  ├─ Torch/
├─ Rust Server/
│  ├─ src/
│  │  ├─ Polars/
│  │  ├─ Torch/
```
You can find more information about the **roadmap** of the project [here](https://mithril-security.notion.site/513af0ada2584e0f837776a7f6649ab4?v=cf664187c13149a4b667d9c0ae3ed1c0).

### 📚 Useful resources
We highly encourage you to take a look at this resources for further information about BlindAI ⚙️🔒. 

It is also recommeneded to see the [examples](https://github.com/mithril-security/blindai/tree/master/examples) that demonstrate how BlindAI works before submitting your first contribution. 

* [Documentation - BlindAI official documentation](https://blindai.readthedocs.io)
* [Blog - Mithril Security blog](https://blog.mithrilsecurity.io/)
* [Article - Mithril Security roadmap](https://blog.mithrilsecurity.io/our-roadmap-to-build-a-unified-framework-for-privacy-friendly-data-science-collaboration/)
* [Notebooks and Python code - BlindAI examples](https://github.com/mithril-security/blindai/tree/master/examples)

## 💻 Contributing code
____________________________

This section presents the different options that you can follow in order to contribute to the  BlindAI🚀🔐 project. You can either **Report Bugs**, **Suggest Enhancements** or **Open Pull Requests**.

### 🐞 Reporting bugs
This section helps you through reporting Bugs for BlindAI. Following the guidelines helps the maintainers to understand your report, reproduce the bug and work on fixing at as soon as possible. 

!!! bug "Important!"

	Before reporting a bug, please take a look at the [existing issues](https://github.com/mithril-security/BlindAI/issues). You may find that the bug has already been reported and that you don't need to create a new one.

#### How to report a bug? 
To report a Bug, you can either:

- Follow this [link](https://github.com/mithril-security/blindai/issues/new?assignees=&labels=&template=bug-report.md&title=) and fill the bug report with the required information.

- Go to BlindAI GitHub repository:

	* Go to `Issues` tab.
	* Click on `New Issue` button.
	* Choose the `Bug` option.
	* Fill the report with the required information.

#### How to submit a good bug report?
- Follow the [bug report template](https://github.com/mithril-security/blindai/issues/new?assignees=&labels=&template=bug-report.md&title=) as much as possible (*You can add further details if needed*).
- Use a clear and descriptive title.
- Describe the expected behavior, the behaviour that's actually happening, and how often it reproduces.
- Describe the exact steps to reproduce the problem.
- Specify the versions of BlindAI Client and Server that produced the bug.
- Add any other relevant information about the context, your development environment (*operating system, language version, Libtorch version, platform, etc*).
- Attach screenshots, code snippets and any helpful resources.  

### 💯 Suggesting enhancements 
This section guides you through suggesting enhancements for the BlindAI project. You can suggest one or many by opening a **GitHub Issue**. 

!!! example "Important!"

	Before opening an issue, please take a look at the [existing issues](https://github.com/mithril-security/blindai/issues). You may find that the same suggestion has already been done and that you don't need to create a new one.

#### How to suggest an enhancement? 
To suggest enhancement for BlindAI Project, you can either:

- Follow this [link](https://github.com/mithril-security/blindai/issues/new/choose), choose the most relevant option and fill the report with the required information.

- Go to BlindAI GitHub repository:

  * Go to `Issues` tab.
  * Click on `New Issue` button.
  * Choose the most relevant option.
  * Fill the description with the required information.

#### How to submit a good enhancement suggestion?
- Choose the most relevant issue type for your suggestion.
- Follow the provided template as much as possible.
- Use a clear and descriptive title.
- Add any other relevant information about the context, your development environment (*operating system, language version, etc*).
- Attach screenshots, code snippets and any helpful resources. 

### 💎 Pull requests
This section helps you through the process of opening a pull request and contributing with code to BlindAI!

#### How to open a pull request? 
- Go to BlindAI GitHub repository.
- Fork BlindAI project.
- [Setup your local development environment.](#setting-your-local-development-environment)
- Do your magic ✨ and push your changes. 
- Open a pull request.
- Fill the description with the required information.

#### How to submit a good pull request?
- Make sure your pull request solves an open issue or fixes a bug. If no related issue exists, please consider opening an issue first so that we can discuss your suggestions. 
- Follow the [style guidelines](#style-guidelines). 
- Make sure to use a clear and descriptive title.
- Follow the instructions in the pull request template.
- Provide as many relevant details as possible.
- Make sure to [link the related issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/about-issues#efficient-communication) in the description.

!!! warning "Important!"

	While the prerequisites above must be satisfied prior to having your pull request reviewed, the reviewer(s) may ask you to complete additional work, tests, or other changes before your pull request can be accepted.

### 🛠️ Setting your local development environment
You can find detailed explanation of how to install BlindAI in your local machine in the [official documentation](../../getting-started/installation.md).

If you encounter any difficulties with that, don't hesitate to reach out to us through [Discord](https://discord.gg/TxEHagpWd4) and ask your questions. 


## 🏷️ Issue tracker tags
____________________________

Issue type tags:

|             |                                                             |
| ----------- | ----------------------------------------------------------- |
| question    | Any questions about the project                             |
| bug         | Something isn't working                                     |
| enhancement | Improving performance, usability, consistency               |
| docs        | Documentation, tutorials, and example projects              |
| new feature | Feature requests or pull request implementing a new feature |
| test        | Improving unit test coverage, e2e test, CI or build         |