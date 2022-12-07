DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

if [ -f $DIR/wav2vec2.onnx ]; then
    echo "Nothing to do."
    exit
fi

[ ! -d $DIR/tmp ] && mkdir $DIR/tmp
wget -q -O $DIR/tmp/hello_world.wav https://github.com/mithril-security/blindai/raw/master/examples/wav2vec2/hello_world.wav

python $DIR/_setup.py
python -m onnxsim $DIR/tmp/wav2vec2_nooptim.onnx $DIR/wav2vec2.onnx
