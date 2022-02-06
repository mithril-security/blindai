use tonic_rpc::tonic_rpc;

// The `tonic_rpc` attribute says that we want to build an RPC defined by this trait.
// The `json` option says that we should use the `tokio-serde` Json codec for serialization.
#[tonic_rpc(json)]
pub trait UntrustedLocalApp {
    fn set_token(token:  String);
}
