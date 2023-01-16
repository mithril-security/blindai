> **Warning : BlindAI Preview**
>  This is a preview version of BlindAI named blindai-preview. It is still under development and has not yet all the features of the current BlindAI. 
> You can found the current blindAI [here](https://github.com/mithril-security/blindai).
    

# 👋 Welcome

**BlindAI** is a **fast, easy-to-use,** and **confidential inference server**, allowing you to easily and quickly deploy your AI models with privacy, **all in Python**. Thanks to the **end-to-end protection guarantees**, data owners can send private data to be analyzed by AI models, without fearing exposing their data to anyone else.

To interact with an AI model hosted on a remote secure enclave, we provide the `blindai.client` API. This client will:

- check that we are talking to a genuine secure enclave with the right security features
- upload an AI model that was previously converted to ONNX
- query the model securely

You can deploy BlindAI on [your own infra](docs/deploy-on-premise.md).

### Installing BlindAI

BlindAI can easily be installed from [PyPI](https://pypi.org/project/blindai/):

```bash
pip install blindai-preview
```

This package is enough for the deployment and querying of models on our managed infrastructure. For on-premise deployment, you will have to deploy our [Docker](https://hub.docker.com/u/mithrilsecuritysas) images, while you can build them yourself as demonstrated in [this section](docs/advanced/build-from-sources/server.md), it is recommanded to start with the prebuilt images.

### Uploading a ResNet18

The model in the GPT2 example had already been loaded by us, but BlindAI also allows you to upload your own models to our managed Cloud solution. 

You can find a [Colab notebook](https://colab.research.google.com/drive/1c8pBM5gN5zL_AT0s4kBZjEGdivWW3hSt?usp=sharing) showing how to deploy and query a ResNet18 on BlindAI.

To be able to upload your model to our Cloud, you will need to [first register](https://cloud.mithrilsecurity.io/) to get an API key.

Once you have the API key, you just have to provide it to our backend. 

```python
import torch
import blindai-preview

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
import blindai-preview
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
with blindai-preview.connect() as client:
  # Send data to the GPT2 model
  response = client.predict("resnet18", input_batch)

>>> response.output[0].argmax()
```