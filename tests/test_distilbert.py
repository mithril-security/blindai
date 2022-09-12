import shutil
from transformers import DistilBertForSequenceClassification
from transformers import DistilBertTokenizer
import torch
import blindai
from blindai import ModelDatumType
import unittest
import os
from server import (
    launch_server,
    policy_file,
    certificate_file,
    has_hardware_support,
    close_server,
    bin_dir,
    with_server_config,
)

model_path = os.path.join(os.path.dirname(__file__), "distilbert-base-uncased.onnx")


class TestDistilBertBase:
    def setUp(self):
        if not self.simulation and not has_hardware_support:
            self.skipTest("no hardware support")

    def test_base(self):
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:

            response = client.upload_model(
                model=model_path,
            )
            model_id = response.model_id

            response = client.predict(model_id, inputs)
            origin_pred = model(inputs).logits.detach()

            diff = (
                (torch.tensor([response.output[0].as_flat()]) - origin_pred).sum().abs()
            )
            self.assertLess(diff, 0.001)  # difference is <0.1%
            client.delete_model(model_id)

    def test_signed(self):
        with blindai.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:
            response = client.upload_model(
                model=model_path,
                sign=True,
            )
            model_id = response.model_id

            client.enclave_signing_key.verify(response.signature, response.payload)

            response = client.predict(model_id, inputs, sign=True)

            client.enclave_signing_key.verify(response.signature, response.payload)

        origin_pred = model(inputs).logits.detach()

        diff = (torch.tensor([response.output[0].as_flat()]) - origin_pred).sum().abs()
        self.assertLess(diff, 0.001)  # difference is <0.1%


class TestDistilBertSW(TestDistilBertBase, unittest.TestCase):
    simulation = True


class TestDistilBertHW(TestDistilBertBase, unittest.TestCase):
    simulation = False


model, inputs = None, None


def setUpModule():
    global model, inputs
    launch_server()

    # Setup the distilbert model
    model = DistilBertForSequenceClassification.from_pretrained(
        "distilbert-base-uncased"
    )

    # Create dummy input for export
    tokenizer = DistilBertTokenizer.from_pretrained("distilbert-base-uncased")
    sentence = "I love AI and privacy!"
    inputs = tokenizer(
        sentence, padding="max_length", max_length=8, return_tensors="pt"
    )["input_ids"]

    # Export the model
    torch.onnx.export(
        model,
        inputs,
        model_path,
        export_params=True,
        opset_version=11,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={"input": {0: "batch_size"}, "output": {0: "batch_size"}},
    )


if __name__ == "__main__":
    unittest.main()
