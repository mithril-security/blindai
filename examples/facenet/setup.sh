DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

if [ -f $DIR/facenet.onnx ]; then
    echo "Nothing to do."
    exit
fi

[ ! -d $DIR/tmp ] && mkdir $DIR/tmp
wget -q -O $DIR/tmp/woman_0.jpg https://github.com/mithril-security/blindai/raw/master/examples/facenet/woman_0.jpg

python3 $DIR/_setup.py
python3 -m onnxsim $DIR/tmp/facenet_nooptim.onnx $DIR/facenet.onnx
