if [ -f ./wav2vec2.onnx ]; then
    echo "Nothing to do."
    exit
fi

[ ! -d ./tmp ] && mkdir tmp
wget -q -O ./tmp/hello_world.wav https://github.com/mithril-security/blindai/raw/master/examples/wav2vec2/hello_world.wav

python _setup.py
python -m onnxsim ./tmp/wav2vec2_nooptim.onnx ./wav2vec2.onnx
