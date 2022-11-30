## script to get names and shapes of input and output nodes of an onnx model

import onnx
import sys

if len(sys.argv) != 2:
    print("usage: python get_onnx_metadat.py <model_name>")
    exit()
model = onnx.load(sys.argv[1])
output_node =[(node.name, node.type.tensor_type.shape.dim) for node in model.graph.output][0]

input_all = [node.name for node in model.graph.input]
input_initializer =  [node.name for node in model.graph.initializer]
net_feed_input = list(set(input_all)  - set(input_initializer))
input_nodes = []
for node in model.graph.input:
    if node.name in net_feed_input:
        input_nodes += (node.name, node.type.tensor_type.shape.dim)
print('Inputs: ', input_nodes)
print('Outputs: ', output_node)