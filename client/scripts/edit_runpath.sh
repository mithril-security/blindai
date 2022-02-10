#!/bin/sh

find . -name "pybind*.so" | xargs -l1 -t patchelf --set-rpath "\$ORIGIN/blindai/lib"
