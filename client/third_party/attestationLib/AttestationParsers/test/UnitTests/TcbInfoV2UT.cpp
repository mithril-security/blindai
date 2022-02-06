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

struct TcbInfoV2UT : public Test
{
};

TEST_F(TcbInfoV2UT, shouldSuccessfullyParseTcbWhenAllRequiredDataProvided)
{
    auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV2Template, TcbInfoGenerator::generateTcbLevelV2());

    const auto tcbInfo = parser::json::TcbInfo::parse(tcbInfoJson);

    EXPECT_EQ(tcbInfo.getPceId(), DEFAULT_PCEID);
    EXPECT_EQ(tcbInfo.getFmspc(), DEFAULT_FMSPC);
    EXPECT_EQ(tcbInfo.getSignature(), DEFAULT_SIGNATURE);
    EXPECT_EQ(tcbInfo.getTcbType(), DEFAULT_TCB_TYPE);
    EXPECT_EQ(tcbInfo.getTcbEvaluationDataNumber(), DEFAULT_TCB_EVALUATION_DATA_NUMBER);
    EXPECT_EQ(tcbInfo.getIssueDate(), getEpochTimeFromString(DEFAULT_ISSUE_DATE));
    EXPECT_EQ(tcbInfo.getNextUpdate(), getEpochTimeFromString(DEFAULT_NEXT_UPDATE));
    EXPECT_EQ(tcbInfo.getVersion(), 2);
    EXPECT_EQ(1, tcbInfo.getTcbLevels().size());
    auto iterator = tcbInfo.getTcbLevels().begin();
    EXPECT_NE(iterator, tcbInfo.getTcbLevels().end());
    for (unsigned int i=0; i<constants::CPUSVN_BYTE_LEN && iterator != tcbInfo.getTcbLevels().end(); i++)
    {
        EXPECT_EQ(iterator->getSgxTcbComponentSvn(i), DEFAULT_CPUSVN[i]);
    }
    EXPECT_EQ(iterator->getTcbDate(), getEpochTimeFromString(DEFAULT_TCB_DATE));
    EXPECT_EQ(iterator->getPceSvn(), DEFAULT_PCESVN);
    EXPECT_EQ(iterator->getStatus(), "UpToDate");
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbTypeNotExist)
{
    const std::string tcbInfoWithOutTcbType = R"json({
        "tcbInfo": {
            "version": 2,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbEvaluationDataNumber": 1,
            "tcbLevels": [%s]
        },
        %s})json";

    auto expResult = "TCB Info JSON should has [tcbType] field";
    auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoWithOutTcbType, TcbInfoGenerator::generateTcbLevelV2());

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because tcbType is not present";
    }
    catch(const parser::FormatException &err)
    {
        EXPECT_EQ(std::string(err.what()), expResult);
    }
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbTypeInvalid)
{
    const std::string tcbInfoWithOutTcbType = R"json({
        "tcbInfo": {
            "version": 2,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbType" : "1",
            "tcbEvaluationDataNumber": 1,
            "tcbLevels": [%s]
        },
        %s})json";

    auto expResult = "Could not parse [tcbType] field of TCB Info JSON to number";
    auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoWithOutTcbType, TcbInfoGenerator::generateTcbLevelV2());

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because tcbType is invalid";
    }
    catch(const parser::InvalidExtensionException &err)
    {
        EXPECT_EQ(std::string(err.what()), expResult);
    }
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbEvaluationDataNumberNotExist)
{
    const std::string tcbInfoTcbEvaluationData = R"json({
        "tcbInfo": {
            "version": 2,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbType" : 1,
            "tcbLevels": [%s]
        },
        %s})json";

    auto expResult = "TCB Info JSON should has [tcbEvaluationDataNumber] field";
    auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTcbEvaluationData, TcbInfoGenerator::generateTcbLevelV2());

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because tcbEvaluationDataNumber is not present";
    }
    catch(const parser::FormatException &err)
    {
        EXPECT_EQ(std::string(err.what()), expResult);
    }
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbEvaluationDataNumberInvalid)
{
    const std::string tcbInfoTcbEvaluationData = R"json({
        "tcbInfo": {
            "version": 2,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbType" : 1,
            "tcbEvaluationDataNumber": "1",
            "tcbLevels": [%s]
        },
        %s})json";

    auto expResult = "Could not parse [tcbEvaluationDataNumber] field of TCB Info JSON to number";
    auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTcbEvaluationData, TcbInfoGenerator::generateTcbLevelV2());

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because tcbEvaluationDataNumber is invalid";
    }
    catch(const parser::InvalidExtensionException &err)
    {
        EXPECT_EQ(std::string(err.what()), expResult);
    }
}

TEST_F(TcbInfoV2UT, shouldFailWhenGettingSvnComponentOutOfRange)
{
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo();

    const auto tcbInfo = parser::json::TcbInfo::parse(tcbInfoJson);
    auto iterator = tcbInfo.getTcbLevels().begin();
    EXPECT_NE(iterator, tcbInfo.getTcbLevels().end());
    EXPECT_THROW(iterator->getSgxTcbComponentSvn(constants::CPUSVN_BYTE_LEN + 1), parser::FormatException);
    EXPECT_THROW(iterator->getSgxTcbComponentSvn((unsigned int) -1), parser::FormatException);
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbInfoFieldIsMissing)
{
    const std::string json = R"json({"signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557"})json";

    EXPECT_THROW(parser::json::TcbInfo::parse(json), parser::FormatException);
}

