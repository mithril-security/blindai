# BlindAI Client

BlindAI Client is a python library to create client applications for BlindAI Server (Mithril-security's confidential inference server). 

## Installation

### Using pip
```bash
$ pip install blindai
```

### Build from source
In order to build the package from this repository, the following requirements must be satisfied: 
**On Linux**
- CMake >= 3.12
- g++ >= 7.1
- python >= 3.6.8
- python-dev package (or python-devel in CentOs based distros) 

Once the requirements are satisfied, proceed as following:

1. Clone the repository
```bash
$ git clone https://github.com/mithril-security/blindai-client
```
2. Install third party libraries
```bash
$ git submodule init
$ git submodule update
```
3. Create and activate a python virtual environemnt 
```bash
$ python3 -m venv env
$ source env/bin/activate
```
- Check pip version (pip 21 is needed)
```bash
$ pip --version
```
- If the installed version is pip 9.x.x , upgrade pip 
```bash
$ pip install -U pip
```
4. Install development dependencies
```bash
$ pip install -r requirements.txt
```

5. Trigger the build and installation
```bash
pip install .
```

## Usage

```python
from blindai.client import BlindAiClient
from PIL import Image
import numpy as np

#Create the connection
client = BlindAiClient()

client.connect_server(
    "localhost",
    policy="policy.toml",
    certificate="host_server.pem",
    simulation=False
)

#Upload the model to the server
response = client.upload_model(model="./mobilenetv2-7.onnx", shape=(1, 3, 224, 224))

if response.ok:
    print("Model loaded")
    image = Image.open("grace_hopper.jpg").resize((224,224))

    #Preprocess the data 
    a = np.asarray(image, dtype=float)
    mean =np.array([0.485, 0.456, 0.406])
    std = np.array([0.229, 0.224, 0.225])
    a = (a / 255.0 - mean) / std
    a = np.moveaxis(a, 2, 0)

    #Send data for inference
    result = client.send_data(a.flatten())

client.close_connection()
```

In order to connect to the BlindAI server, the client needs to acquire the following files from the server: 

- **policy.toml :** the enclave security policy that defines which enclave is trusted.

- **host_server.pem :** TLS certificate for the connection to the untrusted (app) part of the server.

**Simulation mode** enables to pypass the process of requesting and checking the attestation.

Usage examples can be found in [tutorial](./tutorial) folder.

Before you run an example, make sure to get `policy.toml` and `host_server.pem` that are generated in the server side. 

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.


## License
