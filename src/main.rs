use identity::MyIdentity;
use std::sync::Arc;
use std::thread;
use tiny_http::{Response, Server};
mod identity;
mod model;
mod model_store;
use anyhow::{anyhow, Error, Result};
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

    let untrusted_server = Arc::new(
        Server::https(
            "0.0.0.0:9923",
            tiny_http::SslConfig {
                certificate: include_bytes!("../host_server.pem").to_vec(),
                private_key: include_bytes!("../host_server.key").to_vec(),
            },
        )
        .map_err(|e| anyhow!(e))?,
    );

    let mut untrusted_handles = Vec::new();

    for _ in 0..4 {
        let untrusted_server = untrusted_server.clone();
        let trusted_cert = enclave_identity.cert_der.clone();

        untrusted_handles.push(thread::spawn(move || {
            for rq in untrusted_server.incoming_requests() {
                println!("Retrieve and send attestation report to client here");
                //The report must include the enclave_identity.cert_der
                //For now it returns the trusted server's cert
                rq.respond(Response::from_data(trusted_cert.clone()))
                    .unwrap();
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
        )
        .map_err(|e| anyhow!(e))?,
    );
    println!("Now listening on port 9923 and 9924");

    let mut handles = Vec::new();

    for _ in 0..4 {
        let server = server.clone();
        let exchanger_temp = Arc::clone(&exchanger_temp);
        handles.push(thread::spawn(move || {
            for mut rq in server.incoming_requests() {
                println!("{}", rq.url());
                match rq.url() {
                    "/upload" => {
                        let reply = exchanger_temp.send_model(&mut rq);
                        exchanger_temp.respond(rq, reply);
                    }
                    "/run" => {
                        let reply = exchanger_temp.run_model(&mut rq);
                        exchanger_temp.respond(rq, reply);
                    }
                    "/delete" => {
                        let reply = exchanger_temp.delete_model(&mut rq);
                        exchanger_temp.respond(rq, reply);
                    }
                    _ => exchanger_temp
                        .respond::<()>(rq, Err(Error::msg("unknown request".to_string()))),
                };
            }
        }));
    }

    for u in untrusted_handles {
        u.join().unwrap();
    }

    for h in handles {
        h.join().unwrap();
    }

    Ok(())
}
