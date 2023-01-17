# Build the server from source

!!! info
    If you're building the client because you want to change it, you should first go to [the setting up your dev environment guide](../setting-up-your-dev-environment.md) and then build without docker.

## Using Docker 🐳

### Build process

You can build the whole project by using our Docker image. We have set up the Docker image to have a reproducible build no matter the environment. You can start the process with those commands:


=== "Hardware mode"
    ```bash
    cd server

    DOCKER_BUILDKIT=1 docker build \
        --target hardware \
        -t mithrilsecuritysas/blindai-server:latest \
        -f ./docker/build.dockerfile \
        .
    ```
    This will create a manifest file with `allow_debug = false`. To change that, use `-e manifest_ALLOW_DEBUG=true` when building.

=== "Hardware mode (Azure DCsv3 VMs)"
    ```bash
    cd server

    DOCKER_BUILDKIT=1 docker build \
        --target hardware-dcsv3 \
        -t mithrilsecuritysas/blindai-server-dcsv3:latest \
        -f ./docker/build.dockerfile \
        .
    ```
    This will create a manifest file with `allow_debug = false`. To change that, use `-e manifest_ALLOW_DEBUG=true` when building.

!!! info
    If your goal is to obtain a manifest.toml file to connect to a distant server. You should build the image in hardware mode (sgx support isn't needed for compilation). You can then extract it by running:
    ```bash
    docker run --rm <image_name> cat /root/manifest.toml > manifest.toml
    ```

### Running
You can use these images by following the instructions of either the [deploy on premise guide](../../deploy-on-premise.md) or the [cloud deployment guide](../../cloud-deployment.md).


## Without docker

### Build process

Make sure to [set up a dev environment](../setting-up-your-dev-environment.md "mention") to easily install the build dependencies.


Before building the project, some dependencies and service must be up and running, and the hardware requirements must be installed ([hardware-requirements](../../deploy-on-premise.md#hardware-requirements)).

Apart from the SGX drivers, Rust must also be installed and running in default in nightly. 

And finally the fortanix EDP dependencies must also be installed. You can check the official fortanix [documentation here](https://edp.fortanix.com/docs/installation/guide/). 

The SGX configuration and services can be viewed using the command : 

```bash
sgx-detect
```



=== "Hardware mode"

    You will also need to install the Provisioning Certificate Caching Service (PCCS) [using this documentation](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/pccs/README.md).

    You will need the SGX Default Quote Provider Library as well. This can be installed with this command:

    ```bash
    apt update && apt install -y libsgx-dcap-default-qpl-dev
    ```    

    We can then build the server using :

    ```bash

    cd server
    just build 

    # for generating the manifest file 
    just generate-manifest-dev 
    ```

=== "Azure DCsv3 VMs mode"
    Make sure to have the DCs v3 quote provision library:

    ```bash
    curl -sSL https://packages.microsoft.com/keys/microsoft.asc | sudo apt-key add -
    sudo apt-add-repository https://packages.microsoft.com/ubuntu/18.04/prod
    sudo apt-get update
    sudo apt-get install az-dcap-client
    ln -s /usr/lib/libdcap_quoteprov.so /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so.1
    ```
    We can then build the server using : 

    ```bash

    cd server
    just build 

    # for generating the manifest file 
    just generate-manifest-dev 
    ```

Two files will be generated after the building process:

* `manifest.toml`: the enclave security manifest that defines which enclave is trusted.

You will need these two files for running the client in non-simulation mode.

More informations about them on [this page](../../main-concepts/privacy.md)

### Running


Once you are sure to have everything ready, you can run BlindAI.

=== "Hardware mode"

    We can run blindai using : 

    ```bash
    cd server 

    just run
    ```


=== "Hardware mode (Azure DCsv3 VMs)"



    ```bash
    cd server

    just run 
    ```


!!! info
    If you have trouble building and installing from source, don't hesitate to open an issue on our github.  
