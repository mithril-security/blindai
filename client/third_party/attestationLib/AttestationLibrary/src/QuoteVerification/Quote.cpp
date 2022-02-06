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

#include "Quote.h"
#include "ByteOperands.h"

#include <algorithm>
#include <iterator>
#include "QuoteConstants.h"

namespace intel { namespace sgx { namespace dcap {

namespace {
    using namespace constants;

    constexpr size_t HEADER_BYTE_LEN = 48;
    constexpr size_t AUTH_DATA_SIZE_BYTE_LEN = 4;

    constexpr size_t ECDSA_SIGNATURE_BYTE_LEN = 64;
    constexpr size_t ECDSA_PUBKEY_BYTE_LEN = 64;
    constexpr size_t QE_REPORT_BYTE_LEN = ENCLAVE_REPORT_BYTE_LEN;
    constexpr size_t QE_REPORT_SIG_BYTE_LEN = ECDSA_SIGNATURE_BYTE_LEN;
    constexpr size_t QE_AUTH_DATA_SIZE_BYTE_LEN = 2;
    constexpr size_t QE_CERT_DATA_TYPE_BYTE_LEN = 2;
    constexpr size_t QE_CERT_DATA_SIZE_BYTE_LEN = 4;

    constexpr size_t AUTH_DATA_MIN_BYTE_LEN =
            ECDSA_SIGNATURE_BYTE_LEN +
            ECDSA_PUBKEY_BYTE_LEN +
            QE_REPORT_BYTE_LEN +
            QE_REPORT_SIG_BYTE_LEN +
            QE_AUTH_DATA_SIZE_BYTE_LEN +
            QE_CERT_DATA_TYPE_BYTE_LEN +
            QE_CERT_DATA_SIZE_BYTE_LEN;

    constexpr size_t QUOTE_MIN_BYTE_LEN =
            HEADER_BYTE_LEN +
            ENCLAVE_REPORT_BYTE_LEN +
            AUTH_DATA_SIZE_BYTE_LEN +
            AUTH_DATA_MIN_BYTE_LEN;

    template<typename T>
    bool copyAndAdvance(T& val, std::vector<uint8_t>::const_iterator& from, size_t amount, const std::vector<uint8_t>::const_iterator& totalEnd)
    {
        const auto available = std::distance(from, totalEnd);
        if (available < 0 || (unsigned) available < amount)
        {
            return false;
        }
        const auto end = std::next(from, static_cast<long>(amount));
        return val.insert(from, end);
    }

    template<size_t N>
    bool copyAndAdvance(std::array<uint8_t, N>& arr, std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& totalEnd)
    {
        const auto capacity = std::distance(arr.cbegin(), arr.cend());
        if (std::distance(from, totalEnd) < capacity)
        {
            return false;
        }
        const auto end = std::next(from, capacity);
        std::copy(from, end, arr.begin());
        std::advance(from, capacity);
        return true;
    }

    bool copyAndAdvance(uint16_t& val, std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& totalEnd)
    {
        const auto available = std::distance(from, totalEnd);
        const auto capacity = sizeof(uint16_t);
        if (available < 0 || (unsigned) available < capacity)
        {
            return false;
        }

        val = swapBytes(toUint16(*from, *(std::next(from))));
        std::advance(from, capacity);
        return true;
    }


