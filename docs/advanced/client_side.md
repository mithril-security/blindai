# Build the BlindAI Client SDK from source

**BlindAI Client SDK** can currently be built from source on **Linux** and **Windows** platforms.&#x20;

## Requirements

Before proceeding to build the client, make sure the following requirements are installed in your environment.&#x20;



* CMake >= 3.12
* Make >= 4.0
* g++ >= 7.1
* python >= 3.6.8
* python3-dev package (or python-devel in CentOs based distros) - The version of python3-dev depends on the version of python you are using.



* Microsoft visual Studio 2017 15 with CMake installed
* Windows PowerShell
* Perl
* python >= 3.6.8



## Building and installing the package

### **Clone the repository**

```bash
git clone https://github.com/mithril-security/blindai
cd blindai/client
```

### Install third party libraries

```bash
git submodule init
git submodule update
```

### Create and activate a virtual environment



```bash
python3 -m venv env
source env/bin/activate
```



```powershell
python -m venv env
.\env\Scripts\activate.exe
```



### Check pip version


**pip >= 21** is needed, so make sure to check your pip version,**i**s and to update it in case a prior version was installed.


* Check pip version

```bash
$ pip --version
```

* If the installed version is pip 9.x.x , upgrade pip

```bash
$ pip install -U pip
```

### Install development dependencies

```bash
$ pip install -r requirements.txt
```

### Trigger the Build process

```bash
pip install . --use-feature=in-tree-build
```

!!! info
    If you are building on windows, administrator access will be needed at a certain point of the build process.

BlindAI Client SDK will be then built and installed in the virtual environment.
