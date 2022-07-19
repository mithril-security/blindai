# Run the BlindAI server

This page explains how to work with the simulation mode. This simulates Intel SGX in software, and enables you to run this on any hardware you want.

To deploy on real hardware in non-simulation mode, take a look at [deploy-on-hardware.md](../deploy-on-hardware.md "mention") and skip the first step.

To quickly setup an SGX-enabled virtual machine on Azure, take a look at [cloud-deployment](../cloud-deployment/ "mention").

Launch the server using the simulation docker image:

```bash
docker run -it \
    -p 50051:50051 \
    -p 50052:50052 \
    mithrilsecuritysas/blindai-server-sim:latest
```

Please make sure the ports 50051 and 50052 are available :)

**Please keep in mind that this image is not secure, since it simulates Intel SGX in software. It is lighter than hardware mode, and should not be used in production.**
