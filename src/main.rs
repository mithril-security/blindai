use std::{thread};
use tiny_http::{Response, Server};
use std::sync::Arc;
use identity::MyIdentity;
mod identity;
mod model;
use model::{ModelDatumType};
use serde_derive::{Deserialize};
mod model_store;
use model_store::ModelStore;
use ring::digest::{self, Digest};
#[cfg(not(target_env = "sgx"))]
use std::sync::RwLock;
#[cfg(target_env = "sgx")]
use std::sync::SgxRwLock as RwLock;

use serde_cbor;

use crate::client_communication::Exchanger;
mod client_communication;


fn main() {

    // Make debugging easier by enabling rust backtrace inside enclave
    std::env::set_var("RUST_BACKTRACE", "full");

    let test_list = [216, 28, 136, 24, 101, 25, 4, 21, 25, 8, 245, 25, 38, 204, 25, 7, 206, 25, 36, 178, 25, 3, 231, 24, 102];
    let deser_list:Vec<i32> = serde_cbor::from_slice(&test_list).unwrap();
    println!("test deserialization{:?}",deser_list);

    let (certificate, storage_identity, signing_key_seed) = identity::create_certificate().unwrap();
    let my_identity = Arc::new(MyIdentity::from_cert(
        certificate,
        storage_identity,
        signing_key_seed,
    ));
    let enclave_identity = my_identity.tls_identity.clone();
    let exchanger_temp = Arc::new(Exchanger::new(Arc::new(ModelStore::new()),my_identity,1000000000,100000));
    
    let server = Arc::new(
        Server::https(
        "0.0.0.0:9976",
        tiny_http::SslConfig {
            certificate: enclave_identity.cert_der, 
            private_key: enclave_identity.private_key_der,
        },
    ).unwrap());

    println!("Now listening on port 9976");

    let mut handles = Vec::new();

    for _ in 0..4 {
        let server = server.clone();
        let exchanger_temp = Arc::clone(&exchanger_temp);
        handles.push(thread::spawn(move || {
            for mut rq in server.incoming_requests() {
                println!("{}",rq.url());
                println!("Received request");
                
                if rq.url() == "/upload" {
                    Exchanger::send_model(&exchanger_temp, rq).unwrap();
                }

                else if rq.url() == "/run" {
                    Exchanger::run_model(&exchanger_temp, rq).unwrap();
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
