use std::sync::Arc;

use ureq::Agent;
/*
   The following modules `fixed_resolver` and `certificate_verifier_no_hostname` are from this
   website:
   https://scvalex.net/posts/48/
*/
mod fixed_resolver {
    //! A DNS resolver that always returns the same value regardless of
    //! the host it was queried for.

    use std::net::SocketAddr;

    pub struct FixedResolver(pub SocketAddr);

    impl ureq::Resolver for FixedResolver {
        fn resolve(&self, _netloc: &str) -> std::io::Result<Vec<SocketAddr>> {
            Ok(vec![self.0])
        }
    }
}
mod certificate_verifier_no_hostname {
    use rustls::{
        client::{ServerCertVerified, ServerCertVerifier},
        Certificate, Error as TlsError, ServerName,
    };
    use std::time::SystemTime;
    use webpki::{EndEntityCert, SignatureAlgorithm, TlsServerTrustAnchors};

    pub struct CertificateVerifierNoHostname<'a> {
        pub trustroots: &'a TlsServerTrustAnchors<'a>,
    }

    static SUPPORTED_SIG_ALGS: &[&SignatureAlgorithm] = &[
        &webpki::ECDSA_P256_SHA256,
        &webpki::ECDSA_P256_SHA384,
        &webpki::ECDSA_P384_SHA256,
        &webpki::ECDSA_P384_SHA384,
        &webpki::ED25519,
        &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
        &webpki::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
        &webpki::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
        &webpki::RSA_PKCS1_2048_8192_SHA256,
        &webpki::RSA_PKCS1_2048_8192_SHA384,
        &webpki::RSA_PKCS1_2048_8192_SHA512,
        &webpki::RSA_PKCS1_3072_8192_SHA384,
    ];

    impl ServerCertVerifier for CertificateVerifierNoHostname<'_> {
        /// Will verify the certificate is valid in the following ways:
        /// - Signed by a valid root
        /// - Not Expired
        ///
        /// Based on a https://github.com/ctz/rustls/issues/578#issuecomment-816712636
        fn verify_server_cert(
            &self,
            end_entity: &Certificate,
            intermediates: &[Certificate],
            _server_name: &ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            ocsp_response: &[u8],
            _now: SystemTime,
        ) -> Result<ServerCertVerified, TlsError> {
            let end_entity_cert = webpki::EndEntityCert::try_from(end_entity.0.as_ref())
                .map_err(|err| TlsError::General(err.to_string()))?;

            let chain: Vec<&[u8]> = intermediates.iter().map(|cert| cert.0.as_ref()).collect();

            // Validate the certificate is valid, signed by a trusted root, and not
            // expired.
            let now = SystemTime::now();
            let webpki_now =
                webpki::Time::try_from(now).map_err(|_| TlsError::FailedToGetCurrentTime)?;

            let _cert: EndEntityCert = end_entity_cert
                .verify_is_valid_tls_server_cert(
                    SUPPORTED_SIG_ALGS,
                    self.trustroots,
                    &chain,
                    webpki_now,
                )
                .map_err(|err| TlsError::General(err.to_string()))
                .map(|_| end_entity_cert)?;

            if !ocsp_response.is_empty() {
                //trace!("Unvalidated OCSP response: {:?}", ocsp_response.to_vec());
            }
            Ok(ServerCertVerified::assertion())
        }
    }
}

pub struct InternalAgent(Agent);

impl InternalAgent {
    pub fn new(ip: &str, port: &str) -> Self {
        use certificate_verifier_no_hostname::CertificateVerifierNoHostname;
        use fixed_resolver::FixedResolver;

        let mut root_store = rustls::RootCertStore::empty();

        // This adds webpki_roots certs.
        root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        // See rustls documentation for more configuration options.
        let tls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(CertificateVerifierNoHostname {
                trustroots: &webpki_roots::TLS_SERVER_ROOTS,
            }))
            .with_no_client_auth();

        // Build a ureq agent with the rustls config.
        let agent = ureq::builder()
            .tls_config(Arc::new(tls_config))
            .resolver(FixedResolver(format!("{ip}:{port}").parse().unwrap()))
            .build();

        Self(agent)
    }
}

impl std::ops::Deref for InternalAgent {
    type Target = Agent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
