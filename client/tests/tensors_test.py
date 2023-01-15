from blindai_preview.client import *
import numpy
import torch


def testTensorDeserialization():
    serialized = {
        "info": {"fact": [1, 2], "datum_type": "F32", "node_name": "output"},
        "bytes_data": [130, 250, 60, 145, 103, 64, 250, 190, 46, 46, 234],
    }
    tensor = Tensor(TensorInfo(**serialized["info"]), serialized["bytes_data"])
    expected = [0.017749428749084473, -0.1701008379459381]

    assert tensor.as_flat() == expected
    print(tensor.as_torch(), torch.tensor([expected]))
    torch_results = torch.isclose(tensor.as_torch(), torch.tensor([expected])).tolist()
    if isinstance(torch_results, list):
        assert all(torch_results)
    else:
        assert torch_results


def testTensorSerialization():
    tensor1 = [1, 2, 3, 4]
    o = translate_tensors(tensor1, ModelDatumType.I64, (4,))
    assert tensor1 == cbor.loads(bytes(o[0]["bytes_data"]))
    assert o[0]["info"] == {
        "fact": (4,),
        "datum_type": ModelDatumType.I64,
        "node_name": None,
    }

    import numpy

    tensor2 = numpy.array([1, 2, 3, 4])
    o = translate_tensors(tensor2, None, None)
    assert tensor2.tolist() == cbor.loads(bytes(o[0]["bytes_data"])), o[0]["info"]
    assert o[0]["info"] == {
        "fact": (4,),
        "datum_type": ModelDatumType.I64,
        "node_name": None,
    }

    import torch

    tensor3 = torch.tensor([1, 2, 3, 4])
    o = translate_tensors(tensor3, None, None)
    assert tensor3.tolist() == cbor.loads(bytes(o[0]["bytes_data"])), o[0]["info"]
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
    for t in o:
        t["bytes_data"] = cbor.loads(bytes(t["bytes_data"]))
    assert o == [
        {
            "info": {"fact": (4,), "datum_type": ModelDatumType.I64, "node_name": None},
            "bytes_data": [1, 2, 3, 4],
        },
        {
            "info": {"fact": (4,), "datum_type": ModelDatumType.I64, "node_name": None},
            "bytes_data": [1, 2, 3, 4],
        },
        {
            "info": {
                "fact": torch.Size([4]),
                "datum_type": ModelDatumType.I64,
                "node_name": None,
            },
            "bytes_data": [1, 2, 3, 4],
        },
    ]

    o = translate_tensors(
        {"tensor1": tensor1, "tensor2": tensor2, "tensor3": tensor3},
        {"tensor1": ModelDatumType.I64, "tensor2": None, "tensor3": None},
        {"tensor1": (4,), "tensor2": None, "tensor3": None},
    )
    for t in o:
        t["bytes_data"] = cbor.loads(bytes(t["bytes_data"]))
    assert o == [
        {
            "info": {
                "fact": (4,),
                "datum_type": ModelDatumType.I64,
                "node_name": "tensor1",
            },
            "bytes_data": [1, 2, 3, 4],
        },
        {
            "info": {
                "fact": (4,),
                "datum_type": ModelDatumType.I64,
                "node_name": "tensor2",
            },
            "bytes_data": [1, 2, 3, 4],
        },
        {
            "info": {
                "fact": torch.Size([4]),
                "datum_type": ModelDatumType.I64,
                "node_name": "tensor3",
            },
            "bytes_data": [1, 2, 3, 4],
        },
    ]
