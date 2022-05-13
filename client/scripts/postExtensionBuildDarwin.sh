#!/bin/sh

# install_name_tool is used on OSX
__realpath(){
    local path=$1
    dir=$(dirname $path)
    if [ $? -eq 0 ];
    then
        cd $dir
        full_dir=$(pwd .)
        echo $full_dir    
    fi
}
QUOTE_LIB=$(find . -name "_quote_verification*.so")
BASE_DIR=$(__realpath $(basename ${QUOTE_LIB}))
cd -
install_name_tool -change libverify.dylib "$BASE_DIR/blindai/lib/libverify.dylib" ${QUOTE_LIB}
install_name_tool -change @rpath/libQuoteVerification.dylib "$BASE_DIR/blindai/lib/libQuoteVerification.dylib" ${QUOTE_LIB}
