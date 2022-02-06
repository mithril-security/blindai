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

#include "EnclaveIdentityV1.h"
#include "CertVerification/X509Constants.h"
#include "Utils/TimeUtils.h"

#include <rapidjson/writer.h>
#include <rapidjson/stringbuffer.h>

#include <tuple>

namespace intel { namespace sgx { namespace dcap {

    EnclaveIdentityV1::EnclaveIdentityV1(const ::rapidjson::Value &p_body) : isvSvn(0)
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
           || !parseIsvsvn(p_body))
        {
            status = STATUS_SGX_ENCLAVE_IDENTITY_INVALID;
            return;
        }

        rapidjson::StringBuffer buffer;
        rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);
        p_body.Accept(writer);

        body = std::vector<uint8_t>{buffer.GetString(), &buffer.GetString()[buffer.GetSize()]};
        status = STATUS_OK;
    }

    unsigned int EnclaveIdentityV1::getIsvSvn() const
    {
        return isvSvn;
    }

    bool EnclaveIdentityV1::parseIsvsvn(const rapidjson::Value &input)
    {
        return parseUintProperty(input, "isvsvn", isvSvn);
    }

    TcbStatus EnclaveIdentityV1::getTcbStatus(unsigned int p_isvSvn) const
    {
        if (this->isvSvn <= p_isvSvn)
        {
            return TcbStatus::UpToDate;
        }
        return TcbStatus::OutOfDate;
    }

}}}