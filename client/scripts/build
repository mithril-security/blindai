#!/bin/sh

cd blindai

mkdir lib

cd ../third_party/attestationLib

mkdir build

cd build

cmake ..

make

cd AttestationApp/CMakeFiles/AppCore.dir/src/AppCore/

g++ -shared -o libverify.so *.o ../../../../../../Build/out/lib/libargtable3.a

cp libverify.so ../../../../../../../../blindai/lib

cd ../../../../../../Build/out/lib

cp libQuoteVerification.so ../../../../../blindai/lib