TEST_F(TcbInfoV2UT, shouldFailWhenJSONRootIsNotAnObject)
{
    const std::string tcbInfoTemplate = R"json([{
        "tcbInfo": {},
        "signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557"}])json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV2UT, shouldFailWhenTCBInfoIsNotAnObject)
{
    const std::string json = R"json({"tcbInfo": "text", "signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557"})json";

    EXPECT_THROW(parser::json::TcbInfo::parse(json), parser::FormatException);
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbLevelsArrayElementIsMissingStatusField)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV2(validTcbLevelV2Template, validSgxTcb, R"json("missing": "tcbStatus")json");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV2Template, tcbLevels);
    auto expErrMsg = "TCB level JSON should has [tcbStatus] field";

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because tcbStatus is not present";
    }
    catch(const parser::FormatException &err)
    {
        EXPECT_EQ(std::string(err.what()), expErrMsg);
    }
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbLevelsArrayElementIsMissingTcbDateField)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV2(
            validTcbLevelV2Template, validSgxTcb, R"("tcbStatus": "UpToDate")", R"("missing": "tcbDate")");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV2Template, tcbLevels);
    auto expErrMsg = "TCB level JSON should has [tcbDate] field";

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because " << expErrMsg;
    }
    catch(const parser::FormatException &err)
    {
        EXPECT_EQ(std::string(err.what()), expErrMsg);
    }
}

TEST_F(TcbInfoV2UT, shouldSuccessWhenTcbLevelsAdvisoryIDsFieldIsPresent)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV2(
            validTcbLevelV2Template, validSgxTcb, R"("tcbStatus": "UpToDate")", R"("tcbDate": "2019-05-23T10:36:02Z")", R"("advisoryIDs": ["adv"])");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV2Template, tcbLevels);

    auto tcbInfo = parser::json::TcbInfo::parse(tcbInfoJson);

    EXPECT_EQ(tcbInfo.getPceId(), DEFAULT_PCEID);
    EXPECT_EQ(tcbInfo.getFmspc(), DEFAULT_FMSPC);
    EXPECT_EQ(tcbInfo.getSignature(), DEFAULT_SIGNATURE);
    EXPECT_EQ(tcbInfo.getTcbType(), DEFAULT_TCB_TYPE);
    EXPECT_EQ(tcbInfo.getTcbEvaluationDataNumber(), DEFAULT_TCB_EVALUATION_DATA_NUMBER);
    EXPECT_EQ(tcbInfo.getIssueDate(), getEpochTimeFromString(DEFAULT_ISSUE_DATE));
    EXPECT_EQ(tcbInfo.getNextUpdate(), getEpochTimeFromString(DEFAULT_NEXT_UPDATE));
    EXPECT_EQ(tcbInfo.getVersion(), 2);
    EXPECT_EQ(1, tcbInfo.getTcbLevels().size());
    auto iterator = tcbInfo.getTcbLevels().begin();
    EXPECT_NE(iterator, tcbInfo.getTcbLevels().end());
    for (unsigned int i=0; i<constants::CPUSVN_BYTE_LEN && iterator != tcbInfo.getTcbLevels().end(); i++)
    {
        EXPECT_EQ(iterator->getSgxTcbComponentSvn(i), DEFAULT_CPUSVN[i]);
    }
    EXPECT_EQ(iterator->getTcbDate(), getEpochTimeFromString(DEFAULT_TCB_DATE));
    EXPECT_EQ(iterator->getPceSvn(), DEFAULT_PCESVN);
    EXPECT_EQ(iterator->getStatus(), "UpToDate");
    EXPECT_EQ(iterator->getAdvisoryIDs().size(), 1);
    EXPECT_EQ(iterator->getAdvisoryIDs()[0], "adv");
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbLevelsArrayElementIsMissingTcbIDsField)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV2(
            validTcbLevelV2Template, R"("missing": "tcb")");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV2Template, tcbLevels);
    auto expErrMsg = "TCB level JSON should has [tcb] field";

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because advisoryIDs is not present";
    }
    catch(const parser::FormatException &err)
    {
        EXPECT_EQ(std::string(err.what()), expErrMsg);
    }
}

TEST_F(TcbInfoV2UT, shouldFailWhenAdvisoryIDsIsNotArray)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV2(
            validTcbLevelV2Template, validSgxTcb, R"("tcbStatus": "UpToDate")", R"("tcbDate": "2019-05-23T10:36:02Z")", R"("advisoryIDs": "advisoryIDs")");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV2Template, tcbLevels);
    auto expErrMsg = "Could not parse [advisoryIDs] field of TCB info JSON to an array.";

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because advisoryIDs is not array";
    }
    catch(const parser::InvalidExtensionException &err)
    {
        EXPECT_EQ(std::string(err.what()), expErrMsg);
    }
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbDateHasWrongFormat)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV2(
            validTcbLevelV2Template, validSgxTcb, R"("tcbStatus": "UpToDate")", R"("tcbDate": "2019-05-23T10:3")");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV2Template, tcbLevels);
    auto expErrMsg = "Could not parse [tcbDate] field of TCB info JSON to date. [tcbDate] should be ISO formatted date";

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because tcbDate has wrong format";
    }
    catch(const parser::InvalidExtensionException &err)
    {
        EXPECT_EQ(std::string(err.what()), expErrMsg);
    }
}

TEST_F(TcbInfoV2UT, shouldFailWhenTcbDateIsNotString)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV2(
            validTcbLevelV2Template, validSgxTcb, R"("tcbStatus": "UpToDate")", R"("tcbDate": 2019)");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV2Template, tcbLevels);
    auto expErrMsg = "Could not parse [tcbDate] field of TCB info JSON to date. [tcbDate] should be ISO formatted date";

    try
    {
        parser::json::TcbInfo::parse(tcbInfoJson);
        FAIL() << "Should throw, because tcbDate is not string";
    }
    catch(const parser::InvalidExtensionException &err)
    {
        EXPECT_EQ(std::string(err.what()), expErrMsg);
    }
}
