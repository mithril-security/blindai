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

#include <gtest/gtest.h>
#include <gmock/gmock.h>

#include <SgxEcdsaAttestation/QuoteVerification.h>
#include <EnclaveIdentityGenerator.h>
#include <Verifiers/EnclaveIdentityParser.h>
#include <Verifiers/EnclaveIdentityV1.h>

using namespace testing;
using namespace intel::sgx::dcap;
using namespace intel::sgx::dcap::test;

struct EnclaveIdentityV1ParserFormatUT : public Test
{
    EnclaveIdentityParser parser;
};

TEST_F(EnclaveIdentityV1ParserFormatUT, positive)
{
    auto json = qeIdentityJsonWithSignature();

    std::vector<uint8_t> expectedMiscSelect = {0x8f, 0xa6, 0x44, 0x72};
    std::vector<uint8_t> expectedMiscSelectMask = {0x00, 0x00, 0xff, 0xfa};
    std::vector<uint8_t> expectedAttributes = {0x12, 0x54, 0x86, 0x35, 0x48, 0xaf, 0x4a, 0x6b, 0x2f, 0xcc, 0x2d, 0x32, 0x44, 0x78, 0x44, 0x52};
    std::vector<uint8_t> expectedAttributesMask = {0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff};
    std::vector<uint8_t> expectedMrSigner = {0xaa, 0xff, 0x34, 0xff, 0xa5, 0x19, 0x81, 0x95, 0x1a, 0x61, 0xd6, 0x16, 0xb1,
        0x6c, 0x16, 0xf1, 0x65, 0x1c, 0x65, 0x16, 0xe5, 0x1f, 0x65, 0x1d, 0x26, 0xa6, 0x16, 0x6e, 0xd5, 0x67, 0x9c, 0x79};
    unsigned int expectedIsvProdId = 3;
	unsigned int expectedIsvSvn = 22;
    try
    {
        auto enclaveIdentity = parser.parse(json);
        auto *jsonObject = dynamic_cast<EnclaveIdentityV1 *>(enclaveIdentity.get());
        EXPECT_EQ(jsonObject->getVersion(), 1);
        EXPECT_EQ(jsonObject->getMiscselect(), expectedMiscSelect);
        EXPECT_EQ(jsonObject->getMiscselectMask(), expectedMiscSelectMask);
        EXPECT_EQ(jsonObject->getAttributes(), expectedAttributes);
        EXPECT_EQ(jsonObject->getAttributesMask(), expectedAttributesMask);
        EXPECT_EQ(jsonObject->getMrsigner(), expectedMrSigner);
        EXPECT_EQ(jsonObject->getIsvProdId(), expectedIsvProdId);
        EXPECT_EQ(jsonObject->getIsvSvn(), expectedIsvSvn);
    }
    catch(const ParserException &ex)
    {
        FAIL() << "Unexpected status: " << ex.getStatus();
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, positiveWithExtraField)
{
    auto jsonV1 = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22,
            "extraField": "ExtraValue"
        })json";

    EXPECT_EQ(STATUS_OK, parser.parse(qeIdentityJsonWithSignature(jsonV1))->getStatus());
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenInitializedWithEmptyString)
{
    try {
        parser.parse("");
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_FORMAT, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWHenInitializedWithInvalidJSON)
{
    try {
        parser.parse("Plain string.");
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_FORMAT, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenQEIdentityFieldIsMissing)
{
    const std::string json = R"json({"signature": "adad"})json";

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenSignatureFieldIsMissing)
{
    auto json = R"json({"qeIdentity": {
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        }})json";

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_FORMAT, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenVersionFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenIssueDateFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenNextUpdateFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectMaskFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesMaskFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMrsignerFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenIsvprodidFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenIsvsvnFieldIsMissing)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenVersionFieldIsNotEqual1or2)
{
    auto qeidTemplate = R"json({
            "version": 23,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_VERSION, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenVersionFieldIsNotANumber)
{
    auto qeidTemplate = R"json({
            "version": "1",
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenIssueDateIsMalformed)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45:00",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenIssueDateIsNotAString)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": 123,
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenNextUpdateIsMalformed)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "219-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenNextUpdateIsNotAString)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": 2019,
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectIsMalformed)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "qwe-4472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectIsNotAString)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": 44,
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectIsTooShort)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa6447",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectIsTooLong)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472f",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectMaskIsMalformed)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "asdfgh56",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectMaskIsNotAString)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": 234,
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectMaskIsTooShort)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fff",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMiscselectMaskIsTooLong)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "000012345",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesAreMalformed)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "qwp4863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesAreNotAString)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": true,
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesAreTooShort)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d324478445",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesAreTooLong)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d32447844521",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesMaskIsMalformed)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffff****",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesMaskIsNotAString)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": 0,
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesMaskIsTooShort)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "fffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenAttributesMaskIsTooLong)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff0",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMrsignerIsMalformed)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "**++lkffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMrsignerIsNotAString)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": 45,
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMrsignerIsTooShort)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c7",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenMrSignerIsTooLong)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c790",
            "isvprodid": 3,
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenIsvprodidIsNotANumber)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": "3",
            "isvsvn": 22
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityV1ParserFormatUT, shouldFailWhenIsvsvnIsNotANumber)
{
    auto qeidTemplate = R"json({
            "version": 1,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "isvsvn": "22"
        })json";
    auto json = qeIdentityJsonWithSignature(qeidTemplate);

    try {
        parser.parse(json);
        FAIL() << "Test should throw exception";
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}
