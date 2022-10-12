use std::{thread, ptr};
use tiny_http::{Response, Server};
use std::sync::Arc;

fn main() {
    // Make debugging easier by enabling rust backtrace inside enclave
    std::env::set_var("RUST_BACKTRACE", "1");

    let server = Arc::new(
        Server::https(
        "0.0.0.0:9975",
        tiny_http::SslConfig {
            certificate: include_bytes!("../server.pem").to_vec(),
            private_key: include_bytes!("../server2.key").to_vec(),
        },
    ).unwrap());
    
    println!("Now listening on port 9975");

    let mut handles = Vec::new();

    for _ in 0..4 {
        let server = server.clone();

        handles.push(thread::spawn(move || {
            for mut rq in server.incoming_requests() {
                let msg = "hello world";             
                let response = tiny_http::Response::from_string(msg.to_string());
                let _ = rq.respond(response);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