    bool copyAndAdvance(uint32_t& val, std::vector<uint8_t>::const_iterator& position, const std::vector<uint8_t>::const_iterator& totalEnd)
    {
        const auto available = std::distance(position, totalEnd);
        const auto capacity = sizeof(uint32_t);
        if (available < 0 || (unsigned) available < capacity)
        {
            return false;
        }

        val = swapBytes(toUint32(*position, *(std::next(position)), *(std::next(position, 2)), *(std::next(position, 3))));
        std::advance(position, capacity);
        return true;
    }

} // anonymous namespace

bool Quote::parse(const std::vector<uint8_t>& rawQuote)
{
    if(rawQuote.size() < QUOTE_MIN_BYTE_LEN)
    {
        return false;
    }

    auto from = rawQuote.cbegin();
    Header localHeader;
    if (!copyAndAdvance(localHeader, from, HEADER_BYTE_LEN, rawQuote.cend())) {
        return false;
    }

    EnclaveReport localEnclaveReport = {};
    if (localHeader.teeType == TEE_TYPE_SGX)
    {
        if (!copyAndAdvance(localEnclaveReport, from, ENCLAVE_REPORT_BYTE_LEN, rawQuote.cend())) {
            return false;
        }
    }

    uint32_t localAuthDataSize;
    if (!copyAndAdvance(localAuthDataSize, from, rawQuote.cend())) {
        return false;
    }
    const auto remainingDistance = std::distance(from, rawQuote.cend());
    if(localAuthDataSize != remainingDistance)
    {
        return false;
    }

    Ecdsa256BitQuoteAuthData localQuoteAuth;
    if (!copyAndAdvance(localQuoteAuth, from, static_cast<size_t>(localAuthDataSize), rawQuote.cend())) {
        return false;
    }

    // parsing done, we should be precisely at the end of our buffer
    // if we're not it means inconsistency in internal structure
    // and it means invalid format
    if(from != rawQuote.cend())
    {
        return false;
    }

    header = localHeader;
    bodyEnclaveReport = localEnclaveReport;
    authDataSize = localAuthDataSize;
    authData = localQuoteAuth;
    if (localHeader.teeType == TEE_TYPE_SGX)
    {
        signedData = getDataToSignatureVerification(rawQuote, HEADER_BYTE_LEN + QE_REPORT_BYTE_LEN);
    }

    return true;
}

bool Quote::parseEnclaveReport(const std::vector<uint8_t> &enclaveReport)
{
    if (header.teeType != TEE_TYPE_SGX)
    {
        return false;
    }

    if(enclaveReport.size() < ENCLAVE_REPORT_BYTE_LEN)
    {
        return false;
    }

    EnclaveReport localBody;
    auto from = enclaveReport.cbegin();
    auto end = enclaveReport.cend();
    if (!copyAndAdvance(localBody, from, ENCLAVE_REPORT_BYTE_LEN, end)) {
        return false;
    }

    if(from != end)
    {
        return false;
    }

    bodyEnclaveReport = localBody;

    return true;
}

bool Quote::validate() const
{
    if(std::find(ALLOWED_QUOTE_VERSIONS.begin(), ALLOWED_QUOTE_VERSIONS.end(), header.version) ==
       ALLOWED_QUOTE_VERSIONS.end())
    {
        return false;
    }

    if(std::find(ALLOWED_ATTESTATION_KEY_TYPES.begin(), ALLOWED_ATTESTATION_KEY_TYPES.end(), header.attestationKeyType) ==
       ALLOWED_ATTESTATION_KEY_TYPES.end())
    {
        return false;
    }

    if(std::find(ALLOWED_TEE_TYPES.begin(), ALLOWED_TEE_TYPES.end(), header.teeType) == ALLOWED_TEE_TYPES.end())
    {
        return false;
    }

    if(header.qeVendorId != INTEL_QE_VENDOR_ID) {
        return false;
    }

    if(header.version == QUOTE_VERSION_3 && header.teeType != TEE_TYPE_SGX)
    {
        return false;
    }

    return true;
}

const Quote::Header& Quote::getHeader() const
{
    return header;
}

const Quote::EnclaveReport& Quote::getEnclaveReport() const
{
    return bodyEnclaveReport;
}

uint32_t Quote::getAuthDataSize() const
{
    return authDataSize;
}

const Quote::Ecdsa256BitQuoteAuthData& Quote::getQuoteAuthData() const
{
    return authData;
}

const std::vector<uint8_t>& Quote::getSignedData() const
{
    return signedData;
}

bool Quote::Header::insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end)
{
    if (!copyAndAdvance(version, from, end)) { return false; }
    if (!copyAndAdvance(attestationKeyType, from, end)) { return false; }
    if (!copyAndAdvance(teeType, from, end)) { return false; }
    if (!copyAndAdvance(reserved, from, end)) { return false; }
    if (!copyAndAdvance(qeSvn, from, end)) { return false; }
    if (!copyAndAdvance(pceSvn, from, end)) { return false; }
    if (!copyAndAdvance(qeVendorId, from, end)) { return false; }
    if (!copyAndAdvance(userData, from, end)) { return false; }
    return true;
}

bool Quote::EnclaveReport::insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end)
{
    if (!copyAndAdvance(cpuSvn, from, end)) { return false; }
    if (!copyAndAdvance(miscSelect, from, end)) { return false; }
    if (!copyAndAdvance(reserved1, from, end)) { return false; }
    if (!copyAndAdvance(attributes, from, end)) { return false; }
    if (!copyAndAdvance(mrEnclave, from, end)) { return false; }
    if (!copyAndAdvance(reserved2, from, end)) { return false; }
    if (!copyAndAdvance(mrSigner, from, end)) { return false; }
    if (!copyAndAdvance(reserved3, from, end)) { return false; }
    if (!copyAndAdvance(isvProdID, from, end)) { return false; }
    if (!copyAndAdvance(isvSvn, from, end)) { return false; }
    if (!copyAndAdvance(reserved4, from, end)) { return false; }
    if (!copyAndAdvance(reportData, from, end)) { return false; }
    return true;
}

std::array<uint8_t,ENCLAVE_REPORT_BYTE_LEN> Quote::EnclaveReport::rawBlob() const
{
    std::array<uint8_t, ENCLAVE_REPORT_BYTE_LEN> ret{};
    auto to = ret.begin();
    std::copy(cpuSvn.begin(), cpuSvn.end(), to);
    std::advance(to, (unsigned) cpuSvn.size());

    const auto arrMiscSelect = toArray(swapBytes(miscSelect));
    std::copy(arrMiscSelect.begin(), arrMiscSelect.end(), to);
    std::advance(to, arrMiscSelect.size());

    std::copy(reserved1.begin(), reserved1.end(), to);
    std::advance(to, (unsigned) reserved1.size());

    std::copy(attributes.begin(), attributes.end(), to);
    std::advance(to, (unsigned) attributes.size());

    std::copy(mrEnclave.begin(), mrEnclave.end(), to);
    std::advance(to, (unsigned) mrEnclave.size());

    std::copy(reserved2.begin(), reserved2.end(), to);
    std::advance(to, (unsigned) reserved2.size());

    std::copy(mrSigner.begin(), mrSigner.end(), to);
    std::advance(to, (unsigned) mrSigner.size());

    std::copy(reserved3.begin(), reserved3.end(), to);
    std::advance(to, (unsigned) reserved3.size());

    const auto arrIsvProdId = toArray(swapBytes(isvProdID));
    std::copy(arrIsvProdId.begin(), arrIsvProdId.end(), to);
    std::advance(to, arrIsvProdId.size());

    const auto arrIsvSvn = toArray(swapBytes(isvSvn));
    std::copy(arrIsvSvn.begin(), arrIsvSvn.end(), to);
    std::advance(to, arrIsvSvn.size());

    std::copy(reserved4.begin(), reserved4.end(), to);
    std::advance(to, (unsigned) reserved4.size());

    std::copy(reportData.begin(), reportData.end(), to);
    std::advance(to, (unsigned) reportData.size());

    return ret;
}

bool Quote::Ecdsa256BitSignature::insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end)
{
    return copyAndAdvance(signature, from, end);
}

