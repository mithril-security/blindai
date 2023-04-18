# Setting up your dev environment

## Development environment
If you want to make changes to the code, it is recommended you use our pre-configured development container, which has all the dependencies you need to run and use BlindAI.

Firstly, in order to be able to run tests with the server, you'll need to work on an SGX2-ready machine. To find out more about how to do this, check out our [installation page](../../tutorials/core/installation.md).

Now you're ready to set up your development environment!

Click the option on which you will be working on:

-  <a href="#Standard-dev-environment">![](../../../assets/vscode.png){ width=18 } - VSCode and Docker on your local machine</a>**
-  <a href="#Bare-bones-dev-environment"> 🖥️  - Directly on your local machine</a>**
-  <a href="#Azure-dev-environment">![](../../../assets/azure.png){ width=18 } - Azure DCsv3 VM</a>**
____________________________________

### Standard setup [ 🐳 ![](../../../assets/vscode.png){ width=22 } ] <a name="Standard-dev-environment"></a>
____________________________________

To set up our pre-configured development container, you can follow these instructions:

1. Clone blindai github repo and submodules.
```bash
git clone https://github.com/mithril-security/blindai --recursive
cd blindai
```

2. Make sure you have Docker installed on your machine.

    If it is not the case, you can follow [the official Docker installation guide](https://docs.docker.com/engine/install).

    You also need to make sure you haver the correct permissions to run docker commands without `sudo`.
    To check this, try running `docker run hello-world`. If this works, you can skip straight to the next step. If it doesn't, you need to add yourself to docker group:
    ```bash
    sudo usermod -aG docker $USER && newgrp docker
    ```

3. Open the `blindai` folder in VSCode.

4. Make sure you have the `remote container VSCode extension` installed. If you don't, install this from the VSCode extensions marketplace.

5. Open the green menu at the bottom-left by clicking on &ensp;![](../../../assets/vscode-menu.svg){ width=20 align=top }&ensp; in Visual Studio Code.

    ![menu](../../../assets/Screenshot-vscode.png)

6. Select `Dev Containers: Reopen in Container`.

    ![reopen in container option](../../../assets/container.png)

    This may take some time since there are several dependencies that must be installed.

>If you have any issues with this process, make sure you have the BlindAI folder open in your VSCode window.

____________________________________

### Bare-bones setup [ 🖥️  ] <a name="Bare-bones-dev-environment"></a>
____________________________________

To setup the dev environment by hand, you will need to install the following:

=== "General"

    **Intel SGX**

    * Intel SGX DCAP **1.41** Driver
    * Intel SGX SDK **v2.15.1**
    * Intel SGX PSW (version **2.15.101.1** for the PSW librairies and **1.12.101.1** for the PSW-DCAP librairies)

    **Rust**

    * Nightly toolchain

        ??? abstract "Installation instructions"

            === "Installing rustup and nightly toolchain"

                ```bash
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain=nightly -y
                ```
            === "Installing only nightly toolchain"

                ```bash
                rustup default nightly
                ```

    * Fortanix Rust EDP and its dependencies

        ??? abstract "Installation instructions"
            **Fortanix EDP target**
            ```bash
            rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly
            ```
            **Intel SGX driver**
            ```bash
            echo "deb https://download.fortanix.com/linux/apt xenial main" | sudo tee -a /etc/apt/sources.list.d/fortanix.list >/dev/null
            curl -sSL "https://download.fortanix.com/linux/apt/fortanix.gpg" | sudo -E apt-key add -

            sudo apt-get update
            sudo apt-get install intel-sgx-dkms
            ```
            **Fortranix EDP utilities**

            Dependencies
            ```bash
            sudo apt-get install pkg-config libssl-dev protobuf-compiler
            ```
            Utilities
            ```bash
            cargo install fortanix-sgx-tools sgxs-tools
            ```

=== "ubuntu 18.04"

    **Intel SGX**

    * Intel SGX DCAP [**1.41** Driver](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/sgx_linux_x64_driver_1.41.bin)
    * Intel SGX SDK [**v2.15.1**](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/sgx_linux_x64_sdk_2.15.101.1.bin)
    * Intel SGX PSW (version **2.15.101.1** for the PSW librairies and **1.12.101.1** for the PSW-DCAP librairies)

        ??? abstract "Installation instructions"

            **Repository setup**
            ```bash
            echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main' | sudo tee -a /etc/apt/sources.list.d/intel-sgx.list
            wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -

            sudo apt-get update
            ```
            **Installation command**
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
            sudo apt-get install "${psw_dep[@]}"
            ```

    **Rust**

    * Nightly toolchain

        ??? abstract "Installation instructions"

            === "Installing rustup and nightly toolchain"

                ```bash
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain=nightly -y
                ```
            === "Installing only nightly toolchain"

                ```bash
                rustup default nightly
                ```

    * Fortanix Rust EDP and its dependencies

        ??? abstract "Installation instructions"
            **Fortanix EDP target**
            ```bash
            rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly
            ```
            **Intel SGX driver**
            ```bash
            echo "deb https://download.fortanix.com/linux/apt xenial main" | sudo tee -a /etc/apt/sources.list.d/fortanix.list >/dev/null
            curl -sSL "https://download.fortanix.com/linux/apt/fortanix.gpg" | sudo -E apt-key add -

            sudo apt-get update
            sudo apt-get install intel-sgx-dkms
            ```
            **Fortranix EDP utilities**

            Dependencies
            ```bash
            sudo apt-get install pkg-config libssl-dev protobuf-compiler
            ```
            Utilities
            ```bash
            cargo install fortanix-sgx-tools sgxs-tools
            ```

=== "ubuntu 20.04"

    **Intel SGX**

    * Intel SGX DCAP [**1.41** Driver](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu20.04-server/sgx_linux_x64_driver_1.41.bin)
    * Intel SGX SDK [**v2.15.1**](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu20.04-server/sgx_linux_x64_sdk_2.15.101.1.bin)
    * Intel SGX PSW (version **2.15.101.1** for the PSW librairies and **1.12.101.1** for the PSW-DCAP librairies)

        ??? abstract "Installation instructions"

            **Repository setup**
            ```bash
            echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list
            wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -

            sudo apt-get update
            ```
            **Installation command**
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
            sudo apt-get install "${psw_dep[@]}"
            ```

    **Rust**

    * Nightly toolchain

        ??? abstract "Installation instructions"

            === "Installing rustup and nightly toolchain"

                ```bash
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain=nightly -y
                ```
            === "Installing only nightly toolchain"

                ```bash
                rustup default nightly
                ```

    * Fortanix Rust EDP and its dependencies

        ??? abstract "Installation instructions"
            **Fortanix EDP target**
            ```bash
            rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly
            ```
            **Intel SGX driver**
            ```bash
            echo "deb https://download.fortanix.com/linux/apt xenial main" | sudo tee -a /etc/apt/sources.list.d/fortanix.list >/dev/null
            curl -sSL "https://download.fortanix.com/linux/apt/fortanix.gpg" | sudo -E apt-key add -

            sudo apt-get update
            sudo apt-get install intel-sgx-dkms
            ```
            **Fortranix EDP utilities**

            Dependencies
            ```bash
            sudo apt-get install pkg-config libssl-dev protobuf-compiler
            ```
            Utilities
            ```bash
            cargo install fortanix-sgx-tools sgxs-tools
            ```

=== "ubuntu 22.04"

    **Intel SGX**

    * Intel SGX DCAP [**1.41** Driver](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu20.04-server/sgx_linux_x64_driver_1.41.bin)
    * Intel SGX SDK [**v2.15.1**](https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu20.04-server/sgx_linux_x64_sdk_2.15.101.1.bin)
    * Intel SGX PSW (version **2.15.101.1** for the PSW librairies and **1.12.101.1** for the PSW-DCAP librairies)

        ??? abstract "Installation instructions"

            **Dependencies**

            * [libprotobuf17](https://packages.ubuntu.com/focal/amd64/libprotobuf17/download)
            * [libssl1.1](https://packages.ubuntu.com/focal/amd64/libssl1.1/download)

            ```bash
            echo 'deb http://security.ubuntu.com/ubuntu focal-security main' | sudo tee -a /etc/apt/sources.list.d/ubuntu-security.list

            sudo apt-get update
            sudo apt-get install libprotobuf17 libssl1.1
            ```

            **Repository setup**
            ```bash
            echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | sudo tee -a /etc/apt/sources.list.d/intel-sgx.list
            wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -

            sudo apt-get update
            ```
            **Installation command**
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
            sudo apt-get install "${psw_dep[@]}"
            ```

    **Rust**

    * Nightly toolchain

        ??? abstract "Installation instructions"

            === "Installing rustup and nightly toolchain"

                ```bash
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain=nightly -y
                ```
            === "Installing only nightly toolchain"

                ```bash
                rustup default nightly
                ```

    * Fortanix Rust EDP and its dependencies

        ??? abstract "Installation instructions"
            **Fortanix EDP target**
            ```bash
            rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly
            ```
            **Intel SGX driver**
            ```bash
            echo "deb https://download.fortanix.com/linux/apt xenial main" | sudo tee -a /etc/apt/sources.list.d/fortanix.list >/dev/null
            curl -sSL "https://download.fortanix.com/linux/apt/fortanix.gpg" | sudo -E apt-key add -

            sudo apt-get update
            sudo apt-get install intel-sgx-dkms
            ```
            **Fortranix EDP utilities**

            Dependencies
            ```bash
            sudo apt-get install pkg-config libssl-dev protobuf-compiler
            ```
            Utilities
            ```bash
            cargo install fortanix-sgx-tools sgxs-tools
            ```

Then you can install the Intel SGX related dependencies with the following code block, using the [sgx-install.sh](https://github.com/mithril-security/blindai/tree/main/devenvironment/sgx-install.sh) install script.
=== "From BlindAI's directory"

    ```bash
    ./devenvironment/sgx-install.sh
    ```

=== "With curl"

    ```bash
    curl -sSL https://raw.githubusercontent.com/mithril-security/blindai/main/devenvironment/sgx-install.sh | bash
    ```


Or you can find:

* The [installation guides](https://download.01.org/intel-sgx/sgx-linux/2.15.1/docs/) for Intel SGX software on the 01.org website for more specific needs.

* The [installation guides](https://edp.fortanix.com/docs/installation/guide/) for fortanix EDP on their official website.

!!! info "Running without SGX support"
    If you are running on a machine without SGX support, you will need the simulation versions of the Intel PSW and SDK.

____________________________________

### Azure cloud setup [ ☁️  ![](../../../assets/azure.png){ width=22 } ] <a name="Azure-dev-environment"></a>
____________________________________

To set up our pre-configured development container for your Azure VM, you can follow these instructions:

1. Clone blindai github repo and submodules.
```bash
git clone https://github.com/mithril-security/blindai --recursive
cd blindai
```

2. Make sure you have Docker installed on your machine.

    If it is not the case, you can follow [the official Docker installation guide](https://docs.docker.com/engine/install).

    You also need to make sure you haver the correct permissions to run docker commands without `sudo`.
    To check this, try running `docker run hello-world`. If this works, you can skip straight to the next step. If it doesn't, you need to add yourself to docker group:
    ```bash
    sudo usermod -aG docker $USER && newgrp docker
    ```

3. Open the `blindai` folder in VSCode.

4. Make sure you have the `remote container VSCode extension` installed. If you don't, install this from the VSCode extensions marketplace.

5. Make sure you are connected to your VM as host. To do this, open the green menu at the bottom-left by clicking on &ensp;![](../../../assets/vscode-menu.svg){ width=20 align=top }&ensp; in Visual Studio Code.

    ![menu](../../../assets/Screenshot-vscode.png)

6. Select `Connect to host` and select your host.

    ![connect to host](../../../assets/host.png)

7. Next, open the green menu at the bottom-left by clicking on &ensp;![](../../../assets/vscode-menu.svg){ width=20 align=top }&ensp; in Visual Studio Code again and choose:
`Dev Containers: Reopen in Container`.

    ![reopen in container](../../../assets/container.png)


    This may take some time since there are several dependencies that must be installed.

>If you have any issues with this process, make sure you have the BlindAI folder open in your VSCode window.

## Building client from source

To compile the client code locally:
```bash
cd client
poetry install
```

## Building server from source

You can build and run the server from source using the `justfile`:
```bash
just run
```

>Make sure you are in the root of the blindai directory to make use of the justfile commands.

>Note that by default the port opened in 9923 is running on http only. For production, we strongly recommend setting up a ***reverse-proxy*** that will manage and encrypt the traffic from the client to the blindAI server. Many free reverse-proxy implementations exist, such as **caddy**, **Nginx** and **Apache**:

- [https://caddyserver.com/docs/quick-starts/reverse-proxy](https://caddyserver.com/docs/quick-starts/reverse-proxy)
- [Nginx reverse proxy set-up guide](https://docs.nginx.com/nginx/admin-guide/web-server/reverse-proxy/)
- [Apache reverse proxy set-up guide](https://httpd.apache.org/docs/2.4/howto/reverse_proxy.html)

If you do not set up a reverse proxy, users will need to set the `hazmat_http_on_untrusted_port` option to `True` when using blindai's `connect()` function. Again, this is **not recommended** for production.

>Note that if you make any changes to the server code, a new `manifest.toml` file will be created when you build the server. In order to be able to connect with the server instance using the BlindAI Core `connect()` method, you will need to supply a path to a copy of this file in the `hazmat_manifest_path` option. The manifest.toml files are used during the verification step of the connection progress to check that the server is not running any unexpected and potentially malicious code. You can learn more about this verification process [here](../../getting-started/confidential_computing.md).
