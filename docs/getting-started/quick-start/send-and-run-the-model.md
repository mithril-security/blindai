# Send and run the model

Now that the model is prepared and exported in **ONNX Format.** We will see how to use **BlindAI** to **deploy** and **run** DistilBERT.&#x20;

## Step 1: Install the client SDK

BlindAI Client is a python package that provides a simple and straightforward way to connect with BlindAI Server.

You can install the latest version of the client using `pip`.

```bash
pip install blindai 
```

For now, the library is only compatible with Linux. We are working on a native Windows version, but If you are using Windows, you can still use the library with [Windows Subsystem for Linux](https://docs.microsoft.com/fr-fr/windows/wsl/install) in the meantime.&#x20;

## &#x20;Step 2: Send the model

You can run the following script in order to send the model to the server:

```python
from blindai.client import BlindAiClient, ModelDatumType

client = BlindAiClient()

client.connect_server(addr="localhost", simulation=True)

client.upload_model(
    model="./distilbert-base-uncased.onnx", 
    shape=inputs.shape, 
    dtype=ModelDatumType.I64
    )
```

The client is straightforward, we require an address, so if you have loaded the inference server on the same machine, simply mention "localhost" as we did. For simplicity, in the simulation mode, `connect_server` simply creates an insecure channel with the server. This is meant as a quick way to test without requiring specific hardware, so **do not use the simulation mode in production**.

For the `upload_model` method, we need to specify the ONNX file, the shape of the inputs, and the type of data. Here because we run a BERT model, the inputs would be integers to represent the different tokens sent to the model.

For more details about the client API, check the [API reference](../../resources/client-api-reference/client-interface-1.md).

## Step 3: Run the inference

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
from blindai.client import BlindAiClient

# Load the client
client = BlindAiClient()
client.connect_server("localhost", simulation=True)

# Get prediction
response = client.run_model(inputs)
```

For more details about the client API, check the [API reference](../../resources/client-api-reference/client-interface-1.md).

