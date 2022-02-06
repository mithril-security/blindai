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

#include <rapidjson/writer.h>
#include "EnclaveIdentityV2.h"

#include <tuple>

namespace intel { namespace sgx { namespace dcap {

    EnclaveIdentityV2::EnclaveIdentityV2(const ::rapidjson::Value &p_body)
        : tcbEvaluationDataNumber(0)
    {
        if(!p_body.IsObject())
        {
            status = STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_FORMAT;
            return;
        }

        /// 4.1.2.9.3
        if(!parseVersion(p_body)
           || !parseIssueDate(p_body) || !parseNextUpdate(p_body)
           || !parseMiscselect(p_body) || !parseMiscselectMask(p_body)
           || !parseAttributes(p_body) || !parseAttributesMask(p_body)
           || !parseMrsigner(p_body) || !parseIsvprodid(p_body)
           || !parseID(p_body) || !parseTcbEvaluationDataNumber(p_body)
           || !parseTcbLevels(p_body))
        {
            status = STATUS_SGX_ENCLAVE_IDENTITY_INVALID;
            return;
        }

        rapidjson::StringBuffer buffer;
        rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);
        p_body.Accept(writer);

        this->body = std::vector<uint8_t>{buffer.GetString(), &buffer.GetString()[buffer.GetSize()]};
        status = STATUS_OK;
    }

    bool EnclaveIdentityV2::parseID(const rapidjson::Value &input)
    {
        auto parseSuccessful = JsonParser::ParseStatus::Missing;
        std::string idString;
        std::tie(idString, parseSuccessful) = jsonParser.getStringFieldOf(input, "id");
        if (idString == "QE")
        {
            id = QE;
        }
        else if (idString == "QVE")
        {
            id = QVE;
        }
        else
        {
            return false;
        }
        return parseSuccessful == JsonParser::OK;
    }

    bool EnclaveIdentityV2::parseTcbEvaluationDataNumber(const rapidjson::Value &input)
    {
        return parseUintProperty(input, "tcbEvaluationDataNumber", tcbEvaluationDataNumber);
    }

    bool EnclaveIdentityV2::parseTcbLevels(const rapidjson::Value &input)
    {
        if (!input.HasMember("tcbLevels"))
        {
            return false;
        }

        const ::rapidjson::Value& l_tcbLevels = input["tcbLevels"];

        if (!l_tcbLevels.IsArray() || l_tcbLevels.Empty()) // must be a non empty array
        {
            return false;
        }

        auto l_status = JsonParser::ParseStatus::Missing;
        for (rapidjson::Value::ConstValueIterator itr = l_tcbLevels.Begin(); itr != l_tcbLevels.End(); itr++)
        {
            struct tm tcbDate{};
            std::string tcbStatus;
            unsigned int isvsvn = 0;

            std::tie(tcbDate, l_status) = jsonParser.getDateFieldOf(*itr, "tcbDate");
            if (l_status != JsonParser::OK)
            {
                return false;
            }
            std::tie(tcbStatus, l_status) = jsonParser.getStringFieldOf(*itr, "tcbStatus");
            if (l_status != JsonParser::OK)
            {
                return false;
            }

            if (!(*itr).HasMember("tcb"))
            {
                return false;
            }

            const ::rapidjson::Value& tcb = (*itr)["tcb"];

            if (!tcb.IsObject())
            {
                return false;
            }

            std::tie(isvsvn, l_status) = jsonParser.getUintFieldOf(tcb, "isvsvn");
            if (l_status != JsonParser::OK)
            {
                return false;
            }


            TcbStatus tcbStatusEnum;
            try
            {
                tcbStatusEnum = parseStringToTcbStatus(tcbStatus);
            }
            catch (const std::runtime_error &)
            {
                return false;
            }

            this->tcbLevels.emplace_back(isvsvn, tcbDate, tcbStatusEnum);
        }
        return true;
    }

    TcbStatus EnclaveIdentityV2::getTcbStatus(unsigned int p_isvSvn) const
    {
        for(const auto & tcbLevel : tcbLevels)
        {
            if (tcbLevel.getIsvsvn() <= p_isvSvn)
            {
                return tcbLevel.getTcbStatus();
            }
        }
        return TcbStatus::Revoked;
    }

    unsigned int EnclaveIdentityV2::getTcbEvaluationDataNumber() const
    {
        return tcbEvaluationDataNumber;
    }


    const std::vector<TCBLevel>& EnclaveIdentityV2::getTcbLevels() const
    {
        return tcbLevels;
    }

    unsigned int TCBLevel::getIsvsvn() const
    {
        return isvsvn;
    }

    struct tm TCBLevel::getTcbDate() const
    {
        return tcbDate;
    }

    TcbStatus TCBLevel::getTcbStatus() const
    {
        return tcbStatus;
    }
}}}
