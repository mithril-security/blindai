---
description: "Improve SEO with our Intel SGX Remote Attestation guide: Learn secure enclave verification through ECDSA attestation."
---

# Remote attestation Implementation
__________________________________________

This document details the remote attestation as implemented on *BlindAI* for **intel SGX** platforms. 

The remote attestation is the process by which a remote application verifies that the code running is truly within a secure enclave. 

## Theory
__________________________________________

There is two concepts for achieving remote attestation on intel SGX. The first one, is **EPID attestation** which relies on using Intel services to attest that an enclave on specific platforms is verified. 
The second one is **DCAP (DataCenterAttestationPrimitives)** and it allows data centers to own their own attestation. 
Our implementation relies on the latter, where we have to possibility to build our own attestation infrastructure using public keys algorithm, in our case ECDSA. 

### Overall architecture 
As explained in [1] & [2], ECDSA attestation sequence relies on three different platforms to achieve the verifications needed : 
- The Intel SGX platform, 
- The Data Center Caching Service
- The target service, in our case the client. 

The intel SGX platform provides us with the necessary measurements and functions (see section below) used to generate the signatures related to the code and enclave running on it.


### Instruction needed & structures
__________________________________________

The instruction set for intel SGX defines 18 different instructions. The instructions that are used in the remote attestation are: `EGETKEY` and `EREPORT`. 
The structures that will be used on the remote attestation are `TARGETINFO` & `REPORT`. 

### Measurement 

The measurement represents the enclave's signature. In the design of the quote (signature) it includes the following data [4] :

- Measurement of the code and data in the enclave. 
- A hash of the public key in the ISV (independent software vendor) certificate presented at enclave initialization time. 
- The Product ID and the Security Version Number (SVN) of the enclave. 
- Attributes of the enclave (defining if it is in debug mode for instance).
- 64 bytes of user data included in the `REPORTDATA` of the `REPORT`. 


Quote's information : `MRENCLAVE`, `REPORT`, `MRSIGNER`

### Under the hood - How it works
Each enclave running on an SGX platform can generates its own report, based on
an ECDSA attestation key that is generated by the QE (Quoting Enclave). The key is a 256-bit ECC signing key, and it's generated as follows :  

![attestation key generation](../../assets/ak.png)

- The PCE provides an interface to retrieve the PCK certificate Identifier 
- The PCE provides a mechanism to sign another enclave (for example QE) REPORT using the PCK cert private key. 
- In DCAP attestation, the QE generates the ECDSA Attestation Key and include its hash in the REPORT structure (`QE.REPORT.ReportData`). 

This attestation key (AK) is then used to sign application enclave reports and the result is called the ECDSA quote. this generation is done on the quote generation part. There is two ways to communicate with the architectural enclaves. **The in-process mode**, where the quote generation libraries are loaded into the application's process. The **out-process mode** (___used in this implementation___), uses the AESM service to contact the different AE. 

The quote verification part, is the generation of the quote verification collateral. 

### Quote generation 
After initiating the AESM client, the ECDSA Attestation Key is retrieved by contacting the AESM service. We then use this ECDSA Key to generate the `TARGETINFO` structure. This structure is generated from the QE (Quoting Enclave) using the returned ECDSA key and always through the AESM service. 

The `TARGETINFO` is then sent to the enclave to use it to generate the `REPORT`. In fact, using the `EREPORT` instruction, and with the `TARGETINFO` passed on, we generate the `REPORT` structure that will be then used to generate the quote. 

To be able to sign the `REPORT` to obtain to quote, the quoting enclave must be used again. The return is a success if the `REPORT` is valid and generated with the right `TARGETINFO`. 


### Quote verification Collateral
In addition to the quote itself, the verifier needs some other information:

- The PCK certificate that certified the attestation Key
- The Revocation list that applies to the PCK certificate. 
- Updated SVNs for the CPU and PCE. 
- The identity of the quoting Enclave trusted to generate the attestation key and the related quotes. 

The PCK (Provisioning Certification Key) Certificate API delivers the X.509 certificates for PCKs and the public key for the PCK. In addition, it also provides custom fields with the following information :<br />

- PPID
- CPUSVN
- PCE's ISVSVN
- Family-Model-Stepping-Platform Type-CustomSKU (FMSPC)
- other additionnal structural information

(*for more information see [7]*)

The verification collateral is the data needed by the client to complete the quote verification. It's a structure including the following data : <br />

