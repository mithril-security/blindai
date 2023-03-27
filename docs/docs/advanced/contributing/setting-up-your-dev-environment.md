# Setting up your dev environment

## Using remote container extension on Visual Studio Code 🐳

You can directly clone the repo and open it in VS Code. Using the remote container extension, Visual Studio Code should prompt you to install it.

Open the green menu at the bottom-left of the Visual Studio Code.
Choose: "Open folder in container".

![](../../../assets/Screenshot-vscode.png)

It will build and run the image present in the [`.devcontainer` folder](https://github.com/mithril-security/blindai/tree/main/.devcontainer) and it will run the dev environment directly on VSCode.

!!! Warning
    there is a different one for Azure in the : `devcontainer-azure/` folder

You can check that everything is correctly set-up by [Running the tests](../../../index.md#testing)

## Without Docker

If you don't want to use docker, you will need to install the following:

=== "General"

    **Intel SGX**

    * Intel SGX DCAP **1.41** Driver
    * Intel SGX SDK **v2.15.1**
    * Intel SGX PSW (version **2.15.101.1** for the PSW librairies and **1.12.101.1** for the PSW-DCAP librairies)

    **Fortranix EDP**

    * Fortanix EDP and its dependencies

=== "ubuntu 18.04"

    **Intel SGX**

    * Intel SGX DCAP [**1.41** Driver](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/sgx_linux_x64_driver_1.41.bin)
    * Intel SGX SDK [**v2.15.1**](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/sgx_linux_x64_sdk_2.15.101.1.bin)
    * Intel SGX PSW (version **2.15.101.1** for the PSW librairies and **1.12.101.1** for the PSW-DCAP librairies)
      ```bash
      echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list

      wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -
      ```
      ```bash
      declare -a psw_dep=(
          # PSW
          sgx-aesm-service=2.15.101.1-bionic1
          libsgx-ae-epid=2.15.101.1-bionic1
          libsgx-ae-le=2.15.101.1-bionic1
          libsgx-ae-pce=2.15.101.1-bionic1
          libsgx-aesm-ecdsa-plugin=2.15.101.1-bionic1
          libsgx-aesm-epid-plugin=2.15.101.1-bionic1
          libsgx-aesm-launch-plugin=2.15.101.1-bionic1
          libsgx-aesm-pce-plugin=2.15.101.1-bionic1
          libsgx-aesm-quote-ex-plugin=2.15.101.1-bionic1
          libsgx-enclave-common=2.15.101.1-bionic1
          libsgx-epid=2.15.101.1-bionic1
          libsgx-launch=2.15.101.1-bionic1
          libsgx-quote-ex=2.15.101.1-bionic1
          libsgx-uae-service=2.15.101.1-bionic1
          libsgx-urts=2.15.101.1-bionic1
          libsgx-ae-pce=2.15.101.1-bionic1
          # PSW DCAP
          libsgx-ae-qe3=1.12.101.1-bionic1
          libsgx-pce-logic=1.12.101.1-bionic1
          libsgx-qe3-logic=1.12.101.1-bionic1
          libsgx-ra-network=1.12.101.1-bionic1
          libsgx-ra-uefi=1.12.101.1-bionic1
          libsgx-dcap-ql=1.12.101.1-bionic1
          libsgx-dcap-quote-verify=1.12.101.1-bionic1
          libsgx-dcap-default-qpl=1.12.101.1-bionic1
      )
      apt-get install "${psw_dep[@]}"
      ```

    **Fortanix EDP**

    * Fortanix EDP and its dependencies

=== "ubuntu 20.04"

    **Intel SGX**

    * Intel SGX DCAP [**1.41** Driver](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu20.04-server/sgx_linux_x64_driver_1.41.bin)
    * Intel SGX SDK [**v2.15.1**](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu20.04-server/sgx_linux_x64_sdk_2.15.101.1.bin)
    * Intel SGX PSW (version **2.15.101.1** for the PSW librairies and **1.12.101.1** for the PSW-DCAP librairies)
      ```bash
      echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list

      wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -
      ```
      ```bash
      declare -a psw_dep=(
          # PSW
          sgx-aesm-service=2.15.101.1-focal1
          libsgx-ae-epid=2.15.101.1-focal1
          libsgx-ae-le=2.15.101.1-focal1
          libsgx-ae-pce=2.15.101.1-focal1
          libsgx-aesm-ecdsa-plugin=2.15.101.1-focal1
          libsgx-aesm-epid-plugin=2.15.101.1-focal1
          libsgx-aesm-launch-plugin=2.15.101.1-focal1
          libsgx-aesm-pce-plugin=2.15.101.1-focal1
          libsgx-aesm-quote-ex-plugin=2.15.101.1-focal1
          libsgx-enclave-common=2.15.101.1-focal1
          libsgx-epid=2.15.101.1-focal1
          libsgx-launch=2.15.101.1-focal1
          libsgx-quote-ex=2.15.101.1-focal1
          libsgx-uae-service=2.15.101.1-focal1
          libsgx-urts=2.15.101.1-focal1
          libsgx-ae-pce=2.15.101.1-focal1
          # PSW DCAP
          libsgx-ae-qe3=1.12.101.1-focal1
          libsgx-pce-logic=1.12.101.1-focal1
          libsgx-qe3-logic=1.12.101.1-focal1
          libsgx-ra-network=1.12.101.1-focal1
          libsgx-ra-uefi=1.12.101.1-focal1
          libsgx-dcap-ql=1.12.101.1-focal1
          libsgx-dcap-quote-verify=1.12.101.1-focal1
          libsgx-dcap-default-qpl=1.12.101.1-focal1
      )
      apt-get install "${psw_dep[@]}"
      ```

    **Fortanix EDP**

    * Fortanix EDP and its dependencies


=== "ubuntu 22.04"

    **General**

    * [libprotobuf17](https://packages.ubuntu.com/focal/amd64/libprotobuf17/download)
	* [libssl1.1](https://packages.ubuntu.com/focal/amd64/libssl1.1/download)

    **Intel SGX**

    * Intel SGX DCAP [**1.41** Driver](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu20.04-server/sgx_linux_x64_driver_1.41.bin)
    * Intel SGX SDK [**v2.15.1**](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu20.04-server/sgx_linux_x64_sdk_2.15.101.1.bin)
    * Intel SGX PSW (version **2.15.101.1** for the PSW librairies and **1.12.101.1** for the PSW-DCAP librairies)
      ```bash
      echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list

      wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -
      ```
      ```bash
      declare -a psw_dep=(
          # PSW
          sgx-aesm-service=2.15.101.1-focal1
          libsgx-ae-epid=2.15.101.1-focal1
          libsgx-ae-le=2.15.101.1-focal1
          libsgx-ae-pce=2.15.101.1-focal1
          libsgx-aesm-ecdsa-plugin=2.15.101.1-focal1
          libsgx-aesm-epid-plugin=2.15.101.1-focal1
          libsgx-aesm-launch-plugin=2.15.101.1-focal1
          libsgx-aesm-pce-plugin=2.15.101.1-focal1
          libsgx-aesm-quote-ex-plugin=2.15.101.1-focal1
          libsgx-enclave-common=2.15.101.1-focal1
          libsgx-epid=2.15.101.1-focal1
          libsgx-launch=2.15.101.1-focal1
          libsgx-quote-ex=2.15.101.1-focal1
          libsgx-uae-service=2.15.101.1-focal1
          libsgx-urts=2.15.101.1-focal1
          libsgx-ae-pce=2.15.101.1-focal1
          # PSW DCAP
          libsgx-ae-qe3=1.12.101.1-focal1
          libsgx-pce-logic=1.12.101.1-focal1
          libsgx-qe3-logic=1.12.101.1-focal1
          libsgx-ra-network=1.12.101.1-focal1
          libsgx-ra-uefi=1.12.101.1-focal1
          libsgx-dcap-ql=1.12.101.1-focal1
          libsgx-dcap-quote-verify=1.12.101.1-focal1
          libsgx-dcap-default-qpl=1.12.101.1-focal1
      )
      apt-get install "${psw_dep[@]}"
      ```

    **Fortanix EDP**

    * Fortanix EDP and its dependencies


You can install the Intel SGX related dependencies with the [sgx-install.sh](https://github.com/mithril-security/blindai/tree/main/devenvironment/sgx-install.sh) install script.
```bash
# from the repository's devenvironment directory
./sgx-install.sh
# or with curl
curl -sSL https://raw.githubusercontent.com/mithril-security/blindai-preview/main/devenvironment/sgx-install.sh | bash
```

Or you can find the [installation guides](https://download.01.org/intel-sgx/sgx-linux/2.15.1/docs/) for Intel SGX software on the 01.org website for more specific needs.

You can find the [installation guides](https://edp.fortanix.com/docs/installation/guide/) for fortanix EDP on their official website.

Note: if you are running on a machine without SGX support, you will need the simulation versions of the Intel PSW and SDK.