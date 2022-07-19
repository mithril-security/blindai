import unittest
import os
import shutil
import blindai.client
from blindai.client import ModelDatumType
from transformers import AutoModel, AutoTokenizer
import numpy as np
import torch
from server import (
    policy_file,
    certificate_file,
    has_hardware_support,
    with_server_config,
    bin_dir,
)
from test_covidnet import setUpModule as covidNetSetUpModule

model_path = os.path.join(
    os.path.dirname(__file__), "assets/COVID-Net-CXR-2.onnx"
)


class TestServerConfigBase:
    def setUp(self):
        if not self.simulation and not has_hardware_support:
            self.skipTest("no hardware support")

    @with_server_config(
        os.path.join(os.path.dirname(__file__), "startup_model_config.toml")
    )
    def test_startup_models(self):
        shutil.copyfile(
            model_path,
            os.path.join(bin_dir, "covidnet.onnx"),
        )
        with blindai.client.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            response = client.run_model(
                "covidnet",
                inputs,
                dtype=ModelDatumType.F32,
                shape=(1, 480, 480, 3),
                sign=True,
            )

    @with_server_config(
        os.path.join(os.path.dirname(__file__), "startup_model_config.toml")
    )
    def test_config_no_sendmodel(self):
        with blindai.client.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            with self.assertRaises(ConnectionError):
                # send model disabled, so
                # stream is not consumed in server, so it makes a connection error
                # i guess?
                response = client.upload_model(
                    model=model_path,
                    shape=(1, 480, 480, 3),
                    dtype=ModelDatumType.F32,
                    sign=True,
                )

    @with_server_config(
        os.path.join(os.path.dirname(__file__), "startup_model_gptneox_config.toml")
    )
    def test_gptneox(self):
        with blindai.client.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            response = client.run_model(
                "gpt-neo-2.7b",
                gptneox_inputs.flatten().tolist(),
                dtype=ModelDatumType.I64,
                shape=gptneox_inputs.shape,
                sign=True,
            )


class TestServerConfigSW(TestServerConfigBase, unittest.TestCase):
    simulation = True


class TestServerConfigHW(TestServerConfigBase, unittest.TestCase):
    simulation = False


if __name__ == "__main__":
    unittest.main()

gptneox_inputs, inputs = None, None

def setUpModule():
    global gptneox_inputs, inputs
    inputs = covidNetSetUpModule()

    gptneox_inputs = torch.tensor(np.load(os.path.join(os.path.dirname(__file__), "./gpt-neo-2.7b.npz"))["inputs"])

    if not os.path.exists(os.path.join(bin_dir, "./gpt-neo-2.7b")):
        model = AutoModel.from_pretrained("EleutherAI/gpt-neo-2.7B")

        os.mkdir(os.path.join(bin_dir, "./gpt-neo-2.7b"))
        torch.onnx.export(
            model, 
            gptneox_inputs,
            os.path.join(bin_dir, "./gpt-neo-2.7b/gpt-neo-2.7b.onnx"),
            export_params=True,
        )
