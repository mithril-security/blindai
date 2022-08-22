import os
import blindai.client
import cv2
import numpy as np


def crop_top(img, percent=0.15):
    offset = int(img.shape[0] * percent)
    return img[offset:]


def central_crop(img):
    size = min(img.shape[0], img.shape[1])
    offset_h = int((img.shape[0] - size) / 2)
    offset_w = int((img.shape[1] - size) / 2)
    return img[offset_h : offset_h + size, offset_w : offset_w + size]


def process_image_file(filepath, size, top_percent=0.08, crop=True):
    img = cv2.imread(filepath)
    img = crop_top(img, percent=top_percent)
    if crop:
        img = central_crop(img)
    img = cv2.resize(img, (size, size))
    return img


img = process_image_file(
    os.path.join(os.path.dirname(__file__), "../../tests/assets/ex-covid.jpeg"),
    size=480,
)
img = img.astype("float32") / 255.0
img = img[np.newaxis, :, :, :]

model_path = os.path.join(os.path.dirname(__file__), "../../tests/assets/COVID-Net-CXR-2.onnx")

def get_input():
    return img

def get_model():
    with open(model_path, 'rb') as f:
        return f.read()
