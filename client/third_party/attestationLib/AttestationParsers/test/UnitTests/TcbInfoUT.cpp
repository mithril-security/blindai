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

#include "TcbInfoGenerator.h"
#include "SgxEcdsaAttestation/AttestationParsers.h"
#include "X509Constants.h"
#include <Utils/TimeUtils.h>

#include <gtest/gtest.h>
#include <gmock/gmock.h>

using namespace testing;
using namespace intel::sgx::dcap;

struct TcbInfoUT : public Test
{
};

TEST_F(TcbInfoUT, shouldFailWhenInitializedWithEmptyString)
{
    EXPECT_THROW(parser::json::TcbInfo::parse(""), parser::FormatException);
}

TEST_F(TcbInfoUT, shouldFailWHenInitializedWithInvalidJSON)
{
    EXPECT_THROW(parser::json::TcbInfo::parse("Plain string."), parser::FormatException);
}

TEST_F(TcbInfoUT, shouldFailWhenVersionIsMissing)
{
    const std::string tcbInfoWithoutVersion = R"json({
        "tcbInfo": {
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoWithoutVersion);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoUT, shouldFailWhenVersionIsNotAnInteger)
{
    const std::string tcbInfoInvalidVersion = R"json({
        "tcbInfo": {
            "version": "asd",
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoInvalidVersion);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoUT, shouldFailWhenVersionIsNotSupported)
{
    const std::string invalidTcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 4,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(invalidTcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}
