#ifndef SGXECDSAATTESTATION_VERIFICATION_H
#define SGXECDSAATTESTATION_VERIFICATION_H

#include <string>
#include <vector>
#include <iostream>
#include <memory>
#include <chrono>
#include <optional>
#include "AttestationLibraryAdapter.h"
#include "../../../AttestationLibrary/src/QuoteVerification/Quote.h"
#include "../../../AttestationLibrary/src/QuoteVerification/QuoteConstants.h"

namespace intel { namespace sgx { namespace dcap {
class VerificationStatus {
public:
    bool ok = false;
    Status pckCertificateStatus = STATUS_INVALID_PCK_CERT;
    Status tcbInfoStatus = STATUS_TCB_UNRECOGNIZED_STATUS;
    Status qeIdentityStatus = STATUS_INVALID_QE_REPORT_DATA;
    Status qveIdentityStatus = STATUS_INVALID_QE_REPORT_DATA;
    Status quoteStatus = STATUS_INVALID_QUOTE_SIGNATURE;
    std::optional<std::array<uint8_t, 64>> reportData;
    std::optional<std::array<uint8_t, 32>> mrEnclave;
    std::optional<std::array<uint8_t, 16>> attributes;
    std::optional<uint32_t> miscSelect;
    VerificationStatus() = default;
};

class Verification
{
public:
    std::string pckCertificate;
    std::string pckSigningChain;
    std::string rootCaCrl;
    std::string intermediateCaCrl;
    std::string trustedRootCACertificate;
    std::string tcbInfo;
    std::string tcbSigningChain;
    std::string quote;
    std::string qeIdentity;
    std::string qveIdentity;
    time_t expirationDate = std::chrono::system_clock::to_time_t(std::chrono::system_clock::now());
    std::shared_ptr<IAttestationLibraryAdapter> attestationLib = std::make_shared<AttestationLibraryAdapter>();

    VerificationStatus verify();
};

}}}

#endif //SGXECDSAATTESTATION_VERIFICATION_H
