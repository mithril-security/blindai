from facenet_pytorch import MTCNN, InceptionResnetV1
import torch
import numpy as np
from PIL import Image

workers = 1

mtcnn = MTCNN(
    image_size=160, margin=0, min_face_size=20,
    thresholds=[0.6, 0.7, 0.7], factor=0.709, post_process=True,
)

def collate_fn(x):
    return x[0]

model = InceptionResnetV1(pretrained='vggface2').eval()

img = Image.open("./tmp/woman_0.jpg")

img_aligned = mtcnn(torch.tensor(np.asarray(img)))
inputs = img_aligned.unsqueeze(0)

torch.onnx.export(
    model,
    inputs,
    "./tmp/facenet_nooptim.onnx",
    export_params=True,
)

npz_inputs = {}

npz_inputs["input.1"] = inputs

np.savez("./facenet.npz", **npz_inputs)
print(inputs.shape, inputs.type())