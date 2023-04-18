# Build BlindAI server from source
________________________________________

!!! info

    If you're building the client because you **want to change it**, you should first go [set up your dev environment](../contributing/setting-up-your-dev-environment.md) and follow the instructions from the section "Without Docker" there.

## Build process with Docker

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

## Build without Docker
____________________

Once you've [set up your dev environment](../contributing/setting-up-your-dev-environment.md "mention"), be sure to fulfill the [hardware requirements](../../tutorials/core/installation.md) and to have the necessary dependencies and services up and running.

You can find how to install the Intel SDK and Fortanix EDP in the [dev environment setup guide](../contributing/setting-up-your-dev-environment.md#bare-bones-dev-environment "mention").

Afterwards, you can view the SGX configuration and services using the command :

```bash
sgx-detect
```
You will see something like this :
```
Detecting SGX, this may take a minute...
✔  SGX instruction set
  ✔  CPU support
  ✔  CPU configuration
  ✔  Enclave attributes
  ✔  Enclave Page Cache
  SGX features
    ✔  SGX2  ✔  EXINFO  ✘  ENCLV  ✘  OVERSUB  ✔  KSS
    Total EPC size: 2.0GiB (no integrity protection)
✔  Flexible launch control
  ✔  CPU support
  ？ CPU configuration
  ✔  Able to launch production mode enclave
✔  SGX system software
  ✔  SGX kernel device (/dev/sgx_enclave)
  ✔  libsgx_enclave_common
  ✔  AESM service
  ✔  Able to launch enclaves
    ✔  Debug mode
    ✔  Production mode
    ✔  Production mode (Intel whitelisted)

You're all set to start running SGX programs!
```

Before running the BlindAi project, the packages [jq](https://stedolan.github.io/jq/) and [just](https://just.systems/man/en/) must be installed:
```bash
sudo apt-get install jq

cargo install just
```

=== "Hardware mode"


    You will need the SGX Default Quote Provider Library. This can be installed with this command:

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
    just build

    # for generating the manifest file
    just generate-manifest-dev
    ```

The manifest will be generated at the build process and will be essential to the remote attestation process:

* `manifest.toml`: the enclave security manifest that defines which enclave is trusted.


You can learn more about them in [our guide on confidential computing](../../getting-started/confidential_computing.md) and in the [remote attestation implementation](../../security/remote_attestation.md) in depth security explanation.

## Run the server
____________________________

Once everything is ready, you can run BlindAI!

=== "Hardware mode"

    We can run blindai using :

    ```bash
    just run
    ```


=== "Hardware mode (Azure DCsv3 VMs)"

	We can run blindai using :

    ```bash
    BLINDAI_AZURE_DCS3_PATCH=1 just run
    ```


!!! info

    If you have trouble building and installing from source, don't hesitate to open an issue on our [GitHub](https://github.com/mithril-security/blindai/issues) or reach us on [Discord](https://discord.com/invite/TxEHagpWd4).
