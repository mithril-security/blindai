# Build the server from source

!!! info
    If you're building the client because you want to change it, you should first go to [the setting up your dev environment guide](../setting-up-your-dev-environment.md) and then build without docker.

## Using Docker 🐳

### Build process

You can build the whole project by using our Docker image. We have set up the Docker image to have a reproducible build no matter the environment. You can start the process with those commands:

=== "Simulation mode"
    ```bash
    cd server
    make init # create the TLS certificates and pull teaclave
    DOCKER_BUILDKIT=1 docker build \
        --target software \
        -t mithrilsecuritysas/blindai-server-sim:latest \
        -f ./docker/build.dockerfile \
        .
    ```

=== "Hardware mode"
    ```bash
    cd server
    make init # create the TLS certificates and pull teaclave
    DOCKER_BUILDKIT=1 docker build \
        --target hardware \
        -t mithrilsecuritysas/blindai-server:latest \
        -f ./docker/build.dockerfile \
        .
    ```
    This will create a policy file with `allow_debug = false`. To change that, use `-e POLICY_ALLOW_DEBUG=true` when building.

=== "Hardware mode (Azure DCsv3 VMs)"
    ```bash
    cd server
    make init # create the TLS certificates and pull teaclave
    DOCKER_BUILDKIT=1 docker build \
        --target hardware-dcsv3 \
        -t mithrilsecuritysas/blindai-server-dcsv3:latest \
        -f ./docker/build.dockerfile \
        .
    ```
    This will create a policy file with `allow_debug = false`. To change that, use `-e POLICY_ALLOW_DEBUG=true` when building.

!!! info
    If your goal is to obtain a policy.toml file to connect to a distant server. You should build the image in hardware mode (sgx support isn't needed for compilation). You can then extract it by running:
    ```bash
    docker run --rm <image_name> cat /root/policy.toml > policy.toml
    ```

### Running

You can use these images by following the instructions of either the [deploy on premise guide](../../deploy-on-premise.md) or the [cloud deployment guide](../../cloud-deployment.md)


## Without docker

### Build process

Make sure to [set up a dev environment](../setting-up-your-dev-environment.md "mention") to easily install the build dependencies.

Then first of all :
```bash
make init
```
It will pull the right version of teaclave, alongside creating the tls certificate

=== "Software mode"

    ```bash
    cd server
    make SGX_MODE=SW
    ```

=== "Hardware mode"

    ```bash
    cd server
    make
    ```

=== "Hardware mode (Azure DCsv3 VMs)"

    ```bash
    cd server
    make
    ```

Two files will be generated after the building process:

* `policy.toml`: the enclave security policy that defines which enclave is trusted.
* `host_server.pem`: TLS certificate for the connection to the untrusted (app) part of the server.

You will need these two files for running the client in non-simulation mode.

More informations about them on [this page](../../main-concepts/privacy.md)

### Running

=== "Software mode"

    ```bash
    cd bin
    ./blindai_app
    ```

=== "Hardware mode"

    Make sure you have the correct hardware and drivers (see [hardware-requirements](../../deploy-on-premise.md#hardware-requirements))

    You will also need to install the Provisionning Certificate Caching Service (PCCS) [using this documentation](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/pccs/README.md).

    You will need the SGX Default Quote Provider Library as well. This can be installed with this command:

    ```bash
    apt update && apt install -y libsgx-dcap-default-qpl-dev
    ```

    Once you are sure to have everything ready, you can run BlindAI.

    ```bash
    cd bin
    ./blindai_app
    ```

=== "Hardware mode (Azure DCsv3 VMs)"

    Make sure to have the DCs v3 quote provision library:

    ```bash
    curl -sSL https://packages.microsoft.com/keys/microsoft.asc | sudo apt-key add -
    sudo apt-add-repository https://packages.microsoft.com/ubuntu/18.04/prod
    sudo apt-get update
    sudo apt-get install az-dcap-client
    ln -s /usr/lib/libdcap_quoteprov.so /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so.1
    ```

    Once you are sure to have everything ready, you can run BlindAI.

    ```bash
    cd bin
    export BLINDAI_AZURE_DCSV3_PATCH=1
    ./blindai_app
    ```


