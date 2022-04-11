#!/bin/sh
# Remove patchelf from requirements.txt as it cannot be installed on windows
pip install patchelf
find . -name "_quote_verification*.so" | xargs -l1 -t patchelf --set-rpath "\$ORIGIN/blindai/lib"
