#include "Verification.h"
namespace intel::sgx::dcap {

inline std::string bytesToHexString(const std::vector<uint8_t> &vector)
{
    std::string result;
    result.reserve(vector.size() * 2);   // two digits per character

    static constexpr char hex[] = "0123456789ABCDEF";

    for (const uint8_t c : vector)
    {
        result.push_back(hex[c / 16]);
        result.push_back(hex[c % 16]);
    }

    return result;
}

VerificationStatus Verification::verify()
{
    const auto pckCertChain = this->pckSigningChain + this->pckCertificate;
    const auto pckVerifyStatus = this->attestationLib->verifyPCKCertificate(pckCertChain, rootCaCrl, intermediateCaCrl, trustedRootCACertificate, this->expirationDate);
    const auto tcbVerifyStatus = this->attestationLib->verifyTCBInfo(tcbInfo, tcbSigningChain, rootCaCrl, trustedRootCACertificate, this->expirationDate);

    const auto qeIdentityPresent = this->qeIdentity != "";
    Status qeIdentityVerifyStatus = STATUS_OK;
    if (qeIdentityPresent)
    {
        qeIdentityVerifyStatus = this->attestationLib->verifyQeIdentity(qeIdentity, tcbSigningChain, rootCaCrl, trustedRootCACertificate, this->expirationDate);
    }

    const auto qveIdentityPresent = this->qveIdentity != "";
    Status qveIdentityVerifyStatus = STATUS_OK;
    if (qveIdentityPresent)
    {
        qveIdentityVerifyStatus = this->attestationLib->verifyQeIdentity(qveIdentity, tcbSigningChain, rootCaCrl, trustedRootCACertificate, this->expirationDate);
    }
    std::vector<uint8_t> vecQuote(this->quote.begin(), this->quote.end());
    const auto quoteVerifyStatus = this->attestationLib->verifyQuote(vecQuote, this->pckCertificate, intermediateCaCrl, tcbInfo, qeIdentity);
    dcap::Quote quote_data;
    if(quote_data.parse(vecQuote) && quote_data.validate())
    {
        return VerificationStatus {(pckVerifyStatus == STATUS_OK) && (tcbVerifyStatus == STATUS_OK) && (quoteVerifyStatus == STATUS_OK) &&
            (qeIdentityVerifyStatus == STATUS_OK) && (qveIdentityVerifyStatus == STATUS_OK), pckVerifyStatus, tcbVerifyStatus, qeIdentityVerifyStatus, qveIdentityVerifyStatus, quoteVerifyStatus, quote_data.getEnclaveReport().reportData, quote_data.getEnclaveReport().mrEnclave, quote_data.getEnclaveReport().attributes, quote_data.getEnclaveReport().miscSelect};
    }
    return VerificationStatus {(pckVerifyStatus == STATUS_OK) && (tcbVerifyStatus == STATUS_OK) && (quoteVerifyStatus == STATUS_OK) &&
            (qeIdentityVerifyStatus == STATUS_OK) && (qveIdentityVerifyStatus == STATUS_OK), pckVerifyStatus, tcbVerifyStatus, qeIdentityVerifyStatus, qveIdentityVerifyStatus, quoteVerifyStatus, std::nullopt, std::nullopt, std::nullopt, std::nullopt};


}

}