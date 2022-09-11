import itertools
import struct
from typing import Iterable, Iterator, TypeVar
from blindai.utils.utils import ModelDatumType


CHUNK_SIZE = 32 * 1024  # 32kb

bytes_per_item = {
    ModelDatumType.F32: 4,
    ModelDatumType.F64: 8,
    ModelDatumType.I32: 4,
    ModelDatumType.I64: 8,
    ModelDatumType.U32: 4,
    ModelDatumType.U64: 8,
    ModelDatumType.U8: 1,
    ModelDatumType.U16: 2,
    ModelDatumType.I8: 1,
    ModelDatumType.I16: 2,
    ModelDatumType.Bool: 1,
}

format_per_item = {
    ModelDatumType.F32: "f",
    ModelDatumType.F64: "d",
    ModelDatumType.I32: "i",
    ModelDatumType.I64: "q",
    ModelDatumType.U32: "I",
    ModelDatumType.U64: "Q",
    ModelDatumType.U8: "B",
    ModelDatumType.U16: "U",
    ModelDatumType.I8: "b",
    ModelDatumType.I16: "h",
    ModelDatumType.Bool: "?",
}

T = TypeVar("T")


def serialize_tensor(tensor: Iterable[T], type: ModelDatumType) -> Iterator[bytes]:
    num_items_per_chunk = CHUNK_SIZE // bytes_per_item[type]
    fmt = format_per_item[type]
    it = iter(tensor)

    first_round = True
    while True:
        items = list(itertools.islice(it, num_items_per_chunk))
        # first_round condition: this enabled the function to return an empty byte array when there are no element in the tensor
        if len(items) == 0 and not first_round:
            break
        yield struct.pack(f"<{len(items)}{fmt}", *items)
        first_round = False


def deserialize_tensor(data: bytes, type: ModelDatumType) -> Iterator[T]:
    item_size = bytes_per_item[type]
    fmt = format_per_item[type]
    num_items_per_chunk = len(data) // item_size
    if len(data) % item_size != 0:
        raise ValueError("Invalid data length")

    # flatten into an iterator using double list comprehension
    return (
        y for ys in struct.iter_unpack(f"<{num_items_per_chunk}{fmt}", data) for y in ys
    )
