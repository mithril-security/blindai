#!/bin/sh

cd blindai

mkdir lib

cd ../third_party/attestationLib

mkdir build

cd build

cmake ..

make

cd AttestationApp/CMakeFiles/AppCore.dir/src/AppCore/

g++ -dynamiclib -o libverify.dylib *.o ../../../../../../Build/out/lib/libQuoteVerification.dylib ../../../../../../Build/out/lib/libQuoteVerificationStatic.a ../../../../../../Build/out/lib/libargtable3.a

cp libverify.dylib ../../../../../../../../blindai/lib

cd ../../../../../../Build/out/lib

cp libQuoteVerification.dylib ../../../../../blindai/lib