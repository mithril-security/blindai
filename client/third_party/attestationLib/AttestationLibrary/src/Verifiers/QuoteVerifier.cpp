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

#include "QuoteVerifier.h"
#include "EnclaveIdentity.h"
#include "Utils/RuntimeException.h"

#include <algorithm>
#include <functional>

#include <CertVerification/X509Constants.h>
#include <QuoteVerification/QuoteConstants.h>
#include <OpensslHelpers/DigestUtils.h>
#include <OpensslHelpers/KeyUtils.h>
#include <OpensslHelpers/SignatureVerification.h>
#include <Verifiers/PckCertVerifier.h>

namespace intel { namespace sgx { namespace dcap {

namespace {

constexpr int CPUSVN_LOWER = false;
constexpr int CPUSVN_EQUAL_OR_HIGHER = true;

bool isCpuSvnHigherOrEqual(const dcap::parser::x509::PckCertificate& pckCert,
                           const dcap::parser::json::TcbLevel& tcbLevel)
{
    for(unsigned int index = 0; index < constants::CPUSVN_BYTE_LEN; ++index)
    {
        const auto componentValue = pckCert.getTcb().getSgxTcbComponentSvn(index);
        const auto otherComponentValue = tcbLevel.getSgxTcbComponentSvn(index);
        if(componentValue < otherComponentValue)
        {
            // If *ANY* CPUSVN component is lower then CPUSVN is considered lower
            return CPUSVN_LOWER;
        }
    }
    // but for CPUSVN to be considered higher it requires that *EVERY* CPUSVN component to be higher or equal
    return CPUSVN_EQUAL_OR_HIGHER;
}

const std::string& getMatchingTcbLevel(const dcap::parser::json::TcbInfo &tcbInfo,
                                       const dcap::parser::x509::PckCertificate &pckCert)
{
    const auto &tcbs = tcbInfo.getTcbLevels();
    const auto certPceSvn = pckCert.getTcb().getPceSvn();

    for (const auto& tcb : tcbs)
    {
        if(isCpuSvnHigherOrEqual(pckCert, tcb) && certPceSvn >= tcb.getPceSvn())
        {
            return tcb.getStatus();
        }
    }

    /// 4.1.2.4.16.3
    throw RuntimeException(STATUS_TCB_NOT_SUPPORTED);
}

Status checkTcbLevel(const dcap::parser::json::TcbInfo& tcbInfoJson, const dcap::parser::x509::PckCertificate& pckCert)
{
    /// 4.1.2.4.16.1 & 4.1.2.4.16.2
    const auto& tcbLevelStatus = getMatchingTcbLevel(tcbInfoJson, pckCert);

    if (tcbLevelStatus == "OutOfDate")
    {
        return STATUS_TCB_OUT_OF_DATE;
    }

    if (tcbLevelStatus == "Revoked")
    {
        return STATUS_TCB_REVOKED;
    }

    if (tcbLevelStatus == "ConfigurationNeeded")
    {
        return STATUS_TCB_CONFIGURATION_NEEDED;
    }

    if (tcbLevelStatus == "ConfigurationAndSWHardeningNeeded")
    {
        return STATUS_TCB_CONFIGURATION_AND_SW_HARDENING_NEEDED;
    }

    if (tcbLevelStatus == "UpToDate")
    {
        return STATUS_OK;
    }

    if (tcbLevelStatus == "SWHardeningNeeded")
    {
        return STATUS_TCB_SW_HARDENING_NEEDED;
    }

    if(tcbInfoJson.getVersion() > 1 && tcbLevelStatus == "OutOfDateConfigurationNeeded")
    {
        return STATUS_TCB_OUT_OF_DATE_CONFIGURATION_NEEDED;
    }

    throw RuntimeException(STATUS_TCB_UNRECOGNIZED_STATUS);
}

Status convergeTcbStatus(Status tcbLevelStatus, Status qeTcbStatus)
{
    if (qeTcbStatus == STATUS_SGX_ENCLAVE_REPORT_ISVSVN_OUT_OF_DATE)
    {
        if (tcbLevelStatus == STATUS_OK ||
            tcbLevelStatus == STATUS_TCB_SW_HARDENING_NEEDED)
        {
            return STATUS_TCB_OUT_OF_DATE;
        }
        if (tcbLevelStatus == STATUS_TCB_CONFIGURATION_NEEDED ||
            tcbLevelStatus == STATUS_TCB_CONFIGURATION_AND_SW_HARDENING_NEEDED)
        {
            return STATUS_TCB_OUT_OF_DATE_CONFIGURATION_NEEDED;
        }
    }
    if (qeTcbStatus == STATUS_SGX_ENCLAVE_REPORT_ISVSVN_REVOKED)
    {
            return STATUS_TCB_REVOKED;
    }

    switch (tcbLevelStatus)
    {
        case STATUS_TCB_OUT_OF_DATE:
        case STATUS_TCB_REVOKED:
        case STATUS_TCB_CONFIGURATION_NEEDED:
        case STATUS_TCB_OUT_OF_DATE_CONFIGURATION_NEEDED:
        case STATUS_TCB_SW_HARDENING_NEEDED:
        case STATUS_TCB_CONFIGURATION_AND_SW_HARDENING_NEEDED:
        case STATUS_OK:
            return tcbLevelStatus;
        default:
            /// 4.1.2.4.16.4
            return STATUS_TCB_UNRECOGNIZED_STATUS;
    }
}

}//anonymous namespace

Status QuoteVerifier::verify(const Quote& quote,
                             const dcap::parser::x509::PckCertificate& pckCert,
                             const pckparser::CrlStore& crl,
                             const dcap::parser::json::TcbInfo& tcbInfoJson,
                             const EnclaveIdentity *enclaveIdentity,
                             const EnclaveReportVerifier& enclaveReportVerifier)
{
    Status qeIdentityStatus = STATUS_QE_IDENTITY_MISMATCH;

    /// 4.1.2.4.4
    if (!_baseVerififer.commonNameContains(pckCert.getSubject(), constants::SGX_PCK_CN_PHRASE)) {
        return STATUS_INVALID_PCK_CERT;
    }

    /// 4.1.2.4.6
    if (!PckCrlVerifier{}.checkIssuer(crl) || crl.getIssuer().raw != pckCert.getIssuer().getRaw()) {
        return STATUS_INVALID_PCK_CRL;
    }

    /// 4.1.2.4.7
    if(crl.isRevoked(pckCert))
    {
        return STATUS_PCK_REVOKED;
    }

    /// 4.1.2.4.10
    if(pckCert.getFmspc() != tcbInfoJson.getFmspc())
    {
        return STATUS_TCB_INFO_MISMATCH;
    }

    if(pckCert.getPceId() != tcbInfoJson.getPceId())
    {
        return STATUS_TCB_INFO_MISMATCH;
    }

    const auto qeCertData = quote.getQuoteAuthData().qeCertData;
    auto qeCertDataVerificationStatus = verifyQeCertData(qeCertData);
    if(qeCertDataVerificationStatus != STATUS_OK)
    {
        return qeCertDataVerificationStatus;
    }

    auto pubKey = crypto::rawToP256PubKey(pckCert.getPubKey());
    if (pubKey == nullptr)
    {
        return STATUS_INVALID_PCK_CERT; // if there were issues with parsing public key it means cert was invalid.
                                        // Probably it will never happen because parsing cert should fail earlier.
    }

    /// 4.1.2.4.11
    if(!crypto::verifySha256EcdsaSignature(quote.getQuoteAuthData().qeReportSignature.signature,
                                           quote.getQuoteAuthData().qeReport.rawBlob(),
                                           *pubKey))
    {
        return STATUS_INVALID_QE_REPORT_SIGNATURE;
    }

    /// 4.1.2.4.12
    const auto hashedConcatOfAttestKeyAndQeReportData = [&]() -> std::vector<uint8_t>
    {
        const auto attestKeyData = quote.getQuoteAuthData().ecdsaAttestationKey.pubKey;
        const auto qeAuthData = quote.getQuoteAuthData().qeAuthData.data;
        std::vector<uint8_t> ret;
        ret.reserve(attestKeyData.size() + qeAuthData.size());
        std::copy(attestKeyData.begin(), attestKeyData.end(), std::back_inserter(ret));
        std::copy(qeAuthData.begin(), qeAuthData.end(), std::back_inserter(ret));

        return crypto::sha256Digest(ret);
    }();

    if(hashedConcatOfAttestKeyAndQeReportData.empty() || !std::equal(hashedConcatOfAttestKeyAndQeReportData.begin(),
                                                                     hashedConcatOfAttestKeyAndQeReportData.end(),
                                                                     quote.getQuoteAuthData().qeReport.reportData.begin()))
    {
        return STATUS_INVALID_QE_REPORT_DATA;
    }

    if (enclaveIdentity)
    {
        /// 4.1.2.4.13
        if(quote.getHeader().teeType == dcap::constants::TEE_TYPE_SGX)
        {
            if(enclaveIdentity->getID() != EnclaveID::QE)
            {
                return STATUS_QE_IDENTITY_MISMATCH;
            }
        }
        else
        {
            return STATUS_QE_IDENTITY_MISMATCH;
        }

        /// 4.1.2.4.14
        qeIdentityStatus = enclaveReportVerifier.verify(enclaveIdentity, quote.getQuoteAuthData().qeReport);
        switch(qeIdentityStatus) {
            case STATUS_SGX_ENCLAVE_REPORT_UNSUPPORTED_FORMAT:
                return STATUS_UNSUPPORTED_QUOTE_FORMAT;
            case STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_FORMAT:
            case STATUS_SGX_ENCLAVE_IDENTITY_INVALID:
            case STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_VERSION:
                return STATUS_UNSUPPORTED_QE_IDENTITY_FORMAT;
            case STATUS_SGX_ENCLAVE_REPORT_MISCSELECT_MISMATCH:
            case STATUS_SGX_ENCLAVE_REPORT_ATTRIBUTES_MISMATCH:
            case STATUS_SGX_ENCLAVE_REPORT_MRSIGNER_MISMATCH:
            case STATUS_SGX_ENCLAVE_REPORT_ISVPRODID_MISMATCH:
                return STATUS_QE_IDENTITY_MISMATCH;
            case STATUS_SGX_ENCLAVE_REPORT_ISVSVN_OUT_OF_DATE:
            case STATUS_SGX_ENCLAVE_REPORT_ISVSVN_REVOKED:
            default:
                break;
        }
    }

    const auto attestKey = crypto::rawToP256PubKey(quote.getQuoteAuthData().ecdsaAttestationKey.pubKey);
    if(!attestKey)
    {
        return STATUS_UNSUPPORTED_QUOTE_FORMAT;
    }

    /// 4.1.2.4.15
    if (!crypto::verifySha256EcdsaSignature(quote.getQuoteAuthData().ecdsa256BitSignature.signature,
                                            quote.getSignedData(),
                                            *attestKey))
    {
        return STATUS_INVALID_QUOTE_SIGNATURE;
    }

    try
    {
        /// 4.1.2.4.16
        const auto tcbLevelStatus = checkTcbLevel(tcbInfoJson, pckCert);

        if (enclaveIdentity)
        {
            return convergeTcbStatus(tcbLevelStatus, qeIdentityStatus);
        }

        return tcbLevelStatus;
    }
    catch (const RuntimeException &ex)
    {
        return ex.getStatus();
    }
}

Status QuoteVerifier::verifyQeCertData(const Quote::QeCertData& qeCertData) const
{
    if(qeCertData.parsedDataSize != qeCertData.data.size())
    {
        return STATUS_UNSUPPORTED_QUOTE_FORMAT;
    }

    return STATUS_OK;
}

}}} // namespace intel { namespace sgx { namespace dcap {
