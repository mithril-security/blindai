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

#include <SgxEcdsaAttestation/QuoteVerification.h>
#include <SgxEcdsaAttestation/AttestationParsers.h>
#include <CertVerification/X509Constants.h>
#include <QuoteGenerator.h>
#include <EnclaveIdentityGenerator.h>
#include <EcdsaSignatureGenerator.h>
#include <QuoteVerification/QuoteConstants.h>
#include <TcbInfoJsonGenerator.h>
#include <X509CertGenerator.h>
#include <X509CrlGenerator.h>
#include <DigestUtils.h>
#include <KeyHelpers.h>

using namespace std;
using namespace testing;
using namespace intel::sgx::dcap;
using namespace intel::sgx::dcap::test;
using namespace intel::sgx::dcap::parser::test;

struct VerifyQuoteIT : public Test
{
    ~VerifyQuoteIT() override
    {
        delete quotePlaceHolder;
    };

    const char* placeHolder = "placeHolder";
    uint8_t* quotePlaceHolder = new uint8_t;

    int timeNow = 0;
    int timeOneHour = 3600;

    X509CertGenerator certGenerator;
    X509CrlGenerator crlGenerator;
    Bytes sn {0x23, 0x45};
    Bytes ppid = Bytes(16, 0xaa);
    Bytes cpusvn = Bytes(16, 0xff);
    Bytes pceId = {0x04, 0xf3};
    Bytes fmspc = {0x04, 0xf3, 0x44, 0x45, 0xaa, 0x00};
    Bytes pcesvnLE = {0x01, 0x02};
    Bytes pcesvnBE = {0x02, 0x01};

    crypto::EVP_PKEY_uptr keyInt = crypto::make_unique<EVP_PKEY>(nullptr);
    crypto::EVP_PKEY_uptr key = crypto::make_unique<EVP_PKEY>(nullptr);
    crypto::X509_uptr cert = crypto::make_unique<X509>(nullptr);
    crypto::X509_uptr interCert = crypto::make_unique<X509>(nullptr);

    QuoteGenerator quoteGenerator;
    int version = 1;
    int pcesvnStr = 1;
    uint16_t isvprodid = 1;
    uint16_t isvsvn = 1;
    string issueDate = "2018-08-22T10:09:10Z";
    string nextUpdate = "2118-08-23T10:09:10Z";
    string fmspcStr = "04F34445AA00";
    string pceIdStr = "04F3";
    string status = "UpToDate";
    string miscselect = "";
    string miscselectMask = "";
    string attributes = "";
    string attributesMask = "";
    string mrsigner = "";
    string positiveTcbInfoJsonBody;
    string postiveQEIdentityJsonBody;

    test::QuoteGenerator::EnclaveReport enclaveReport;

    VerifyQuoteIT()
    {
        keyInt = certGenerator.generateEcKeypair();
        key = certGenerator.generateEcKeypair();

        cert = certGenerator.generatePCKCert(2, sn, timeNow, timeOneHour, key.get(), keyInt.get(),
                                             constants::PCK_SUBJECT, constants::PLATFORM_CA_SUBJECT,
                                             ppid, cpusvn, pcesvnBE, pceId, fmspc, 0);

        intel::sgx::dcap::parser::x509::DistinguishedName subject =
                {"", "Intel SGX PCK Platform CA", "US", "Intel Corporation", "Santa Clara", "CA"};
        intel::sgx::dcap::parser::x509::DistinguishedName issuer =
                {"", "Intel SGX PCK Platform CA", "US", "Intel Corporation", "Santa Clara", "CA"};

        interCert = certGenerator.generateCaCert(2, sn, timeNow, timeOneHour, key.get(), keyInt.get(), subject, issuer);

        positiveTcbInfoJsonBody = tcbInfoJsonV1Body(version, issueDate, nextUpdate, fmspcStr, pceIdStr,
                                                  getRandomTcb(), pcesvnStr, status);

        EnclaveIdentityVectorModel model;
        postiveQEIdentityJsonBody = model.toJSON();
        model.applyTo(enclaveReport);
    }

