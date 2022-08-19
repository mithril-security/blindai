# Quick start

This section guides you through deploying your first model with **BlindAI Inference Server!**&#x20;

We will use the example of [DistilBert](https://huggingface.co/docs/transformers/model\_doc/distilbert) model for demonstration purposes.

## Prepare the model

Let's assume we want to deploy a [**DistilBERT**](https://huggingface.co/docs/transformers/model\_doc/distilbert) model for classification, within our confidential inference server. This could be useful for instance to analyze medical records in a privacy-friendly manner and compliant way.

BlindAI uses the [ONNX format](https://onnx.ai/), which is an open and interoperable AI model format. Pytorch or Tensorflow models can be easily exported to ONNX.

### Step 1: Load the BERT model

```python
from transformers import DistilBertForSequenceClassification

# Load the model
model = DistilBertForSequenceClassification.from_pretrained("distilbert-base-uncased")
```

For simplicity, we will take a pre-trained DistilBERT without finetuning it, as the purpose is to show how to deploy a model with confidentiality.

### Step 2: Export it in ONNX format

Because DistilBert uses tracing behind the scenes, we need to feed it an example input.

```python
from transformers import DistilBertTokenizer
import torch

# Create dummy input for export
tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
inputs = tokenizer(sentence, padding = "max_length", max_length = 8, return_tensors="pt")["input_ids"]

# Export the model
torch.onnx.export(
	model, inputs, "./distilbert-base-uncased.onnx",
	export_params=True, opset_version=11,
	input_names = ['input'], output_names = ['output'],
	dynamic_axes={'input' : {0 : 'batch_size'},
	'output' : {0 : 'batch_size'}})

print(inputs.shape) # We print inputs.shape because we will need it to upload the model to the server
```

## Run the BlindAI server

This page explains how to work with the simulation mode. This simulates Intel SGX in software, and enables you to run this on any hardware you want.

To deploy on real hardware in non-simulation mode, take a look at [deploy-on-hardware.md](deploy-on-hardware.md "mention") and skip the first step.

To quickly setup an SGX-enabled virtual machine on Azure, take a look at [cloud-deployment](cloud-deployment.md "mention").

Launch the server using the simulation docker image:

```bash
docker run -it \
    -p 50051:50051 \
    -p 50052:50052 \
    mithrilsecuritysas/blindai-server-sim:latest
```

!!! info
    Please make sure the ports 50051 and 50052 are available.

**Please keep in mind that this image is not secure, since it simulates Intel SGX in software. It is lighter than hardware mode, and should not be used in production.**

## Send and run the model

Now that the model is prepared and exported in **ONNX Format.** We will see how to use **BlindAI** to **deploy** and **run** DistilBERT.

### Step 1: Install the client SDK

BlindAI Client is a python package that provides a simple and straightforward way to connect with BlindAI Server.

You can install the latest version of the client using `pip`.

```bash
pip install blindai 
```

For now, the library is only compatible with Linux. We are working on a native Windows version, but If you are using Windows, you can still use the library with [Windows Subsystem for Linux](https://docs.microsoft.com/fr-fr/windows/wsl/install) in the meantime.&#x20;

### &#x20;Step 2: Send the model

You can run the following script in order to send the model to the server:

```python
from blindai.client import BlindAiConnection, ModelDatumType
import torch

client = BlindAiConnection(addr="localhost", simulation=True)

response = client.upload_model(
    model="./distilbert-base-uncased.onnx", 
    shape=torch.Size([1, 8]), # replace by the previously printed values if necessary
    dtype=ModelDatumType.I64
    )

print(response.model_id) # we need the model_id to run the inference
```

The client is straightforward, we require an address, so if you have loaded the inference server on the same machine, simply mention "localhost" as we did. For simplicity, in the simulation mode, `connect_server` simply creates an insecure channel with the server. This is meant as a quick way to test without requiring specific hardware, so **do not use the simulation mode in production**.

For the `upload_model` method, we need to specify the ONNX file, the shape of the inputs, and the type of data. Here because we run a BERT model, the inputs would be integers to represent the different tokens sent to the model.

For more details about the client API, check the [API reference](../resources/blindai/client.html).

### Step 3: Run the inference

The process is as straightforward as before, simply tokenize the input you want before sending it. As of now, the tokenization must happen at the client-side, but we will implement it shortly in the server-side, so that the client interface remains lightweight.

```python
from transformers import DistilBertTokenizer

# Prepare the inputs
tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
sentence = "I love AI and privacy!"
inputs = tokenizer(sentence, padding = "max_length", max_length = 8)["input_ids"]
```

Now we simply have to create our client, connect and send data to be analyzed. In the same fashion as before, we will create a client in simulation, and simply send data to be analyzed with the proper communication channel.

```python
from blindai.client import BlindAiConnection

# Load the client
client = BlindAiConnection("localhost", simulation=True)

# Get prediction
response = client.run_model(model_id, inputs) # replace model_id by its previously printed value.

print(response.output)
```

For more details about the client API, check the [API reference](../resources/blindai/client.html).

