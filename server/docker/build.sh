#!/bin/sh

# Run this file to build the software & hardware images
# note: make sure to load the git submodules using `git submodule init && git submodule update` beforehand

docker build --target software -t mithrilsecuritysas/blindai-server-sim:latest . -f ./docker/build.dockerfile
docker build --target hardware -t mithrilsecuritysas/blindai-server:latest . -f ./docker/build.dockerfile

# Run `docker run mithrilsecuritysas/blindai-server /usr/bin/sha256sum /root/bin/enclave.signed.so`
#  to check the integrity of the enclave file
