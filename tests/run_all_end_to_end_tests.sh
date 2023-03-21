#!/bin/bash

set -e

# Run generate_all_onnx_and_npz.sh before running this script
# Use it to run all the correctness tests
# You must have the server up

cd ../client
for d in ../tests/*/ ; do
	if [[ "$d" == *"mobilenet"* ]]; then
		continue
	fi
	
	onnx_files=($d*.onnx)
	npz_files=($d*.npz)
	poetry run python ../tests/assert_correctness.py "${onnx_files[0]}" "${npz_files[0]}"
done

# Run API test for whisper
poetry run python ../tests/audio/api_test.py
