import torch
import blindai
from blindai import Connection, ModelDatumType
import unittest
from server import (
    close_server,
    launch_server,
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

    @unittest.skipIf(
        os.getenv("BLINDAI_TEST_SKIP_COVIDNET") is not None, "skipped by env var"
    )
    def test_base(self):
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            model = os.path.join(
                os.path.dirname(__file__), "assets/COVID-Net-CXR-2.onnx"
            )

            upload_response = client.upload_model(
                model=model,
                shape=(1, 480, 480, 3),
                dtype=ModelDatumType.F32,
                sign=True,
            )

            if not self.simulation and os.getenv("BLINDAI_DUMPRES") is not None:
                upload_response.save_to_file("./client/tests/exec_upload.proof")

            response = client.predict(
                upload_response.model_id,
                img,
                sign=True,
            )

        if not self.simulation and os.getenv("BLINDAI_DUMPRES") is not None:
            response.save_to_file("./client/tests/exec_run.proof")

        ort_session = onnxruntime.InferenceSession(model)
        ort_inputs = {ort_session.get_inputs()[0].name: img}

        ort_outs = ort_session.run(None, ort_inputs)

        diff = abs(sum(np.array([response.output[0].as_flat()]) - ort_outs))[0][0]
        self.assertLess(diff, 0.001)  # difference is <0.1%

    @unittest.skipIf(
        os.getenv("BLINDAI_TEST_SKIP_COVIDNET") is not None, "skipped by env var"
    )
    def test_multiple(self):
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:

            model = os.path.join(
                os.path.dirname(__file__), "assets/COVID-Net-CXR-2.onnx"
            )

            models = []
            for _ in range(5):
                upload_response = client.upload_model(
                    model=model,
                    shape=(1, 480, 480, 3),
                    dtype=ModelDatumType.F32,
                )
                models.append(upload_response.model_id)

            for i in range(5):
                response = client.predict(
                    models[i],
                    img,
                    dtype=ModelDatumType.F32,
                    shape=(1, 480, 480, 3),
                )

                ort_session = onnxruntime.InferenceSession(model)
                ort_inputs = {ort_session.get_inputs()[0].name: img}

                ort_outs = ort_session.run(None, ort_inputs)

                diff = abs(sum(np.array([response.output[0].as_flat()]) - ort_outs))[0][
                    0
                ]
                self.assertLess(diff, 0.001)  # difference is <0.1%

    @unittest.skipIf(
        os.getenv("BLINDAI_TEST_SKIP_COVIDNET") is not None, "skipped by env var"
    )
    def test_seal(self):
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            model = os.path.join(
                os.path.dirname(__file__), "assets/COVID-Net-CXR-2.onnx"
            )

            upload_response = client.upload_model(
                model=model,
                shape=(1, 480, 480, 3),
                dtype=ModelDatumType.F32,
                sign=True,
                save_model=True,
            )

        # restart server :)
        close_server()
        launch_server()

        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            response = client.predict(
                upload_response.model_id,
                img,
                dtype=ModelDatumType.F32,
                shape=(1, 480, 480, 3),
                sign=True,
            )

        ort_session = onnxruntime.InferenceSession(model)
        ort_inputs = {ort_session.get_inputs()[0].name: img}

        ort_outs = ort_session.run(None, ort_inputs)

        diff = abs(sum(np.array([response.output[0].as_flat()]) - ort_outs))[0][0]
        self.assertLess(diff, 0.001)  # difference is <0.1%

    @unittest.skipIf(
        os.getenv("BLINDAI_TEST_SKIP_COVIDNET") is not None, "skipped by env var"
    )
    def test_feat(self):
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            model = os.path.join(
                os.path.dirname(__file__), "assets/COVID-Net-CXR-2.onnx"
            )

            client.upload_model(
                model=model,
                shape=(1, 480, 480, 3),
                dtype=ModelDatumType.F32,
                sign=True,
                model_id="Salut",
            )

            response = client.predict(
                "Salut",
                img,
                dtype=ModelDatumType.F32,
                shape=(1, 480, 480, 3),
                sign=True,
            )

        ort_session = onnxruntime.InferenceSession(model)
        ort_inputs = {ort_session.get_inputs()[0].name: img}

        ort_outs = ort_session.run(None, ort_inputs)

        diff = abs(sum(np.array([response.output[0].as_flat()]) - ort_outs))[0][0]
        self.assertLess(diff, 0.001)  # difference is <0.1%

    @unittest.skipIf(
        os.getenv("BLINDAI_TEST_SKIP_COVIDNET") is not None, "skipped by env var"
    )
    def test_torch_inputs(self):
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            model = os.path.join(
                os.path.dirname(__file__), "assets/COVID-Net-CXR-2.onnx"
            )

            upload_response = client.upload_model(
                model=model,
                shape=(1, 480, 480, 3),
                dtype=ModelDatumType.F32,
                sign=True,
            )

            inputs = img
            response1 = client.predict(upload_response.model_id, inputs)

            inputs = torch.tensor(img, dtype=torch.float32)
            response2 = client.predict(upload_response.model_id, inputs)

        ort_session = onnxruntime.InferenceSession(model)
        ort_inputs = {ort_session.get_inputs()[0].name: img}

        ort_outs = ort_session.run(None, ort_inputs)

        diff = abs(sum(np.array([response1.output[0].as_flat()]) - ort_outs))[0][0]
        self.assertLess(diff, 0.001)  # difference is <0.1%
        diff = abs(sum(np.array([response2.output[0].as_flat()]) - ort_outs))[0][0]
        self.assertLess(diff, 0.001)  # difference is <0.1%


class TestCovidNetSW(TestCovidNetBase, unittest.TestCase):
    simulation = True


class TestCovidNetHW(TestCovidNetBase, unittest.TestCase):
    simulation = False


img = None


def setUpModule():
    global img
    if os.getenv("BLINDAI_TEST_SKIP_COVIDNET") is not None:
        return
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

    return img


if __name__ == "__main__":
    unittest.main()
