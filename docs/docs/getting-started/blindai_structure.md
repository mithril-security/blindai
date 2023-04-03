# How is BlindAI structured?
________________

BlindAI’s goal is to allow users to consume AI models without showing their data to the AI provider, thanks to [Confidential Computing](confidential_computing.md).

The project has been historically intended for AI engineers but has been modified to onboard developers easily with ready-to-use AI APIs.

BlindAI API is a simple client-side SDK to query popular AI models that we host inside a [Trusted Execution Environment](confidential_computing.md) to help developers get started quickly with AI whilst keeping their data private. BlindAI API uses BlindAI Core behind the scenes.

BlindAI Core contains the server and client implementations to upload a model inside a [Trusted Execution Environment](confidential_computing.md) and query it securely.

## BlindAI API
__________________________

BlindAI API includes an open-source Python library, allowing developpers to query popular AI models without needing to trust us with access to your data, thanks to [Confidential Computing](confidential_computing.md). 

So far, we have released [Whisper on BlindAI](quick_tour.ipynb), a pre-trained model for automatic speech recognition (ASR) and speech translation. More models are set to join Whisper on the platform in the future.

## BlindAI Core
________________________________

BlindAI Core facilitates privacy-friendly AI model deployment by letting AI engineers upload and delete models to their secure BlindAI server instance. Users can then connect to the server, upload their data and run models on it without giving any party access to their data.

We have made [a guide using a life-like example](../how-to-guides/covid_net_confidential.ipynb) to give you an idea of what using BlindAI Core could look like. You can also read more in details about the features of BlindAI Core [here](../concepts/BlindAI_Core.md). 


## BlindAI vs BlindAI Core: Key differences
__________________________________________________________

Here is a summary of the key differences between BlindAI API and BlindAI Core:

![BlindAI API vs Core](../../assets/blindai_core_table.png)

With both BlindAI and BlindAI Core, data confidentiality is assured by hardware-enforced Trusted Execution Environments. We explain how they keep data and models safe in detail [here](confidential_computing.md).
