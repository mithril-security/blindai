use anyhow::{anyhow, bail, ensure, Error, Result};
use common::SgxCollateral;
use der_parser::{
    der::*,
    error::{self, BerError, BerResult},
    nom::combinator::map,
};
use sgx_quote::{QeCertificationData::CertChain, Quote};
use sgx_types::*;
use std::{convert::TryInto, ffi::CString, fmt::Debug, ptr, slice, str};
use x509_parser::{oid_registry::Oid, prelude::X509Certificate, traits::FromDer};

#[derive(Debug)]
enum SgxExtension {
    Fmspc([u8; 6]), // FMSPC ::= OCTET STRING (SIZE (6))
}

const FMSPC_SIZE: usize = 6; // FMSPC is 6 bytes long

/// Parse the SGX extension from X509 certificate extension for SGX
///
/// # Arguments
/// * `i` - DER encoded SGXExtentions as defined in
///    Chapter 1.5.1 Appendix A of https://api.trustedservices.intel.com/documents/Intel_SGX_PCK_Certificate_CRL_Spec-1.4.pdf
///
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

/// Convert a PCS V3.{0,1} CRL to the V1 and V2 PEM format
///
/// Motivation :
/// Intel CRLs from sgx_ql_qve_collateral_t are encoded differently depending on the version
/// of the PCS.
/// For PCS V1 and V2 APIs, the major_version = 1 and minor_version = 0 and the CRLs will be formatted in PEM.
/// For PCS V3 APIs, the major_version = 3 and the minor_version can be either 0 or 1.
/// A minor_verion of 0 indicates the CRL’s are formatted in Base16 encoded DER.
/// A minorversion of 1 indicates the CRL’s are formatted in raw binary DER.
///
fn crl_base16_to_pem(crl: &[u8]) -> String {
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
/// * `i` - DER encoded sGXExtensionValue as defined in
///    Chapter 1.5.1 Appendix A of https://api.trustedservices.intel.com/documents/Intel_SGX_PCK_Certificate_CRL_Spec-1.4.pdf
///
fn parse_sgx_extension(i: &[u8]) -> BerResult<Option<SgxExtension>> {
    parse_der_sequence_defined_g(|i: &[u8], _| {
        let (i, a) = parse_der_oid(i)?;
        let sgx_extension_id = a.as_oid()?;

        let sgx_extension_fmspc: Oid =
            Oid::from(sgx_pkix::oid::SGX_EXTENSION_FMSPC.components()).unwrap();
        let sgx_value = if sgx_extension_id == &sgx_extension_fmspc {
            let (_, fmspc) = parse_der_octetstring(i)?;
            let fmspc = fmspc.content.as_slice()?;

            if fmspc.len() != FMSPC_SIZE {
                panic!("FMSPC size should be {}, got {}", FMSPC_SIZE, fmspc.len());
            }
            Some(SgxExtension::Fmspc(
                fmspc
                    .try_into()
                    .map_err(|_| error::BerError::Custom(FMSPC_PARSING_ERROR))?,
            ))
        } else {
            None
        };
        Ok((i, sgx_value))
    })(i)
}

fn get_fmspc_ca_from_quote(quote: &[u8]) -> Result<([u8; 6], CString, String, String)> {
    // The following is basically what the internal QVL function get_fmspc_ca_from_quote does :
    // https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteVerification/QvE/Enclave/qve.cpp#L478
    let quote = Quote::parse(quote).map_err(|_| anyhow!("Quote parsing failed!"))?;

    // qe_certification_data is the Certification Data Variable Byte Array Data required
    // to verify the QE Report Signature depending on the value of the Certification Data Type:
    // . * 5: Concatenated PCK Cert Chain (PEM formatted). PCK Leaf Cert||Intermediate CA Cert|| Root CA Cert
    let cert_chain = match quote.signature {
        sgx_quote::Signature::EcdsaP256 {
            qe_certification_data: CertChain(x),
            ..
        } => x,
        _ => bail!("Unexpected quote format !"),
    };

    // Get  PCK Leaf Cert
    let pems = pem::parse_many(cert_chain)?;
    ensure!(
        pems.len() == 3,
        "Wrong number of certificates in the CertChain"
    );

    let pck_certificate = pem::encode(&pems[0]);
    let pck_signing_chain = pem::encode_many(&pems[1..]);

    let pck_cert_der = &pems[0].contents;

    let (_, pck_cert) = X509Certificate::from_der(&pck_cert_der)?;

    // If the Certificate has no SGX extension, it is the wrong certificate probably Root CA or Intermediate CA
    let sgx_extension_oid: Oid = Oid::from(sgx_pkix::oid::SGX_EXTENSION.components())
        .map_err(|e| Error::msg(format!("{:?})", e)))?;
    let sgx_ext = pck_cert
        .extensions()
        .into_iter()
        .find(|ext| ext.oid == sgx_extension_oid)
        .ok_or(anyhow!(
            "SGX extension not found in the X509 Certificate, hint: is it the"
        ))?
        .value;

    let (_, extension) = parse_sgx_extensions(&sgx_ext)?;
    let fmspc = extension
        .iter()
        .find_map(|v| {
            #[allow(unreachable_patterns)]
            match v {
                SgxExtension::Fmspc(fmspc) => Some(fmspc),
                _ => None,
            }
        })
        .ok_or(anyhow!("SGX FMSPC not found in the SGX extensions"))?;

    let issuer_cn = pck_cert
        .issuer()
        .iter_common_name()
        .next()
        .ok_or(anyhow!("No Issuer common name in pck_cert"))?
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
        pck_certificate,
        pck_signing_chain,
    ))
}

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

