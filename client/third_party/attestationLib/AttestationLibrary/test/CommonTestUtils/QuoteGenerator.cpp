/*
 * Copyright (C) 2011-2021 Intel Corporation. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 *   * Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 *   * Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in
 *     the documentation and/or other materials provided with the
 *     distribution.
 *   * Neither the name of Intel Corporation nor the names of its
 *     contributors may be used to endorse or promote products derived
 *     from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 */

#include "QuoteGenerator.h"
#include <type_traits>
#include <algorithm>

namespace intel { namespace sgx { namespace dcap { namespace test {

namespace {

class ConvertToBytesDetail
{
public:
    template<class DataType>
    static Bytes convert(const DataType& data)
    {
        return ensureIntegersAreLittleEndian<DataType>(toBytes(data));
    }

private:
    static bool isSystemLittleEndian()
    {
        uint32_t i = 0x01020304;
        return reinterpret_cast<char*>(&i)[0] == 4;
    }

    template<class DataType>
    static typename std::enable_if<!std::is_integral<DataType>::value, Bytes>::type ensureIntegersAreLittleEndian(Bytes bytes)
    {
        return bytes;
    }

    template<class DataType>
    static typename std::enable_if<std::is_integral<DataType>::value, Bytes>::type ensureIntegersAreLittleEndian(Bytes bytes)
    {
        if (!isSystemLittleEndian())
        {
            std::reverse(bytes.begin(), bytes.end());
        }
        return bytes;
    }
};

template<class DataType>
Bytes convertToBytes(DataType& data)
{
    return ConvertToBytesDetail::convert(data);
}

constexpr uint16_t DEFAULT_VERSION = 3;
constexpr uint16_t DEFAULT_ATTESTATION_KEY_TYPE = 2;
constexpr char INTEL_QE_VENDOR_UUID[] = "939A7233F79C4CA9940A0DB3957F0607";

QuoteGenerator::EnclaveReport defaultEnclaveReport()
{
    return {
            {}, //cpusvn
            0,  //miscselect
            {}, //reserved1
            {}, //attributes
            {}, //mrenclave
            {}, //reserved2
            {}, //mrsigner
            {}, //reserved3
            0,  //isvProdId
            0,  //isvSvn
            {}, //reserved4
            {}  //reportdata
    };
}

QuoteGenerator::QuoteHeader defaultHeader()
{
    return {
            DEFAULT_VERSION,
            DEFAULT_ATTESTATION_KEY_TYPE,
            0,
            0,
            0,
            0,
            {{0}},
            {{0}}
    };
}

QuoteGenerator::EcdsaSignature defaultSignature()
{
    QuoteGenerator::EcdsaSignature ret{ {{0}} }; 
    return ret;
}

QuoteGenerator::EcdsaPublicKey defaultPubKey()
{
    return {
        {{0}}
    };
}

QuoteGenerator::QeAuthData defaultQeAuthData()
{
    return {
        0,
        {}
    };
}

QuoteGenerator::QeCertData defaultQeCertData()
{
    return {
        0,
        0,
        {}
    };
}

} //anonymous namespace

QuoteGenerator::QuoteGenerator() :
        header(defaultHeader()),
        enclaveReport(defaultEnclaveReport()),
        quoteAuthData{
            test::QUOTE_AUTH_DATA_MIN_SIZE,
            defaultSignature(),
            defaultPubKey(),
            defaultEnclaveReport(), //qeReport
            defaultSignature(), //qeReportSignature
            defaultQeAuthData(),
            defaultQeCertData()
        }
{
    static_assert(sizeof(QuoteHeader) == QUOTE_HEADER_SIZE, "Incorrect header size");
    static_assert(sizeof(EnclaveReport) == ENCLAVE_REPORT_SIZE, "Incorrect enclave report size");
    static_assert(sizeof(EcdsaSignature) == ENCLAVE_REPORT_SIGNATURE_SIZE, "Incorrect enclave report signature size");
    static_assert(sizeof(EcdsaPublicKey) == ECDSA_PUBLIC_KEY_SIZE, "Incorrect public key size");

    auto uuid = hexStringToBytes(INTEL_QE_VENDOR_UUID);
    std::copy(uuid.begin(), uuid.end(), header.qeVendorId.begin());
}

QuoteGenerator& QuoteGenerator::withQeSvn(uint16_t qeSvn)
{
    header.qeSvn = qeSvn;
    return *this;
}

QuoteGenerator& QuoteGenerator::withPceSvn(uint16_t pceSvn)
{
    header.pceSvn = pceSvn;
    return *this;
}

QuoteGenerator& QuoteGenerator::withHeader(const QuoteHeader& _header)
{
    this->header = _header;
    return *this;
}

QuoteGenerator& QuoteGenerator::withEnclaveReport(const EnclaveReport& _body)
{
    this->enclaveReport = _body;
    return *this;
}

QuoteGenerator& QuoteGenerator::withAuthDataSize(uint32_t size)
{
    quoteAuthData.authDataSize = size;
    return *this;
}

QuoteGenerator& QuoteGenerator::withQeReport(const EnclaveReport& report)
{
    quoteAuthData.qeReport = report;
    return *this;
}

QuoteGenerator& QuoteGenerator::withQeReportSignature(const EcdsaSignature& sign)
{
    quoteAuthData.qeReportSignature = sign;
    return *this;
}

QuoteGenerator& QuoteGenerator::withQuoteSignature(const EcdsaSignature& signature)
{
    quoteAuthData.ecdsaSignature = signature;
    return *this; 
}

QuoteGenerator& QuoteGenerator::withAttestationKey(const EcdsaPublicKey& pubKey)
{
    quoteAuthData.ecdsaAttestationKey = pubKey;
    return *this;
}

QuoteGenerator& QuoteGenerator::withAuthData(const QuoteGenerator::QuoteAuthData& authData)
{
    quoteAuthData = authData;
    return *this;
}

QuoteGenerator& QuoteGenerator::withQeAuthData(const QuoteGenerator::QeAuthData& qeAuth)
{
    quoteAuthData.qeAuthData = qeAuth;
    return *this;
}

QuoteGenerator& QuoteGenerator::withQeAuthData(const Bytes& authData)
{
    quoteAuthData.qeAuthData.data = authData;
    quoteAuthData.qeAuthData.size = static_cast<uint16_t>(authData.size());
    return *this;
}

QuoteGenerator& QuoteGenerator::withQeCertData(const QeCertData& qeCertData)
{
    quoteAuthData.qeCertData = qeCertData;
    return *this;
}

QuoteGenerator& QuoteGenerator::withQeCertData(uint16_t type, const Bytes& keyData)
{
    quoteAuthData.qeCertData.keyDataType = type;
    quoteAuthData.qeCertData.keyData = keyData;
    quoteAuthData.qeCertData.size = static_cast<uint32_t>(keyData.size());
    return *this;
}

Bytes QuoteGenerator::buildSgxQuote()
{
	return header.bytes() + enclaveReport.bytes() + quoteAuthData.bytes();
}

Bytes QuoteGenerator::QuoteHeader::bytes() const
{
    return
            convertToBytes(version) +
            convertToBytes(attestationKeyType) +
            convertToBytes(teeType) +
            convertToBytes(reserved) +
            convertToBytes(qeSvn) +
            convertToBytes(pceSvn) +
            convertToBytes(qeVendorId) +
            convertToBytes(userData);
}

Bytes QuoteGenerator::EnclaveReport::bytes() const
{
    return
        convertToBytes(cpuSvn) +
        convertToBytes(miscSelect) +
        convertToBytes(reserved1) +
        convertToBytes(attributes) +
        convertToBytes(mrEnclave) +
        convertToBytes(reserved2) +
        convertToBytes(mrSigner) +
        convertToBytes(reserved3) +
        convertToBytes(isvProdID) +
        convertToBytes(isvSvn) +
        convertToBytes(reserved4) +
        convertToBytes(reportData);
}

Bytes QuoteGenerator::QuoteAuthData::bytes() const
{
    return
        convertToBytes(authDataSize) +
        ecdsaSignature.bytes() +
        ecdsaAttestationKey.bytes() +
        qeReport.bytes() +
        qeReportSignature.bytes() +
        qeAuthData.bytes() +
        qeCertData.bytes();
}

Bytes QuoteGenerator::EcdsaSignature::bytes() const
{
    return convertToBytes(signature);
}

Bytes QuoteGenerator::EcdsaPublicKey::bytes() const
{
    return convertToBytes(publicKey);
}

Bytes QuoteGenerator::QeAuthData::bytes() const
{
    return convertToBytes(size) + data;
}

Bytes QuoteGenerator::QeCertData::bytes() const
{
    return convertToBytes(keyDataType) + convertToBytes(size) + keyData;
}

}}}}
