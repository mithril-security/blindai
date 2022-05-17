#!/bin/sh

# A replacement for realpath on Linux
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

# install_name_tool is used on OSX
OLD_PWD=$(pwd)
QUOTE_LIBS=$(find . -name "_quote_verification*.so")
for QUOTE_LIB in ${QUOTE_LIBS}
do
    BASE_DIR=$(__realpath $(basename ${QUOTE_LIB}))
    cd ${OLD_PWD}

    # install_name_tool replaces patchelf on Linux
    install_name_tool -change libverify.dylib "$BASE_DIR/blindai/lib/libverify.dylib" ${QUOTE_LIB}
    install_name_tool -change @rpath/libQuoteVerification.dylib "$BASE_DIR/blindai/lib/libQuoteVerification.dylib" ${QUOTE_LIB}
done