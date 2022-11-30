if [ -f ./facenet.onnx ]; then
    echo "Nothing to do."
    exit
fi

[ ! -d ./tmp ] && mkdir tmp
wget -q -O ./tmp/woman_0.jpg https://github.com/mithril-security/blindai/raw/master/examples/facenet/woman_0.jpg

python3 ./_setup.py
python3 -m onnxsim ./tmp/facenet_nooptim.onnx ./facenet.onnx
