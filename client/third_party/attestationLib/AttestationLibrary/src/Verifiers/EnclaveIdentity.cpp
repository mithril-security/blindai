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

#include "EnclaveIdentity.h"
#include "CertVerification/X509Constants.h"
#include "Utils/TimeUtils.h"
#include "QuoteVerification/QuoteConstants.h"

#include <rapidjson/writer.h>
#include <rapidjson/stringbuffer.h>

#include <tuple>

namespace intel { namespace sgx { namespace dcap {

    void EnclaveIdentity::setSignature(std::vector<uint8_t> &p_signature)
    {
        signature = p_signature;
    }

    std::vector<uint8_t> EnclaveIdentity::getBody() const
    {
        return body;
    }

    std::vector<uint8_t> EnclaveIdentity::getSignature() const
    {
        return signature;
    }

    const std::vector<uint8_t>& EnclaveIdentity::getMiscselect() const
    {
        return miscselect;
    }

    const std::vector<uint8_t>& EnclaveIdentity::getMiscselectMask() const
    {
        return miscselectMask;
    }

    const std::vector<uint8_t>& EnclaveIdentity::getAttributes() const
    {
        return attributes;
    }

    const std::vector<uint8_t>& EnclaveIdentity::getAttributesMask() const
    {
        return attributesMask;
    }

    const std::vector<uint8_t>& EnclaveIdentity::getMrsigner() const
    {
        return mrsigner;
    }

    unsigned int EnclaveIdentity::getIsvProdId() const
    {
        return isvProdId;
    }

    int EnclaveIdentity::getVersion() const
    {
        return version;
    }

    time_t EnclaveIdentity::getIssueDate() const
    {
        return issueDate;
    }

    time_t EnclaveIdentity::getNextUpdate() const
    {
        return nextUpdate;
    }

    EnclaveID EnclaveIdentity::getID() const
    {
        return id;
    }

    bool EnclaveIdentity::parseVersion(const rapidjson::Value &input)
    {
        auto l_status = JsonParser::ParseStatus::Missing;
        std::tie(version, l_status) = jsonParser.getIntFieldOf(input, "version");
        return l_status == JsonParser::OK;
    }

    bool EnclaveIdentity::parseIssueDate(const rapidjson::Value &input)
    {
        auto l_status = JsonParser::ParseStatus::Missing;
        struct tm issueDateTm{};
        std::tie(issueDateTm, l_status) = jsonParser.getDateFieldOf(input, "issueDate");
        issueDate = dcap::mktime(&issueDateTm);
        return l_status == JsonParser::OK;
    }

    bool EnclaveIdentity::parseNextUpdate(const rapidjson::Value &input)
    {
        auto l_status = JsonParser::ParseStatus::Missing;
        struct tm nextUpdateTm{};
        std::tie(nextUpdateTm, l_status) = jsonParser.getDateFieldOf(input, "nextUpdate");
        nextUpdate = dcap::mktime(&nextUpdateTm);
        return l_status == JsonParser::OK;
    }

    bool EnclaveIdentity::parseMiscselect(const rapidjson::Value &input)
    {
        return parseHexstringProperty(input, "miscselect", constants::MISCSELECT_BYTE_LEN * 2, miscselect);
    }

    bool EnclaveIdentity::parseMiscselectMask(const rapidjson::Value &input)
    {
        return parseHexstringProperty(input, "miscselectMask", constants::MISCSELECT_BYTE_LEN * 2, miscselectMask);
    }

    bool EnclaveIdentity::parseAttributes(const rapidjson::Value &input)
    {
        return parseHexstringProperty(input, "attributes", constants::ATTRIBUTES_BYTE_LEN * 2, attributes);
    }

    bool EnclaveIdentity::parseAttributesMask(const rapidjson::Value &input)
    {
        return parseHexstringProperty(input, "attributesMask", constants::ATTRIBUTES_BYTE_LEN * 2, attributesMask);
    }

    bool EnclaveIdentity::parseMrsigner(const rapidjson::Value &input)
    {
        return parseHexstringProperty(input, "mrsigner", constants::MRSIGNER_BYTE_LEN * 2, mrsigner);
    }

    bool EnclaveIdentity::parseHexstringProperty(const rapidjson::Value &object, const std::string &propertyName, const size_t length, std::vector<uint8_t> &saveAs)
    {
        auto parseSuccessful = JsonParser::ParseStatus::Missing;
        std::tie(saveAs, parseSuccessful) = jsonParser.getHexstringFieldOf(object, propertyName, length);
        return parseSuccessful == JsonParser::OK;
    }

    bool EnclaveIdentity::parseIsvprodid(const rapidjson::Value &input)
    {
        return parseUintProperty(input, "isvprodid", isvProdId);
    }

    bool EnclaveIdentity::parseUintProperty(const rapidjson::Value &object, const std::string &propertyName, unsigned int &saveAs)
    {
        auto parseSuccessful = JsonParser::ParseStatus::Missing;
        std::tie(saveAs, parseSuccessful) = jsonParser.getUintFieldOf(object, propertyName);
        return parseSuccessful == JsonParser::OK;
    }

    bool EnclaveIdentity::checkDateCorrectness(const time_t expirationDate) const
    {
        if (expirationDate > nextUpdate)
        {
            return false;
        }

        if (expirationDate <= issueDate)
        {
            return false;
        }

        return true;
    }

    Status EnclaveIdentity::getStatus() const
    {
        return status;
    }

}}}