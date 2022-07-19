// Copyright 2022 Mithril Security. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::net::SocketAddr;

use anyhow::{Context, Result};
use http::Uri;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::net::ToSocketAddrs;
use tonic_rpc::tonic_rpc;

#[derive(Deserialize, Clone, Debug)]
pub struct ModelFactsConfig {
    pub datum_type: Option<String>,
    pub dims: Option<Vec<usize>>,
    pub index: Option<usize>,
    pub index_name: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LoadModelConfig {
    pub path: String,
    pub model_id: String,
    pub model_name: Option<String>,
    #[serde(default)]
    pub input_facts: Vec<ModelFactsConfig>,
    #[serde(default)]
    pub output_facts: Vec<ModelFactsConfig>,
    #[serde(default)]
    pub no_optim: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlindAIConfig {
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
    pub models_path: String,
    pub allow_sendmodel: bool,
    #[serde(default)]
    pub load_models: Vec<LoadModelConfig>,
}

fn uri_to_socket(uri: &Uri) -> Result<SocketAddr> {
    uri.authority()
        .context("No authority")?
        .as_str()
        .to_socket_addrs()?
        .next()
        .context("Uri could not be converted to socket")
}

impl BlindAIConfig {
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

// The `tonic_rpc` attribute says that we want to build an RPC defined by this
// trait. The `json` option says that we should use the `tokio-serde` Json codec
// for serialization.
#[tonic_rpc(json)]
pub trait UntrustedLocalApp {
    fn get_collateral_from_quote(quote: Vec<u8>) -> SgxCollateral;
}
