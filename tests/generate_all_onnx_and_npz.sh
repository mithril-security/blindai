#!/bin/bash

set -e

# Generates onnx and inputs files for end to end tests

cd ../client
find ../tests -name "setup.py" -print0 | xargs -0 -n1 poetry run python
