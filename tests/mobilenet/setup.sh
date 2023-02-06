mkdir -p .cache
pushd cache
wget -N https://github.com/onnx/models/raw/main/vision/classification/mobilenet/model/mobilenetv2-7.onnx
wget -N https://github.com/sonos/tract/raw/main/examples/onnx-mobilenet-v2/grace_hopper.jpg
popd
ln -f -rs .cache/* .