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

#![crate_name = "inference_server"]
#![crate_type = "staticlib"]
#![feature(once_cell)]

extern crate env_logger;
#[cfg(target_env = "sgx")]
extern crate sgx_libc;
#[cfg(target_env = "sgx")]
extern crate sgx_tseal;
#[cfg(target_env = "sgx")]
extern crate sgx_types;
extern crate tract_core;
extern crate tract_onnx;

extern crate serde_cbor;
extern crate serde_derive;

use env_logger::Env;
use std::backtrace::{self, PrintFormat};
use std::ffi::CStr;

use log::*;
use sgx_types::*;
use std::io::Read;
use tonic::transport::ServerTlsConfig;

use tonic::transport::{Identity, Server};

use sgx_types::sgx_status_t;

#[cfg(target_env = "sgx")]
use std::untrusted::fs::File;

#[cfg(not(target_env = "sgx"))]
use std::fs::File;

#[cfg(target_env = "sgx")]
use std::untrusted::fs;

#[cfg(not(target_env = "sgx"))]
use std::fs;

use crate::client_communication::{secured_exchange::exchange_server::ExchangeServer, Exchanger};

use crate::dcap_quote_provider::DcapQuoteProvider;
use crate::telemetry::TelemetryEventProps;

use untrusted::MyAttestation;

use identity::MyIdentity;

mod client_communication;
mod dcap_quote_provider;
mod identity;
mod telemetry;
mod untrusted;

#[no_mangle]
pub extern "C" fn start_server(
    telemetry_platform: *const c_char,
    telemetry_uid: *const c_char,
) -> sgx_status_t {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let _ = backtrace::enable_backtrace("enclave.signed.so", PrintFormat::Full);
    info!("Switched to enclave context");

    let telemetry_platform = unsafe { CStr::from_ptr(telemetry_platform) };
    let telemetry_uid = unsafe { CStr::from_ptr(telemetry_uid) };

    let telemetry_platform = telemetry_platform.to_owned().into_string().unwrap();
    let telemetry_uid = telemetry_uid.to_owned().into_string().unwrap();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main(telemetry_platform, telemetry_uid))
        .unwrap();

    sgx_status_t::SGX_SUCCESS
}

async fn main(
    telemetry_platform: String,
    telemetry_uid: String,
) -> Result<(), Box<dyn std::error::Error>> {
    teaclave_attestation::logger_info("azaz");
    let _ = backtrace::enable_backtrace("enclave.signed.so", PrintFormat::Full);
    let (certificate, storage_identity) = identity::create_certificate()?;
    let my_identity = MyIdentity::from_cert(certificate, storage_identity);
    let enclave_identity = my_identity.tls_identity.clone();

    // Read network config into network_config
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let network_config: common::NetworkConfig = toml::from_str(&contents)?;

    let dcap_quote_provider = DcapQuoteProvider::new(&enclave_identity.cert_der);
    let dcap_quote_provider: &'static DcapQuoteProvider = Box::leak(Box::new(dcap_quote_provider));

    // Identity for untrusted (non-attested) communication
    let untrusted_cert = fs::read("tls/host_server.pem")?;
    let untrusted_key = fs::read("tls/host_server.key")?;
    let untrusted_identity = Identity::from_pem(&untrusted_cert, &untrusted_key);

    tokio::spawn({
        let network_config = network_config.clone();
        async move {
            info!(
                "Starting server for User --> Enclave (unattested) untrusted communication at {}",
                network_config.client_to_enclave_untrusted_url
            );
            Server::builder()
                .tls_config(ServerTlsConfig::new().identity(untrusted_identity))?
                .add_service(untrusted::AttestationServer::new(MyAttestation {
                    quote_provider: &dcap_quote_provider,
                }))
                .serve(network_config.client_to_enclave_untrusted_socket()?)
                .await?;
            Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
        }
    });

    let exchanger = Exchanger::new(network_config.max_model_size, network_config.max_input_size);

    let server_future = Server::builder()
        .tls_config(ServerTlsConfig::new().identity((&enclave_identity).into()))?
        .max_frame_size(Some(65536))
        .add_service(ExchangeServer::new(exchanger))
        .serve(network_config.client_to_enclave_attested_socket()?);

    info!(
        "Starting server for User --> Enclave (attested TLS) trusted communication at {}",
        network_config.client_to_enclave_attested_url
    );
    println!("Server started, waiting for commands");

    if cfg!(SGX_MODE = "SW") {
        info!("Server running in simulation mode, attestation not available.");
    }

    if !std::env::var("BLINDAI_DISABLE_TELEMETRY").is_ok() {
        telemetry::setup(telemetry_platform, telemetry_uid)?;
    } else {
        debug!("Telemetry is disabled.")
    }
    telemetry::add_event(TelemetryEventProps::Started {});

    server_future.await?;

    Ok(())
}