fn sgx_get_quote_verification_collateral(
    fmspc: &[u8; 6],
    ca_from_quote: &CString,
) -> Result<SgxQlQveCollateral> {
    // Retrieve verification collateral using QPL

    let mut pp_quote_collateral: *mut sgx_ql_qve_collateral_t = ptr::null_mut();

    let (quote_collateral, qv_ret) = unsafe {
        let qv_ret = sgx_ql_get_quote_verification_collateral(
            fmspc.as_ptr(),
            fmspc.len() as u16,
            ca_from_quote.as_ptr(),
            &mut pp_quote_collateral as *mut *mut sgx_ql_qve_collateral_t,
        );
        let quote_collateral = &*pp_quote_collateral;
        (quote_collateral, qv_ret)
    };

    ensure!(
        qv_ret == sgx_quote3_error_t::SGX_QL_SUCCESS,
        "sgx_ql_get_quote_verification_collateral failed!"
    );

    // The strings inside quote_collateral struct are described by
    // a *char and the size in bytes of the string including the terminating NULL character
    // We don't want the ending NULL character in our Rust slices so we construct the slice with the ..._size - 1
    let (
        pck_crl_issuer_chain,
        root_ca_crl,
        pck_crl,
        tcb_info_issuer_chain,
        tcb_info,
        qe_identity_issuer_chain,
        qe_identity,
    );
    unsafe {
        pck_crl_issuer_chain = slice::from_raw_parts(
            quote_collateral.pck_crl_issuer_chain as *const u8,
            quote_collateral.pck_crl_issuer_chain_size as usize - 1,
        )
        .to_owned();

        root_ca_crl = slice::from_raw_parts(
            quote_collateral.root_ca_crl as *const u8,
            quote_collateral.root_ca_crl_size as usize - 1,
        )
        .to_owned();

        pck_crl = slice::from_raw_parts(
            quote_collateral.pck_crl as *const u8,
            quote_collateral.pck_crl_size as usize - 1,
        )
        .to_owned();

        let tcb_info_issuer_chain_slice = slice::from_raw_parts(
            quote_collateral.tcb_info_issuer_chain as *const u8,
            quote_collateral.tcb_info_issuer_chain_size as usize - 1,
        );
        tcb_info_issuer_chain = str::from_utf8(tcb_info_issuer_chain_slice)?.to_owned();

        let tcb_info_slice = slice::from_raw_parts(
            quote_collateral.tcb_info as *const u8,
            quote_collateral.tcb_info_size as usize - 1,
        );
        tcb_info = str::from_utf8(tcb_info_slice)?.to_owned();

        let qe_identity_issuer_chain_slice = slice::from_raw_parts(
            quote_collateral.qe_identity_issuer_chain as *const u8,
            quote_collateral.qe_identity_issuer_chain_size as usize - 1,
        );
        qe_identity_issuer_chain = str::from_utf8(qe_identity_issuer_chain_slice)?.to_owned();

        let qe_identity_slice = slice::from_raw_parts(
            quote_collateral.qe_identity as *const u8,
            quote_collateral.qe_identity_size as usize - 1,
        );
        qe_identity = str::from_utf8(qe_identity_slice)?.to_owned();

        // We have copied every pointers to Rust owned String, so we can free the C allocated collateral struct
        let ret = sgx_ql_free_quote_verification_collateral(pp_quote_collateral);
        ensure!(
            ret == sgx_quote3_error_t::SGX_QL_SUCCESS,
            "sgx_ql_free_quote_verification_collateral failed!"
        );
    }

    let version = quote_collateral.version;
    let pck_crl_issuer_chain = crl_base16_to_pem(&pck_crl_issuer_chain);
    let root_ca_crl = crl_base16_to_pem(&root_ca_crl);
    let pck_crl = crl_base16_to_pem(&pck_crl);

    Ok(SgxQlQveCollateral {
        version: version,
        pck_crl_issuer_chain: pck_crl_issuer_chain,
        root_ca_crl: root_ca_crl,
        pck_crl: pck_crl,
        tcb_info_issuer_chain: tcb_info_issuer_chain,
        tcb_info: tcb_info,
        qe_identity_issuer_chain: qe_identity_issuer_chain,
        qe_identity: qe_identity,
    })
}
pub(crate) fn get_collateral_from_quote(quote: &[u8]) -> Result<SgxCollateral> {
    // First we need to parse the quote to extract the fmspc and the right CA
    // This will be neeeded to get the collateral from the SGX Quote provider library
    let (fmspc, ca_from_quote, pck_certificate, pck_signing_chain) =
        get_fmspc_ca_from_quote(quote)?;

    let SgxQlQveCollateral {
        version,
        pck_crl_issuer_chain,
        root_ca_crl,
        pck_crl,
        tcb_info_issuer_chain,
        tcb_info,
        qe_identity_issuer_chain,
        qe_identity,
    } = sgx_get_quote_verification_collateral(&fmspc, &ca_from_quote)?;

    Ok(SgxCollateral {
        version: version,
        pck_crl_issuer_chain: pck_crl_issuer_chain,
        root_ca_crl: root_ca_crl,
        pck_crl: pck_crl,
        tcb_info_issuer_chain: tcb_info_issuer_chain,
        tcb_info: tcb_info,
        qe_identity_issuer_chain: qe_identity_issuer_chain,
        qe_identity: qe_identity,
        // added
        pck_certificate: pck_certificate,
        pck_signing_chain: pck_signing_chain,
    })
}
