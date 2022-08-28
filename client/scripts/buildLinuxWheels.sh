#!/bin/sh

export CIBW_MANYLINUX_X86_64_IMAGE=manylinux2014
export CIBW_SKIP="cp311-* pp* *i686 *musllinux*"

cibuildwheel --platform linux --output-dir dist

