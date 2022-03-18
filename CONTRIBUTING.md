# Contributing to BlindAI

🎉 Hello there! thanks for taking the time to contribute to BlindAI! 🎉 

The following is a set of guidelines for contributing to [BlindAI](https://github.com/mithril-security/blindai) project. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

#### Table Of Contents

[Code of Conduct](#code-of-conduct)

[What should I know before I get started?](#what-should-i-know-before-i-get-started)
  * [I just have a question!](#i-just-have-a-question)
  * [BlindAI Project](#blindai-project)
  * [Useful Resources](#useful-resources)

[How Can I Contribute?](#how-can-i-contribute)
  * [Reporting Bugs](#reporting-bugs)
  * [Suggesting Enhancements](#suggesting-enhancements)
  * [Your First Code Contribution](#your-first-code-contribution)
  * [Pull Requests](#pull-requests)

[Styleguides](#styleguides)
  * [Git Commit Messages](#git-commit-messages)
  * [Python Styleguide](#python-styleguide)
  * [Rust Styleguide](#rust-styleguide)

[Additional Notes](#additional-notes)
  * [Issue and Pull Request Labels](#issue-and-pull-request-labels)

## Code of Conduct

This project and everyone participating in it is governed by the [Mithril Security Code Of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [contact@mithrilsecurity.io](mailto:contact@mithrilsecurity.io).

## What should I know before I get started?

### I just have a question
If you have a question to ask or you want to open a discussion about Blindai or confidential computing in general, we have a dedicated [Discord Community](https://discord.gg/TxEHagpWd4) in which all these kind of exchanges are more than welcome!

### BlindAI Project

BlindAI is a fast, easy to use and confidential inference server, allowing to deploy models that works on sensitive data. Thanks to the end-to-end protection guarantees, data owners can send private data to be analyzed by AI models, without fearing exposing their data to anyone else.

The solution has two parts:
- A secure inference solution to serve AI models with privacy guarantees (Built with **Rust**).
- A client SDK to securely consume the remote AI models (Built with **Python**).

You can find more information about the **Roadmap** of the project [here](https://blog.mithrilsecurity.io/our-roadmap-at-mithril-security/).

### Useful Resources
We highly encourage you to take a look at this resources for further information about BlindAI Solution. 

It is also recommeneded to see the [examples](https://github.com/mithril-security/blindai/tree/master/examples) that demonstrates how BlindAI works before submitting your first contribution. 

* [Documentation - BlindAI Official Documentation](https://docs.mithrilsecurity.io)
* [Blog - Mithril Security Blog](https://blog.mithrilsecurity.io/)
* [Article - Mithril Security Roadmap](https://blog.mithrilsecurity.io/our-roadmap-at-mithril-security/)
* [Notebooks - BlindAI Examples](https://github.com/mithril-security/blindai/tree/master/examples)

## How Can I Contribute?
This section presents the different ways you can take in order to contribute to BlindAI project. You can either **Report Bugs**, **Suggest Enhancements** or do **Pull Requests**.

### Reporting Bugs

### Suggesting Enhancements 

### Pull Requests

## Styleguides

### Git Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line

### Python Styleguide

All Python code is linted with [Black](https://github.com/psf/black).

### Rust Styleguide

All Rust code is formatted with [Rust Formatter](https://github.com/rust-lang/rustfmt)

## Additional Notes

### Issue and Pull Request Labels

This section lists the labels we use to help us track and manage issues and pull requests.

[GitHub search](https://help.github.com/articles/searching-issues/) makes it easy to use labels for finding groups of issues or pull requests you're interested in.

The labels are organised in 4 groups : `Info`, `Type`, `Status` and `priority`.

The labels are loosely grouped by their purpose, but it's not required that every issue have a label from every group or that an issue can't have more than one label from the same group.

#### Issue and Pull Request Labels

| Label name | Description |
| --- | --- |
| Info : Client 🐍 | The issue/PR affects the client side |
| Info : Server 🦀 | The issue/PR affects the server side |
| Info : Build 🏗️ | The issue/PR is related to the build process  |
| Info : Good First Issue 🎓 | Good for beginners and new incomers |
| Info : Duplicate❕ | The Issue/PR is duplicate |
| Info : Invalid 😕 | The issue/PR doesn’t seem to be relevant |
| Type : Bug 🐞 | The Issue/PR reports/fixes a bug |
| Type : Refactor 🏭 |  The Issue/PR only refactors the codebase, not additional feature of bug fixes are provided |
| Type : Improvement 📈 | The Issue/PR suggests an improvement of an existing functionality |
| Type : New Feature ➕ | The issue/PR proposes a new feature that wasn’t in the codebase.  |
| Type : Documentation 📝 | The Issue/PR concerns the documentation (README, docstrings, CHANGELOG ...) |
| Type : Testing 🧪 | The issue/PR adds, improves or edits tests. |
| Status : Available 🤚 | The issue hasn’t been assigned yet |
| Status : In progress 👨‍🔧 | The work on the Issue/PR is in progress |
| Status : blocked 🚫 | The work one the Issue/PR is blocked by other tasks that haven’t been finished |
| Status : Completed | The work on the Issue/PR is completed |
| Status : Review needed 🙋‍♂️ | A review is needed in order to complete the work / approve it. |
| Status : To merge ✅ | Approved PR and will be merged!  |
| Priority : High 🔴 | The issue is urgent, must be fixed as soon as possible |
| Priority : Medium 🟠 | The issue is of a medium priority |
| Priority : Low 🟢 | The issue is of a low priority and can wait a bit |
