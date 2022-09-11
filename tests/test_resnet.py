import torch
import os
import blindai
import unittest
from server import (
    close_server,
    launch_server,
    policy_file,
    certificate_file,
    has_hardware_support,
)


class TestResnetBase:
    def setUp(self):
        if not self.simulation and not has_hardware_support:
            self.skipTest("no hardware support")

    def test_upload(self):
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            upload_response = client.upload_model(model_path)
            client.predict(upload_response.model_id, torch.zeros(3, 3, 224, 224))
            client.predict(upload_response.model_id, torch.zeros(5, 3, 224, 224))
            client.predict(upload_response.model_id, torch.zeros(10, 3, 224, 224))
            client.predict(upload_response.model_id, torch.zeros(1, 3, 224, 224))
            with self.assertRaises(ConnectionError):
                client.predict(upload_response.model_id, torch.zeros(1, 3, 224, 100))
            with self.assertRaises(ConnectionError):
                client.predict(upload_response.model_id, torch.zeros(3, 224, 224))
            with self.assertRaises(ConnectionError):
                client.predict(upload_response.model_id, torch.zeros(1, 1, 3, 224, 224))
            with self.assertRaises(ConnectionError):
                client.predict(
                    upload_response.model_id,
                    torch.zeros(1, 3, 224, 224, dtype=torch.int64),
                )


class TestResnetSW(TestResnetBase, unittest.TestCase):
    simulation = True


class TestResnetHW(TestResnetBase, unittest.TestCase):
    simulation = False


model_path = None


def setUpModule():
    global model_path
    launch_server()

    model_path = os.path.join(os.path.dirname(__file__), "resnet18.onnx")

    import urllib

    url = "https://github.com/onnx/models/raw/main/vision/classification/resnet/model/resnet18-v2-7.onnx"
    try:
        urllib.URLopener().retrieve(url, model_path)
    except Exception:
        urllib.request.urlretrieve(url, model_path)


if __name__ == "__main__":
    unittest.main()
