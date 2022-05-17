#!/bin/sh

python -m grpc_tools.protoc -I./proto --python_out=blindai --grpc_python_out=blindai untrusted.proto

python -m grpc_tools.protoc -I./proto --python_out=blindai --grpc_python_out=blindai securedexchange.proto

export CIBW_ARCHS_MACOS=x86_64

python -m pip install cibuildwheel
cibuildwheel --platform macos --output-dir dist
