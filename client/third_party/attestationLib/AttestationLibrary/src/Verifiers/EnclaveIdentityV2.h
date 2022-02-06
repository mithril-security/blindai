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

#ifndef SGXECDSAATTESTATION_ENCLAVEIDENTITYV2_H
#define SGXECDSAATTESTATION_ENCLAVEIDENTITYV2_H


#include "EnclaveIdentity.h"
#include "TcbStatus.h"

namespace intel { namespace sgx { namespace dcap {

    class TCBLevel
    {
    public:
        TCBLevel(const unsigned int p_isvsvn,
                 const struct tm &p_tcbDate,
                 const TcbStatus p_tcbStatus):
                    isvsvn(p_isvsvn), tcbDate(p_tcbDate), tcbStatus(p_tcbStatus) {};
        unsigned int getIsvsvn() const;
        struct tm getTcbDate() const;
        TcbStatus getTcbStatus() const;
    protected:
        unsigned int isvsvn;
        struct tm tcbDate;
        TcbStatus tcbStatus;
    };

    class EnclaveIdentityV2 : virtual public EnclaveIdentity
    {
    public:
        explicit EnclaveIdentityV2(const ::rapidjson::Value &p_body);

        virtual TcbStatus getTcbStatus(unsigned int isvSvn) const;
        virtual unsigned int getTcbEvaluationDataNumber() const;
        virtual const std::vector<TCBLevel>& getTcbLevels() const;

    protected:
        EnclaveIdentityV2() = default;

        bool parseID(const rapidjson::Value &input);
        bool parseTcbEvaluationDataNumber(const rapidjson::Value &input);
        bool parseTcbLevels(const rapidjson::Value &input);

        unsigned int tcbEvaluationDataNumber;
        std::vector<TCBLevel> tcbLevels;
    };
}}}


#endif //SGXECDSAATTESTATION_ENCLAVEIDENTITYV2_H
