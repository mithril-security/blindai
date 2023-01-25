from blindai_preview.client import *
import numpy
import torch


def testTensorDeserialization():
    serialized = {
        "info": {"fact": [4], "datum_type": "I64", "node_name": "output"},
        "bytes_data": b"\x01\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00",
    }
    tensor = Tensor(TensorInfo(**serialized["info"]), serialized["bytes_data"])
    expected = [1, 2, 3, 4]

    torch_results = torch.isclose(
        tensor.as_torch(), torch.tensor(expected, dtype=torch.int64)
    ).tolist()
    if isinstance(torch_results, list):
        assert all(torch_results)
    else:
        assert torch_results


def testTensorSerialization():
    expected_bytes = b"\x01\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00"

    tensor1 = [1, 2, 3, 4]
    o = translate_tensors(tensor1, ModelDatumType.I64, (4,))
    assert o[0]["bytes_data"] == expected_bytes
    assert o[0]["info"] == {
        "fact": (4,),
        "datum_type": ModelDatumType.I64,
        "node_name": None,
    }

    import numpy

    tensor2 = numpy.array([1, 2, 3, 4])
    o = translate_tensors(tensor2, None, None)
    assert o[0]["bytes_data"] == expected_bytes
    assert o[0]["info"] == {
        "fact": (4,),
        "datum_type": ModelDatumType.I64,
        "node_name": None,
    }

    import torch

    tensor3 = torch.tensor([1, 2, 3, 4])
    o = translate_tensors(tensor3, None, None)
    assert o[0]["bytes_data"] == expected_bytes
    assert o[0]["info"] == {
        "fact": torch.Size([4]),
        "datum_type": ModelDatumType.I64,
        "node_name": None,
    }

    o = translate_tensors(
        [tensor1, tensor2, tensor3],
        [ModelDatumType.I64, None, None],
        [(4,), None, None],
    )
    assert o == [
        {
            "info": {"fact": (4,), "datum_type": ModelDatumType.I64, "node_name": None},
            "bytes_data": expected_bytes,
        },
        {
            "info": {"fact": (4,), "datum_type": ModelDatumType.I64, "node_name": None},
            "bytes_data": expected_bytes,
        },
        {
            "info": {
                "fact": torch.Size([4]),
                "datum_type": ModelDatumType.I64,
                "node_name": None,
            },
            "bytes_data": expected_bytes,
        },
    ]

    o = translate_tensors(
        {"tensor1": tensor1, "tensor2": tensor2, "tensor3": tensor3},
        {"tensor1": ModelDatumType.I64, "tensor2": None, "tensor3": None},
        {"tensor1": (4,), "tensor2": None, "tensor3": None},
    )
    assert o == [
        {
            "info": {
                "fact": (4,),
                "datum_type": ModelDatumType.I64,
                "node_name": "tensor1",
            },
            "bytes_data": expected_bytes,
        },
        {
            "info": {
                "fact": (4,),
                "datum_type": ModelDatumType.I64,
                "node_name": "tensor2",
            },
            "bytes_data": expected_bytes,
        },
        {
            "info": {
                "fact": torch.Size([4]),
                "datum_type": ModelDatumType.I64,
                "node_name": "tensor3",
            },
            "bytes_data": expected_bytes,
        },
    ]
