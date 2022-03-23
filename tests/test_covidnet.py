from blindai.client import BlindAiClient, ModelDatumType
import unittest
from server import (
    launch_server,
    close_server,
    policy_file,
    certificate_file,
    has_hardware_support,
)
import os
import onnxruntime
import cv2
import numpy as np


class TestCovidNetBase:
    def setUp(self):
        if not self.simulation and not has_hardware_support:
            self.skipTest("no hardware support")

    def test_base(self):
        client = BlindAiClient()

        client.connect_server(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        )

        model = os.path.join(os.path.dirname(__file__), "assets/COVID-Net-CXR-2.onnx")

        client.upload_model(
            model=model,
            shape=(1, 480, 480, 3),
            dtype=ModelDatumType.F32,
        )

        response = client.run_model(
            flattened_img,
        )

        ort_session = onnxruntime.InferenceSession(model)
        ort_inputs = {ort_session.get_inputs()[0].name: img}

        ort_outs = ort_session.run(None, ort_inputs)

        diff = abs(sum(np.array([response.output]) - ort_outs))[0][0]
        self.assertLess(diff, 0.001)  # difference is <0.1%


class TestCovidNetSW(TestCovidNetBase, unittest.TestCase):
    simulation = True


class TestCovidNetHW(TestCovidNetBase, unittest.TestCase):
    simulation = False


img, flattened_img = None, None


def setUpModule():
    global flattened_img, img
    launch_server()

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
        os.path.join(os.path.dirname(__file__), "assets/ex-covid.jpeg"), size=480
    )
    img = img.astype("float32") / 255.0
    img = img[np.newaxis, :, :, :]

    flattened_img = img.flatten().tolist()


def tearDownModule():
    close_server()


if __name__ == "__main__":
    unittest.main()
