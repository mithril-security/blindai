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

#ifndef INTEL_SGX_QVL_QUOTE_H_
#define INTEL_SGX_QVL_QUOTE_H_

#include <cstdint>
#include <array>
#include <vector>
#include "QuoteConstants.h"

namespace intel { namespace sgx { namespace dcap {


class Quote
{
public:

    struct Header
    {
        uint16_t version;
        uint16_t attestationKeyType;
        uint16_t teeType;
        uint16_t reserved;
        uint16_t qeSvn;
        uint16_t pceSvn;
        std::array<uint8_t, 16> qeVendorId;
        std::array<uint8_t, 20> userData;

        bool insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end);
    };

    struct EnclaveReport
    {
        std::array<uint8_t, 16> cpuSvn;
        uint32_t miscSelect;
        std::array<uint8_t, 28> reserved1;
        std::array<uint8_t, 16> attributes;
        std::array<uint8_t, 32> mrEnclave;
        std::array<uint8_t, 32> reserved2;
        std::array<uint8_t, 32> mrSigner;
        std::array<uint8_t, 96> reserved3;
        uint16_t isvProdID;
        uint16_t isvSvn;
        std::array<uint8_t, 60> reserved4;
        std::array<uint8_t, 64> reportData;

        bool insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end);
        std::array<uint8_t, constants::ENCLAVE_REPORT_BYTE_LEN> rawBlob() const;
    };

    struct Ecdsa256BitSignature
    {
        std::array<uint8_t, dcap::constants::ECDSA_P256_SIGNATURE_BYTE_LEN> signature;

        bool insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end);
    };

    struct Ecdsa256BitPubkey
    {
        std::array<uint8_t, 64> pubKey;

        bool insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end);
    };

    struct QeAuthData
    {
        uint16_t parsedDataSize;
        std::vector<uint8_t> data;
        bool insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end);
    };

    struct QeCertData
    {
        uint16_t type;
        uint32_t parsedDataSize;
        std::vector<uint8_t> data;
        bool insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end);
    };

    struct Ecdsa256BitQuoteAuthData
    {
        Ecdsa256BitSignature ecdsa256BitSignature;
        Ecdsa256BitPubkey ecdsaAttestationKey;
        
        EnclaveReport qeReport;
        Ecdsa256BitSignature qeReportSignature;
        QeAuthData qeAuthData;
        QeCertData qeCertData;

        bool insert(std::vector<uint8_t>::const_iterator& from, const std::vector<uint8_t>::const_iterator& end);
    }; 

    bool parse(const std::vector<uint8_t>& rawQuote);
    bool parseEnclaveReport(const std::vector<uint8_t>& rawQuote);
    bool validate() const;

    const Header& getHeader() const;
    const EnclaveReport& getEnclaveReport() const;
    uint32_t getAuthDataSize() const;
    const Ecdsa256BitQuoteAuthData& getQuoteAuthData() const;
    const std::vector<uint8_t>& getSignedData() const;

private:
    std::vector<uint8_t> getDataToSignatureVerification(const std::vector<uint8_t>& rawQuote,
                                                        const std::vector<uint8_t>::difference_type ) const;

protected:
    std::vector<uint8_t> signedData{};
    Header header{};
    EnclaveReport bodyEnclaveReport{};
    uint32_t authDataSize{0};
    Ecdsa256BitQuoteAuthData authData{};
};

}}} // namespace intel { namespace sgx { namespace dcap { namespace test {


#endif //SGXECDSAATTESTATION_QUOTEGENERATOR_H
