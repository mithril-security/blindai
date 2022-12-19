cd ../client
for d in ../tests/*/ ; do
	onnx_files=($d*.onnx)
	npz_files=($d*.npz)
	poetry run python ../tests/assert_correctness.py "${onnx_files[0]}" "${npz_files[0]}"
done
