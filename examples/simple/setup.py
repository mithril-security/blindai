#
# Export a simple onnx model and save its inputs in npz format
#

import numpy as np
import onnx
from onnx import numpy_helper
from onnx.helper import *
import os

node1 = make_node(
    'Sub',
    ['input', 'sub'],
    ['result']
)

input1 = make_tensor_value_info('input', onnx.TensorProto.INT64, [])
sub = make_tensor_value_info('sub', onnx.TensorProto.INT64, [])
result = make_tensor_value_info('result', onnx.TensorProto.INT64, [])

graph = make_graph(
    [node1], 'test_graph',
    [input1, sub],
    [result]
)
model = make_model(graph)
model.opset_import[0].version = 12

with open( "simple.onnx", "wb") as f:
    f.write(model.SerializeToString())

# print(model)
# print(printable_graph(model.graph))

inputs = {"input":np.array(42), "sub":np.array(40)}

np.savez( "simple.npz", **inputs)