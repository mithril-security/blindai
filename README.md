<p align="center">
  <img src="assets/logo.png" alt="BlindAI" width="200" height="200" />
</p>

<h1 align="center">Mithril Security – BlindAI</h1>

<h4 align="center">
  <a href="https://www.mithrilsecurity.io">Website</a> |
  <a href="cloud.mithrilsecurity.io/">Cloud</a> |
  <a href="https://blindai.mithrilsecurity.io/">Documentation</a> |
  <a href="https://blog.mithrilsecurity.io/">Blog</a> |
  <a href="https://hub.docker.com/u/mithrilsecuritysas">Docker Hub</a> |
  <a href="https://www.linkedin.com/company/mithril-security-company">LinkedIn</a> | 
  <a href="https://www.twitter.com/mithrilsecurity">Twitter</a> | 
  <a href="https://discord.gg/TxEHagpWd4">Discord</a>
</h4>

<h3 align="center">Fast, accessible and privacy friendly AI deployment 🚀🔒</h3>

> **Warning**
> 
> This version of BlindAI is no longer maintained. **This repository is being replaced by [blindai-preview](https://github.com/mithril-security/blindai-preview)**. 
> 
> **blindai-preview** is a slimmer and more secure version of BlindAI. You can [try it](https://github.com/mithril-security/blindai-preview). 
>
> This version will no longer receive any security updates and we are aware of security weaknesses. In short **this version should NOT be used on sensitive data**. 

BlindAI is a confidential AI inference server. Like regular AI inference solutions, BlindAI helps AI engineers serve models for end-users to benefit from their predictions, but with an added privacy layer. Data sent by users to the AI model is kept confidential at all times, from the transfer to the analysis. This way, users can benefit from AI models without ever having to expose their data in clear to anyone: neither the AI service provider, nor the Cloud provider (if any), can see the data.

Confidentiality is assured by using special hardware-enforced Trusted Execution Environments. To read more about those, read our blog series [here](https://blog.mithrilsecurity.io/confidential-computing-explained-part-1-introduction/).

Our solution comes in two parts:

- A secure inference server to serve AI models with privacy guarantees, developed using [**the Rust Programming Language**](https://www.rust-lang.org/). 🦀🦀
- A Python client SDK to securely consume the remote AI models.

## :round_pushpin: Table of content

- [:lock: Motivation](#lock-motivation)
- [:rocket: Getting started](#rocket-getting-started)
- [📖 Which part of the AI workflow do we cover?](#book-which-part-of-the-ai-workflow-do-we-cover)
- [:sunny: Models covered by BlindAI](#sunny-models-covered-by-blindai)
- [:page_facing_up: Documentation](#page_facing_up-documentation)
- [:white_check_mark: What you can do with BlindAI](#white_check_mark-what-you-can-do-with-blindai)
- [:negative_squared_cross_mark: What you cannot do with BlindAI](#negative_squared_cross_mark-what-you-cannot-do-with-blindai)
- [:computer: Current hardware support](#computer-current-hardware-support)
- [:satellite: What next](#satellite-what-next)
- [:question:FAQ](#questionfaq)
- [Telemetry](#telemetry)
- [Disclaimer](#disclaimer)

## :lock: Motivation

Today, most AI tools offer no privacy by design mechanisms, so when data is sent to be analysed by third parties, the data is exposed to malicious usage or potential leakage. 

We illustrate it below with the use of AI for voice assistants. Audio recordings are often sent to the Cloud to be analysed, leaving conversations exposed to leaks and uncontrolled usage without users’ knowledge or consent.

Currently, even though data can be sent securely with TLS, some stakeholders in the loop can see and expose data : the AI company renting the machine, the Cloud provider or a malicious insider. 

![Before / after BlindAI](https://github.com/mithril-security/animations/raw/main/With_and_without_blindai.gif)

By using BlindAI, data remains always protected as it is only decrypted inside a Trusted Execution Environment, called an enclave, whose contents are protected by hardware. While data is in clear inside the enclave, it is inaccessible to the outside thanks to isolation and memory encryption. This way, data can be processed, enriched, and analysed by AI, without exposing it to external parties.

## Gradio Demo

You can test and see how BlindAI secures AI application through our [hosted demo of GPT2](https://huggingface.co/spaces/mithril-security/blindai), built using Gradio.

In this demo, you can see how BlindAI works and make sure that your data is protected. Thanks to the attestation mechanism, even before sending data, our Python client will check that:

- We are talking to a secure enclave with the hardware protection enabled.
- The right code is loaded inside the enclave, and not a malicious one.

You can find more on [secure enclaves attestation here](https://blog.mithrilsecurity.io/confidential-computing-explained-part-2-attestation/).

![GPT2 demo](https://raw.githubusercontent.com/mithril-security/animations/main/gradio_demo.gif)

## Quick tour

BlindAI allows you to easily and quickly **deploy your AI models with privacy, all in Python**. 
To interact with an AI model hosted on a remote secure enclave, we provide the `blindai.client` API. This client will:
- check that we are talking to a genuine secure enclave with the right security features
- upload an AI model that was previously converted to ONNX
- query the model securely

BlindAI is configured by default to connect to our managed Cloud backend to make it easy for users to upload and query models inside our secure enclaves. Even though we managed users AI models, thanks to the protection provided by the use of secure enclaves, data and models sent to our Cloud remain private. 

You can also deploy BlindAI on [your own infra](#on-premise-deployment).

### Installing BlindAI

BlindAI can easily be installed from [PyPI](https://pypi.org/project/blindai/):

```bash
pip install blindai
```

This package is enough for the deployment and querying of models on our managed infrastructure. For on-premise deployment, you will have to deploy our [Docker](https://hub.docker.com/u/mithrilsecuritysas) images.

### Querying a GPT2

We can see how it works with our GPT2 model for text generation. It is already loaded inside our managed Cloud, so we will simply need to query it. We will be using the `transformers` library for tokenizing.

```python
import blindai
from transformers import GPT2Tokenizer

tokenizer = GPT2Tokenizer.from_pretrained('gpt2')
example = "I like the Rust programming language because"

def get_example_inputs(example, tokenizer):
  # Detailled tokenizing here https://gist.github.com/dhuynh95/4357aec425bd30fbb41db0bc6ce0f8b2
  ...

input_list = get_example_inputs([example], tokenizer)

# Connect to a remote model. If security checks fail, an exception is raised
with blindai.Connection() as client:
  # Send data to the GPT2 model
  response = client.predict("gpt2", input_list)

example = tokenizer.decode(response.output[0].as_torch(), skip_special_tokens=True)

# We can see how GPT2 completed our sentence 🦀
>>> example
"I like the Rust programming language because it's easy to write and maintain."
```

### Uploading a ResNet18

The model in the GPT2 example had already been loaded by us, but BlindAI also allows you to upload your own models to our managed Cloud solution. 

You can find a [Colab notebook](https://colab.research.google.com/drive/1c8pBM5gN5zL_AT0s4kBZjEGdivWW3hSt?usp=sharing) showing how to deploy and query a ResNet18 on BlindAI.

To be able to upload your model to our Cloud, you will need to [first register](https://cloud.mithrilsecurity.io/) to get an API key.

Once you have the API key, you just have to provide it to our backend. 

```python
import torch
import blindai

# Get the model and export it locally in ONNX format
model = torch.hub.load('pytorch/vision:v0.10.0', 'resnet18', pretrained=True)
dummy_inputs = torch.zeros(1,3,224,224)
torch.onnx.export(model, dummy_inputs, "resnet18.onnx")

# Upload the ONNX file along with specs and model name
with blindai.connect(api_key=...) as client:
    client.upload_model(
      model="resnet18.onnx",
    )
```

The first block of code pulls a model from [PyTorch Hub](https://pytorch.org/hub/), and export it in ONNX format. Because [tracing](https://pytorch.org/tutorials/advanced/super_resolution_with_onnxruntime.html) is used, we need to provide a dummy input for the model to know the shape of inputs used live.

Before uploading, we need to provide information on the expected inputs and outputs.

Finally, we can connect to the managed backend and upload the model. You can provide a model name to know which one to query, for instance with `model_name="resnet18"`. Because we have already uploaded a model with the name `"resnet18"`, you should not try to upload a model with that exact name as it is already taken on our main server.

### Querying a ResNet18

Now we can consume this model securely. We can now have a ResNet18 analyze an image of our dog, without showing the image of the dog in clear. 

<img src="https://github.com/pytorch/hub/raw/master/images/dog.jpg" alt="Dog to analyze" width="200"/>

We will first pull the dog image, and preprocess it before sending it our enclave. The code is similar to [PyTorch ResNet18 example](https://pytorch.org/hub/pytorch_vision_resnet/):

```python
# Source: https://pytorch.org/hub/pytorch_vision_resnet/
import blindai
import urllib
from PIL import Image
from torchvision import transforms

# Download an example image from the pytorch website
url, filename = ("https://github.com/pytorch/hub/raw/master/images/dog.jpg", "dog.jpg")
try: urllib.URLopener().retrieve(url, filename)
except: urllib.request.urlretrieve(url, filename)

# sample execution (requires torchvision)
input_image = Image.open(filename)
preprocess = transforms.Compose([
    transforms.Resize(256),
    transforms.CenterCrop(224),
    transforms.ToTensor(),
    transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
])
input_tensor = preprocess(input_image)
input_batch = input_tensor.unsqueeze(0) # create a mini-batch as expected by the model
```

Now we that we have the input tensor, we simply need to send it to the pre-uploaded ResNet18 model inside our secure enclave:

```python
with blindai.connect() as client:
  # Send data to the GPT2 model
  response = client.predict("resnet18", input_batch)

>>> response.output[0].as_numpy().argmax()
```

### On-premise deployment

If you do not wish to use our managed Cloud, it is possible to deploy BlindAI yourself. We provide Docker images to help you with the deployment.

To run our solution with the full security features, you will need access to the [proper hardware](#computer-current-hardware-support).

You can still try our solution without the right hardware through our simulation version. *Be careful*, as the simulation provides no added security guarantees, and is meant for tests only.

## :book: Which part of the AI workflow do we cover?

![Position AI toolkit](assets/position_ai_toolkit.PNG)

BlindAI is currently a solution for AI model deployment. We suppose the model has already been trained and wants to be deployed but requires privacy guarantees for the data owners sending data to the model. We focus mostly on deep learning models, though inference of random forests can be covered by BlindAI.

This scenario often comes up once you have been able to train a model on a specific dataset, most likely on premise, like on biometric, medical or financial data, and now want to deploy it at scale as a Service to your users.

BlindAI can be seen as a variant of current serving solutions, like Nvidia Triton, Torchserve, TFserve, Kserve and so on. We provide the networking layer and the client SDK to consume the service remotely and securely, thanks to our secure AI backend.

## :sunny: Models covered by BlindAI

Here is a list of models BlindAI supports, the use cases it unlocks and articles to provide more context on each case. The articles are in preparation and we welcome all contributions to show how BlindAI can be used to deploy AI models with confidentiality!

| Model name           | Model family  | Link to model                                                 | Example use case                        | Article                                                                                                                               | Link to the notebook                                                                                 | Inference time (ms) | Hardware                        |
|----------------------|---------------|---------------------------------------------------------------|-----------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------|---------------------|---------------------------------|
| DistilBERT           | BERT          | https://huggingface.co/docs/transformers/model_doc/distilbert | Sentiment analysis                      | [Deploy Transformers models with confidentiality](https://blog.mithrilsecurity.io/transformers-with-confidentiality/)                 | https://github.com/mithril-security/blindai/blob/master/examples/distilbert/BlindAI-DistilBERT.ipynb | 28.435              | Intel(R) Xeon(R) Platinum 8370C |
| COVID-Net-CXR-2      | 2D CNN        | https://github.com/lindawangg/COVID-Net                       | Chest XRAY analysis for COVID detection | [Confidential medical image analysis with COVID-Net and BlindAI](https://blog.mithrilsecurity.io/confidential-covidnet-with-blindai/) | https://github.com/mithril-security/blindai/blob/master/examples/covidnet/BlindAI-COVID-Net.ipynb    | To be announced     | To be announced                 |
| Wav2vec2             | Wav2vec       | https://huggingface.co/docs/transformers/model_doc/wav2vec2   | Speech to text                          | [Build a privacy-by-design voice assistant with BlindAI](https://blog.mithrilsecurity.io/privacy-voice-ai-with-blindai/)                                                                                                                       | https://github.com/mithril-security/blindai/blob/master/examples/wav2vec2/BlindAI-Wav2vec2.ipynb     | 617.04              | Intel(R) Xeon(R) Platinum 8370C |
| Facenet              | Resnet        | https://github.com/timesler/facenet-pytorch                   | Facial recognition                      | To be announced                                                                                                                       | https://github.com/mithril-security/blindai/blob/master/examples/facenet/BlindAI-Facenet.ipynb                                                                                      | 47.135              | Intel(R) Xeon(R) Platinum 8370C |
| YoloV5               | Yolo          | https://github.com/ultralytics/yolov5                         | Object detection                        | To be announced                                                                                                                       | To be announced                                                                                      | To be announced     | To be announced                 |
| Word2Vec             | Word2Vec      | https://spacy.io/usage/embeddings-transformers                | Document search                         | To be announced                                                                                                                       | To be announced                                                                                      | To be announced     | To be announced                 |
| Neural Random Forest | Random Forest | https://arxiv.org/abs/1604.07143                              | Credit scoring                          | To be announced                                                                                                                       | To be announced                                                                                      | To be announced     | To be announced                 |
| M5 network           | 1D CNN        | https://arxiv.org/pdf/1610.00087.pdf                          | Speaker recognition                     | To be announced                                                                                                                       | To be announced                                                                                      | To be announced     | To be announced                 |                                                                                  |

We will publish soon the scripts to run the benchmarks. 

## :page_facing_up: Documentation

To learn more about our project, do not hesitate to read our [documentation](https://blindai.mithrilsecurity.io/).

## :white_check_mark: What you can do with BlindAI

- Easily deploy state-of-the-art models with confidentiality. Run any [ONNX model](https://onnx.ai/), from **BERT** for text to **ResNets** for **images**, and much more.
- Provide guarantees to third parties, for instance clients or regulators, that you are indeed providing **data protection**, through **code attestation**.
- Explore different scenarios from confidential **Sentiment analysis**, to **medical imaging** with our pool of examples.

## :negative_squared_cross_mark: What you cannot do with BlindAI

- Our solution aims to be modular but we have yet to incorporate tools for generic pre/post processing. Specific pipelines can be covered but will require additional handwork for now.
- We do not cover training and federated learning yet, but if this feature interests you do not hesitate to show your interest through the [roadmap](https://blog.mithrilsecurity.io/our-roadmap-at-mithril-security/) or [Discord](https://discord.gg/TxEHagpWd4) channel.
- The examples we provide are simple, and do not take into account complex mechanisms such as secure storage of confidential data with sealing keys, advanced scheduler for inference requests, or complex key management scenarios. If your use case involves more than what we show, do not hesitate to **contact us** for more information.

## :computer: Current hardware support 

Our solution currently leverages Intel SGX enclaves to protect data.

If you want to deploy our solution with real hardware protection and not only simulation, you can either deploy it on premise with the right hardware specs, or rent a machine adapted for Confidential Computing in the Cloud.

You can go to [Azure Confidential Computing VMs to try](https://docs.microsoft.com/en-us/azure/confidential-computing/confidential-computing-enclaves), with our [guides available here](https://blindai.mithrilsecurity.io/en/latest/docs/cloud-deployment/) for deployment on DCsv2 and DCsv3.

## :satellite: What next

We intend to cover AMD SEV and Nitro Enclave in the future, which would make our solution available on GCP and AWS. 

While we only cover deployment for now, we will start working on covering more complex pre/post processing pipelines inside enclaves, and training with Nvidia secure GPUs. More information about our roadmap can be found [here](https://blog.mithrilsecurity.io/our-roadmap-at-mithril-security/).

## :question:FAQ

**Q: How do I make sure data that I send is protected**

**A:** We leverage secure enclaves to provide end-to-end protection. This means that even while your data is sent to someone else for them to apply an AI on it, your data remains protected thanks to hardware memory isolation and encryption.

We provide some information in our workshop [Reconcile AI and privacy with Confidential Computing](https://www.youtube.com/watch?v=tAT23GKMi_0).

You can also have a look on our series [Confidential Computing explained](https://blog.mithrilsecurity.io/confidential-computing-explained-part-1-introduction/).

**Q: Why should I trust you?**

**A**: The client and server are open source. You can find them on our [GitHub](https://github.com/mithril-security/blindai).
    
Thanks to this, you can verify yourself that the client does implement the right encryption mechanisms.
    
In addition, the server side code can be verified thanks to remote attestation. Our server code is open, and any version will generate a specific hash that you can reproduce yourself. You can compile BlindAI and get the hash of the code yourself.

Then you will just need to verify that this trustful source is indeed loaded in the remote server by using remote attestation.

**Q: How much slowdown should we expect when using BlindAI?**

**A:** We will provide a detailled benchmark soon. Usually you should see a negligeable slowdown with some simple models, and we have observed up to 30-40% slowdown for complex models.

**Q: What is the maximal data/model size with BlindAI?**

**A:** With the latest Intel Xeon Icelake 3rd Gen, the enclaves can now protect up to 1TB of code and data. This means that most models, even the biggest ones, can be made confidential with our solution. 

**Q: What do I need to do to use BlindAI?**

**A:** The general workflow of BlindAI is described [here](#wrench-how-do-i-use-it). Basically you need to export your model in ONNX, upload it to the server and then you can send data to be analyzed securely.

**Q: Can I use Python script with BlindAI?**

**A:** We only support ONNX models for now, but most of the time preprocessing or postprocessing workflows can be expressed using ONNX operators. In that case you just have to include it in your model before exporting it to ONNX. You can see example for instance in the [Wav2vec2 example](https://github.com/mithril-security/blindai/blob/master/examples/wav2vec2/BlindAI-Wav2vec2.ipynb).

**Q: Do you do training or federated learning?**

**A:** We will cover this topic soon. We have multy party learning framework leveraging secure enclave in development. You should learn about it in the near future.

## Telemetry

BlindAI collects anonymous data regarding general usage, this allows us to understand how you are using the project. We only collect data regarding the execution mode (Hardware/Software) and the usage metrics.

This feature can be easily disabled, by settin up the environment variable `BLINDAI_DISABLE_TELEMETRY` to 1.

You can find more information about the telemetry in our [**documentation**](https://blindai.mithrilsecurity.io/en/latest/getting-started/telemetry/).

## Disclaimer

BlindAI is still in alpha and is being actively developed. It is provided as is, use it at your own risk.