- *The root CA CRL*<br />
- *The PCK Cert CRL*<br />
- *The PCK Cert CRL signing chain*<br />
- *The signing cert chain for the TCBInfo structure*<br />
- *The signing cert chain for the QEIdentity structure*<br />
- *The TCBInfo structure*<br />
- *The QEIdentity structure* <br />

The first step to get the verification collateral is to extract the FMSPc and CA from the computed quote. 

To be able to get the verification Collateral, we use the `sgx_ql_get_quote_verification_collateral` function present in QV lib API. And the function requires the `fmspc` data and the CA which are extracted from the computed quote [6]. 

The collateral structure is then returned to the client to complete the verification process. 

### Attestation verification 
The Attestation verification is done on the client side. The quote & the verification collateral are sent to the client, which then uses the verification library to verifying with the information that are available to it. 

According to the client platform, there is two different ways to verify the remote enclave. The first one, is when using the QvE (Quote Verification Enclave) and thus the client supports an SGX platform. The Second one, is using the QvL (Quote Verification Library). Supposing that the client doesn't actually have access to an SGX platform, we chose to implement the QvL verification method, which is more suitable in this case (see the source code in the DCAP [8]). 

The library contains all the verification necessary for ECDSA-quotes generated by an Intel provided Quoting Enclave. However, it requires the use of the collateral (the PCK certificate Chain, Revocation Lists, TCB info...). 


After receiving the quote and the collateral from the server, the client then verifies it using the verify function from the QvL. It then uses the results to establish a secure TLS connection, with the certificate verified for the server. That second connection will then be used to run the inference engine. 

The sequence diagram below illustrate the different steps taken to establish the remote attestation and the secure connection between the client and the server.


## Remote attestation in Fortanix (Server-side)
__________________________________________

Our implementation in *Fortanix EDP* relies on AESM (Application Enclave Service Manager) to manage the architectural enclaves (LE, PvE, PcE, QE, PSE). The AESM service makes it possible to communicate with the architectural enclaves from the application enclave [3].  

_**to review**_ : Currently AESM is bound to the host's. We have to see how we can use it with future kubernetes deployments. Questions : What happens it multiple AESM services are run in the same time? is it possible to have multiple architectural enclaves in the same time ? 

### Quote generation

The quote generation begins by generating an AESM client using the `aesm_client` crate. 

We then call to the `get_supported_att_key_ids` method to get the ECDSA attestation key. This method is a rewrite of the official intel sgx function `sgx_get_supported_att_key_ids` as defined in [9]. 

