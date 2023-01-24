# Deploy on premise

The docker images used here are prebuilt ones from our dockerhub, you can take a look at the [build the server from source section]('build-from-sources/server.md')

!!! warning
    The unsecure connection is on HTTP only. In production mode, it is highly recommended to connect it to a **reverse-proxy** that creates a TLS connection between the end user and the BlindAI server.  

<!-- ## Simulation mode

This section explains how to work with the simulation mode. This simulates Intel SGX in software and enables you to run this on any hardware you want.

Launch the server using the simulation docker image:

```bash
docker run -it \
    -p 50051:50051 \
    -p 50052:50052 \ 
    mithrilsecuritysas/blindai-server-sim:latest # make sure the ports 50051 and 50052 are available.
```

!!! warning
    Please keep in mind that this image is not secure, since it simulates Intel SGX in software. It is lighter than hardware mode, and should not be used in production. -->

## Hardware mode

### Hardware requirements

!!! info 
    In some cases (Linux kernel >5.15) the execution of the binary returns `in-kernel drivers support`, and it means that the drivers are already installed and must appear in `/dev/sgx/`. 


=== "Hardware mode"


    You will need to have an Intel SGX-ready device, with `SGX+FLC` (Flexible Launch Control) support. Read [this Intel documentation page](https://www.intel.com/content/www/us/en/support/articles/000057420/software/intel-security-products.html) to see if your Intel processor supports it.

    Please make sure to have the `SGX+FLC` drivers (preferably with version **1.41**) installed on your system before running the docker image.

    !!! success
        If you can find the drivers named "enclave" and "provision" (or sgx\_enclave and sgx\_provision) in /dev/, you are good to go!

    !!! failure
        If on the other hand, you can find a driver named "isgx", that means your system is not supported. This driver is for the first generation of SGX, which lacks the security features we rely on. You can still boot the server in hardware mode and benefit from the isolation offered by SGX enclaves, but you will need to use the client in simulation mode.

    In case you don't have any drivers installed, you can install the drivers with this:

    ```bash
    wget https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/sgx_linux_x64_driver_1.41.bin
    chmod +x sgx_linux_x64_driver_1.41.bin
    ./sgx_linux_x64_driver_1.41.bin
    ```

    The binary file contains the drivers signed by Intel and will proceed to the installation transparently.



=== "Hardware mode (Azure DCsv3 VMs)"

    There is no need to do anything, the drivers are already installed.


### Installation of the AESM service

!!! info
    The AESM service is currently only supported outside a docker container and thus must be installed separately. We're working on making it more easier to install by running it directly with BlindAi, in the next iterations. 

To install the AESM service and run it, you can follow the steps described below: 

```bash
echo "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main" | sudo tee -a /etc/apt/sources.list.d/intel-sgx.list >/dev/null \ 
curl -sSL "https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key" | sudo apt-key add -
sudo apt-get update \
sudo apt-get install -y sgx-aesm-service libsgx-aesm-launch-plugin
```
You can verify that the service is running by typing :
```bash
service aesmd status
```
The current user must also be added to the aesm group to be able to function properly : 
```bash
sudo usermod -a -G aesmd $USER
```

### Running the server

Please make sure you have [Docker](https://docs.docker.com/get-docker/) installed on your machine.

=== "Hardware mode"

    A [Quote Provisioning Certificate Caching Service (PCCS)](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/pccs/README.md) is built-in inside the Docker Image in order to generate the DCAP attestation from the enclave. You need to provide an API Key in order for the PCCS server to function. [You can get one from Intel here.](https://api.portal.trustedservices.intel.com/provisioning-certification)

    ```bash
    docker run -it \
        -p 9923:9923 \
        -p 9924:9924 \
        --device /dev/sgx/enclave \
        --device /dev/sgx/provision \
        -v /var/run/aesmd:/var/run/aesmd \
        mithrilsecuritysas/blindai-server:latest /root/start.sh PCCS_API_KEY
    ```

    !!! info
        The `PCCS_API_KEY` needs to be replaced with the PCCS API Key.

=== "Hardware mode (Azure DCsv3 VMs)"

    ```bash
    docker run -it \
        -v $(pwd)/bin/tls:/root/tls \
        -p 9923:9923 \
        -p 9924:9924 \
        --device /dev/sgx/enclave \
        --device /dev/sgx/provision \
        -v /var/run/aesmd:/var/run/aesmd \
        mithrilsecuritysas/blindai-server-dcsv3:latest
    ```

!!! info
    If you built this image locally you can allow debug by running with -e MANIFEST_ALLOW_DEBUG=true. Building from sources is documented [here](advanced/build-from-sources/server.md).

!!! warning
    You should only allow debug if your manifest.toml allows debug.

### Extract manifest and default TLS Certificate from the Hardware docker image

You can extract the manifest directly from the prebuilt Docker Image using:

=== "Hardware mode"

    ```bash
    docker run --rm mithrilsecuritysas/blindai-server:latest /bin/cat /root/manifest.toml > manifest.toml
    ```

=== "Hardware mode (Azure DCsv3 VMs)"

    ```bash
    docker run --rm mithrilsecuritysas/blindai-server-dcsv3:latest /bin/cat /root/manifest.toml > manifest.toml
    ```

You can also extract the default TLS certificate like this:

=== "Hardware mode"

    ```bash
    docker run --rm mithrilsecuritysas/blindai-server:latest /bin/cat /root/tls/host_server.pem > host_server.pem
    ```

=== "Hardware mode (Azure DCsv3 VMs)"

    ```bash
    docker run --rm mithrilsecuritysas/blindai-server-dcsv3:latest /bin/cat /root/tls/host_server.pem > host_server.pem
    ```

### Connect to the hardware mode server

You can start from the python code of [the quick-start section](../index.md). You should then replace the instances of :
```py
client = BlindAiConnection(addr="localhost", simulation=True)
```

by

```py
client = BlindaiConnection(addr="localhost", manifest="/path/to/manifest.toml")
```

Your client will use your TLS certificate and will only be able to connect to an enclave generated with the exact same manifest.toml.

!!! note
    If you want to deploy for production you should check out [the privacy section](main-concepts/privacy.md). You will learn how to check the authenticity of the manifest and how to inject your own TLS certificate.