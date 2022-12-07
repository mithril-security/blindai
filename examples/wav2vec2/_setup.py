from transformers import Wav2Vec2Processor, Wav2Vec2ForCTC
import librosa
import numpy as np
import torch
import os
path = os.path.dirname(os.path.realpath(__file__))

# load model and processor
processor = Wav2Vec2Processor.from_pretrained("facebook/wav2vec2-base-960h")
model = Wav2Vec2ForCTC.from_pretrained("facebook/wav2vec2-base-960h")


audio, rate = librosa.load(path + "/tmp/hello_world.wav", sr = 16000)

# Tokenize sampled audio to input into model
inputs = processor(audio, sampling_rate=rate, return_tensors="pt", padding="longest").input_values

torch.onnx.export(
    model,
    inputs,
    path + '/tmp/wav2vec2_nooptim.onnx',
    export_params=True,
    opset_version=11)

npz_inputs = {}

npz_inputs["onnx::Unsqueeze_0"] =  inputs

np.savez(path + "/wav2vec2.npz", **npz_inputs)
print(inputs.shape, inputs.type())
