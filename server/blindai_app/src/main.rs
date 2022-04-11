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

extern crate blindai_sgx_attestation;
extern crate sgx_types;
extern crate sgx_urts;

use std::{
    collections::hash_map::DefaultHasher,
    ffi::CString,
    hash::{Hash, Hasher},
    os::raw::c_char,
};

use blindai_common::{untrusted_local_app_server, SgxCollateral};
use env_logger::Env;
use log::{error, info};

use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::{env, fs::File, io::Read};

use tonic::transport::Server;

use anyhow::Result;

mod dcap;
mod self_signed_tls;

static ENCLAVE_FILE: &str = "enclave.signed.so";

extern "C" {
    fn start_server(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        telemetry_platform: *const c_char,
        telemetry_uid: *const c_char,
    ) -> sgx_status_t;
}

#[derive(Default)]
pub struct State {}

#[tonic::async_trait]
impl untrusted_local_app_server::UntrustedLocalApp for State {
    // The request type gets wrapped in a `tonic::Request`.
    // The response type gets wrapped in a `Result<tonic::Response<_>,
    // tonic::Status>`.
    async fn get_collateral_from_quote(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<SgxCollateral>, tonic::Status> {
        dcap::get_collateral_from_quote(&request.into_inner())
            .await
            .map(tonic::Response::new)
            .map_err(|e| tonic::Status::internal(e.to_string()))
    }
}

fn fill_blank_and_print(content: &str, size: usize)
{
    let trail_char = "#";
    let trail: String = trail_char.repeat((size - 2 - content.len()) / 2);
    let trail2: String = trail_char.repeat(((size - 2 - content.len()) as f32 / 2.0).ceil() as usize);
    println!("{} {} {}", trail, content, trail2);
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let sgx_mode = match env::var_os("SGX_MODE") {
        Some(v) => v.into_string().unwrap(),
        None => "HW".to_string(),
    };

    let logo_str: &str = include_str!("../logo.txt");
    let version_str: String = format!("VERSION : {}", env!("CARGO_PKG_VERSION"));
    let text_size : usize = 58;
    println!("{}\n", logo_str);
    fill_blank_and_print("BlindAI - INFERENCE SERVER", text_size);
    fill_blank_and_print("MADE BY MITHRIL SECURITY", text_size);
    fill_blank_and_print("GITHUB: https://github.com/mithril-security/blindai", text_size);
    fill_blank_and_print(&version_str, text_size);
    println!();
    info!("Starting Enclave...");

    let enclave = match init_enclave() {
        Ok(r) => {
            info!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            error!("[-] Init Enclave Failed {}!", x.as_str());
            return Ok(());
        }
    };

    // Read network config
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let network_config: blindai_common::NetworkConfig = toml::from_str(&contents)?;

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

    let platform: CString =
        CString::new(format!("{} - SGX {}", whoami::platform(), sgx_mode)).unwrap();
    let uid: CString = {
        let mut hasher = DefaultHasher::new();
        whoami::username().hash(&mut hasher);
        whoami::hostname().hash(&mut hasher);
        platform.hash(&mut hasher);
        CString::new(format!("{:X}", hasher.finish())).unwrap()
    };

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        start_server(
            enclave.geteid(),
            &mut retval,
            platform.into_raw(),
            uid.into_raw(),
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            error!("[-] ECALL Enclave Failed {}!", result.as_str());
            return Ok(());
        }
    }
    info!("[+] start_server success...");
    enclave.destroy();
    Ok(())
}
