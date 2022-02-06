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

#include "TcbInfoJsonGenerator.h"

#include <iomanip>
#include <sstream>

static std::string getTcb(std::array<int, 16> tcb)
{
    std::string sgxtcbcompXXsvn;
    std::string sgxtcbcomp = R"("sgxtcbcomp)";
    std::string svn = R"(svn")";
    std::stringstream number;
    std::string result;

    for(unsigned int i = 0; i < tcb.size(); i++)
    {
        number << std::setw(2) << std::setfill('0') << i + 1;
        sgxtcbcompXXsvn = sgxtcbcomp + number.str() + svn;
        result += sgxtcbcompXXsvn + ":" + std::to_string(tcb[i]) + R"(,)";
        number.str("");
    }

    return result;
}

static std::string getTcbLevelsV1(std::array<int, 16> tcb, int pcesvn, std::string status)
{
    std::string result;
    result = R"([{"tcb":{)" + getTcb(tcb) + R"("pcesvn":)" + std::to_string(pcesvn);
    result += R"(},"status":")" + status + R"("}]})";

    return result;
}

static std::string getTcbLevels(std::array<int, 16> tcb, int pcesvn, std::string status, std::string tcbDate)
{
    std::string result;
    result = R"([{"tcb":{)" + getTcb(tcb) + R"("pcesvn":)" + std::to_string(pcesvn);
    result += R"(},"tcbDate":")" + tcbDate;
    result += R"(","tcbStatus":")" + status + R"("}]})";

    return result;
}

std::string tcbInfoJsonV1Body(int version, std::string issueDate, std::string nextUpdate, std::string fmspc,
                              std::string pceId, std::array<int, 16> tcb, int pcesvn, std::string status)
{
    std::string result;
    result = R"({"version":)" + std::to_string(version) + + R"(,"issueDate":")" + issueDate;
    result +=  R"(","nextUpdate":")" + nextUpdate + R"(","fmspc":")" + fmspc + R"(","pceId":")" + pceId;
    result += R"(","tcbLevels":)" + getTcbLevelsV1(tcb, pcesvn, status);

    return result;
}

std::string tcbInfoJsonV2Body(int version, std::string issueDate, std::string nextUpdate, std::string fmspc,
                              std::string pceId, std::array<int, 16> tcb, int pcesvn, std::string tcbStatus,
                              int tcbType, int tcbEvaluationDataNumber, std::string tcbDate)
{
    std::string result;
    result = R"({"version":)" + std::to_string(version) + + R"(,"issueDate":")" + issueDate;
    result += R"(","nextUpdate":")" + nextUpdate + R"(","fmspc":")" + fmspc + R"(","pceId":")" + pceId;
    result += R"(","tcbType":)" + std::to_string(tcbType);
    result += R"(,"tcbEvaluationDataNumber":)" + std::to_string(tcbEvaluationDataNumber);
    result += R"(,"tcbLevels":)" + getTcbLevels(tcb, pcesvn, tcbStatus, tcbDate);

    return result;
}

std::string tcbInfoJsonGenerator(int version, std::string issueDate, std::string nextUpdate, std::string fmspc,
                                 std::string pceId, std::array<int, 16> tcb, int pcesvn, std::string status,
                                 std::string signature)
{
    std::string result;
    result = R"({"tcbInfo":{"version":)" + std::to_string(version) + + R"(,"issueDate":")" + issueDate;
    result += R"(","nextUpdate":")" + nextUpdate + R"(","fmspc":")" + fmspc + R"(","pceId":")" + pceId;
    result += R"(","tcbLevels":)" + getTcbLevelsV1(tcb, pcesvn, status) + R"(,"signature":")" + signature + R"("})";

    return result;
}

std::string tcbInfoJsonGenerator(std::string tcbInfoBody, std::string signature)
{
    std::string result;
    result = R"({"tcbInfo":)" + tcbInfoBody + R"(,"signature":")" + signature + R"("})";

    return result;
}

std::array<int, 16> getRandomTcb()
{
    std::array<int, 16> tcb;
    for(unsigned long i = 0; i < 16; i ++)
    {
        tcb[i] = (rand() % 255) + 1;
    }

    return tcb;
}