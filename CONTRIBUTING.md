# Contributing to BlindAI

🎉 Hello there! thanks for taking the time to contribute to BlindAI! 🎉 

The following is a set of guidelines for contributing to [BlindAI](https://github.com/mithril-security/blindai) project. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

#### Table Of Contents

[Code of Conduct](#code-of-conduct)

[What should I know before I get started?](#what-should-i-know-before-i-get-started)
  * [I only have a question!](#i-only-have-a-question)
  * [BlindAI Project](#blindai-project)
  * [Useful Resources](#useful-resources)

[How Can I Contribute?](#how-can-i-contribute)
  * [Reporting Bugs](#reporting-bugs)
  * [Suggesting Enhancements](#suggesting-enhancements)
  * [Pull Requests](#pull-requests)
  * [Setting Your Local Development Environment](#setting-your-local-development-environment)

[Style Guidelines](#style-guidelines)
  * [Git Commit Messages](#git-commit-messages)
  * [Linting and Formatting](#linting-and-formatting)
  * [Pre-commit hook](#pre-commit-hook)

[Additional Notes](#additional-notes)
  * [Issue and Pull Request Labels](#issue-and-pull-request-labels)

## Code of Conduct

This project and everyone participating in it is governed by the [Mithril Security Code Of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [contact@mithrilsecurity.io](mailto:contact@mithrilsecurity.io).

## What should I know before I get started?

### I only have a question
If you have a question to ask or you want to open a discussion about BlindAI or confidential computing in general, we have a dedicated [Discord Community](https://discord.gg/TxEHagpWd4) in which all these kind of exchanges are more than welcome!

### BlindAI Project

BlindAI is a fast, easy to use and confidential inference server, allowing to deploy models that works on sensitive data. Thanks to the end-to-end protection guarantees, data owners can send private data to be analyzed by AI models, without fearing exposing their data to anyone else.

The solution has two parts:
- A secure inference solution to serve AI models with privacy guarantees (Built with **Rust**).
- A client SDK to securely consume the remote AI models (Built with **Python**).

You can find more information about the **Roadmap** of the project [here](https://blog.mithrilsecurity.io/our-roadmap-at-mithril-security/).

### Useful Resources

We highly encourage you to take a look at this resources for further information about BlindAI. 

It is also recommeneded to see the [examples](https://github.com/mithril-security/blindai/tree/master/examples) that demonstrate how BlindAI works before submitting your first contribution. 

* [Documentation - BlindAI Official Documentation](https://docs.mithrilsecurity.io)
* [Blog - Mithril Security Blog](https://blog.mithrilsecurity.io/)
* [Article - Mithril Security Roadmap](https://blog.mithrilsecurity.io/our-roadmap-at-mithril-security/)
* [Notebooks - BlindAI Examples](https://github.com/mithril-security/blindai/tree/master/examples)

## How Can I Contribute?
This section presents the different options that you can follow in order to contribute to BlindAI project. You can either **Report Bugs**, **Suggest Enhancements** or **Open Pull Requests**.

### Reporting Bugs
This section helps you through reporting Bugs for BlindAI. Following the guidelines helps the maintainers to understand your report, reproduce the Bug and work on fixing at as soon as possible. 

> **Important!**
> Before reporting a bug, please take a look at the [existing issues](https://github.com/mithril-security/blindai/issues). You may find that the bug has already been reported and that you don't need to create a new one.

#### How to report a bug? 
To report a Bug, you can either:
- Follow this [link](https://github.com/mithril-security/blindai/issues/new?assignees=&labels=&template=bug-report.md&title=) and fill the report with the required information.
- In BlindAI github repository:
  * Go to `Issues` tab.
  * Click on `New Issue` button.
  * Choose the `Bug` option.
  * Fill the report with the required information.

#### How to submit a good Bug Report?
- Follow the Bug Report template as much as possible (You can add further details if needed).
- Use a clear and descriptive title.
- Describe the expected behavior, the one that's actually happening and how often does it reproduce.
- Describe the exact steps which reproduce the problem.
- Specify the versions of BlindAI Client and Server that produced the bug.
- Add any other relevant information about the context, your development environment (Operating system, Language version ...)
- Attach screenshots, code snippets and any helpful resources.  

### Suggesting Enhancements 
This section guides you through suggesting enhancements for BlindAI project. You can suggest an enhancement by opening a **GitHub Issue**. 

> **Important!**
> Before opening an issue, please take a look at the [existing issues](https://github.com/mithril-security/blindai/issues). You may find that the same suggestion has already been done and that you don't need to create a new one.

#### How to suggest an enhancement? 
To suggest enhamcement for BlindAI Project, you can either:

- Follow this [link](https://github.com/mithril-security/blindai/issues/new/choose), choose the most relevant option and fill the report with the required information
- In BlindAI GitHub repository:
  * Go to `Issues` tab.
  * Click on `New Issue` button.
  * Choose the most relevant option.
  * Fill the description with the required information.

#### How to submit a good enhancement suggestion?
- Choose the most relevant issue type for your suggestion.
- Follow the provided template as much as possible.
- Use a clear and descriptive title.
- Add any other relevant information about the context, your development environment (Operating system, Language version ...)
- Attach screenshots, code snippets and any helpful resources.  

### Pull Requests
This section helps you through the process of opening a Pull Request and contributing with code to BlindAI Project!

#### How to open a pull request? 
In order to open a pull request:
- Go to BlindAI GitHub repository.
- Fork BlindAI project.
- [Setup your local development environment.](#setting-your-local-development-environment)
- Do you magic! and push your changes. 
- Open a Pull Request
- Fill the description with the required information.

#### How to submit a good pull request?
- Make sure your pull request solves an open issue or fixes a bug. If no related issue exists, please consider opening an issue first so that we can discuss your suggestions. 
- Follow the [style guidelines](#style-guidelines). 
- Make sure to use a clear and descriptive title.
- Follow the instructions in the pull request template.
- Provide as many relevant details as possible.
- Make sure to [link the related issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/about-issues#efficient-communication) in the description.

While the prerequisites above must be satisfied prior to having your pull request reviewed, the reviewer(s) may ask you to complete additional work, tests, or other changes before your pull request can be ultimately accepted.

### Setting Your Local Development Environment
You can find detailed explanation of how to install BlindAI in your local machine in the [official documentation](https://docs.mithrilsecurity.io/started/installation).

If you encounter any difficulties within that, don't hesitate to reach out to us through [Discord](https://discord.gg/TxEHagpWd4) and ask your questions. 

## Style Guidelines

### Git Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line

### Linting and formatting

This repository uses the following:
- flake8 and black to lint/format python code.
- Cargo formatter to format Rust code.

### Pre-commit hook
To ensure the style guidelines are maintained in the codebase, **pre-commit hook** is used.
Once cloning the repository, make sure to follow the following steps:

#### Install pre-commit package manager

```bash
pip install pre-commit
```
#### Install the git hooks script

```bash
pre-commit install
```
## Additional Notes

### Issue and Pull Request Labels

This section lists the labels we use to help us track and manage issues and pull requests.

[GitHub search](https://help.github.com/articles/searching-issues/) makes it easy to use labels for finding groups of issues or pull requests you're interested in.

The labels are organised in 4 groups : `Info`, `Type`, `Status` and `priority`.

The labels are loosely grouped by their purpose, but it's not required that every issue has a label from every group or that an issue can't have more than one label from the same group.

#### Issue and Pull Request Labels

| Label name | Description |
| --- | --- |
| Info : Client 🐍 | The Issue/PR affects the client-side |
| Info : Server 🦀 | The Issue/PR affects the server-side |
| Info : Build 🏗️ | The Issue/PR is related to the build process  |
| Info : Good First Issue 🎓 | Good for beginners and new incomers |
| Info : Duplicate❕ | The Issue/PR is duplicate |
| Info : Invalid 😕 | The Issue/PR doesn’t seem to be relevant |
| Type : Bug 🐞 | The Issue/PR reports/fixes a bug |
| Type : Refactor 🏭 |  The Issue/PR only refactors the codebase, not additional features of bug fixes are provided |
| Type : Improvement 📈 | The Issue/PR suggests an improvement of an existing functionality |
| Type : New Feature ➕ | The Issue/PR proposes a new feature that wasn’t in the codebase.  |
| Type : Documentation 📝 | The Issue/PR concerns the documentation (README, docstrings, CHANGELOG ...) |
| Type : Testing 🧪 | The Issue/PR adds, improves or edits tests. |
| Status : Available 🤚 | The Issue hasn’t been assigned yet |
| Status : In progress 👨‍🔧 | The work on the Issue/PR is in progress |
| Status : blocked 🚫 | The work on the Issue/PR is blocked by other tasks that haven’t been finished yet |
| Status : Completed 💯 | The work on the Issue/PR is completed |
| Status : Review needed 🙋‍♂️ | A review is needed in order to complete the work / approve it. |
| Status : To merge ✅ | Approved PR and will be merged!  |
| Priority : High 🔴 | The Issue is urgent, must be fixed as soon as possible |
| Priority : Medium 🟠 | The Issue is of a medium priority |
| Priority : Low 🟢 | The Issue is of a low priority and can wait a bit |
