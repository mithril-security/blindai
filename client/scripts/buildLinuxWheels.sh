#!/bin/sh

python -m grpc_tools.protoc -I./proto --python_out=blindai --grpc_python_out=blindai untrusted.proto

python -m grpc_tools.protoc -I./proto --python_out=blindai --grpc_python_out=blindai securedexchange.proto

export CIBW_MANYLINUX_X86_64_IMAGE=manylinux2014

cibuildwheel --platform linux --output-dir dist

