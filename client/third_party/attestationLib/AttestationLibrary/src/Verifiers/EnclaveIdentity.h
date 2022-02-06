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

#ifndef SGXECDSAATTESTATION_ENCLAVEIDENTITY_H
#define SGXECDSAATTESTATION_ENCLAVEIDENTITY_H

#include "OpensslHelpers/Bytes.h"
#include "Utils/JsonParser.h"
#include "TcbStatus.h"

#include <SgxEcdsaAttestation/QuoteVerification.h>

#include <rapidjson/document.h>

#include <string>
#include <vector>
#include <cstdint>

namespace intel { namespace sgx { namespace dcap {

    enum EnclaveID
    {
        QE, QVE
    };

    class EnclaveIdentity
    {
    public:
        enum Version
        {
            V1 = 1,
            V2,
        };

        virtual ~EnclaveIdentity() = default;

        virtual void setSignature(std::vector<uint8_t> &p_signature);
        virtual std::vector<uint8_t> getBody() const;
        virtual std::vector<uint8_t> getSignature() const;

        virtual time_t getIssueDate() const;
        virtual time_t getNextUpdate() const;
        virtual const std::vector<uint8_t>& getMiscselect() const;
        virtual const std::vector<uint8_t>& getMiscselectMask() const;
        virtual const std::vector<uint8_t>& getAttributes() const;
        virtual const std::vector<uint8_t>& getAttributesMask() const;
        virtual const std::vector<uint8_t>& getMrsigner() const;
        virtual unsigned int getIsvProdId() const;
        virtual int getVersion() const;
        virtual TcbStatus getTcbStatus(unsigned int p_isvSvn) const = 0;
        virtual bool checkDateCorrectness(time_t expirationDate) const;
        virtual Status getStatus() const;
        virtual EnclaveID getID() const;

    protected:
        EnclaveIdentity() = default;

        std::vector<uint8_t> signature;
        std::vector<uint8_t> body;

        bool parseMiscselect(const rapidjson::Value &input);
        bool parseMiscselectMask(const rapidjson::Value &input);
        bool parseAttributes(const rapidjson::Value &input);
        bool parseAttributesMask(const rapidjson::Value &input);
        bool parseMrsigner(const rapidjson::Value &input);
        bool parseIsvprodid(const rapidjson::Value &input);
        bool parseVersion(const rapidjson::Value &input);
        bool parseIssueDate(const rapidjson::Value &input);
        bool parseNextUpdate(const rapidjson::Value &input);

        bool parseHexstringProperty(const rapidjson::Value &object, const std::string &propertyName, size_t length, std::vector<uint8_t> &saveAs);
        bool parseUintProperty(const rapidjson::Value &object, const std::string &propertyName, unsigned int &saveAs);

        JsonParser jsonParser;
        std::vector<uint8_t> miscselect;
        std::vector<uint8_t> miscselectMask;
        std::vector<uint8_t> attributes;
        std::vector<uint8_t> attributesMask;
        std::vector<uint8_t> mrsigner;
        time_t issueDate;
        time_t nextUpdate;
        unsigned int isvProdId;
        int version;
        EnclaveID id = EnclaveID::QE;

        Status status = STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_FORMAT;
    };
}}}

#endif //SGXECDSAATTESTATION_ENCLAVEIDENTITY_H
