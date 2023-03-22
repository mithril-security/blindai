# Build the server from source

!!! info
    If you're building the client because you want to change it, you should first go to [the setting up your dev environment guide](../contributing/setting-up-your-dev-environment.md) and then build without docker.

<!-- ## Using Docker 🐳

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
    This will create a manifest file with `allow_debug = false`. To change that, use `-e MANIFEST_ALLOW_DEBUG=true` when building.

=== "Hardware mode (Azure DCsv3 VMs)"
    ```bash
    cd server

    DOCKER_BUILDKIT=1 docker build \
        --target hardware-dcsv3 \
        -t mithrilsecuritysas/blindai-server-dcsv3:latest \
        -f ./docker/build.dockerfile \
        .
    ```
    This will create a manifest file with `allow_debug = false`. To change that, use `-e MANIFEST_ALLOW_DEBUG=true` when building.

!!! info
    If your goal is to obtain a manifest.toml file to connect to a distant server. You should build the image in hardware mode (sgx support isn't needed for compilation). You can then extract it by running:
    ```bash
    docker run --rm <image_name> cat /root/manifest.toml > manifest.toml
    ``` -->
<!-- 
### Running
You can use these images by following the instructions of either the [deploy on premise guide](../../deploy-on-premise.md) or the [cloud deployment guide](../../cloud-deployment.md).
 -->

## Without docker

### Build process

Make sure to [set up a dev environment](../contributing/setting-up-your-dev-environment.md "mention") to easily install the build dependencies.


Before building the project, some dependencies and service must be up and running, and the hardware requirements must be installed ([hardware-requirements](../../getting-started/installation.md)).

The installation of the Intel SDK can be found by following this [link](https://github.com/intel/linux-sgx).

The fortanix EDP dependencies must also be installed. You can check the official fortanix [documentation here](https://edp.fortanix.com/docs/installation/guide/). 

The SGX configuration and services can be viewed using the command : 

```bash
sgx-detect
```

Before running the BlindAi project, some other packages must be installed: 
```bash
sudo apt install jq

cargo install just
```



=== "Hardware mode"


    You will need the SGX Default Quote Provider Library as well. This can be installed with this command:

    ```bash
    sudo apt-get install -y software-properties-common 
    curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add - 

    sudo add-apt-repository "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main" 
    sudo apt-get install -y libsgx-dcap-ql-dev libsgx-dcap-default-qpl-dev libsgx-uae-service libsgx-dcap-default-qpl
    ```    
    You will also need to install the Provisioning Certificate Caching Service (PCCS) [by following this documentation](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/pccs/README.md) (The PCCS must be installed directly from the Github repo as it is not yet updated by Intel on their repo). During installation, a PCCS Key will be asked. This key is delivered by Intel [here](https://api.portal.trustedservices.intel.com/provisioning-certification). 

    We can clone the BlindAi repo on Github then build the server using the following steps:

    ```bash
    git submodule init
    git submodule update

    cd server
    just build 

    # for generating the manifest file 
    just generate-manifest-dev 
    ```

=== "Azure DCsv3 VMs mode"
    Make sure to have the DCs v3 quote provision library:

    ```bash
    curl -sSL https://packages.microsoft.com/keys/microsoft.asc | sudo apt-key add -
    sudo apt-add-repository https://packages.microsoft.com/ubuntu/20.04/prod
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

The manifest will be generated at the build process and will serve as essential to the remote attestation process:

* `manifest.toml`: the enclave security manifest that defines which enclave is trusted.


More informations about them on [this page](../../concepts/privacy.md) and in the [remote attestation implementation](../security/remote_attestation.md).

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

    BLINDAI_AZURE_DCS3_PATCH=1 just run 
    ```


!!! info
    If you have trouble building and installing from source, don't hesitate to open an issue on our github.  
