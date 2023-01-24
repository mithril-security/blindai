# Cloud deployment

You will find here some tutorials to deploy **BlindAI** on Azure DCsv3.

### Create the VM

You can easily deploy BlindAI on [Azure DCsv3 VMs](https://docs.microsoft.com/en-us/azure/virtual-machines/dcv3-series). BlindAI works out of the box, all you need to do is to follow those steps to deploy a VM :&#x20;

First thing first, you need to create an account on Azure. If you want to try the service for free, it is strongly advised to subscribe [to the free trial.](https://azure.microsoft.com/en-us/free/) Click on the link to have more information.

Once you created your account and activated the free credits of $200, you can start searching for `Azure Confidential Computing` and then click on "Create".

![Confidential Computing VM](../assets/2022-02-24_11_09_07.png)

![Start the creation process.](../assets/2022-02-24_11_09_26.png)

After this, you will start to see a configuration screen. Please select either **Ubuntu 18.04 or 20.04. For security reasons, it is strongly advised to use a SSH public key in order to use the VM.**

![Basic configuration](../assets/2022-02-24_11_57_19.png)

On the next page, you will choose the VM you want to use. We strongly advise you to pick the **DC2s v3 VM** to test BlindAI. Before going to the next page, please remember to **allow the connection from SSH**.

![VM settings](../assets/2022-02-24_11_12_12.png)

![Choose a VM](../assets/2022-02-24_11_10_26.png)

After this screen, please validate and create the VM.

![Validate and create the VM.](../assets/2022-03-02_16_41_19.png)

After a few minutes, the VM will be successfully deployed. Before connecting to the VM, **it is strongly advised to set up a DNS name, in order to simplify the connection as much as possible.**

![Setting up DNS name - 1](../assets/2022-03-02_16_38_31.png)

![Setting up DNS name - 2](../assets/2022-02-24_12_07_22.png)

Once you are done with this, we have to **open the ports used by BlindAI.** You need to open the ports **9923 and 9924.**

![](../assets/image.png)

![](../assets/image_1.png)


### Using the VM

You can now start the VM. In order to have a good experience with SSH, we recommend you download [**Visual Studio Code**](https://code.visualstudio.com/) and get the extension [**Remote - SSH**](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-ssh). 

Setting up a SSH connection is fairly easy in Visual Studio Code. All you need to do is add a SSH Host (you can find this option in **the Command Palette**):&#x20;

![](../assets/2022-02-24_12_15_41.png)

![The DNS name shows its usefulness here as you won't need to update the host after the first configuration.](../assets/2022-02-24_12_15_35.png)

After that, you need to select "Connect to Host" in **the Command Palette** and select your DNS name.

![](../assets/2022-02-24_12_53_38.png)

Once you are online, we need to make sure that the SGX drivers are installed. You can do it very easily like this:&#x20;

![](../assets/2022-02-24_12_17_25.png)

If you can see "`SGX`" in the list, in the same way it appears on the screenshot above, **you're good to go**! If `SGX` is missing, you can simply install the drivers yourself with those commands:&#x20;

```
wget https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/sgx_linux_x64_driver_1.41.bin
chmod +x sgx_linux_x64_driver_1.41.bin
./sgx_linux_x64_driver_1.41.bin
```

**If you are getting an error while installing the drivers, it might mean that you picked the wrong VM. Please restart the process in that case.**

If you are good to go, you just need to install Docker on the VM. [Please follow these instructions to get started quickly. ](https://docs.docker.com/engine/install/ubuntu/#install-using-the-repository)

### Security requirements


**The port opened in 9923 is considered as unsecure. It is also running on http only and must not be considered secure on a post-production environment.** 

!!! Warning
    The unsecure port must be linked to a predefined ***reverse-proxy*** that manages the ingress and egress traffic and also that will be responsible of encrypting the traffic from the client to the blindAI server. Multiple reverse-proxy implementations exist, among them **Nginx** and **Apache**. 

Once Docker is installed, [set up your dev-environment](advanced/setting-up-your-dev-environment.md).
