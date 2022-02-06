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


#include <algorithm>
#include <ios>
#include <sstream>
#include "EnclaveIdentityGenerator.h"

namespace intel { namespace sgx { namespace dcap { namespace test {

static std::string createEnclaveIdentityJSON(const std::string &version,
                                             const std::string &issueDate,
                                             const std::string &nextUpdate,
                                             const std::string &miscselect,
                                             const std::string &miscselectMask,
                                             const std::string &attributes,
                                             const std::string &attributesMask,
                                             const std::string &mrsigner,
                                             const std::string &isvprodid,
                                             const std::string &isvsvn)
{
    std::string result;
    result =  R"({"version":)" + version + R"(,"issueDate":")" + issueDate + R"(","nextUpdate":")" + nextUpdate;
    result += R"(","miscselect":")" + miscselect + R"(","miscselectMask":")" + miscselectMask;
    result += R"(","attributes":")" + attributes + R"(","attributesMask":")" + attributesMask;
    result += R"(","mrsigner":")" + mrsigner + R"(","isvprodid":)" + isvprodid;
    result += R"(,"isvsvn":)" + isvsvn + R"(})";
    return result;
}

std::string EnclaveIdentityVectorModel::toJSON()
{
    return createEnclaveIdentityJSON(std::to_string(version),
                                     issueDate,
                                     nextUpdate,
                                     bytesToHexString(miscselect),
                                     bytesToHexString(miscselectMask),
                                     bytesToHexString(attributes),
                                     bytesToHexString(attributesMask),
                                     bytesToHexString(mrsigner),
                                     std::to_string(isvprodid),
                                     std::to_string(isvsvn)
    );
}

void EnclaveIdentityVectorModel::applyTo(intel::sgx::dcap::test::QuoteGenerator::EnclaveReport& enclaveReport)
{
    std::copy_n(attributes.begin(), enclaveReport.attributes.size(), enclaveReport.attributes.begin());
    std::copy_n(mrsigner.begin(), enclaveReport.mrSigner.size(), enclaveReport.mrSigner.begin());
    enclaveReport.miscSelect = vectorToUint32(miscselect);
    enclaveReport.isvSvn = isvsvn;
    enclaveReport.isvProdID = isvprodid;
}

std::string EnclaveIdentityStringModel::toJSON()
{
    return createEnclaveIdentityJSON(version,
                                     issueDate,
                                     nextUpdate,
                                     miscselect,
                                     miscselectMask,
                                     attributes,
                                     attributesMask,
                                     mrsigner,
                                     isvprodid,
                                     isvsvn
    );
}

std::string qeIdentityJsonWithSignature(const std::string &qeIdentityBody, const std::string &signature)
{
    return R"({"qeIdentity":)" + qeIdentityBody + R"(,"signature":")" + signature + R"("})";
}

std::string enclaveIdentityJsonWithSignature(const std::string &enclaveIdentityBody, const std::string &signature)
{
    return R"({"enclaveIdentity":)" + enclaveIdentityBody + R"(,"signature":")" + signature + R"("})";
}

uint32_t vectorToUint32(const std::vector<uint8_t> &input) {
    auto position = input.cbegin();
    return intel::sgx::dcap::swapBytes(intel::sgx::dcap::toUint32(*position, *(std::next(position)),
                                                                *(std::next(position, 2)), *(std::next(position, 3))));
}

void removeWordFromString(std::string word, std::string &input)
{
    while (input.find(word) != std::string::npos)
        input.replace(input.find(word), word.length(), "");
}

}}}}