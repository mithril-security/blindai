# Deploy on Hardware

## Hardware requirements

You will need to have an Intel SGX ready device, with `SGX+FLC` (Flexible Launch Control) support. Read [this Intel documentation page](https://www.intel.com/content/www/us/en/support/articles/000057420/software/intel-security-products.html) to see if your Intel processor supports it.

Please make sure to have the `SGX+FLC` drivers (preferably with the version **1.41**) installed on your system before running the docker image. [Check this link to get more information about the drivers](https://github.com/intel/SGXDataCenterAttestationPrimitives/tree/master/driver/linux).

If you are using an Azure DCXS VM, the drivers are already installed.


If the drivers are named "enclave" and "provision" (or sgx\_enclave and sgx\_provision), you are good to go!



If the drivers are named "isgx", that means your system is not supported. This driver is for the first generation of SGX, which lacks security features we rely on.


Otherwise, here is a way to install the drivers quickly:

```
wget https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/sgx_linux_x64_driver_1.41.bin
chmod +x sgx_linux_x64_driver_1.41.bin
./sgx_linux_x64_driver_1.41.bin
```

The binary file contains the drivers signed by Intel, and will proceed to the installation transparently.

## Running the server

Please make sure you have [Docker ](https://docs.docker.com/get-docker/)installed on your machine.



A [Quote Provisioning Certificate Caching Service (PCCS)](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/pccs/README.md) is built-in inside the Docker Image in order to generate the DCAP attestation from the enclave. You need to provide an API Key in order for the PCCS server to function. [You can get one from Intel here.](https://api.portal.trustedservices.intel.com/provisioning-certification)

```bash
docker run -it \
    -p 50051:50051 \
    -p 50052:50052 \
    --device /dev/sgx/enclave \
    --device /dev/sgx/provision \
    mithrilsecuritysas/blindai-server:latest /root/start.sh PCCS_API_KEY
```

The `PCCS_API_KEY` needs to be replaced with your PCCS API Key.



There is no need for a PCCS API Key, just run the following:

```bash
docker run -it \
    -v $(pwd)/bin/tls:/root/tls \
    -p 50051:50051 \
    -p 50052:50052 \
    --device /dev/sgx/enclave \
    --device /dev/sgx/provision \
    mithrilsecuritysas/blindai-server-dcsv3:latest
```



### Get the policy and TLS Certificate

In hardware mode, we are required to pass two files that were generated previously by the server to the client: `policy.toml` and `host_server.pem`. Read more about what these files are used for here: [certificate-and-policy.md](../advanced/certificate-and-policy.md "mention")

You may pull the policy for the latest prebuilt server binary with this command:

```
docker run --rm mithrilsecuritysas/blindai-server:latest cat /root/policy.toml > policy.toml
```

If you wish to use the default built-in TLS certificate, you need to pull the certificate first as well.

```
docker run --rm mithrilsecuritysas/blindai-server:latest cat /root/tls/host_server.pem > host_server.pem
```


**Please remember that this certificate is not secure, it is strongly recommended to [generate your own certificate](../advanced/certificate-and-policy.md#inject-your-own-tls-certificate-to-blindai).**