bool Quote::Ecdsa256BitPubkey::insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end)
{
    return copyAndAdvance(pubKey, from, end);
}

bool Quote::QeAuthData::insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end)
{
    const size_t amount = static_cast<size_t>(std::distance(from, end));
    if(from > end || amount < QE_AUTH_DATA_SIZE_BYTE_LEN)
    {
        return false;
    }

    this->data.clear();
    if (!copyAndAdvance(parsedDataSize, from, end))
    {
        return false;
    }

    if(parsedDataSize != amount - QE_AUTH_DATA_SIZE_BYTE_LEN)
    {
        // invalid format
        // moving back pointer
        from = std::prev(from, sizeof(decltype(parsedDataSize)));
        return false;
    }

    if(parsedDataSize == 0)
    {
        // all good, parsed size is zero
        // data are cleared and from is moved
        return true;
    }

    data.reserve(parsedDataSize);
    std::copy_n(from, parsedDataSize, std::back_inserter(data));
    std::advance(from, parsedDataSize);
    return true;
}

bool Quote::QeCertData::insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end)
{
    const auto minLen = QE_CERT_DATA_SIZE_BYTE_LEN + QE_CERT_DATA_TYPE_BYTE_LEN;
    const size_t amount = static_cast<size_t>(std::distance(from, end));
    if(from > end || amount < minLen)
    {
        return false;
    }

    data.clear();
    if (!copyAndAdvance(type, from, end)) { return false; }
    if (!copyAndAdvance(parsedDataSize, from, end)) { return false; }
    if(parsedDataSize != amount - minLen)
    {
        // invalid format, moving back pointer
        from = std::prev(from, sizeof(decltype(type)) + sizeof(decltype(parsedDataSize)));
        return false;
    }

    if(parsedDataSize == 0)
    {
        // all good, parsed size is 0
        // data cleared and pointer moved
        return true;
    }

    data.reserve(parsedDataSize);
    std::copy_n(from, parsedDataSize, std::back_inserter(data));
    std::advance(from, parsedDataSize);
    return true;
}

