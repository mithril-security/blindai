# Build the BlindAI Client SDK from source

**BlindAI Client SDK** can currently be built from source on **Linux** and **Windows** platforms.&#x20;

!!! info
    If you're building the client because you want to change it, you should rather got to [the setting up your dev environment page](../contributing/setting-up-your-dev-environment.md)

## Requirements

Before proceeding to build the client, make sure the following requirements are installed in your environment.&#x20;

* python >= 3.7
* python3-dev package (or python-devel in CentOs based distros) - The version of python3-dev depends on the version of python you are using.
* poetry



## Building and installing the package

### **Clone the repository**

```bash
git clone https://github.com/mithril-security/blindai-preview
cd blindai-preview/client
```

### Install third party libraries

```bash
git submodule init
git submodule update
```

### Check pip version


**pip >= 21** is needed, so make sure to check what your pip version is and to update it in case a prior version was installed.

* Check pip version

```bash
$ pip --version
```

* If the installed version is pip 9.x.x , upgrade pip

```bash
$ pip install -U pip
```

### Installing the client using Poetry

On the client, we deployed a Poetry virtual environment to make it easier to build and run blindAI Client. 

You just have to install poetry by using pip and run it by following these steps :


```bash
$ pip install poetry && cd client/

(client-py)$ poetry shell 

(client-py)$ poetry install

```


BlindAI Client SDK will be then built and installed in the virtual environment.
