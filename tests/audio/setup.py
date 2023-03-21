import whisper
import numpy as np
import os
import onnx
import gdown
import torch

# Dowload the ONNX model using the Drive ID and save it as `whisper-model-20-tokens.onnx`
output = os.path.join(os.path.dirname(__file__), "whisper.onnx")
gdown.download(id="1wqg1F0UkEdm3KB7n1BjfRLHnzKU2-G5S", output=output)

# Load the ONNX model with onnx load
model = onnx.load(output)

# Download the `taunt.wav` file from the UCI repository

output = os.path.join(os.path.dirname(__file__), "taunt.wav")
os.system(f"wget https://www2.cs.uic.edu/~i101/SoundFiles/taunt.wav -O {output}")

audio = whisper.load_audio(output).flatten()

# Trim/pad audio to 30s with `pad_or_trim`
audio = whisper.pad_or_trim(audio).flatten()

# Convert loaded audio to log_mel_spectrogram
input_mel = whisper.log_mel_spectrogram(audio).unsqueeze(0)

# Export input_mel to NPZ file
inputs = dict(mel=input_mel.numpy())

output = os.path.join(os.path.dirname(__file__), "whisper.npz")
np.savez(output, **inputs)
