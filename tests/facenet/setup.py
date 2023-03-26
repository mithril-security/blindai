from facenet_pytorch import MTCNN, InceptionResnetV1
import torch
import numpy as np
from PIL import Image
import os
import runcmd
from onnxsim import simplify
import onnx

path = os.path.dirname(os.path.realpath(__file__))

runcmd.run(["wget", "-P", path + "/tmp", "https://github.com/mithril-security/blindai/raw/blindai-legacy/examples/facenet/woman_0.jpg"])

mtcnn = MTCNN(
    image_size=160, margin=0, min_face_size=20,
    thresholds=[0.6, 0.7, 0.7], factor=0.709, post_process=True,
)

def collate_fn(x):
    return x[0]

model = InceptionResnetV1(pretrained='vggface2').eval()

img = Image.open(path + "/tmp/woman_0.jpg")

img_aligned = mtcnn(torch.tensor(np.asarray(img)))
inputs = img_aligned.unsqueeze(0)

torch.onnx.export(
    model,
    inputs,
    path + "/facenet.onnx",
    export_params=True,
)

model = onnx.load(path + "/facenet.onnx")
model_simp, check = simplify(model)
assert check, "Simplified ONNX model could not be validated"
onnx.save(model_simp, path + "/facenet.onnx")

npz_inputs = {}

npz_inputs["input.1"] = inputs

np.savez(path + "/facenet.npz", **npz_inputs)
print(inputs.shape, inputs.type())