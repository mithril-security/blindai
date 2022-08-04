#!/bin/sh

export CIBW_MANYLINUX_X86_64_IMAGE=manylinux2014

cibuildwheel --platform linux --output-dir dist

