use std::net::SocketAddr;

use anyhow::{Context, Result};
use http::Uri;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::net::ToSocketAddrs;
use tonic_rpc::tonic_rpc;

#[derive(Deserialize, Clone, Debug)]
pub struct NetworkConfig {
    // Internal connection for Host -> Enclave communication
    #[serde(deserialize_with = "deserialize_uri")]
    pub internal_host_to_enclave_url: Uri,
    // Internal connection for Enclave -> Host communication
    #[serde(deserialize_with = "deserialize_uri")]
    pub internal_enclave_to_host_url: Uri,
    // Untrusted connection for Client -> Enclave communication
    #[serde(deserialize_with = "deserialize_uri")]
    pub client_to_enclave_untrusted_url: Uri,
    // Attested connection for Client -> Enclave communication
    #[serde(deserialize_with = "deserialize_uri")]
    pub client_to_enclave_attested_url: Uri,
    pub max_model_size: usize,
    pub max_input_size: usize,
}

fn uri_to_socket(uri: &Uri) -> Result<SocketAddr> {
    Ok(uri
        .authority()
        .context("No authority")?
        .as_str()
        .to_socket_addrs()?
        .next()
        .context("Uri could not be converted to socket")?)
}

impl NetworkConfig {
    pub fn internal_host_to_enclave_socket(&self) -> Result<SocketAddr> {
        uri_to_socket(&self.internal_host_to_enclave_url)
    }
    pub fn internal_enclave_to_host_socket(&self) -> Result<SocketAddr> {
        uri_to_socket(&self.internal_enclave_to_host_url)
    }
    pub fn client_to_enclave_untrusted_socket(&self) -> Result<SocketAddr> {
        uri_to_socket(&self.client_to_enclave_untrusted_url)
    }
    pub fn client_to_enclave_attested_socket(&self) -> Result<SocketAddr> {
        uri_to_socket(&self.client_to_enclave_attested_url)
    }
}

fn deserialize_uri<'de, D>(deserializer: D) -> Result<Uri, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<Uri>().map_err(D::Error::custom)
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

// The `tonic_rpc` attribute says that we want to build an RPC defined by this trait.
// The `json` option says that we should use the `tokio-serde` Json codec for serialization.
#[tonic_rpc(json)]
pub trait UntrustedLocalApp {
    // fn set_token(token:  String);
    fn get_collateral_from_quote(quote: Vec<u8>) -> SgxCollateral;
}

#[tonic_rpc(json)]
pub trait LocalEnclave {
    // fn get_quote() -> Vec<u8>;
}
