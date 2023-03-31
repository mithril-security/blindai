# How BlindAI is structured
________________

## Introduction

BlindAI’s goal is to allow users to consume AI models without showing their data to the AI provider, thanks to Confidential Computing.

The project has been historically intended for AI engineers but has been modified to onboard developers easily with ready-to-use AI APIs.

BlindAI API is a simple client-side SDK to query popular AI models that we host inside a [Trusted Execution Environment](confidential_computing.md) to help developers get started quickly with AI whilst keeping their data private. BlindAI API uses BlindAI Core behind the scenes.
BlindAI Core contains the server and client implementations to upload a model inside a [Trusted Execution Environment](confidential_computing.md) and query it securely.

## BlindAI API

BlindAI API includes an open-source Python library, allowing you to query popular AI models without needing to trust us with access to your data, thanks to Confidential Computing. 

So far, we have released Whisper on BlindAI, a pre-trained model for automatic speech recognition (ASR) and speech translation. More models are set to join Whisper on the platform in the future.

Help developers benefit from popular off-the-shelf AI models like Whisper without giving away access to their data
### Use cases
You want to use off-the-shelf AI for your organization, for example, to analyze your code with [CodeGen](https://github.com/salesforce/CodeGen), but you don’t want to manage the model or give an AI vendor access to your code.
You want to add AI into your apps for privacy-demanding clients, such as hospitals, with assurances that no one will have access to their data during this process.

## BlindAI Core

BlindAI Core facilitates privacy-friendly AI model deployment by letting AI engineers upload and delete models to their secure BlindAI server instance. Users can then connect to the server, upload their data and run models on it without giving any party access to their data.
### Goal
Allows AI engineers and AI service providers to deploy custom AI workloads in a trusted execution environment.

### Use case
You want to provide your own model, for instance, ECG analysis with AI to hospitals, and you want to prove to them that no one will have access to their data. 


### BlindAI vs BlindAI Core: Key differences

Here is a summary of the key differences between BlindAI API and BlindAI Core:

![BlindAI API vs Core](../../assets/blindai_core_table.png)

With both BlindAI and BlindAI Core, data confidentiality is assured by hardware-enforced Trusted Execution Environments. We explain how they keep data and models safe in detail [here](confidential_computing.md).
