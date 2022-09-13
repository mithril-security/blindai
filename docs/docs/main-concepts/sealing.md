# Model sealing

## Model sealing in BlindAI

When you upload a model on a BlindAI server instance, it is by default **sealed**. This means that the model will be serialized, encrypted with a hardware-backed sealing key and saved on the server's machine. At start up, blindAI will unseal previously saved models so that you don't have a to re-upload them at each server restart.

In SGX, once something is sealed it can only be unsealed by an enclave having the exact same policy than the one who sealed it. This ensure that sealed models can't be compromised.

## Deactivating sealing

if you want to upload a model without sealing it you can specify it at the upload like so :

```python
client.upload_model(model='your_model', save_model=False)
```