Also to contact the AESM, the Fortanix EDP defines protobuf messages that tries to contact the service, otherwise in which case it returns an error with the error code (more information here : [*https://github.com/Fortanix/rust-sgx/blob/64100155aa8e0e9379fd66c6128e6f1605442e75/intel-sgx/aesm-client/src/imp/aesm_protobuf/mod.rs*](https://github.com/Fortanix/rust-sgx/blob/64100155aa8e0e9379fd66c6128e6f1605442e75/intel-sgx/aesm-client/src/imp/aesm_protobuf/mod.rs)). 

From the resulted array, we extract the ECDSA attestation key, identified by the constant `const SGX_QL_ALG_ECDSA_P256 : u32 = 2;` (As defined in the quote generation DCAP enum here : [*https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/quote_wrapper/common/inc/sgx_quote_3.h*](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/quote_wrapper/common/inc/sgx_quote_3.h)). The initialization of the runner context completes with the `init_quote_ex` function which, given the attestation key, returns the target info. 

According to the quote generation section above, the computed target info is sent to the enclave application by creating an HTTP communication channel between the enclave and the runner (This channel will be the same for the other data that must be delivered between the two). Hence the bind channel at port `11000` for the runner.  

The enclave then request the target info using the channel and returns report via a `POST` request to the runner. 

The last step in the generation is performed with the `get_quote` method where, using the report received is used to calculate the quote and return it in a vec format. 


### Quote verification collateral 

The quote verification collateral is not done on the Fortanix EDP. 
The overall goal was to be able to link DCAP functions to our code. These specific functions are `sgx_ql_get_quote_verification_collateral`, for the collateral, and  `sgx_ql_free_quote_verification_collateral` to free memory when the desired operations are done. These functions request the PCCS service to retrieve the PCK certificates necessary when not already cached. 


The steps to generate the collateral are the following. 
We request the `fmspc`, `ca_from_quote` (certificate inside the quote), and the `pck_signing_chain` from the quote.  

Using this, we retrieve the structure `SgxQlQveCollateral` that is used to populate the collateral `SgxCollateral`.

## Remote attestation verification in the client
__________________________________________

The goal here was to be able to have a python client that can verify the quote and collateral received from the server. So we had to rebuild the Quote Verification library with python bindings to be able to use the verification API functions. 

Moreover, when the server part is launched, a manifest file is generated. This manifest file includes some information (such as the `mrenclave`) that is needed in the verification process.  

When the unsecure TLS is established, the server calculate the quote and collateral of the current platform. The remote attestation process begin by sending the quote and collateral to the client through the insecure TLS connection. 

The client then verifies the quote and collateral against the manifest file. It does so by requesting the QvL (Quote Verification Library).

If the verification is valid, another TLS connection is established to run the models with the data securely.


## References
__________________________________________

- [1] John P Mechalas, *"Intel DCAP overview"*, 2021 : [*Quote Generation, Verification, and Attestation with Intel® Software Guard Extensions Data Center Attestation Primitives (Intel® SGX DCAP)*](https://www.intel.com/content/www/us/en/developer/articles/technical/quote-verification-attestation-with-intel-sgx-dcap.html)

- [2] Vinnie Scarlata, Simon Johnson, James Beaney, Piotr Zmijewski
Intel Corporation 2018 *"Supporting Third Party Attestation for Intel® SGX
with Intel® Data Center Attestation Primitives"* : [(*https://www.intel.com/content/dam/develop/external/us/en/documents/intel-sgx-support-for-third-party-attestation-801017.pdf*](https://www.intel.com/content/dam/develop/external/us/en/documents/intel-sgx-support-for-third-party-attestation-801017.pdf)

- [3] SSLab, Georgia Institute of Technology, 2017 *"Communication between Architectural and Application Enclaves"* : [*https://sgx101.gitbook.io/sgx101/sgx-bootstrap/enclave/interaction-between-pse-and-application-enclaves*](https://sgx101.gitbook.io/sgx101/sgx-bootstrap/enclave/interaction-between-pse-and-application-enclaves)

- [4] Intel, *Intel® Software Guard Extensions Developer Guide* : [*https://download.01.org/intel-sgx/linux-1.7/docs/Intel_SGX_Developer_Guide.pdf*](https://download.01.org/intel-sgx/linux-1.7/docs/Intel_SGX_Developer_Guide.pdf)


- [6] Intel, Linux 1.15, *Intel SGX ECDSA QuoteLibReference DCAP API* :  [*https://download.01.org/intel-sgx/sgx-dcap/1.15/linux/docs/Intel_SGX_ECDSA_QuoteLibReference_DCAP_API.pdf*](https://download.01.org/intel-sgx/sgx-dcap/1.15/linux/docs/Intel_SGX_ECDSA_QuoteLibReference_DCAP_API.pdf)

- [7] Intel, *Intel SGX PCK Certificate CRL Spec-1.4*: [*https://api.trustedservices.intel.com/documents/Intel_SGX_PCK_Certificate_CRL_Spec-1.4.pdf*](https://api.trustedservices.intel.com/documents/Intel_SGX_PCK_Certificate_CRL_Spec-1.4.pdf)

- [8] intel/SGXDataCenterAttestationPrimitives : [*https://github.com/intel/SGXDataCenterAttestationPrimitives/tree/master/QuoteVerification/QVL*](https://github.com/intel/SGXDataCenterAttestationPrimitives/tree/master/QuoteVerification/QVL)

- [9] intel SGX developer Reference linux 2.9.1: [*https://download.01.org/intel-sgx/sgx-linux/2.9.1/docs/Intel_SGX_Developer_Reference_Linux_2.9.1_Open_Source.pdf*](https://download.01.org/intel-sgx/sgx-linux/2.9.1/docs/Intel_SGX_Developer_Reference_Linux_2.9.1_Open_Source.pdf)


- [9] intel SGX developer Reference linux 2.9.1: [*https://download.01.org/intel-sgx/sgx-linux/2.9.1/docs/Intel_SGX_Developer_Reference_Linux_2.9.1_Open_Source.pdf*](https://download.01.org/intel-sgx/sgx-linux/2.9.1/docs/Intel_SGX_Developer_Reference_Linux_2.9.1_Open_Source.pdf)