    std::string getValidPEMCrl(const crypto::X509_uptr &ucert)
    {
        auto revokedList = std::vector<Bytes>{{0x12, 0x10, 0x13, 0x11}, {0x11, 0x33, 0xff, 0x56}};
        auto rootCaCRL = crlGenerator.generateCRL(CRLVersion::CRL_VERSION_2, 0, 3600, ucert, revokedList);

        return X509CrlGenerator::x509CrlToPEMString(rootCaCRL.get());
    }

    std::string getValidDERCrl(const crypto::X509_uptr &ucert)
    {
        auto revokedList = std::vector<Bytes>{{0x12, 0x10, 0x13, 0x11}, {0x11, 0x33, 0xff, 0x56}};
        auto rootCaCRL = crlGenerator.generateCRL(CRLVersion::CRL_VERSION_2, 0, 3600, ucert, revokedList);

        return X509CrlGenerator::x509CrlToDERString(rootCaCRL.get());
    }

    std::vector<uint8_t> concat(const std::vector<uint8_t>& rhs, const std::vector<uint8_t>& lhs)
    {
        std::vector<uint8_t> ret = rhs;
        std::copy(lhs.begin(), lhs.end(), std::back_inserter(ret));
        return ret;
    }

    template<size_t N>
    std::vector<uint8_t> concat(const std::array<uint8_t,N>& rhs, const std::vector<uint8_t>& lhs)
    {
        std::vector<uint8_t> ret(std::begin(rhs), std::end(rhs));
        std::copy(lhs.begin(), lhs.end(), std::back_inserter(ret));
        return ret;
    }

    std::array<uint8_t,64> assingFirst32(const std::array<uint8_t,32>& in)
    {
        std::array<uint8_t,64> ret{};
        std::copy_n(in.begin(), 32, ret.begin());
        return ret;
    }

    std::array<uint8_t,64> signEnclaveReport(const test::QuoteGenerator::EnclaveReport& report, EVP_PKEY& ukey)
    {
        return signAndGetRaw(report.bytes(), ukey);
    }

    std::array<uint8_t,64> signAndGetRaw(const std::vector<uint8_t>& data, EVP_PKEY& ukey)
    {
        auto usignature = EcdsaSignatureGenerator::signECDSA_SHA256(data, &ukey);
        std::array<uint8_t, 64> signatureArr{};
        std::copy_n(usignature.begin(), signatureArr.size(), signatureArr.begin());
        return signatureArr;
    }

};

