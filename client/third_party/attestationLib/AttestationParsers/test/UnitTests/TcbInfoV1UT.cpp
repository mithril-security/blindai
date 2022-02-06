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

struct TcbInfoV1UT : public Test
{
};

TEST_F(TcbInfoV1UT, shouldSuccessfullyParseTcbWhenAllRequiredDataProvided)
{
    auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo();

    const auto tcbInfo = parser::json::TcbInfo::parse(tcbInfoJson);

    EXPECT_THROW(tcbInfo.getTcbType(), parser::FormatException);
    EXPECT_THROW(tcbInfo.getTcbEvaluationDataNumber(), parser::FormatException);
    EXPECT_EQ(tcbInfo.getPceId(), DEFAULT_PCEID);
    EXPECT_EQ(tcbInfo.getFmspc(), DEFAULT_FMSPC);
    EXPECT_EQ(tcbInfo.getSignature(), DEFAULT_SIGNATURE);
    EXPECT_EQ(tcbInfo.getInfoBody(), DEFAULT_INFO_BODY);
    EXPECT_EQ(tcbInfo.getIssueDate(), getEpochTimeFromString(DEFAULT_ISSUE_DATE));
    EXPECT_EQ(tcbInfo.getNextUpdate(), getEpochTimeFromString(DEFAULT_NEXT_UPDATE));
    EXPECT_EQ(tcbInfo.getVersion(), 1);
    EXPECT_EQ(1, tcbInfo.getTcbLevels().size());
    auto iterator = tcbInfo.getTcbLevels().begin();
    EXPECT_NE(iterator, tcbInfo.getTcbLevels().end());
    for (unsigned int i=0; i<constants::CPUSVN_BYTE_LEN && iterator != tcbInfo.getTcbLevels().end(); i++)
    {
        EXPECT_EQ(iterator->getSgxTcbComponentSvn(i), DEFAULT_CPUSVN[i]);
        EXPECT_EQ(iterator->getCpuSvn(), DEFAULT_CPUSVN);
    }
    EXPECT_EQ(iterator->getPceSvn(), DEFAULT_PCESVN);
    EXPECT_EQ(iterator->getStatus(), "UpToDate");
    EXPECT_EQ(iterator->getAdvisoryIDs().size(), 0);
}

