# BlindAI Client
## Introduction
BlindAI client is a python package that simplifies the connection to BlindAI inference server from python client applications, by providing a simple interface for it. 

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
| addr| ```str``` | The address of BlindAI server you want to reach. |
| server_name | ``str`` | Contains the CN expected by the server TLS certificate. |
| certificate | ``str``| path to the public key of the untrusted inference server. Generated in the server side. |
| policy | ``str`` | path to the toml file describing the policy of the server. Generated in the server side. |
| simulation | ``bool`` | Connect to the server in simulation mode (default False). If set to yes, the args policy and certificate will be ignored.|

Returns True if the connection was successful. False otherwise.

---
### **upload_model (model, shape) -> SimpleReply**
Upload a pretrained model in ONNX format to BlindAI server.

| Param | Type | description |
| --- | --- | --- |
| model | ``str``| path to Onnx model file|
| shape | ``(int,)`` | the shape of the model input |
| dtype | ``ModelDatumType`` | the type of the model input data |

Returns a **``SimpleReply``** object with the following fields:

| field | Type | description |
| ----- | --- | --- |
|  ok   | ``bool`` | Set to True if model was loaded successfully, False otherwise |
|  msg  | ``str`` | Error message if any. | 
---
### **run_model (data) -> ModelResult**
Send data to the server to make a secure inference.

| Param | Type | description |
| --- | --- | --- |
| data_list | ``[number]``| array of numbers, the numbers must be of the same type ``dtype`` specified in `upload_model`| 

Returns a **``ModelResult``** object with the following fields:
| field | Type | description |
| ----- | --- | --- |
| output | ``[float]`` | array of floats. The inference results returned by the model. | 
|  ok   | ``bool`` | Set to True if the inference was run successfully, False otherwise |
|  msg  | ``str`` | Error message if any. | 

---
### **close_connection ( )**
Close the connection between the client and the inference server. 
