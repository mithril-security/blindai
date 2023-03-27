use std::sync::Arc;
use ureq::Agent;

mod fixed_resolver {

    //! A DNS resolver that always returns the same value regardless of
    //! the host it was queried for.
    //!
    //! NB: The following modules `fixed_resolver` is from this website:
    //! https://scvalex.net/posts/48/
    use std::net::SocketAddr;

    pub struct FixedResolver(pub SocketAddr);

    impl ureq::Resolver for FixedResolver {
        fn resolve(&self, _netloc: &str) -> std::io::Result<Vec<SocketAddr>> {
            Ok(vec![self.0])
        }
    }
}

pub struct InternalAgent(Agent);

impl InternalAgent {
    pub fn new(ip: &str, port: &str) -> Self {
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
            .with_root_certificates(root_store)
            // .with_custom_certificate_verifier(Arc::new(cert_verifier))
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
