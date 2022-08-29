import unittest
import os
import shutil
import blindai
from blindai import ModelDatumType
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

model_path = os.path.join(os.path.dirname(__file__), "assets/COVID-Net-CXR-2.onnx")


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
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            client.predict(
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
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            with self.assertRaises(ConnectionError):
                # send model disabled, so
                # stream is not consumed in server, so it makes a connection error
                # i guess?
                client.upload_model(
                    model=model_path,
                    shape=(1, 480, 480, 3),
                    dtype=ModelDatumType.F32,
                    sign=True,
                )


class TestServerConfigSW(TestServerConfigBase, unittest.TestCase):
    simulation = True


class TestServerConfigHW(TestServerConfigBase, unittest.TestCase):
    simulation = False


if __name__ == "__main__":
    unittest.main()

inputs = None


def setUpModule():
    global inputs
    inputs = covidNetSetUpModule()