bool Quote::Ecdsa256BitQuoteAuthData::insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end)
{
    if (!copyAndAdvance(ecdsa256BitSignature, from, ECDSA_SIGNATURE_BYTE_LEN, end)) { return false; }
    if (!copyAndAdvance(ecdsaAttestationKey, from, ECDSA_PUBKEY_BYTE_LEN, end)) { return false; }
    if (!copyAndAdvance(qeReport, from, ENCLAVE_REPORT_BYTE_LEN, end)) { return false; }
    if (!copyAndAdvance(qeReportSignature, from, ECDSA_SIGNATURE_BYTE_LEN, end)) { return false; }

    uint16_t authSize = 0;
    if (!copyAndAdvance(authSize, from, end))
    {
        return false;
    }
    from = std::prev(from, sizeof(uint16_t));
    if (!copyAndAdvance(qeAuthData, from, authSize + sizeof(uint16_t), end))
    {
        return false;
    }

    uint32_t qeCertSize = 0;
    const auto available = std::distance(from, end);
    if (available < 0 || (unsigned) available < sizeof(uint16_t))
    {
        return false;
    }
    std::advance(from, sizeof(uint16_t)); // skip type
    if (!copyAndAdvance(qeCertSize, from, end))
    {
        return false;
    }
    from = std::prev(from, sizeof(uint32_t) + sizeof(uint16_t)); // go back to beg of struct data
    if (!copyAndAdvance(qeCertData, from, qeCertSize + sizeof(uint16_t) + sizeof(uint32_t), end))
    {
        return false;
    }
    return true;
}

std::vector<uint8_t> Quote::getDataToSignatureVerification(const std::vector<uint8_t>& rawQuote,
                                                           const std::vector<uint8_t>::difference_type sizeToCopy) const
{
    // private method, we call it at the end of parsing, so
    // here we assume format is valid
    const std::vector<uint8_t> ret(rawQuote.begin(), std::next(rawQuote.begin(), sizeToCopy));
    return ret;
}

}}} //namespace intel { namespace sgx { namespace dcap {
