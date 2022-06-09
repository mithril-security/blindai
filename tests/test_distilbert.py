from transformers import DistilBertForSequenceClassification
from transformers import DistilBertTokenizer
import torch
import blindai.client
from blindai.client import ModelDatumType
import unittest
import os
from server import (
    launch_server,
    policy_file,
    certificate_file,
    has_hardware_support,
)

model_path = os.path.join(os.path.dirname(__file__), "distilbert-base-uncased.onnx")


class TestDistilBertBase:
    def setUp(self):
        if not self.simulation and not has_hardware_support:
            self.skipTest("no hardware support")

    def test_base(self):
        with blindai.client.connect(
            addr="localhost",
            simulation=self.simulation,
            policy=policy_file,
            certificate=certificate_file,
        ) as client:

            response = client.upload_model(
                model=model_path,
                shape=inputs.shape,
                dtype=ModelDatumType.I64,
                model_name="test.onnx",
            )
            model_id = response.model_id

            response = client.run_model(model_id, run_inputs)
            origin_pred = model(torch.tensor(run_inputs).unsqueeze(0)).logits.detach()

            diff = (torch.tensor([response.output]) - origin_pred).sum().abs()
            self.assertLess(diff, 0.001)  # difference is <0.1%
            client.delete_model(model_id)

        def test_signed(self):
            with blindai.client.connect(
                addr="localhost",
                simulation=self.simulation,
                policy=policy_file,
                certificate=certificate_file,
            ) as client:
                response = client.upload_model(
                    model=model_path,
                    shape=inputs.shape,
                    dtype=ModelDatumType.I64,
                    sign=True,
                )
                model_id = response.model_id

                print(response)
                client.enclave_signing_key.verify(response.signature, response.payload)

                response = client.run_model(model_id, run_inputs, sign=True)

                client.enclave_signing_key.verify(response.signature, response.payload)

            origin_pred = model(torch.tensor(run_inputs).unsqueeze(0)).logits.detach()

            diff = (torch.tensor([response.output]) - origin_pred).sum().abs()
            self.assertLess(diff, 0.001)  # difference is <0.1%


class TestDistilBertSW(TestDistilBertBase, unittest.TestCase):
    simulation = True


class TestDistilBertHW(TestDistilBertBase, unittest.TestCase):
    simulation = False


model, inputs, run_inputs = None, None, None


def setUpModule():
    global model, inputs, run_inputs
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

    run_inputs = tokenizer(sentence, padding="max_length", max_length=8)["input_ids"]


if __name__ == "__main__":
    unittest.main()
