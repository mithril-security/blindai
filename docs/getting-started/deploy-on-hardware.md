# Deploy on Hardware

## Hardware requirements

=== "Hardware mode"

    !!! info
        If you are using Azure DCsV2 vms, you can ignore all of this. The drivers and the PCCS server are built-in the vms.

    You will need to have an Intel SGX ready device, with `SGX+FLC` (Flexible Launch Control) support. Read [this Intel documentation page](https://www.intel.com/content/www/us/en/support/articles/000057420/software/intel-security-products.html) to see if your Intel processor supports it.

    Please make sure to have the `SGX+FLC` drivers (preferably with the version **1.41**) installed on your system before running the docker image.

    !!! success
        If you can find the drivers named "enclave" and "provision" (or sgx\_enclave and sgx\_provision) in /dev/, you are good to go!

    !!! failure
        If on the other hand, you can find a driver named "isgx", that means your system is not supported. This driver is for the first generation of SGX, which lacks security features we rely on. You can still boot the server in hardware mode and benefit from the isolation offered by SGX enclaves, but you will need to use the client in simulation mode.

    In case you don't have any drivers installed, you can install the drivers with this:

    ```bash
    wget https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/sgx_linux_x64_driver_1.41.bin
    chmod +x sgx_linux_x64_driver_1.41.bin
    ./sgx_linux_x64_driver_1.41.bin
    ```

    The binary file contains the drivers signed by Intel, and will proceed to the installation transparently.


=== "Hardware mode (Azure DCsv3 VMs)"

    There is no need to do anything, the drivers are already installed.

## Running the server

Please make sure you have [Docker ](https://docs.docker.com/get-docker/)installed on your machine.

=== "Hardware mode"

    A [Quote Provisioning Certificate Caching Service (PCCS)](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/pccs/README.md) is built-in inside the Docker Image in order to generate the DCAP attestation from the enclave. You need to provide an API Key in order for the PCCS server to function. [You can get one from Intel here.](https://api.portal.trustedservices.intel.com/provisioning-certification)

    ```bash
    docker run -it \
        -p 50051:50051 \
        -p 50052:50052 \
        --device /dev/sgx/enclave \
        --device /dev/sgx/provision \
        mithrilsecuritysas/blindai-server:latest /root/start.sh PCCS_API_KEY
    ```

    !!! info
        The `PCCS_API_KEY` needs to be replaced with the PCCS API Key.

=== "Hardware mode (Azure DCsv3 VMs)"

    ```bash
    docker run -it \
        -v $(pwd)/bin/tls:/root/tls \
        -p 50051:50051 \
        -p 50052:50052 \
        --device /dev/sgx/enclave \
        --device /dev/sgx/provision \
        mithrilsecuritysas/blindai-server-dcsv3:latest
    ```


## Get the policy and TLS Certificate



In hardware mode, we are required to pass two files that were generated previously by the server to the client: `policy.toml` and `host_server.pem`. Read more about what these files are used for here: [certificate-and-policy.md](../advanced/certificate-and-policy.md "mention")

You may pull the policy for the latest prebuilt server binary with this command:

=== "Hardware mode"

    ```
    docker run --rm mithrilsecuritysas/blindai-server:latest cat /root/policy.toml > policy.toml
    ```

    If you wish to use the default built-in TLS certificate, you need to pull the certificate first as well.

    ```
    docker run --rm mithrilsecuritysas/blindai-server:latest cat /root/tls/host_server.pem > host_server.pem
    ```

=== "Hardware mode (Azure DCsv3 VMs)"

    ```
    docker run --rm mithrilsecuritysas/blindai-server-dcsv3:latest cat /root/policy.toml > policy.toml
    ```

    If you wish to use the default built-in TLS certificate, you need to pull the certificate first as well.

    ```
    docker run --rm mithrilsecuritysas/blindai-server-dcsv3:latest cat /root/tls/host_server.pem > host_server.pem
    ```


**Please remember that this certificate is not secure, it is strongly recommended to [generate your own certificate](../advanced/certificate-and-policy.md#inject-your-own-tls-certificate-to-blindai).**

