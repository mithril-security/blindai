# Build the server from source

## Using Docker 🐳

### Build process

You can build the whole project by using our Docker image. We have set up the Docker image to have a reproducible build no matter the environment. You can start the process with those commands:

=== "Simulation mode"
    ```bash
    cd server
    make init # create the TLS certificates
    DOCKER_BUILDKIT=1 docker build \
        --target software \
        -t mithrilsecuritysas/blindai-server-sim:latest \
        -f ./docker/build.dockerfile \
        .
    ```

=== "Hardware mode"
    ```bash
    cd server
    make init # create the TLS certificates
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
    make init # create the TLS certificates
    DOCKER_BUILDKIT=1 docker build \
        --target hardware-dcsv3 \
        -t mithrilsecuritysas/blindai-server-dcsv3:latest \
        -f ./docker/build.dockerfile \
        .
    ```
    This will create a policy file with `allow_debug = false`. To change that, use `-e POLICY_ALLOW_DEBUG=true` when building.


### Run the compiled server

To run the client, you will want to get the `policy.toml` and `host_server.pem`  file from the server using:

=== "Simulation mode"
    This step can be ignored when running in software mode.

=== "Hardware mode"
    ```bash
    docker run mithrilsecuritysas/blindai-server:latest /bin/cat /root/policy.toml > policy.toml
    ```
    ```bash
    docker run mithrilsecuritysas/blindai-server:latest /bin/cat /root/tls/host_server.pem > host_server.pem
    ```

=== "Hardware mode (Azure DCsv3 VMs)"
    ```bash
    docker run mithrilsecuritysas/blindai-server-dcsv3:latest /bin/cat /root/policy.toml > policy.toml
    ```
    ```bash
    docker run mithrilsecuritysas/blindai-server-dcsv3:latest /bin/cat /root/tls/host_server.pem > host_server.pem
    ```
You need to run this command to start the docker image:

=== "Simulation mode"
    ```bash
    docker run -it \
        -p 50051:50051 \
        -p 50052:50052 \
        mithrilsecuritysas/blindai-server-sim:latest
    ```

=== "Hardware mode"
    Make sure you have the correct hardware and drivers (see [#hardware-requirements](../getting-started/deploy-on-hardware.md#hardware-requirements "mention")), and run:

    ```bash
    docker run -it \
        -p 50051:50051 \
        -p 50052:50052 \
        --device /dev/sgx/enclave \
        --device /dev/sgx/provision \
        mithrilsecuritysas/blindai-server:latest /root/start.sh PCCS_API_KEY
    ```

    A [Quote Provisioning Certificate Caching Service (PCCS)](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/pccs/README.md) is built-in inside the Docker Image in order to generate the DCAP attestation from the enclave. You need to provide an API Key in order for the PCCS server to function. [You can get one from Intel here.](https://api.portal.trustedservices.intel.com/provisioning-certification)

    !!! info
        The `PCCS_API_KEY` needs to be replaced with the PCCS API Key.

    !!! info
        This will launch the enclave in non debug-mode. If you wish to launch in debug mode, use `-e ENCLAVE_DEBUG_MODE=true` when launching.

=== "Hardware mode (Azure DCsv3 VMs)"
    ```bash
    docker run -it \
        -p 50051:50051 \
        -p 50052:50052 \
        --device /dev/sgx/enclave \
        --device /dev/sgx/provision \
        mithrilsecuritysas/blindai-server-dcsv3:latest
    ```

    !!! info
        This will launch the enclave in non debug-mode. If you wish to launch in debug mode, use `-e ENCLAVE_DEBUG_MODE=true` when launching.


## Without docker

Make sure to follow [setting-up-your-dev-environment.md](setting-up-your-dev-environment.md "mention") first to set up your environment and install the build dependencies.


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

### Running

=== "Software mode"

    ```bash
    cd bin
    ./blindai_app
    ```

=== "Hardware mode"

    Make sure you have the correct hardware and drivers (see [#hardware-requirements](../getting-started/deploy-on-hardware.md#hardware-requirements "mention"))

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


