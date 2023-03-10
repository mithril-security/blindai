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

#![forbid(unsafe_code)]

use std::sync::Arc;
use std::thread;
mod identity;
mod model;
mod model_store;
use crate::client_communication::Exchanger;
use anyhow::Result;
use model_store::ModelStore;
mod client_communication;
use log::debug;

// ra
use env_logger::Env;
use ring::digest;
use serde::{Deserialize, Serialize};
use serde_bytes::Bytes;
use sgx_isa::{Report, Targetinfo};

#[derive(Serialize)]
struct GetQuoteRequest {
    enclave_report: Report,
}

#[derive(Serialize)]
struct GetCollateralRequest {
    quote: Vec<u8>,
}

// "Native" Rust type for sgx_ql_qve_collateral_t
#[derive(Debug, Serialize, Deserialize, Clone)]
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

const RUNNER_ADDRESS: &str = "http://127.0.0.1:11000";

fn get_target_info() -> Result<Targetinfo> {
    Ok(ureq::post(&format!("{RUNNER_ADDRESS}/get_target_info"))
        .call()?
        .into_json()?)
}

fn get_quote(report: Report) -> Result<Vec<u8>> {
    Ok(ureq::post(&format!("{RUNNER_ADDRESS}/get_quote"))
        .send_json(GetQuoteRequest {
            enclave_report: report,
        })?
        .into_json()?)
}

fn get_collateral(quote: &[u8]) -> Result<SgxCollateral> {
    Ok(ureq::post(&format!("{RUNNER_ADDRESS}/get_collateral"))
        .send_json(GetCollateralRequest {
            quote: quote.to_vec(),
        })?
        .into_json()?)
}

fn main() -> Result<()> {
    println!("BlindAI server is running at : 0.0.0.0:9923 and 0.0.0.0:9924");

    const SERVER_NAME: &str = if cfg!(target_env = "sgx") {
        "blindai_preview"
    } else {
        "blindai_preview mock (testing)"
    };

    // Make debugging easier by enabling rust backtrace inside enclave
    std::env::set_var("RUST_BACKTRACE", "full");
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();

    let certificate_with_secret = identity::create_tls_certificate()?;
    let enclave_cert_der = Arc::new(certificate_with_secret.serialize_der()?);
    let enclave_private_key_der = certificate_with_secret.serialize_private_key_der();

    let exchanger_temp = Arc::new(Exchanger::new(
        Arc::new(ModelStore::new()),
        1_000_000_000,
        1_000_000,
    ));

    fn respond(x: &(impl Serialize + ?Sized)) -> rouille::Response {
        match serde_cbor::to_vec(&x) {
            Ok(ser_data) => rouille::Response::from_data("application/cbor", ser_data),
            Err(e) => rouille::Response::from_data(
                "application/cbor",
                serde_cbor::to_vec(&format!("{:?}", &e)).unwrap(),
            )
            .with_status_code(500),
        }
        .with_additional_header("Server", SERVER_NAME)
    }

    // Remote attestation
    // Connecting to the runner

    // Enclave held data hash
    let report_binding = digest::digest(&digest::SHA256, &enclave_cert_der);
    let mut report_data = [0u8; 64];
    report_data[0..32].copy_from_slice(report_binding.as_ref());

    cfg_if::cfg_if! {
        if #[cfg(target_env = "sgx")] {
            let target_info = get_target_info()?;
            debug!("target info = {:?} ", &target_info);
            let report = Report::for_target(&target_info, &report_data);

            let quote = get_quote(report)?;
            debug!("Attestation : Quote is {:?} ", &quote);

            let collateral = get_collateral(&quote)?;
            debug!("Attestation : Collateral is {:?} ", collateral);

            let router = {
                let enclave_cert_der = Arc::clone(&enclave_cert_der);
                move |request: &rouille::Request| {
                    rouille::router!(request,
                        (GET)(/) => {
                            debug!("Requested enclave TLS certificate");
                            respond(Bytes::new(&enclave_cert_der))
                        },
                        (GET)(/quote) => {
                            debug!("Attestation : Sending quote to client.");
                            respond(Bytes::new(&quote))
                        },
                        (GET)(/collateral) => {
                            debug!("Attestation : Sending collateral to client.");
                            respond(&collateral)
                        },
                        _ => {
                            rouille::Response::empty_404()
                        },
                    )
                }
            };
        } else {
            let router = {
                let enclave_cert_der = Arc::clone(&enclave_cert_der);
                move |request: &rouille::Request| {
                    rouille::router!(request,
                        (GET)(/) => {
                            debug!("Requested enclave TLS certificate");
                            respond(Bytes::new(&enclave_cert_der))
                        },
                        _ => {
                            rouille::Response::empty_404()
                        },
                    )
                }
            };
        }
    };

    let untrusted_server = rouille::Server::new("0.0.0.0:9923", router)
        .expect("Failed to start untrusted server")
        .pool_size(4);

    let (_untrusted_handle, _untrusted_sender) = untrusted_server.stoppable();

    let router = move |request: &rouille::Request| {
        rouille::router!(request,
            (POST) (/upload) => {
                let reply = exchanger_temp.send_model(request);
                exchanger_temp.respond(request, reply)
            },

            (POST) (/run) => {
                let reply = exchanger_temp.run_model(request);
                exchanger_temp.respond(request, reply)
            },

            (POST) (/delete) => {
                let reply = exchanger_temp.delete_model(request);
                exchanger_temp.respond(request, reply)
            },

            _ => rouille::Response::empty_404()
        )
    };
    thread::spawn({
        let enclave_cert_der = Arc::clone(&enclave_cert_der);
        move || {
            let trusted_server = rouille::Server::new_ssl(
                "0.0.0.0:9924",
                router,
                tiny_http::SslConfig::Der(tiny_http::SslConfigDer {
                    certificates: vec![enclave_cert_der.to_vec()],
                    private_key: enclave_private_key_der,
                }),
            )
            .expect("Failed to start trusted server");
            let (_trusted_handle, _trusted_sender) = trusted_server.stoppable();
            _trusted_handle.join().unwrap();
        }
    });
    _untrusted_handle.join().unwrap();

    Ok(())
}
