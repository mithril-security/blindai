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

        // Build a ureq agent with the rustls config.
        let agent = ureq::builder()
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
