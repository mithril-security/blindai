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

use identity::MyIdentity;
use std::sync::Arc;
use std::thread;
mod identity;
mod model;
mod model_store;
use anyhow::Result;
use model_store::ModelStore;

use crate::client_communication::Exchanger;
mod client_communication;

fn main() -> Result<()> {
    // Make debugging easier by enabling rust backtrace inside enclave
    std::env::set_var("RUST_BACKTRACE", "full");

    let (certificate, signing_key_seed) = identity::create_certificate().unwrap();
    let my_identity = Arc::new(MyIdentity::from_cert(certificate, signing_key_seed));
    let enclave_identity = my_identity.tls_identity.clone();
    let exchanger_temp = Arc::new(Exchanger::new(
        Arc::new(ModelStore::new()),
        my_identity,
        1000000000,
        1000000,
    ));

    let cert_untrusted = enclave_identity.cert_der.clone();

    let untrusted_server = rouille::Server::new_ssl(
        "0.0.0.0:9923",
        move |_request| {
            println!("Requested enclave TLS certificate");
            rouille::Response::from_data("application/octet-stream", cert_untrusted.clone())
            // TODO: Change it to something more appropriate?
        },
        include_bytes!("../host_server.pem").to_vec(),
        include_bytes!("../host_server.key").to_vec(),
    )
    .expect("Failed to start untrusted server")
    .pool_size(4);
    let (_untrusted_handle, _untrusted_sender) = untrusted_server.stoppable();

    thread::spawn(move || {
        let trusted_server = rouille::Server::new_ssl(
            "0.0.0.0:9924",
            move |request| {
                rouille::router!(request,
                    (POST) (/upload) => {
                        let reply = exchanger_temp.send_model(&request);
                        exchanger_temp.respond(request, reply)
                    },

                    (POST) (/run) => {
                        let reply = exchanger_temp.run_model(&request);
                        exchanger_temp.respond(request, reply)
                    },

                    (POST) (/delete) => {
                        let reply = exchanger_temp.delete_model(&request);
                        exchanger_temp.respond(request, reply)
                    },

                    _ => rouille::Response::empty_404()
                )
            },
            enclave_identity.cert_der,
            enclave_identity.private_key_der,
        )
        .expect("Failed to start trusted server");
        let (_trusted_handle, _trusted_sender) = trusted_server.stoppable();
        _trusted_handle.join().unwrap();
    });
    _untrusted_handle.join().unwrap();

    Ok(())
}