TEST_F(VerifyQuoteIT, shouldReturnedMissingParmatersWhenQuoteIsNull)
{
    // GIVEN / WHEN
    auto result = sgxAttestationVerifyQuote(nullptr, 0, placeHolder, placeHolder, placeHolder, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_MISSING_PARAMETERS, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedMissingParmatersWhenPckCertificateIsNull)
{
    // GIVEN / WHEN
    auto result = sgxAttestationVerifyQuote(quotePlaceHolder, 0, nullptr, placeHolder, placeHolder, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_MISSING_PARAMETERS, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedMissingParmatersWhenPckCrlIsNull)
{
    // GIVEN / WHEN
    auto result = sgxAttestationVerifyQuote(quotePlaceHolder, 0, placeHolder, nullptr, placeHolder, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_MISSING_PARAMETERS, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedMissingParmatersWhenTcbInfoJsonIsNull)
{
    // GIVEN / WHEN
    auto result = sgxAttestationVerifyQuote(quotePlaceHolder, 0, placeHolder, placeHolder, nullptr, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_MISSING_PARAMETERS, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedUnsuportedQuoteFormatWhenQuoteParseFail)
{
    // GIVEN / WHEN
    auto result = sgxAttestationVerifyQuote(quotePlaceHolder, 0, placeHolder, placeHolder, placeHolder, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_UNSUPPORTED_QUOTE_FORMAT, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedUnsuportedQuoteFormatWhenQuoteSizeIsIncorrect)
{
    // GIVEN
    auto incorrectQouteSize = 0;
    auto quote = quoteGenerator.buildSgxQuote();

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (unsigned) incorrectQouteSize, placeHolder, placeHolder, placeHolder, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_UNSUPPORTED_QUOTE_FORMAT, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedUnsuportedQuoteFormatWhenQuoteHeaderVersionIsWrong)
{
    // GIVEN
    QuoteGenerator::QuoteHeader quoteHeader{};
    quoteHeader.version = 999;
    quoteGenerator.withHeader(quoteHeader);
    auto quote = quoteGenerator.buildSgxQuote();


    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), placeHolder, placeHolder, placeHolder, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_UNSUPPORTED_QUOTE_FORMAT, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedUnsuportedPckCertFormatWhenVerifyPckCertFail)
{
    // GIVEN
    auto pckCertPubKeyPtr = EVP_PKEY_get0_EC_KEY(key.get());
    auto pckCertKeyPtr = key.get();

    test::QuoteGenerator::QeCertData qeCertData;
    qeCertData.keyDataType = constants::PCK_ID_PLAIN_PPID;
    qeCertData.keyData = concat(ppid, concat(cpusvn, pcesvnLE));
    qeCertData.size = static_cast<uint16_t>(qeCertData.keyData.size());

    quoteGenerator.withQeCertData(qeCertData);
    quoteGenerator.getAuthSize() += (uint32_t) qeCertData.keyData.size();
    quoteGenerator.getQuoteAuthData().ecdsaAttestationKey.publicKey = test::getRawPub(*pckCertPubKeyPtr);

    enclaveReport.reportData = assingFirst32(DigestUtils::sha256DigestArray(concat(quoteGenerator.getQuoteAuthData().ecdsaAttestationKey.publicKey,
                                                                                   quoteGenerator.getQuoteAuthData().qeAuthData.data)));

    quoteGenerator.getQuoteAuthData().qeReport = enclaveReport;
    quoteGenerator.getQuoteAuthData().qeReportSignature.signature =
            signEnclaveReport(quoteGenerator.getQuoteAuthData().qeReport, *pckCertKeyPtr);
    quoteGenerator.getQuoteAuthData().ecdsaSignature.signature =
            signAndGetRaw(concat(quoteGenerator.getHeader().bytes(), quoteGenerator.getEnclaveReport().bytes()), *pckCertKeyPtr);

    auto quote = quoteGenerator.buildSgxQuote();
    auto pckCrl = getValidPEMCrl(interCert);
    auto tcbInfoBodyBytes = Bytes{};
    tcbInfoBodyBytes.insert(tcbInfoBodyBytes.end(), positiveTcbInfoJsonBody.begin(), positiveTcbInfoJsonBody.end());
    auto signatureTcb = EcdsaSignatureGenerator::signECDSA_SHA256(tcbInfoBodyBytes, key.get());
    auto tcbInfoJsonWithSignature = tcbInfoJsonGenerator(positiveTcbInfoJsonBody,
                                                         EcdsaSignatureGenerator::signatureToHexString(signatureTcb));
    auto qeIdentityBodyBytes = Bytes{};
    qeIdentityBodyBytes.insert(qeIdentityBodyBytes.end(), postiveQEIdentityJsonBody.begin(), postiveQEIdentityJsonBody.end());
    auto signatureQE = EcdsaSignatureGenerator::signECDSA_SHA256(qeIdentityBodyBytes, key.get());
    auto qeIdentityJsonWithSignature = ::qeIdentityJsonWithSignature(postiveQEIdentityJsonBody,
                                                                     EcdsaSignatureGenerator::signatureToHexString(
                                                                           signatureQE));

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), placeHolder, pckCrl.c_str(), tcbInfoJsonWithSignature.c_str(),
                                            qeIdentityJsonWithSignature.c_str());

    // THEN
    EXPECT_EQ(STATUS_UNSUPPORTED_PCK_CERT_FORMAT, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedUnsuportedPckCrlFormatWhenVerifyPckCrlFail)
{
    // GIVEN
    auto quote = quoteGenerator.buildSgxQuote();
    auto pckPem = certGenerator.x509ToString(cert.get());

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), pckPem.c_str(), placeHolder, placeHolder, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_UNSUPPORTED_PCK_RL_FORMAT, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedUnsuportedTcbInfoFormatWhenVerifyTcbInfoFail)
{
    // GIVEN
    auto quote = quoteGenerator.buildSgxQuote();
    auto pckPem = certGenerator.x509ToString(cert.get());
    auto pckCrl = getValidPEMCrl(cert);

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), pckPem.c_str(), pckCrl.c_str(), placeHolder, placeHolder);

    // THEN
    EXPECT_EQ(STATUS_UNSUPPORTED_TCB_INFO_FORMAT, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedUnsuportedQeIdentityFormatWhenVerifyQEidentityFail)
{
    // GIVEN
    auto quote = quoteGenerator.buildSgxQuote();
    auto pckPem = certGenerator.x509ToString(cert.get());
    auto pckCrl = getValidPEMCrl(cert);
    auto tcbInfoBodyBytes = Bytes{};
    tcbInfoBodyBytes.insert(tcbInfoBodyBytes.end(), positiveTcbInfoJsonBody.begin(), positiveTcbInfoJsonBody.end());
    auto signature = EcdsaSignatureGenerator::signECDSA_SHA256(tcbInfoBodyBytes, key.get());
    auto tcbInfoJsonWithSignature = tcbInfoJsonGenerator(positiveTcbInfoJsonBody,
                                                         EcdsaSignatureGenerator::signatureToHexString(signature));

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), pckPem.c_str(), pckCrl.c_str(),
                                            tcbInfoJsonWithSignature.c_str(), placeHolder);

    // THEN
    EXPECT_EQ(STATUS_UNSUPPORTED_QE_IDENTITY_FORMAT, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedUnsuportedQeIdentityFormatWhenQEIdentityIsWrong)
{
    // GIVEN
    auto quote = quoteGenerator.buildSgxQuote();
    auto pckPem = certGenerator.x509ToString(cert.get());
    auto pckCrl = getValidPEMCrl(cert);
    auto tcbInfoBodyBytes = Bytes{};
    tcbInfoBodyBytes.insert(tcbInfoBodyBytes.end(), positiveTcbInfoJsonBody.begin(), positiveTcbInfoJsonBody.end());
    auto signature = EcdsaSignatureGenerator::signECDSA_SHA256(tcbInfoBodyBytes, key.get());
    auto tcbInfoJsonWithSignature = tcbInfoJsonGenerator(positiveTcbInfoJsonBody,
                                                         EcdsaSignatureGenerator::signatureToHexString(signature));

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), pckPem.c_str(), pckCrl.c_str(),
                                            tcbInfoJsonWithSignature.c_str(), placeHolder);

    // THEN
    EXPECT_EQ(STATUS_UNSUPPORTED_QE_IDENTITY_FORMAT, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedStatusOKWhenVerifyQuoteSuccessffulyWhenCrlAsPem)
{
    // GIVEN
    auto pckCertPubKeyPtr = EVP_PKEY_get0_EC_KEY(key.get());
    auto pckCertKeyPtr = key.get();

    test::QuoteGenerator::QeCertData qeCertData;
    qeCertData.keyDataType = constants::PCK_ID_PLAIN_PPID;
    qeCertData.keyData = concat(ppid, concat(cpusvn, pcesvnLE));
    qeCertData.size = static_cast<uint16_t>(qeCertData.keyData.size());

    quoteGenerator.withQeCertData(qeCertData);
    quoteGenerator.getAuthSize() += (uint32_t) qeCertData.keyData.size();
    quoteGenerator.getQuoteAuthData().ecdsaAttestationKey.publicKey = test::getRawPub(*pckCertPubKeyPtr);

    enclaveReport.reportData = assingFirst32(DigestUtils::sha256DigestArray(concat(quoteGenerator.getQuoteAuthData().ecdsaAttestationKey.publicKey,
                                                  quoteGenerator.getQuoteAuthData().qeAuthData.data)));

    quoteGenerator.getQuoteAuthData().qeReport = enclaveReport;
    quoteGenerator.getQuoteAuthData().qeReportSignature.signature =
            signEnclaveReport(quoteGenerator.getQuoteAuthData().qeReport, *pckCertKeyPtr);
    quoteGenerator.getQuoteAuthData().ecdsaSignature.signature =
            signAndGetRaw(concat(quoteGenerator.getHeader().bytes(), quoteGenerator.getEnclaveReport().bytes()), *pckCertKeyPtr);

    auto quote = quoteGenerator.buildSgxQuote();
    auto pckPem = certGenerator.x509ToString(cert.get());
    auto pckCrl = getValidPEMCrl(interCert);
    auto tcbInfoBodyBytes = Bytes{};
    tcbInfoBodyBytes.insert(tcbInfoBodyBytes.end(), positiveTcbInfoJsonBody.begin(), positiveTcbInfoJsonBody.end());
    auto signatureTcb = EcdsaSignatureGenerator::signECDSA_SHA256(tcbInfoBodyBytes, key.get());
    auto tcbInfoJsonWithSignature = tcbInfoJsonGenerator(positiveTcbInfoJsonBody,
                                                         EcdsaSignatureGenerator::signatureToHexString(signatureTcb));

    auto qeIdentityBodyBytes = Bytes{};
    qeIdentityBodyBytes.insert(qeIdentityBodyBytes.end(), postiveQEIdentityJsonBody.begin(), postiveQEIdentityJsonBody.end());
    auto signatureQE = EcdsaSignatureGenerator::signECDSA_SHA256(qeIdentityBodyBytes, key.get());
    auto qeIdentityJsonWithSignature = ::qeIdentityJsonWithSignature(postiveQEIdentityJsonBody,
                                                                     EcdsaSignatureGenerator::signatureToHexString(
                                                                           signatureQE));

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), pckPem.c_str(), pckCrl.c_str(),
                                            tcbInfoJsonWithSignature.c_str(), qeIdentityJsonWithSignature.c_str());

    // THEN
    EXPECT_EQ(STATUS_OK, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedStatusOKWhenVerifyQuoteSuccessffulyWhenCrlAsDer)
{
    // GIVEN
    auto pckCertPubKeyPtr = EVP_PKEY_get0_EC_KEY(key.get());
    auto pckCertKeyPtr = key.get();

    test::QuoteGenerator::QeCertData qeCertData;
    qeCertData.keyDataType = constants::PCK_ID_PLAIN_PPID;
    qeCertData.keyData = concat(ppid, concat(cpusvn, pcesvnLE));
    qeCertData.size = static_cast<uint16_t>(qeCertData.keyData.size());

    quoteGenerator.withQeCertData(qeCertData);
    quoteGenerator.getAuthSize() += (uint32_t) qeCertData.keyData.size();
    quoteGenerator.getQuoteAuthData().ecdsaAttestationKey.publicKey = test::getRawPub(*pckCertPubKeyPtr);

    enclaveReport.reportData = assingFirst32(DigestUtils::sha256DigestArray(concat(quoteGenerator.getQuoteAuthData().ecdsaAttestationKey.publicKey,
                                                                                   quoteGenerator.getQuoteAuthData().qeAuthData.data)));

    quoteGenerator.getQuoteAuthData().qeReport = enclaveReport;
    quoteGenerator.getQuoteAuthData().qeReportSignature.signature =
            signEnclaveReport(quoteGenerator.getQuoteAuthData().qeReport, *pckCertKeyPtr);
    quoteGenerator.getQuoteAuthData().ecdsaSignature.signature =
            signAndGetRaw(concat(quoteGenerator.getHeader().bytes(), quoteGenerator.getEnclaveReport().bytes()), *pckCertKeyPtr);

    auto quote = quoteGenerator.buildSgxQuote();
    auto pckPem = certGenerator.x509ToString(cert.get());
    auto pckCrl = getValidDERCrl(interCert);
    auto tcbInfoBodyBytes = Bytes{};
    tcbInfoBodyBytes.insert(tcbInfoBodyBytes.end(), positiveTcbInfoJsonBody.begin(), positiveTcbInfoJsonBody.end());
    auto signatureTcb = EcdsaSignatureGenerator::signECDSA_SHA256(tcbInfoBodyBytes, key.get());
    auto tcbInfoJsonWithSignature = tcbInfoJsonGenerator(positiveTcbInfoJsonBody,
                                                         EcdsaSignatureGenerator::signatureToHexString(signatureTcb));

    auto qeIdentityBodyBytes = Bytes{};
    qeIdentityBodyBytes.insert(qeIdentityBodyBytes.end(), postiveQEIdentityJsonBody.begin(), postiveQEIdentityJsonBody.end());
    auto signatureQE = EcdsaSignatureGenerator::signECDSA_SHA256(qeIdentityBodyBytes, key.get());
    auto qeIdentityJsonWithSignature = ::qeIdentityJsonWithSignature(postiveQEIdentityJsonBody,
                                                                     EcdsaSignatureGenerator::signatureToHexString(
                                                                             signatureQE));

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), pckPem.c_str(), pckCrl.c_str(),
                                            tcbInfoJsonWithSignature.c_str(), qeIdentityJsonWithSignature.c_str());

    // THEN
    EXPECT_EQ(STATUS_OK, result);
}

