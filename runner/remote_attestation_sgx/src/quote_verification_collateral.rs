use anyhow::{anyhow, ensure, Context, Error, Result};
use dcap_ql::{
    quote::{Qe3CertDataPckCertChain, Quote, Quote3SignatureEcdsaP256},
    Quote3Error,
};
use der_parser::der::{
    parse_der_octetstring, parse_der_oid, parse_der_sequence_defined_g, parse_der_sequence_of_v,
};
use der_parser::error::{BerError, BerResult};
use der_parser::nom::combinator::map;
use serde::{Deserialize, Serialize};

use std::{
    ffi::{c_char, CString},
    ptr, slice, str,
};
use x509_parser::{oid_registry::Oid, prelude::FromDer, prelude::X509Certificate};

#[derive(Debug)]
pub struct VerificationContext {
    pub quote_slice: Vec<u8>,
    pub fmspc: [u8; 6],
    pub ca_from_quote: CString,
    pub pck_certificate: String,
    pub pck_signing_chain: String,
}

#[derive(Debug)]
pub enum VerificationError {
    IO(std::io::Error),
    Serialization(std::boxed::Box<bincode::ErrorKind>),
    IntegrityError,
    SigstructMismatched,
    EnclaveInDebugMode,
    EnclaveNotTrusted,
}

impl From<anyhow::Error> for VerificationError {
    fn from(err: anyhow::Error) -> Self {
        err.into()
    }
}

impl std::convert::From<std::io::Error> for VerificationError {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl std::convert::From<std::boxed::Box<bincode::ErrorKind>> for VerificationError {
    fn from(e: std::boxed::Box<bincode::ErrorKind>) -> Self {
        Self::Serialization(e)
    }
}
impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for VerificationError {}

pub type VerificationResult<T> = Result<T, VerificationError>;

impl VerificationContext {
    pub fn init(quote: &[u8]) -> VerificationResult<Self> {
        let (fmspc, ca_from_quote, pck_certificate, pck_signing_chain) =
            get_fmspc_ca_from_quote(quote)?;
        Ok(Self {
            quote_slice: quote.to_vec(),
            fmspc,
            ca_from_quote,
            pck_certificate,
            pck_signing_chain,
        })
    }

