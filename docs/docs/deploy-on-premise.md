# Deploy on premise

<!-- 
    The docker images used here are prebuilt ones from our dockerhub, you can take a look at the [build the server from source section]('build-from-sources/server.md')
-->

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

<!-- ### Running the server

=== "Hardware mode"

    After running the PCCS, you can directly run the BlindAi server by using : 
    ```bash
    just release 

    #or 
    just run 
    ```

    !!! info
        The `PCCS_API_KEY` needs to be replaced with the PCCS API Key.

=== "Hardware mode (Azure DCsv3 VMs)"

    To run the server on azure, and after installing all the dependencies needed :
    ```bash
    BLINDAI_AZURE_DCS3_PATCH=1 just release 
    # or 
    BLINDAI_AZURE_DCS3_PATCH=1 just run
    ```

### manifest Generation

The manifest is automatically extracted via the `just run` or `just release` command depending on what mode you're in.

This manifest.toml file is generated at the root of the repo and is based on the templates present on the repo. 

### Connect to the hardware mode server

You can start from the python code of [the quick-start section](../index.md). and copy the manifest.toml containing the mrenclave to `/client/`folder (an argument will be added in the next releases to take into account the manifest file directly into the connect function).
```py
client = connect(addr="localhost")
```


Your client will only be able to connect to an enclave generated with the exact same manifest.toml.

!!! note
    If you want to deploy for production you should check out [the privacy section](main-concepts/privacy.md). You will learn how to check the authenticity of the manifest and how to build a secure communication channel. -->

Next you can [set up your dev environment](advanced/setting-up-your-dev-environment.md)