TEST_F(TcbInfoV1UT, shouldSuccessfullyParseMultipleTcbLevels)
{
    std::vector<uint8_t> expectedCpusvn{55, 0, 0, 1, 10, 0, 0, 77, 200, 200, 250, 250, 55, 2, 2, 2};
    unsigned int expectedPcesvn = 66;
    std::vector<uint8_t> expectedRevokedCpusvn{44, 0, 0, 1, 10, 0, 0, 77, 200, 200, 250, 250, 55, 2, 2, 2};
    unsigned int expectedRevokedPcesvn = 65;
    const std::string upToDateTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 55,
        "sgxtcbcomp02svn": 0,
        "sgxtcbcomp03svn": 0,
        "sgxtcbcomp04svn": 1,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 77,
        "sgxtcbcomp09svn": 200,
        "sgxtcbcomp10svn": 200,
        "sgxtcbcomp11svn": 250,
        "sgxtcbcomp12svn": 250,
        "sgxtcbcomp13svn": 55,
        "sgxtcbcomp14svn": 2,
        "sgxtcbcomp15svn": 2,
        "sgxtcbcomp16svn": 2,
        "pcesvn": 66
    })json";
    const std::string revokedTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 44,
        "sgxtcbcomp02svn": 0,
        "sgxtcbcomp03svn": 0,
        "sgxtcbcomp04svn": 1,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 77,
        "sgxtcbcomp09svn": 200,
        "sgxtcbcomp10svn": 200,
        "sgxtcbcomp11svn": 250,
        "sgxtcbcomp12svn": 250,
        "sgxtcbcomp13svn": 55,
        "sgxtcbcomp14svn": 2,
        "sgxtcbcomp15svn": 2,
        "sgxtcbcomp16svn": 2,
        "pcesvn": 65
    })json";
    const std::string configurationNeededTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 48,
        "sgxtcbcomp02svn": 0,
        "sgxtcbcomp03svn": 0,
        "sgxtcbcomp04svn": 1,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 77,
        "sgxtcbcomp09svn": 200,
        "sgxtcbcomp10svn": 200,
        "sgxtcbcomp11svn": 250,
        "sgxtcbcomp12svn": 222,
        "sgxtcbcomp13svn": 55,
        "sgxtcbcomp14svn": 2,
        "sgxtcbcomp15svn": 2,
        "sgxtcbcomp16svn": 2,
        "pcesvn": 66
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, validSgxTcb, validOutOfDateStatus)
        + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, upToDateTcb, validUpToDateStatus)
        + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, revokedTcb, validRevokedStatus)
        + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, configurationNeededTcb, validConfigurationNeededStatus);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);

    const auto tcbInfo = parser::json::TcbInfo::parse(tcbInfoJson);
    EXPECT_EQ(4, tcbInfo.getTcbLevels().size());
    auto iterator = tcbInfo.getTcbLevels().begin();
    EXPECT_NE(iterator, tcbInfo.getTcbLevels().end());
    for (unsigned int i=0; i<constants::CPUSVN_BYTE_LEN && iterator != tcbInfo.getTcbLevels().end(); i++)
    {
        EXPECT_EQ(expectedCpusvn[i], iterator->getSgxTcbComponentSvn(i));
    }
    EXPECT_EQ(expectedPcesvn, iterator->getPceSvn());
    EXPECT_EQ("UpToDate", iterator->getStatus());
    std::advance(iterator, 2);
    for (unsigned int i=0; i<constants::CPUSVN_BYTE_LEN; i++)
    {
        EXPECT_EQ(expectedRevokedCpusvn[i], iterator->getSgxTcbComponentSvn(i));
    }
    EXPECT_EQ(expectedRevokedPcesvn, iterator->getPceSvn());
    EXPECT_EQ("Revoked", iterator->getStatus());
}

TEST_F(TcbInfoV1UT, shouldSuccessfullyParseMultipleRevokedTcbLevels)
{
    std::vector<uint8_t> expectedRevokedCpusvn{44, 0, 0, 1, 10, 0, 0, 77, 200, 222, 111, 121, 55, 2, 2, 2};
    uint16_t expectedRevokedPcesvn = 66;
    const std::string revokedTcbLatest = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 44,
        "sgxtcbcomp02svn": 0,
        "sgxtcbcomp03svn": 0,
        "sgxtcbcomp04svn": 1,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 77,
        "sgxtcbcomp09svn": 200,
        "sgxtcbcomp10svn": 222,
        "sgxtcbcomp11svn": 111,
        "sgxtcbcomp12svn": 121,
        "sgxtcbcomp13svn": 55,
        "sgxtcbcomp14svn": 2,
        "sgxtcbcomp15svn": 2,
        "sgxtcbcomp16svn": 2,
        "pcesvn": 66
    })json";
    const std::string otherRevokedTcb1 = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 44,
        "sgxtcbcomp02svn": 0,
        "sgxtcbcomp03svn": 0,
        "sgxtcbcomp04svn": 1,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 77,
        "sgxtcbcomp09svn": 200,
        "sgxtcbcomp10svn": 222,
        "sgxtcbcomp11svn": 111,
        "sgxtcbcomp12svn": 121,
        "sgxtcbcomp13svn": 55,
        "sgxtcbcomp14svn": 2,
        "sgxtcbcomp15svn": 2,
        "sgxtcbcomp16svn": 2,
        "pcesvn": 65
    })json";
    const std::string otherRevokedTcb2 = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 44,
        "sgxtcbcomp02svn": 0,
        "sgxtcbcomp03svn": 0,
        "sgxtcbcomp04svn": 0,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 77,
        "sgxtcbcomp09svn": 200,
        "sgxtcbcomp10svn": 222,
        "sgxtcbcomp11svn": 111,
        "sgxtcbcomp12svn": 121,
        "sgxtcbcomp13svn": 55,
        "sgxtcbcomp14svn": 2,
        "sgxtcbcomp15svn": 2,
        "sgxtcbcomp16svn": 2,
        "pcesvn": 66
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, validSgxTcb, validUpToDateStatus)
        + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, otherRevokedTcb1, validRevokedStatus)
        + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, revokedTcbLatest, validRevokedStatus)
        + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, otherRevokedTcb2, validRevokedStatus);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);

    const auto tcbInfo = parser::json::TcbInfo::parse(tcbInfoJson);
    EXPECT_EQ(4, tcbInfo.getTcbLevels().size());
    auto iterator = tcbInfo.getTcbLevels().begin();
    std::advance(iterator, 0);
    for (unsigned int i=0; i<constants::CPUSVN_BYTE_LEN; i++)
    {
        EXPECT_EQ(expectedRevokedCpusvn[i], iterator->getSgxTcbComponentSvn(i));
    }
    EXPECT_EQ(expectedRevokedPcesvn, iterator->getPceSvn());
    EXPECT_EQ("Revoked", iterator->getStatus());
}

