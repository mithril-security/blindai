# Why BlindBox?
________________

Recent Large Language Model (LLM) releases, such as Whisper and ChatGPT, have skyrocketed the adoption and democratization of LLMs across all industries. Whilst these are extraordinary models which can be provide great benefits across a multitude of use cases, we believe that we need to take action to ensure we have solutions that protect confidential data whilst using those models. Take ChatGPT's example: thousands of employees used it - unrestricted - to help them in their tasks, feeding it company data that could then be accessed by any OpenAI employee.

This is why we have created BlindBox, so developers can deploy their SAAS solutions in a protected highly isolated environment which ensures no one can access their data.

## Example: Secure LLM application deployment for hospitals
____________________________________________

Imagine you are a SAAS provider that provides a speech-to-text service for customers. You speak to a potential client, a hospital wanting to use speech-to-text to help doctors be more efficient by transcribing voice notes they make about consultations. The issue is that, when sending data to the SAAS provider, no technical guarantee can be provided that the confidential medical information contained in those audios cannot be accessed or compromised by a malicious insider or attacker targeting the SAAS provider's server. The data may be encrypted in transit, but it will be in clear in memory before being analyzed for transcription by the LLM model. Therefore the hospital has to entrust their data to this startup which might not have put in place the proper security measures to ensure data confidentiality.

This key security flaw often cuts off this type of collaboration from the get go. The hospital would likely refuse to send their medical data to the SAAS provider. Being able to use speech-to-text could have saved many hours for the hospital’s personnel and benefit the startup, but healthcare data security is just too sensitive to take any risk.

This is where BlindBox comes in. Using BlindBox, the SAAS server can deploy their application within a highly isolated computing environment, a [Trusted Execution Environment](confidential_computing.md), which ensures that no-one, including their employees, can access the hospital's data. This allows this kind of collaboration to go ahead to the great benefit of both parties.

## Next steps 
____________________________

To see a hands-on demo of BlindBox via our Whisper API, check out our [Quick Tour](./quick-tour.ipynb)!

To find out more about the Confidential technologies behind BlindBox, check out our [guide to Confidential Computing](confidential_computing.md).