    // l'erreur du result doit être modifiée sur Quote3Error
    pub fn get_quote_verification_collateral(
        &self,
    ) -> Result<SgxQlQveCollateral, VerificationError> {
        Ok(sgx_get_quote_verification_collateral(
            &self.fmspc,
            &self.ca_from_quote,
        )?)
    }
}

/// Get SGX ECDSA attestation collateral from an SGX quote
///
/// The verification collateral is the data required needed by the client to
/// complete the quote verification. It includes:
/// * The root CA CRL
/// * The PCK Cert CRL
/// * The PCK Cert CRL signing chain.
/// * The signing cert chain for the TCBInfo structure
/// * The signing cert chain for the QEIdentity structure
/// * The TCBInfo structure
/// * The QEIdentity structure
pub fn get_collateral_from_quote(quote: &[u8]) -> Result<SgxCollateral> {
    let verificationcontext = VerificationContext::init(quote)?;

    let SgxQlQveCollateral {
        version,
        pck_crl_issuer_chain,
        root_ca_crl,
        pck_crl,
        tcb_info_issuer_chain,
        tcb_info,
        qe_identity_issuer_chain,
        qe_identity,
    } = verificationcontext.get_quote_verification_collateral()?;
    Ok(SgxCollateral {
        version,
        pck_crl_issuer_chain,
        root_ca_crl,
        pck_crl,
        tcb_info_issuer_chain,
        tcb_info,
        qe_identity_issuer_chain,
        qe_identity,
        pck_certificate: verificationcontext.pck_certificate,
        pck_signing_chain: verificationcontext.pck_signing_chain,
    })
}

// Linking with dcap prov
#[repr(C)]
pub struct sgx_ql_qve_collateral_t {
    pub version: u32, // version = 1.  PCK Cert chain is in the Quote.
    pub pck_crl_issuer_chain: *mut c_char,
    pub pck_crl_issuer_chain_size: u32,
    pub root_ca_crl: *mut c_char, // Root CA CRL
    pub root_ca_crl_size: u32,
    pub pck_crl: *mut c_char, // PCK Cert CRL
    pub pck_crl_size: u32,
    pub tcb_info_issuer_chain: *mut c_char,
    pub tcb_info_issuer_chain_size: u32,
    pub tcb_info: *mut c_char, // TCB Info structure
    pub tcb_info_size: u32,
    pub qe_identity_issuer_chain: *mut c_char,
    pub qe_identity_issuer_chain_size: u32,
    pub qe_identity: *mut c_char, // QE Identity Structure
    pub qe_identity_size: u32,
}

//#[link(name="dcap_quoteprov")]
extern "C" {
    pub fn sgx_ql_get_quote_verification_collateral(
        fmspc: *const u8,
        fmspc_size: u16,
        pck_ra: *const c_char,
        pp_quote_collateral: *mut *mut sgx_ql_qve_collateral_t,
    ) -> Quote3Error;
    pub fn sgx_ql_free_quote_verification_collateral(
        p_quote_collateral: *const sgx_ql_qve_collateral_t,
    ) -> Quote3Error;
}

#[derive(Debug)]
enum SgxExtension {
    Fmspc([u8; 6]), // FMSPC ::= OCTET STRING (SIZE (6))
}

const FMSPC_SIZE: usize = 6; // FMSPC is 6 bytes long

/// Parse the SGX extension from X509 certificate extension for SGX
///
/// # Arguments
/// * `i` - DER encoded SGXExtentions as defined in Chapter 1.5.1 Appendix A of
///  <https://api.trustedservices.intel.com/documents/Intel_SGX_PCK_Certificate_CRL_Spec-1.4.pdf>
fn parse_sgx_extensions(
    i: &[u8],
) -> Result<(&[u8], Vec<SgxExtension>), der_parser::nom::Err<BerError>> {
    // SGXExtentions ::= SEQUENCE SIZE (1..MAX) OF SEQUENCE {
    // sGXExtensionId SGXExtensionId,
    // sGXExtensionValue ANY DEFINED BY sGXExtensionId }
    map(parse_der_sequence_of_v(parse_sgx_extension), |l| {
        l.into_iter().flatten().collect()
    })(i)
}

/// Convert any PCS CRL version to the V1/V2 PEM format
///
/// Motivation :
/// Intel CRLs from sgx_ql_qve_collateral_t are encoded differently depending on
/// the version of the PCS.
/// * For PCS V1 and V2 APIs, the major_version = 1 and minor_version = 0 and
///   the CRLs will be formatted in PEM.
/// * For PCS V3 APIs, the major_version = 3 and the minor_version can be either
///   0 or 1. A minor_verion of 0 indicates the CRL’s are formatted in Base16
///   encoded DER. A minor version of 1 indicates the CRL’s are formatted in raw
///   binary DER.
fn pcs_crl_to_pem(crl: &[u8]) -> String {
    // if it is already in PEM format (format V1/V2)
    if pem::parse(crl).is_ok() {
        return str::from_utf8(crl).unwrap().to_string();
    }

    // try to decode as base16, if it fails we assume we've got the raw binary DER
    let raw_bytes_crl = match base16::decode(crl) {
        Err(
            base16::DecodeError::InvalidByte { .. } | base16::DecodeError::InvalidLength { .. },
        ) => crl.to_owned(),
        Ok(decoded) => decoded,
    };

    pem::encode(&pem::Pem {
        tag: "X509 CRL".to_owned(),
        contents: raw_bytes_crl,
    })
}

const FMSPC_PARSING_ERROR: u32 = 1;
/// Parse an SGX extension
///
/// Only the FMSPC value is extracted, the other extensions are ignored.
///
/// # Arguments
/// * `i` - DER encoded sGXExtensionValue as defined in Chapter 1.5.1 Appendix A
///   of <https://api.trustedservices.intel.com/documents/Intel_SGX_PCK_Certificate_CRL_Spec-1.4.pdf>
fn parse_sgx_extension(i: &[u8]) -> BerResult<Option<SgxExtension>> {
    parse_der_sequence_defined_g(|i: &[u8], _| {
        let (i, a) = parse_der_oid(i)?;
        let sgx_extension_id = a.as_oid()?;

        let sgx_extension_fmspc: Oid =
            Oid::from(sgx_pkix::oid::SGX_EXTENSION_FMSPC.components()).unwrap();
        let sgx_value = if sgx_extension_id == &sgx_extension_fmspc {
            let (_, fmspc) = parse_der_octetstring(i)?;
            let fmspc = fmspc.content.as_slice()?;

            assert!(
                fmspc.len() == FMSPC_SIZE,
                "FMSPC size should be {}, got {}",
                FMSPC_SIZE,
                fmspc.len()
            );

            Some(SgxExtension::Fmspc(
                fmspc
                    .try_into()
                    .map_err(|_| BerError::Custom(FMSPC_PARSING_ERROR))?,
            ))
        } else {
            None
        };
        Ok((i, sgx_value))
    })(i)
}

// "Native" Rust type for sgx_ql_qve_collateral_t
#[derive(Debug, Serialize, Deserialize)]
pub struct SgxCollateral {
    pub version: u32,                  // version = 1.  PCK Cert chain is in the Quote.
    pub pck_crl_issuer_chain: String,  // PCK CRL Issuer Chain in PEM format
    pub root_ca_crl: String,           // Root CA CRL in PEM format
    pub pck_crl: String,               // PCK Cert CRL in PEM format
    pub tcb_info_issuer_chain: String, // PEM
    pub tcb_info: String,              // TCB Info structure
    pub qe_identity_issuer_chain: String, // PEM
    pub qe_identity: String,           // QE Identity Structure
    pub pck_certificate: String,       // PCK certificate in PEM format
    pub pck_signing_chain: String,     // PCK signing chain in PEM format
}

#[derive(Debug)]
pub struct SgxQlQveCollateral {
    pub version: u32,                  // version = 1.  PCK Cert chain is in the Quote.
    pub pck_crl_issuer_chain: String,  // PCK CRL Issuer Chain in PEM format
    pub root_ca_crl: String,           // Root CA CRL in PEM format
    pub pck_crl: String,               // PCK Cert CRL in PEM format
    pub tcb_info_issuer_chain: String, // PEM
    pub tcb_info: String,              // TCB Info structure
    pub qe_identity_issuer_chain: String, // PEM
    pub qe_identity: String,           // QE Identity Structure
}

/// Safe wrapper around FFI C QV library to get quote collateral
pub fn sgx_get_quote_verification_collateral(
    fmspc: &[u8; 6],
    ca_from_quote: &CString,
) -> Result<SgxQlQveCollateral> {
    // Retrieving verification collateral using QPL
    let mut p_quote_collateral: *mut sgx_ql_qve_collateral_t = ptr::null_mut();
    let qv_ret = unsafe {
        sgx_ql_get_quote_verification_collateral(
            fmspc.as_ptr(),
            fmspc.len() as u16,
            ca_from_quote.as_ptr(),
            &mut p_quote_collateral as *mut *mut sgx_ql_qve_collateral_t,
        )
    };

    ensure!(
        qv_ret == Quote3Error::Success,
        "sgx_ql_get_quote_verification_collateral failed!"
    );
    // SAFETY : p_quote_collateral points to a sgx_ql_qve_collateral_t variable
    // allocated by the C library via sgx_ql_get_quote_verification_collateral
    // It lives until we call sgx_ql_free_quote_verification_collateral, therefore
    // we can dereference it

    // The strings inside the sgx_ql_qve_collateral_t struct are described by a
    // *char and the size in bytes of the string including the terminating NULL
    // character. We don't want the ending NULL character in our Rust slices so we
    // construct the slice with the ..._size - 1
    // The slice content is then copied to Rust strings / Vec<u8>, so that the C QV
    // library can latter free the "C" allocated strings

    let pck_crl_issuer_chain = unsafe {
        slice::from_raw_parts(
            (*p_quote_collateral).pck_crl_issuer_chain as *const u8,
            (*p_quote_collateral).pck_crl_issuer_chain_size as usize - 1,
        )
        .to_owned()
    };

    let root_ca_crl = unsafe {
        slice::from_raw_parts(
            (*p_quote_collateral).root_ca_crl as *const u8,
            (*p_quote_collateral).root_ca_crl_size as usize - 1,
        )
        .to_owned()
    };

    let pck_crl = unsafe {
        slice::from_raw_parts(
            (*p_quote_collateral).pck_crl as *const u8,
            (*p_quote_collateral).pck_crl_size as usize - 1,
        )
        .to_owned()
    };

    let tcb_info_issuer_chain = {
        let slice = unsafe {
            slice::from_raw_parts(
                (*p_quote_collateral).tcb_info_issuer_chain as *const u8,
                (*p_quote_collateral).tcb_info_issuer_chain_size as usize - 1,
            )
        };
        str::from_utf8(slice)?.to_owned()
    };

    let tcb_info = {
        let slice = unsafe {
            slice::from_raw_parts(
                (*p_quote_collateral).tcb_info as *const u8,
                (*p_quote_collateral).tcb_info_size as usize - 1,
            )
        };
        str::from_utf8(slice)?.to_owned()
    };

    let qe_identity_issuer_chain = {
        let slice = unsafe {
            slice::from_raw_parts(
                (*p_quote_collateral).qe_identity_issuer_chain as *const u8,
                (*p_quote_collateral).qe_identity_issuer_chain_size as usize - 1,
            )
        };
        str::from_utf8(slice)?.to_owned()
    };

    let qe_identity = {
        let slice = unsafe {
            slice::from_raw_parts(
                (*p_quote_collateral).qe_identity as *const u8,
                (*p_quote_collateral).qe_identity_size as usize - 1,
            )
        };
        str::from_utf8(slice)?.to_owned()
    };

    let version = unsafe { (*p_quote_collateral).version };

    let pck_crl_issuer_chain = pcs_crl_to_pem(&pck_crl_issuer_chain);
    let root_ca_crl = pcs_crl_to_pem(&root_ca_crl);
    let pck_crl = pcs_crl_to_pem(&pck_crl);

    // SAFETY: C-FFI call to free the allocated sgx_ql_qve_collateral_t
    let ret = unsafe { sgx_ql_free_quote_verification_collateral(p_quote_collateral) };

    ensure!(
        ret == Quote3Error::Success,
        "sgx_ql_free_quote_verification_collateral failed!"
    );

    Ok(SgxQlQveCollateral {
        version,
        pck_crl_issuer_chain,
        root_ca_crl,
        pck_crl,
        tcb_info_issuer_chain,
        tcb_info,
        qe_identity_issuer_chain,
        qe_identity,
    })
}

/// Function to extract the FMSPc and CA from a given quote with cert type 5
pub fn get_fmspc_ca_from_quote(quote: &[u8]) -> Result<([u8; 6], CString, String, String)> {
    // The following is basically what the internal QVL function
    // get_fmspc_ca_from_quote does :
    // <https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteVerification/QvE/Enclave/qve.cpp#L478>
    let quote = Quote::parse(quote).map_err(|_| anyhow!("Quote parsing failed!"))?;

    // qe_certification_data is the Certification Data Variable Byte Array Data
    // required to verify the QE Report Signature depending on the value of the
    // Certification Data Type: . * 5: Concatenated PCK Cert Chain (PEM
    // formatted). PCK Leaf Cert||Intermediate CA Cert|| Root CA Cert

    // In Fortanix the certChain function doesn't exist. And thus must be implemented
    // let cert_chain_sig = cert_chain.parse(EcdsaP256, quote.signature)
    // The certification data is in quote.certification_data()
    // It returns the PCK (platform certification public key) in DER format
    let signature: Quote3SignatureEcdsaP256 = quote.signature().unwrap();

    //ajouter un match pour s'assurer du certification_type ici dans le code final du collateral
    let certification_data = signature.certification_data::<Qe3CertDataPckCertChain>();

    //transforming the certification into pems
    let cert_chain_data = certification_data.unwrap();
    let cert_chain : Vec<String> = cert_chain_data.certs.iter().map(|c| c.to_string()).collect();
    
    ensure!(
        cert_chain.len() == 3,
        "Wrong number of certificates in the CertChain"
    );

    let pck_certificate = &cert_chain[0];
    let pck_signing_chain = cert_chain[1..].join("\n");

    let pck_cert_der = pem::parse(&pck_certificate)?.contents;

    let (_, pck_cert) = X509Certificate::from_der(&pck_cert_der)?;

    let sgx_extension_oid: Oid = Oid::from(sgx_pkix::oid::SGX_EXTENSION.components())
        .map_err(|e| Error::msg(format!("{e:?})")))?;
    let sgx_ext = pck_cert
        .extensions()
        .iter()
        .find(|ext| ext.oid == sgx_extension_oid)
        .context(
            "SGX extension not found in the X509 Certificate, hint: is it the wrong certificate \
            (expecting the PCK cert but maybe got the Root CA, or the Intermediate CA cert instead) ?",
        )?
        .value;

    let (_, extension) = parse_sgx_extensions(sgx_ext)?;
    let fmspc = extension
        .iter()
        .find_map(|v| {
            #[allow(unreachable_patterns)]
            match v {
                SgxExtension::Fmspc(fmspc) => Some(fmspc),
                _ => None,
            }
        })
        .context("SGX FMSPC not found in the SGX extensions")?;

    let issuer_cn = pck_cert
        .issuer()
        .iter_common_name()
        .next()
        .context("No Issuer common name in pck_cert")?
        .as_str()?;

    let ca_from_quote = if issuer_cn.contains("Processor") {
        Ok("processor")
    } else if issuer_cn.contains("Platform") {
        Ok("platform")
    } else {
        Err(anyhow!(
            "Found issuer name {:?}, expected to find an issuer with processor or platform",
            issuer_cn
        ))
    }?;

    let ca_from_quote = CString::new(ca_from_quote)?;
    Ok((
        fmspc.to_owned(),
        ca_from_quote,
        pck_certificate.to_string(),
        pck_signing_chain,
    ))
}