TEST_F(TcbInfoV1UT, shouldSucceedWhenTcbLevelsContainsOnlyRevokedTcbs)
{
    std::vector<uint8_t> expectedRevokedCpusvn{55, 0, 0, 1, 10, 0, 0, 77, 200, 200, 250, 250, 55, 2, 2, 2};
    unsigned int expectedRevokedPcesvn = 66;
    const std::string revokedTcb1 = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 55,
        "sgxtcbcomp02svn": 0,
        "sgxtcbcomp03svn": 0,
        "sgxtcbcomp04svn": 1,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 77,
        "sgxtcbcomp09svn": 200,
        "sgxtcbcomp10svn": 200,
        "sgxtcbcomp11svn": 250,
        "sgxtcbcomp12svn": 250,
        "sgxtcbcomp13svn": 55,
        "sgxtcbcomp14svn": 2,
        "sgxtcbcomp15svn": 2,
        "sgxtcbcomp16svn": 2,
        "pcesvn": 66
    })json";
    const std::string revokedTcb2 = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 44,
        "sgxtcbcomp02svn": 0,
        "sgxtcbcomp03svn": 0,
        "sgxtcbcomp04svn": 1,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 77,
        "sgxtcbcomp09svn": 200,
        "sgxtcbcomp10svn": 200,
        "sgxtcbcomp11svn": 250,
        "sgxtcbcomp12svn": 250,
        "sgxtcbcomp13svn": 55,
        "sgxtcbcomp14svn": 2,
        "sgxtcbcomp15svn": 2,
        "sgxtcbcomp16svn": 2,
        "pcesvn": 65
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, revokedTcb1, validRevokedStatus)
        + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, revokedTcb2, validRevokedStatus);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);

    const auto tcbInfo = parser::json::TcbInfo::parse(tcbInfoJson);
    EXPECT_EQ(2, tcbInfo.getTcbLevels().size());
    auto iterator = tcbInfo.getTcbLevels().begin();
    EXPECT_NE(iterator, tcbInfo.getTcbLevels().end());
    std::advance(iterator, 0);
    for (unsigned int i=0; i<constants::CPUSVN_BYTE_LEN; i++)
    {
        EXPECT_EQ(expectedRevokedCpusvn[i], iterator->getSgxTcbComponentSvn(i));
    }
    EXPECT_EQ(expectedRevokedPcesvn, iterator->getPceSvn());
    EXPECT_EQ("Revoked", iterator->getStatus());
}

