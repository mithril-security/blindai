

|model_name|onnxruntime(ms)|Blindai(ms)|cpu|
|-|-|-|-|
|bert|26.656|36.822|Intel(R) Xeon(R) Gold 6334 CPU|
|wav2vec2|184.482|767.047|Intel(R) Xeon(R) Gold 6334 CPU|
|facenet|20.024|47.634|Intel(R) Xeon(R) Gold 6334 CPU|
|yolov5s|115.610|393.935|Intel(R) Xeon(R) Gold 6334 CPU|
|gpt2_text_gen|381.514|561.434|Intel(R) Xeon(R) Gold 6334 CPU|

These results have been computed using our [inference benchmark repository](https://github.com/mithril-security/inference_backends_benchmarks).

We are comparing the inference time inside an SGX enclave of our inference backend, [tract](https://github.com/sonos/tract.git), against [onnnxruntime](https://onnxruntime.ai/) (without an enclave), known for its high performance.
