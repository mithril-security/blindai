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

#ifndef SGXECDSAATTESTATION_ENCLAVEIDENTITYGENERATOR_H
#define SGXECDSAATTESTATION_ENCLAVEIDENTITYGENERATOR_H

#include "QuoteGenerator.h"

#include <OpensslHelpers/Bytes.h>
#include <Verifiers/EnclaveIdentityV2.h>
#include <QuoteVerification/ByteOperands.h>

#include <random>
#include <string>
#include <array>
#include <vector>

namespace
{
    uint8_t getRandomNumber()
    {
        return (uint8_t)((rand() % 9) + 1);
    }

    std::vector<uint8_t> generateRandomUint8Vector(std::size_t SIZE)
    {
        std::vector<uint8_t> vector;
        std::default_random_engine generator;
        std::uniform_int_distribution<unsigned int > distribution(0, UINT8_MAX);
        for(size_t i = 0; i < SIZE; i++)
        {
            vector.push_back(static_cast<unsigned char>(distribution(generator)));
        }
        return vector;
    }
}

namespace intel { namespace sgx { namespace dcap { namespace test {

class EnclaveIdentityVectorModel {
public:
    int version;
    std::string issueDate;
    std::string nextUpdate;
    std::vector<uint8_t> miscselect;
    std::vector<uint8_t> miscselectMask;
    std::vector<uint8_t> attributes;
    std::vector<uint8_t> attributesMask;

    std::vector<uint8_t> mrsigner;
    uint8_t isvprodid;

    // V1 only
    uint8_t isvsvn;

    // V2 only
    intel::sgx::dcap::EnclaveID id;
    unsigned int tcbEvaluationDataNumber;
    std::vector<intel::sgx::dcap::TCBLevel> tcbLevels;

    EnclaveIdentityVectorModel() {
        version = 1;
        issueDate = "2018-08-22T12:00:00Z";
        nextUpdate = "2029-08-22T12:00:00Z";

        isvprodid = getRandomNumber();
        isvsvn = getRandomNumber();
        attributes = generateRandomUint8Vector(16);
        mrsigner = generateRandomUint8Vector(32);
        miscselect = generateRandomUint8Vector(4);

        miscselectMask = miscselect;
        attributesMask = attributes;
    }

    std::string toJSON();
    void applyTo(intel::sgx::dcap::test::QuoteGenerator::EnclaveReport& enclaveReport);
};

class EnclaveIdentityStringModel
{
public:
    std::string version;
    std::string issueDate;
    std::string nextUpdate;
    std::string miscselect;
    std::string miscselectMask;
    std::string attributes;
    std::string attributesMask;

    std::string mrsigner;
    std::string isvprodid;
    std::string isvsvn;

    EnclaveIdentityStringModel() : EnclaveIdentityStringModel(EnclaveIdentityVectorModel())
    {}

    explicit EnclaveIdentityStringModel(EnclaveIdentityVectorModel vectorModel) {
        version = std::to_string(vectorModel.version);
        issueDate = vectorModel.issueDate;
        nextUpdate = vectorModel.nextUpdate;
        miscselect = bytesToHexString(vectorModel.miscselect);
        miscselectMask = bytesToHexString(vectorModel.miscselectMask);
        attributes = bytesToHexString(vectorModel.attributes);
        mrsigner = bytesToHexString(vectorModel.mrsigner);
        isvprodid = std::to_string(vectorModel.isvprodid);
        isvsvn = std::to_string(vectorModel.isvsvn);
    }

    std::string toJSON();
};

uint32_t vectorToUint32(const std::vector<uint8_t> &input);

const std::string validQEIdentityTemplate = R"json({
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
        })json";

const std::string validEnclaveIdentityTemplate = R"json({
            "id": "QE",
            "version": 2,
            "issueDate": "2018-10-04T11:10:45Z",
            "nextUpdate": "2019-06-21T12:36:02Z",
            "tcbEvaluationDataNumber":0,
            "miscselect": "8fa64472",
            "miscselectMask": "0000fffa",
            "attributes": "1254863548af4a6b2fcc2d3244784452",
            "attributesMask": "ffffffffffffffffffffffffffffffff",
            "mrsigner": "aaff34ffa51981951a61d616b16c16f1651c6516e51f651d26a6166ed5679c79",
            "isvprodid": 3,
            "tcbLevels": [
                {
                    "tcb":{ "isvsvn":8 },
                    "tcbDate":"2019-06-23T10:41:29Z",
                    "tcbStatus":"UpToDate"
                },
                {
                    "tcb":{ "isvsvn":7 },
                    "tcbDate":"2019-06-23T10:41:29Z",
                    "tcbStatus":"OutOfDate"
                },
                {
                    "tcb":{ "isvsvn":6 },
                    "tcbDate":"2019-06-23T10:41:29Z",
                    "tcbStatus":"ConfigurationNeeded"
                },
                {
                    "tcb":{ "isvsvn":5 },
                    "tcbDate":"2019-06-23T10:41:29Z",
                    "tcbStatus":"OutOfDateConfigurationNeeded"
                },
                {
                    "tcb":{ "isvsvn":4 },
                    "tcbDate":"2019-06-23T10:41:29Z",
                    "tcbStatus":"Revoked"
                }
            ]
        })json";

const std::string validSignatureTemplate = "fb1530326344ee4baded1120a7a07b1c7c46941cf5f8abff36a63492610e17f5b9d0f8f8b4b9bf06932e1220a74b72e2ab27d14d8bbfe69334046b38363bb568";

std::string qeIdentityJsonWithSignature(const std::string &qeIdentityBody = validQEIdentityTemplate, const std::string &signature = validSignatureTemplate);
std::string enclaveIdentityJsonWithSignature(const std::string &qeIdentityBody = validEnclaveIdentityTemplate, const std::string &signature = validSignatureTemplate);
void removeWordFromString(std::string word, std::string &input);

}}}}

#endif //SGXECDSAATTESTATION_ENCLAVEIDENTITYGENERATOR_H