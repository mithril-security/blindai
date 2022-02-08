# BlindAI Client
## Introduction
blindai client is a python package that simplifies the connection to BlindAI inference server from python client applications, by providing a simple interface for it. 

## BlindAiClient

### **connect_server (addr, certificate, policy, simulation)-> bool**
Estabilish a connection with BlindAI inference server and perform the process of requesting and verifying the attestation.

| Param | Type | description |
| --- | --- | --- |
| addr| ```str``` | the address of BlindAI server |
| certificate | ``str``| path to the public key of the untrusted inference server. Generated in the server side. |
| policy | ``str`` | path to the toml file describing the policy of the server. Generated in the server side. |
| simulation | ``bool`` | connect to the server in the simulation mode or not (default `False`). |


Returns a boolean describing whether the connection was successful or not.

### **upload_model (model, shape) -> dict**
Upload a pretrained model in ONNX format to BlindAI server.

| Param | Type | description |
| --- | --- | --- |
| model | ``str``| path to onnx trained model |
| shape | ``(int, int, int, int)`` | the shape of the model input |

Returns a **``dict``** with the following keys:

| key | Type | description |
| --- | --- | --- |
| ok  | ``bool`` | True if the model is successfully upload |
| msg | ``str`` | Error message | 

### **send_data (data) -> dict**
Send data to  BlindAI server to perform the inference.

| Param | Type | description |
| --- | --- | --- |
| data | ``[float]``| array of float| 

Returns a **``dict``** with the following keys:
| key | Type | description |
| --- | --- | --- |
| output | ``[float]`` | Output tensor returned by the model | 
| ok | ``bool`` | True if the model is successfully upload |
| msg | ``str`` | Error message | 


### **close_connection ( )**
Close the connection between the client and the inference server. 