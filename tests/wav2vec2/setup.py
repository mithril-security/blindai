from transformers import Wav2Vec2Processor, Wav2Vec2ForCTC
import librosa
import numpy as np
import torch
import os
import runcmd
from onnxsim import simplify
import onnx

path = os.path.dirname(os.path.realpath(__file__))

runcmd.run(["wget", "-P", path + "/tmp", "https://github.com/mithril-security/blindai/raw/blindai-legacy/examples/wav2vec2/hello_world.wav"])

# load model and processor
processor = Wav2Vec2Processor.from_pretrained("facebook/wav2vec2-base-960h")
model = Wav2Vec2ForCTC.from_pretrained("facebook/wav2vec2-base-960h")


audio, rate = librosa.load(path + "/tmp/hello_world.wav", sr = 16000)

# Tokenize sampled audio to input into model
inputs = processor(audio, sampling_rate=rate, return_tensors="pt", padding="longest").input_values

torch.onnx.export(
    model,
    inputs,
    path + '/wav2vec2.onnx',
    export_params=True,
    opset_version=11)

model = onnx.load(path + '/wav2vec2.onnx')
model_simp, check = simplify(model)
assert check, "Simplified ONNX model could not be validated"
onnx.save(model_simp, path + '/wav2vec2.onnx')

npz_inputs = {}

npz_inputs["onnx::Unsqueeze_0"] =  inputs

np.savez(path + "/wav2vec2.npz", **npz_inputs)
print(inputs.shape, inputs.type())
