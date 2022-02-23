// Adapted from https://github.com/Kixunil/tonic_lnd/blob/master/src/lib.rs#L165-L175
// License MITNFA

#![allow(unused)]

use anyhow::Result;
use rustls::{Certificate, RootCertStore, ServerCertVerified, ServerCertVerifier, TLSError};

pub(crate) fn client_tls_config_for_self_signed_server(
    cert: tonic::transport::Certificate,
) -> Result<tonic::transport::ClientTlsConfig> {
    let mut tls_config = rustls::ClientConfig::new();
    let certificate = Certificate(pem::parse(cert)?.contents);
    tls_config
        .dangerous()
        .set_certificate_verifier(std::sync::Arc::new(SingleCertVerifier::new(certificate)));
    tls_config.set_protocols(&["h2".into()]);
    Ok(tonic::transport::ClientTlsConfig::new().rustls_client_config(tls_config))
}

pub(crate) struct SingleCertVerifier {
    certificate: Certificate,
}

impl SingleCertVerifier {
    pub(crate) fn new(certificate: Certificate) -> SingleCertVerifier {
        SingleCertVerifier { certificate }
    }
}

/// Verify the certificate is exactly the one that was provided to SingleCertVerifier
/// Will verify :
///  - the presented certificate match the our certificate
///  -Valid for DNS entry
/// However, it WILL NOT verify :
/// - **Not Expired**
/// - OCSP data is present
impl ServerCertVerifier for SingleCertVerifier {
    fn verify_server_cert(
        &self,
        _roots: &RootCertStore,
        presented_certs: &[Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<ServerCertVerified, TLSError> {
        match presented_certs {
            [end_entity_cert] => {
                if end_entity_cert == &self.certificate {
                    webpki::EndEntityCert::from(end_entity_cert.0.as_ref())
                        .map_err(TLSError::WebPKIError)
                        // .verify_is_valid_for_dns_name(dns_name)
                        // .map_err(TLSError::WebPKIError)
                        .map(|_| ServerCertVerified::assertion())
                } else {
                    Err(rustls::TLSError::General(
                        "unexpected certificate presented".to_owned(),
                    ))
                }
            }
            [] => Err(rustls::TLSError::NoCertificatesPresented),
            [..] => Err(rustls::TLSError::General(format!(
                "more than one certificate presented (got {} certificates, expected exactly one)",
                presented_certs.len()
            ))),
        }
    }
}