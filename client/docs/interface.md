# BlindAI Client
## Introduction
blindai client is a python package that simplifies the connection to BlindAI inference server from python client applications, by providing a simple interface for it. 

## BlindAiClient

### **ModelDatumType**
An enumeration of the acceptable input data types. Used to specify the type of the input data of a model before uploading it to the server.

| Member  | Type | 
|---|---|
| F32 | float32 |
| F64 | float64 |
| I32 | int32 |
| I64 | int64 |
| U32 | unsigned32 |
| U64 | unsigned64 |

### **connect_server (addr, certificate, policy, simulation)-> bool**
Estabilish a connection with BlindAI inference server and perform the process of requesting and verifying the attestation.

| Param | Type | description |
| --- | --- | --- |
| addr| ```str``` | the address of BlindAI server |
| certificate | ``str``| path to the public key of the untrusted inference server. Generated in the server side. |
| policy | ``str`` | path to the toml file describing the policy of the server. Generated in the server side. |
| simulation | ``bool`` | connect to the server in the simulation mode (default `False`). |
| no_untrusted_cert_check |``bool`` | bypass the verification of the untrusted server certificate (default `False`) |

Returns a boolean describing whether the connection was successful or not.

---
### **upload_model (model, shape) -> dict**
Upload a pretrained model in ONNX format to BlindAI server.

| Param | Type | description |
| --- | --- | --- |
| model | ``str``| path to model file|
| shape | ``(int,)`` | the shape of the model input |
| datum | ``ModelDatumType`` | the type of the model input data |

Returns a **``dict``** with the following keys:

| key | Type | description |
| --- | --- | --- |
| ok  | ``bool`` | True if the model is successfully uploaded |
| msg | ``str`` | message from the server | 
---
### **run_model (data) -> dict**
Send data to  BlindAI server to perform the inference.

| Param | Type | description |
| --- | --- | --- |
| data | ``[number]``| array of numbers, the numbers must be of the same type ``datum`` specified in `upload_model`| 

Returns a **``dict``** with the following keys:
| key | Type | description |
| --- | --- | --- |
| output | ``[float]`` | Output tensor returned by the model | 
| ok | ``bool`` | True if the model is successfully upload |
| msg | ``str`` | message from the server | 

---
### **close_connection ( )**
Close the connection between the client and the inference server. 
