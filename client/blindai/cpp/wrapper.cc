// Copyright 2022 Mithril Security. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include "../../third_party/attestationLib/AttestationApp/src/AppCore/Verification.h"

namespace py = pybind11;

PYBIND11_MODULE(_quote_verification, m) {
    py::class_<intel::sgx::dcap::Verification>(m, "Verification")
        .def(py::init<>())
        .def_readwrite("pckCertificate", &intel::sgx::dcap::Verification::pckCertificate)
        .def_readwrite("pckSigningChain", &intel::sgx::dcap::Verification::pckSigningChain)
        .def_readwrite("rootCaCrl", &intel::sgx::dcap::Verification::rootCaCrl)
        .def_readwrite("intermediateCaCrl", &intel::sgx::dcap::Verification::intermediateCaCrl)
        .def_readwrite("trustedRootCACertificate", &intel::sgx::dcap::Verification::trustedRootCACertificate)
        .def_readwrite("tcbInfo", &intel::sgx::dcap::Verification::tcbInfo)
        .def_readwrite("tcbSigningChain", &intel::sgx::dcap::Verification::tcbSigningChain)
        .def_readwrite("quote", &intel::sgx::dcap::Verification::quote)
        .def_readwrite("qeIdentity", &intel::sgx::dcap::Verification::qeIdentity)
        .def_readwrite("qveIdentity", &intel::sgx::dcap::Verification::qveIdentity)
        .def_readwrite("expirationDate", &intel::sgx::dcap::Verification::expirationDate)
        .def("verify", &intel::sgx::dcap::Verification::verify);
    py::class_<intel::sgx::dcap::VerificationStatus>(m, "VerificationStatus")
        .def(py::init<>())
        .def_readwrite("ok", &intel::sgx::dcap::VerificationStatus::ok)
        .def_readwrite("pckCertificateStatus", &intel::sgx::dcap::VerificationStatus::pckCertificateStatus)
        .def_readwrite("tcbInfoStatus", &intel::sgx::dcap::VerificationStatus::tcbInfoStatus)
        .def_readwrite("qeIdentityStatus", &intel::sgx::dcap::VerificationStatus::qeIdentityStatus)
        .def_readwrite("qveIdentityStatus", &intel::sgx::dcap::VerificationStatus::qveIdentityStatus)
        .def_readwrite("quoteStatus", &intel::sgx::dcap::VerificationStatus::quoteStatus)
        .def_readwrite("reportData", &intel::sgx::dcap::VerificationStatus::reportData)
        .def_readwrite("mrEnclave", &intel::sgx::dcap::VerificationStatus::mrEnclave)
        .def_readwrite("attributes", &intel::sgx::dcap::VerificationStatus::attributes)
        .def_readwrite("miscSelect", &intel::sgx::dcap::VerificationStatus::miscSelect);


    py::enum_<_status>(m, "status")
        .value("STATUS_OK", _status::STATUS_OK)
        .value("STATUS_UNSUPPORTED_CERT_FORMAT", _status::STATUS_UNSUPPORTED_CERT_FORMAT)
        .value("STATUS_SGX_ROOT_CA_MISSING", _status::STATUS_SGX_ROOT_CA_MISSING)
        .value("STATUS_SGX_ROOT_CA_INVALID", _status::STATUS_SGX_ROOT_CA_INVALID)
        .value("STATUS_SGX_ROOT_CA_INVALID_EXTENSIONS", _status::STATUS_SGX_ROOT_CA_INVALID_EXTENSIONS)
        .value("STATUS_SGX_ROOT_CA_INVALID_ISSUER", _status::STATUS_SGX_ROOT_CA_INVALID_ISSUER)
        .value("STATUS_SGX_ROOT_CA_UNTRUSTED", _status::STATUS_SGX_ROOT_CA_UNTRUSTED)
        .value("STATUS_SGX_INTERMEDIATE_CA_MISSING", _status::STATUS_SGX_INTERMEDIATE_CA_MISSING)
        .value("STATUS_SGX_INTERMEDIATE_CA_INVALID", _status::STATUS_SGX_INTERMEDIATE_CA_INVALID)
        .value("STATUS_SGX_INTERMEDIATE_CA_INVALID_EXTENSIONS", _status::STATUS_SGX_INTERMEDIATE_CA_INVALID_EXTENSIONS)
        .value("STATUS_SGX_INTERMEDIATE_CA_INVALID_ISSUER", _status::STATUS_SGX_INTERMEDIATE_CA_INVALID_ISSUER) // 10
        .value("STATUS_SGX_INTERMEDIATE_CA_REVOKED", _status::STATUS_SGX_INTERMEDIATE_CA_REVOKED)
        .value("STATUS_SGX_PCK_MISSING", _status::STATUS_SGX_PCK_MISSING)
        .value("STATUS_SGX_PCK_INVALID",_status::STATUS_SGX_PCK_INVALID)
        .value("STATUS_SGX_PCK_INVALID_EXTENSIONS", _status::STATUS_SGX_PCK_INVALID_EXTENSIONS)
        .value("STATUS_SGX_PCK_INVALID_ISSUER", _status::STATUS_SGX_PCK_INVALID_ISSUER)
        .value("STATUS_SGX_PCK_REVOKED", _status::STATUS_SGX_PCK_REVOKED)
        .value("STATUS_TRUSTED_ROOT_CA_INVALID", _status::STATUS_TRUSTED_ROOT_CA_INVALID)
        .value("STATUS_SGX_PCK_CERT_CHAIN_UNTRUSTED", _status::STATUS_SGX_PCK_CERT_CHAIN_UNTRUSTED)
        .value("STATUS_SGX_TCB_INFO_UNSUPPORTED_FORMAT", _status::STATUS_SGX_TCB_INFO_UNSUPPORTED_FORMAT)
        .value("STATUS_SGX_TCB_INFO_INVALID", _status::STATUS_SGX_TCB_INFO_INVALID) // 20
        .value("STATUS_TCB_INFO_INVALID_SIGNATURE", _status::STATUS_TCB_INFO_INVALID_SIGNATURE)
        .value("STATUS_SGX_TCB_SIGNING_CERT_MISSING", _status::STATUS_SGX_TCB_SIGNING_CERT_MISSING)
        .value("STATUS_SGX_TCB_SIGNING_CERT_INVALID", _status::STATUS_SGX_TCB_SIGNING_CERT_INVALID)
        .value("STATUS_SGX_TCB_SIGNING_CERT_INVALID_EXTENSIONS", _status::STATUS_SGX_TCB_SIGNING_CERT_INVALID_EXTENSIONS)
        .value("STATUS_SGX_TCB_SIGNING_CERT_INVALID_ISSUER", _status::STATUS_SGX_TCB_SIGNING_CERT_INVALID_ISSUER)
        .value("STATUS_SGX_TCB_SIGNING_CERT_CHAIN_UNTRUSTED", _status::STATUS_SGX_TCB_SIGNING_CERT_CHAIN_UNTRUSTED)
        .value("STATUS_SGX_TCB_SIGNING_CERT_REVOKED", _status::STATUS_SGX_TCB_SIGNING_CERT_REVOKED)
        .value("STATUS_SGX_CRL_UNSUPPORTED_FORMAT", _status::STATUS_SGX_CRL_UNSUPPORTED_FORMAT)
        .value("STATUS_SGX_CRL_UNKNOWN_ISSUER", _status::STATUS_SGX_CRL_UNKNOWN_ISSUER)
        .value("STATUS_SGX_CRL_INVALID", _status::STATUS_SGX_CRL_INVALID)
        .value("STATUS_SGX_CRL_INVALID_EXTENSIONS", _status::STATUS_SGX_CRL_INVALID_EXTENSIONS)
        .value("STATUS_SGX_CRL_INVALID_SIGNATURE", _status::STATUS_SGX_CRL_INVALID_SIGNATURE)
        .value("STATUS_SGX_CA_CERT_UNSUPPORTED_FORMAT", _status::STATUS_SGX_CA_CERT_UNSUPPORTED_FORMAT)
        .value("STATUS_SGX_CA_CERT_INVALID", _status::STATUS_SGX_CA_CERT_INVALID)
        .value("STATUS_TRUSTED_ROOT_CA_UNSUPPORTED_FORMAT", _status::STATUS_TRUSTED_ROOT_CA_UNSUPPORTED_FORMAT)
        .value("STATUS_MISSING_PARAMETERS", _status::STATUS_MISSING_PARAMETERS)
        .value("STATUS_UNSUPPORTED_QUOTE_FORMAT", _status::STATUS_UNSUPPORTED_QUOTE_FORMAT)
        .value("STATUS_UNSUPPORTED_PCK_CERT_FORMAT", _status::STATUS_UNSUPPORTED_PCK_CERT_FORMAT)
        .value("STATUS_INVALID_PCK_CERT", _status::STATUS_INVALID_PCK_CERT)
        .value("STATUS_UNSUPPORTED_PCK_RL_FORMAT", _status::STATUS_UNSUPPORTED_PCK_RL_FORMAT)
        .value("STATUS_INVALID_PCK_CRL", _status::STATUS_INVALID_PCK_CRL)
        .value("STATUS_UNSUPPORTED_TCB_INFO_FORMAT", _status::STATUS_UNSUPPORTED_TCB_INFO_FORMAT)
        .value("STATUS_PCK_REVOKED", _status::STATUS_PCK_REVOKED)
        .value("STATUS_TCB_INFO_MISMATCH", _status::STATUS_TCB_INFO_MISMATCH)
        .value("STATUS_TCB_OUT_OF_DATE", _status::STATUS_TCB_OUT_OF_DATE)
        .value("STATUS_TCB_REVOKED", _status::STATUS_TCB_REVOKED)
        .value("STATUS_TCB_CONFIGURATION_NEEDED", _status::STATUS_TCB_CONFIGURATION_NEEDED)
        .value("STATUS_TCB_OUT_OF_DATE_CONFIGURATION_NEEDED", _status::STATUS_TCB_OUT_OF_DATE_CONFIGURATION_NEEDED)
        .value("STATUS_TCB_NOT_SUPPORTED", _status::STATUS_TCB_NOT_SUPPORTED)
        .value("STATUS_TCB_UNRECOGNIZED_STATUS", _status::STATUS_TCB_UNRECOGNIZED_STATUS)
        .value("STATUS_UNSUPPORTED_QE_CERTIFICATION", _status::STATUS_UNSUPPORTED_QE_CERTIFICATION)
        .value("STATUS_INVALID_QE_CERTIFICATION_DATA_SIZE", _status::STATUS_INVALID_QE_CERTIFICATION_DATA_SIZE)
        .value("STATUS_UNSUPPORTED_QE_CERTIFICATION_DATA_TYPE", _status::STATUS_UNSUPPORTED_QE_CERTIFICATION_DATA_TYPE)
        .value("STATUS_PCK_CERT_MISMATCH", _status::STATUS_PCK_CERT_MISMATCH)
        .value("STATUS_INVALID_QE_REPORT_SIGNATURE", _status::STATUS_INVALID_QE_REPORT_SIGNATURE)
        .value("STATUS_INVALID_QE_REPORT_DATA", _status::STATUS_INVALID_QE_REPORT_DATA)
        .value("STATUS_INVALID_QUOTE_SIGNATURE", _status::STATUS_INVALID_QUOTE_SIGNATURE)
        .value("STATUS_SGX_QE_IDENTITY_UNSUPPORTED_FORMAT", _status::STATUS_SGX_QE_IDENTITY_UNSUPPORTED_FORMAT)
        .value("STATUS_SGX_QE_IDENTITY_INVALID", _status::STATUS_SGX_QE_IDENTITY_INVALID)
        .value("STATUS_SGX_QE_IDENTITY_INVALID_SIGNATURE", _status::STATUS_SGX_QE_IDENTITY_INVALID_SIGNATURE)
        .value("STATUS_SGX_ENCLAVE_REPORT_UNSUPPORTED_FORMAT", _status::STATUS_SGX_ENCLAVE_REPORT_UNSUPPORTED_FORMAT)
        .value("STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_FORMAT", _status::STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_FORMAT)
        .value("STATUS_SGX_ENCLAVE_IDENTITY_INVALID", _status::STATUS_SGX_ENCLAVE_IDENTITY_INVALID)
        .value("STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_VERSION", _status::STATUS_SGX_ENCLAVE_IDENTITY_UNSUPPORTED_VERSION)
        .value("STATUS_SGX_ENCLAVE_IDENTITY_OUT_OF_DATE", _status::STATUS_SGX_ENCLAVE_IDENTITY_OUT_OF_DATE)
        .value("STATUS_SGX_ENCLAVE_REPORT_MISCSELECT_MISMATCH", _status::STATUS_SGX_ENCLAVE_REPORT_MISCSELECT_MISMATCH)
        .value("STATUS_SGX_ENCLAVE_REPORT_ATTRIBUTES_MISMATCH", _status::STATUS_SGX_ENCLAVE_REPORT_ATTRIBUTES_MISMATCH)
        .value("STATUS_SGX_ENCLAVE_REPORT_MRENCLAVE_MISMATCH", _status::STATUS_SGX_ENCLAVE_REPORT_MRENCLAVE_MISMATCH)
        .value("STATUS_SGX_ENCLAVE_REPORT_MRSIGNER_MISMATCH", _status::STATUS_SGX_ENCLAVE_REPORT_MRSIGNER_MISMATCH)
        .value("STATUS_SGX_ENCLAVE_REPORT_ISVPRODID_MISMATCH", _status::STATUS_SGX_ENCLAVE_REPORT_ISVPRODID_MISMATCH)
        .value("STATUS_SGX_ENCLAVE_REPORT_ISVSVN_OUT_OF_DATE", _status::STATUS_SGX_ENCLAVE_REPORT_ISVSVN_OUT_OF_DATE)
        .value("STATUS_UNSUPPORTED_QE_IDENTITY_FORMAT", _status::STATUS_UNSUPPORTED_QE_IDENTITY_FORMAT)
        .value("STATUS_QE_IDENTITY_OUT_OF_DATE", _status::STATUS_QE_IDENTITY_OUT_OF_DATE)
        .value("STATUS_QE_IDENTITY_MISMATCH", _status::STATUS_QE_IDENTITY_MISMATCH)
        .value("STATUS_SGX_TCB_INFO_EXPIRED", _status::STATUS_SGX_TCB_INFO_EXPIRED)
        .value("STATUS_SGX_ENCLAVE_IDENTITY_INVALID_SIGNATURE", _status::STATUS_SGX_ENCLAVE_IDENTITY_INVALID_SIGNATURE)
        .value("STATUS_INVALID_PARAMETER", _status::STATUS_INVALID_PARAMETER)
        .value("STATUS_SGX_PCK_CERT_CHAIN_EXPIRED", _status::STATUS_SGX_PCK_CERT_CHAIN_EXPIRED)
        .value("STATUS_SGX_CRL_EXPIRED", _status::STATUS_SGX_CRL_EXPIRED)
        .value("STATUS_SGX_SIGNING_CERT_CHAIN_EXPIRED", _status::STATUS_SGX_SIGNING_CERT_CHAIN_EXPIRED)
        .value("STATUS_SGX_ENCLAVE_IDENTITY_EXPIRED", _status::STATUS_SGX_ENCLAVE_IDENTITY_EXPIRED)
        .value("STATUS_TCB_SW_HARDENING_NEEDED", _status::STATUS_TCB_SW_HARDENING_NEEDED)
        .value("STATUS_TCB_CONFIGURATION_AND_SW_HARDENING_NEEDED", _status::STATUS_TCB_CONFIGURATION_AND_SW_HARDENING_NEEDED)
        .value("STATUS_SGX_ENCLAVE_REPORT_ISVSVN_REVOKED", _status::STATUS_SGX_ENCLAVE_REPORT_ISVSVN_REVOKED)
        .export_values();
}