TEST_F(TcbInfoV1UT, shouldFailWhenGettingSvnComponentOutOfRange)
{
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo();

    const auto tcbInfo = parser::json::TcbInfo::parse(tcbInfoJson);
    auto iterator = tcbInfo.getTcbLevels().begin();
    EXPECT_NE(iterator, tcbInfo.getTcbLevels().end());
    EXPECT_THROW(iterator->getSgxTcbComponentSvn(constants::CPUSVN_BYTE_LEN + 1), parser::FormatException);
    EXPECT_THROW(iterator->getSgxTcbComponentSvn((unsigned int) -1), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsContainsNoTcbs)
{
    const std::string tcbLevels = "";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbInfoFieldIsMissing)
{
    const std::string json = R"json({"signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557"})json";

    EXPECT_THROW(parser::json::TcbInfo::parse(json), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenJSONRootIsNotAnObject)
{
    const std::string tcbInfoTemplate = R"json([{
        "tcbInfo": {},
        "signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557"}])json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTCBInfoIsNotAnObject)
{
    const std::string json = R"json({"tcbInfo": "text", "signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557"})json";

    EXPECT_THROW(parser::json::TcbInfo::parse(json), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenSignatureIsMissing)
{
    const std::string missingSignature = R"json("missing": "signature")json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, TcbInfoGenerator::generateTcbLevelV1(), missingSignature);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenSignatureIsNotAString)
{
    const std::string invalidSignature = R"json("signature": 555)json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, TcbInfoGenerator::generateTcbLevelV1(), invalidSignature);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenSignatureIsTooLong)
{
    const std::string invalidSignature = R"json("signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA35570")json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, TcbInfoGenerator::generateTcbLevelV1(), invalidSignature);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenSignatureIsTooShort)
{
    const std::string invalidSignature = R"json("signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA355")json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, TcbInfoGenerator::generateTcbLevelV1(), invalidSignature);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenIssueDateIsMissing)
{
    const std::string tcbInfoWithoutDate = R"json({
        "tcbInfo": {
            "version": 1,
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoWithoutDate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenIssueDateIsNotAString)
{
    const std::string tcbInfoInvalidDate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": true,
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoInvalidDate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenIssueDateIsNotInValidFormat)
{
    const std::string tcbInfoInvalidDate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "20171004T111045Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoInvalidDate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenIssueDateIsNotInUTC)
{
    const std::string tcbInfoInvalidDate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45+01",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoInvalidDate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenNextUpdateIsMissing)
{
    const std::string tcbInfoWithoutDate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoWithoutDate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenNextUpdateIsNotAString)
{
    const std::string tcbInfoInvalidDate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": true,
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoInvalidDate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenNextUpdateIsNotInValidFormat)
{
    const std::string tcbInfoInvalidDate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "20180621T123602Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoInvalidDate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenNextUpdateIsNotInUTC)
{
    const std::string tcbInfoInvalidDate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02+01",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoInvalidDate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenFmspcIsMissing)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenFmspcIsNotAString)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": 23,
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenFmspcIsTooLong)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0123456789ABC",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenFmspcIsTooShort)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0123456789A",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenFmspcIsNotAValidHexstring)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "01invalid9AB",
            "pceId": "0000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenPceIdIsMissing)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenPceIdIsNotAString)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": 23,
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenPceIdIsTooLong)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "00000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenPceIdIsTooShort)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "000",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenPceIdIsNotAValidHexstring)
{
    const std::string tcbInfoTemplate = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "xxxx",
            "tcbLevels": [%s]
        },
        %s})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(tcbInfoTemplate);

    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayIsMissing)
{
    const std::string json = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000"
        },
        "signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557"})json";
    EXPECT_THROW(parser::json::TcbInfo::parse(json), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsIsNotAnArray)
{
    const std::string json = R"json({
        "tcbInfo": {
            "version": 1,
            "issueDate": "2017-10-04T11:10:45Z",
            "nextUpdate": "2018-06-21T12:36:02Z",
            "fmspc": "0192837465AF",
            "pceId": "0000",
            "tcbLevels": 0
        },
        "signature": "ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557ABBA3557"})json";
    EXPECT_THROW(parser::json::TcbInfo::parse(json), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayIsEmpty)
{
    const std::string tcbLevels = R"json()json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementIsNotAnObject)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(R"json("tcblevelString")json");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementIsEmpty)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1("{}");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementHasIncorrectNumberOfFields)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + R"json(, {"status": "UpToDate"})json";
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementIsMissingTcbField)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, R"json("missing": "tcb")json");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementIsMissingStatusField)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, validSgxTcb, R"json("missing": "status")json");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementStatusIsNotAString)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, validSgxTcb, R"json("status": 78763124)json");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementTcbIsNotAnObject)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, R"json("tcb": "qwerty")json");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementStatusIsNotAValidValue)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, validSgxTcb, R"json("status": "unknown value")json");
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementTcbComponentsAreMissing)
{
    const std::string invalidTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 12,
        "sgxtcbcomp02svn": 34,
        "sgxtcbcomp03svn": 56,
        "sgxtcbcomp04svn": 78,
        "sgxtcbcomp08svn": 254,
        "sgxtcbcomp09svn": 9,
        "sgxtcbcomp10svn": 87,
        "sgxtcbcomp11svn": 65,
        "sgxtcbcomp12svn": 43,
        "sgxtcbcomp13svn": 21,
        "sgxtcbcomp14svn": 222,
        "sgxtcbcomp15svn": 184,
        "sgxtcbcomp16svn": 98,
        "pcesvn": 37240
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, invalidTcb);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementTcbComponentIsNotAnInteger)
{
    const std::string invalidTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": "12",
        "pcesvn": 37240
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, invalidTcb);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementTcbComponentIsNegative)
{
    const std::string invalidTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": -23,
        "pcesvn": 37240
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, invalidTcb);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementTcbComponentPcesvnIsMissing)
{
    const std::string invalidTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 12,
        "sgxtcbcomp02svn": 34,
        "sgxtcbcomp03svn": 56,
        "sgxtcbcomp04svn": 78,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 254,
        "sgxtcbcomp09svn": 9,
        "sgxtcbcomp10svn": 87,
        "sgxtcbcomp11svn": 65,
        "sgxtcbcomp12svn": 43,
        "sgxtcbcomp13svn": 21,
        "sgxtcbcomp14svn": 222,
        "sgxtcbcomp15svn": 184,
        "sgxtcbcomp16svn": 98
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, invalidTcb);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementTcbComponentPcesvnIsNegative)
{
    const std::string invalidTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 12,
        "sgxtcbcomp02svn": 34,
        "sgxtcbcomp03svn": 56,
        "sgxtcbcomp04svn": 78,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 254,
        "sgxtcbcomp09svn": 9,
        "sgxtcbcomp10svn": 87,
        "sgxtcbcomp11svn": 65,
        "sgxtcbcomp12svn": 43,
        "sgxtcbcomp13svn": 21,
        "sgxtcbcomp14svn": 222,
        "sgxtcbcomp15svn": 184,
        "sgxtcbcomp16svn": 98,
        "pcesvn": -4
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, invalidTcb);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayElementTcbComponentPcesvnIsNotANumber)
{
    const std::string invalidTcb = R"json(
    "tcb": {
        "sgxtcbcomp01svn": 12,
        "sgxtcbcomp02svn": 34,
        "sgxtcbcomp03svn": 56,
        "sgxtcbcomp04svn": 78,
        "sgxtcbcomp05svn": 10,
        "sgxtcbcomp06svn": 0,
        "sgxtcbcomp07svn": 0,
        "sgxtcbcomp08svn": 254,
        "sgxtcbcomp09svn": 9,
        "sgxtcbcomp10svn": 87,
        "sgxtcbcomp11svn": 65,
        "sgxtcbcomp12svn": 43,
        "sgxtcbcomp13svn": 21,
        "sgxtcbcomp14svn": 222,
        "sgxtcbcomp15svn": 184,
        "sgxtcbcomp16svn": 98,
        "pcesvn": "78xy"
    })json";
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, invalidTcb);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::FormatException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayHasTwoIdenticalElements)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1();
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}

TEST_F(TcbInfoV1UT, shouldFailWhenTcbLevelsArrayHasTwoElementsWithSameSvnsAndDifferentStatus)
{
    const std::string tcbLevels = TcbInfoGenerator::generateTcbLevelV1() + "," + TcbInfoGenerator::generateTcbLevelV1(validTcbLevelV1Template, validSgxTcb, validRevokedStatus);
    const auto tcbInfoJson = TcbInfoGenerator::generateTcbInfo(validTcbInfoV1Template, tcbLevels);
    EXPECT_THROW(parser::json::TcbInfo::parse(tcbInfoJson), parser::InvalidExtensionException);
}
