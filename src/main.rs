use std::{thread};
use tiny_http::{Response, Server};
use std::sync::Arc;
use identity::MyIdentity;
mod identity;
mod model;
//use model::{ModelDatumType};
//use serde_derive::{Deserialize};
mod model_store;
use model_store::ModelStore;
//use ring::digest::{self, Digest};
use std::sync::RwLock;


//use serde_cbor;

use crate::client_communication::Exchanger;
mod client_communication;


fn main() {

    // Make debugging easier by enabling rust backtrace inside enclave
    std::env::set_var("RUST_BACKTRACE", "full");

    let (certificate, storage_identity, signing_key_seed) = identity::create_certificate().unwrap();
    let my_identity = Arc::new(MyIdentity::from_cert(
        certificate,
        storage_identity,
        signing_key_seed,
    ));
    let enclave_identity = my_identity.tls_identity.clone();
    let exchanger_temp = Arc::new(Exchanger::new(Arc::new(ModelStore::new()),my_identity,1000000000,100000));

    
    let untrusted_server = Arc::new(
        Server::https(
            "0.0.0.0:9923",
            tiny_http::SslConfig {
                certificate: include_bytes!("../host_server.pem").to_vec(),
                private_key: include_bytes!("../host_server.key").to_vec(),
            },
        ).unwrap()
    );

    
    let mut untrusted_handles = Vec::new();

    for _ in 0..4 {
        let untrusted_server = untrusted_server.clone();
        let trusted_cert = enclave_identity.cert_der.clone();
        
        untrusted_handles.push(thread::spawn(move || {
            for mut rq in untrusted_server.incoming_requests() {
                println!("Retrieve and send attestation report to client here");
                //The report must include the enclave_identity.cert_der
                //For now it returns the trusted server's cert
                rq.respond(Response::from_data(trusted_cert.clone()));
            }
        }));
    }


    let server = Arc::new(
        Server::https(
        "0.0.0.0:9924",
        tiny_http::SslConfig {
            certificate: enclave_identity.cert_der, 
            private_key: enclave_identity.private_key_der,
        },
    ).unwrap());
    println!("Now listening on port 9923 and 9924");

    let mut handles = Vec::new();

    for _ in 0..4 {
        let server = server.clone();
        let exchanger_temp = Arc::clone(&exchanger_temp);
        handles.push(thread::spawn(move || {
            for mut rq in server.incoming_requests() {
                println!("{}",rq.url());
                
                if rq.url() == "/upload" {
                    Exchanger::send_model(&exchanger_temp, rq).unwrap();
                }

                else if rq.url() == "/run" {
                    Exchanger::run_model(&exchanger_temp, rq).unwrap();
                }

                else if rq.url() == "/delete" {
                    Exchanger::delete_model(&exchanger_temp, rq).unwrap();
                }
            }
        }));
    }

    for u in untrusted_handles {
        u.join().unwrap();
    }

    for h in handles {
        h.join().unwrap();
    }
}