TEST_F(VerifyQuoteIT, shouldReturnedStatusOKWhenVerifyQuoteSuccessffulyWithNoQeIdentityJson)
{
    // GIVEN
    auto pckCertPubKeyPtr = EVP_PKEY_get0_EC_KEY(key.get());
    auto pckCertKeyPtr = key.get();

    test::QuoteGenerator::QeCertData qeCertData;
    qeCertData.keyDataType = constants::PCK_ID_PLAIN_PPID;
    qeCertData.keyData = concat(ppid, concat(cpusvn, pcesvnLE));
    qeCertData.size = static_cast<uint16_t>(qeCertData.keyData.size());

    quoteGenerator.withQeCertData(qeCertData);
    quoteGenerator.getAuthSize() += (uint32_t) qeCertData.keyData.size();
    quoteGenerator.getQuoteAuthData().ecdsaAttestationKey.publicKey = test::getRawPub(*pckCertPubKeyPtr);

    enclaveReport.reportData = assingFirst32(DigestUtils::sha256DigestArray(concat(quoteGenerator.getQuoteAuthData().ecdsaAttestationKey.publicKey,
                                                                                   quoteGenerator.getQuoteAuthData().qeAuthData.data)));

    quoteGenerator.getQuoteAuthData().qeReport = enclaveReport;
    quoteGenerator.getQuoteAuthData().qeReportSignature.signature =
            signEnclaveReport(quoteGenerator.getQuoteAuthData().qeReport, *pckCertKeyPtr);
    quoteGenerator.getQuoteAuthData().ecdsaSignature.signature =
            signAndGetRaw(concat(quoteGenerator.getHeader().bytes(), quoteGenerator.getEnclaveReport().bytes()), *pckCertKeyPtr);

    auto quote = quoteGenerator.buildSgxQuote();
    auto pckPem = certGenerator.x509ToString(cert.get());
    auto pckCrl = getValidPEMCrl(interCert);
    auto tcbInfoBodyBytes = Bytes{};
    tcbInfoBodyBytes.insert(tcbInfoBodyBytes.end(), positiveTcbInfoJsonBody.begin(), positiveTcbInfoJsonBody.end());
    auto signatureTcb = EcdsaSignatureGenerator::signECDSA_SHA256(tcbInfoBodyBytes, key.get());
    auto tcbInfoJsonWithSignature = tcbInfoJsonGenerator(positiveTcbInfoJsonBody,
                                                         EcdsaSignatureGenerator::signatureToHexString(signatureTcb));

    // WHEN
    auto result = sgxAttestationVerifyQuote(quote.data(), (uint32_t) quote.size(), pckPem.c_str(), pckCrl.c_str(),
                                            tcbInfoJsonWithSignature.c_str(), nullptr);

    // THEN
    EXPECT_EQ(STATUS_OK, result);
}
