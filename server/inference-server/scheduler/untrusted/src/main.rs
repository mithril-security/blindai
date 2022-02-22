// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

extern crate sgx_types;
extern crate sgx_urts;
extern crate teaclave_attestation;

use std::sync::{Arc, Mutex};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ffi::{CString};
use std::os::raw::c_char;

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

use env_logger::Env;
use log::{error, info};

use sgx_types::*;
use sgx_urts::SgxEnclave;

use rpc::*;

use std::{fs, fs::File};
use std::env;
use std::io::{Error, ErrorKind, Read};
use common::{untrusted_local_app_server, *};

use tonic::transport::Certificate;
use tonic::{
    transport::{Channel, Identity, Server, ServerTlsConfig}, Response, Status,
};

use anyhow::Result;
use self_signed_tls::client_tls_config_for_self_signed_server;

mod dcap;
mod self_signed_tls;


static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern {
    fn start_server(eid: sgx_enclave_id_t, retval: *mut sgx_status_t, telemetry_platform: *const c_char, telemetry_uid: *const c_char) -> sgx_status_t;
}

#[derive(Default)]
pub struct MyAttestation {
    token: Arc<Mutex<String>>
}

#[derive(Default)]
pub struct State {}

#[tonic::async_trait]
impl untrusted_local_app_server::UntrustedLocalApp for State {
    // The request type gets wrapped in a `tonic::Request`.
    // The response type gets wrapped in a `Result<tonic::Response<_>, tonic::Status>`.
    async fn get_collateral_from_quote(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<SgxCollateral>, tonic::Status> {
        dcap::get_collateral_from_quote(&request.into_inner())
            .map(tonic::Response::new)
            .map_err(|e| tonic::Status::internal(e.to_string()))
    }
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create(ENCLAVE_FILE,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let sgx_mode = match env::var_os("SGX_MODE") {
        Some(v) => v.into_string().unwrap(),
        None => "HW".to_string()
    };

    let enclave = match init_enclave() {
        Ok(r) => {
            info!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            error!("[-] Init Enclave Failed {}!", x.as_str());
            return Ok(());
        },
    };

    // Read network config
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let network_config: common::NetworkConfig = toml::from_str(&contents)?;
     
    info!(
        "Starting server for Enclave --> Host internal communication at {}",
        network_config.internal_enclave_to_host_url
    );
    tokio::spawn(
        Server::builder()
            .add_service(untrusted_local_app_server::UntrustedLocalAppServer::new(
                State {},
            ))
            .serve(network_config.internal_enclave_to_host_socket()?),
    );

    let platform: CString = CString::new(format!("{} - SGX {}", whoami::platform(), sgx_mode)).unwrap();
    let uid: CString = {
        let mut hasher = DefaultHasher::new();
        whoami::username().hash(&mut hasher);
        whoami::hostname().hash(&mut hasher);
        platform.hash(&mut hasher);
        CString::new(format!("{:X}", hasher.finish())).unwrap()
    };

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        start_server(enclave.geteid(),
                    &mut retval, platform.into_raw(), uid.into_raw())
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {},
        _ => {
            error!("[-] ECALL Enclave Failed {}!", result.as_str());
            return Ok(());
        }
    }
    info!("[+] start_server success...");
    enclave.destroy();
    Ok(())
}
