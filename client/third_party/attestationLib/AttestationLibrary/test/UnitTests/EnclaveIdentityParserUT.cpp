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
#include <Verifiers/EnclaveReportVerifier.h>
#include <Verifiers/EnclaveIdentityParser.h>
#include <numeric>
#include <iostream>
#include <QuoteGenerator.h>
#include <QuoteVerification/Quote.h>
#include <EnclaveIdentityGenerator.h>

using namespace testing;
using namespace ::intel::sgx::dcap;
using namespace intel::sgx::dcap::test;
using namespace std;

struct EnclaveIdentityParserUT : public Test
{
    EnclaveIdentityParser parser;
};

TEST_F(EnclaveIdentityParserUT, shouldReturnStatusOkWhenJsonIsOk)
{
    string json = qeIdentityJsonWithSignature(EnclaveIdentityVectorModel().toJSON());
    auto result = parser.parse(json);

    ASSERT_EQ(STATUS_OK, result->getStatus());
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenMiscselectIsWrong)
{
    EnclaveIdentityVectorModel model;
    model.miscselect = {{1, 1}};
    string json = qeIdentityJsonWithSignature(qeIdentityJsonWithSignature(model.toJSON()));
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenOptionalFieldIsInvalid)
{
    EnclaveIdentityVectorModel model;
    string json = qeIdentityJsonWithSignature(model.toJSON());
    removeWordFromString("mrenclave", json);
    removeWordFromString("mrsigner", json);
    removeWordFromString("isvprodid", json);
    removeWordFromString("isvsvn", json);
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenVerionFieldIsInvalid)
{
    EnclaveIdentityVectorModel model;
    string json = qeIdentityJsonWithSignature(model.toJSON());
    removeWordFromString("version", json);
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenMiscselectHasIncorrectSize)
{
    EnclaveIdentityVectorModel model;
    model.miscselect= {{1, 1}};
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenMiscselectIsNotHexString)
{
    EnclaveIdentityStringModel model;
    model.miscselect = "xyz00000";
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenMiscselectMaskHasIncorrectSize)
{
    EnclaveIdentityVectorModel model;
    model.miscselectMask = {{1, 1}};
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenMiscselectMaskIsNotHexString)
{
    EnclaveIdentityStringModel model;
    model.miscselectMask = "xyz00000";
    string json = qeIdentityJsonWithSignature(model.toJSON());

    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenAttributesHasIncorrectSize)
{
    EnclaveIdentityVectorModel model;
    model.attributes = {{1, 1}};
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenAttributesIsNotHexString)
{
    EnclaveIdentityStringModel model;
    model.attributes = "xyz45678900000000000000123456789";
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenAttributesMaskHasIncorrectSize)
{
    EnclaveIdentityVectorModel model;
    model.attributesMask = {{1, 1}};
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenAttributesMaskIsNotHexString)
{
    EnclaveIdentityStringModel model;
    model.attributesMask = "xyz45678900000000000000123456789";
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenIssuedateIsWrong)
{
    EnclaveIdentityStringModel model;
    model.issueDate = "2018-08-22T10:09:";
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityInvalidWhenNextUpdateIsWrong)
{
    EnclaveIdentityStringModel model;
    model.nextUpdate = "2018-08-22T10:09:";
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_INVALID, ex.getStatus());
    }
}

TEST_F(EnclaveIdentityParserUT, shouldReturnEnclaveIdentityUnsuportedVersionWhenVersionIsWrong)
{
    EnclaveIdentityVectorModel model;
    model.version = 5;
    string json = qeIdentityJsonWithSignature(model.toJSON());
    try {
        parser.parse(json);
        FAIL();
    }
    catch (const ParserException &ex)
    {
        EXPECT_EQ(STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_VERSION, ex.getStatus());
    }
}
