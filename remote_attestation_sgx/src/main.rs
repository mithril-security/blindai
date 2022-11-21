
mod quote_generation;
mod quote_verification;

use quote_generation::{RunnerContext}; 
use quote_verification::{VerificationContext, get_collateral_from_quote, SgxCollateral};
use std::time::Duration;
use std::{thread};
use std::fs::File;
use tiny_http::{Server, Response};


unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}


// Testing Remote attestation with the client 
// No tls connection 
fn main() {
    let mut runnercontext = RunnerContext::init().unwrap();
    // Binding the runner 

    let ti = &runnercontext.target_info.clone();

    let server_runner = Server::http("0.0.0.0:11001").unwrap();

    loop {
        let mut request = match server_runner.recv() {
            Ok(rq) => rq, 
            Err(e) => { println!("error: {}", e); break }
        };
        println!("request is : {:?}", request);
        let mut response = Response::from_string("");

        if request.url() =="/targetinfo" && request.method() == &tiny_http::Method::Get {
            response = Response::from_data(&**ti);
        }


        else if request.url() == "/report" && request.method() == &tiny_http::Method::Post {
            let mut report : Vec<u8> = Vec::new();
            request.as_reader().read_to_end(&mut report);
            println!("[DEBUG] Attestation Runner : report is : {:?}", report);
            runnercontext.update_report(report);

            response = Response::from_data(runnercontext.report_slice.clone());
        }

        else if request.url() == "/getquote" && request.method() == &tiny_http::Method::Get {
            let raw_quote = runnercontext.get_quote().unwrap();
            println!("[DEBUG] Attestation Runner : quote is : {:?}", raw_quote);
            response = Response::from_data(raw_quote);
        }

        
        else if request.url() == "/getcollateral" && request.method() == &tiny_http::Method::Get {
            let collateral = get_collateral_from_quote(&runnercontext.quote_slice).unwrap();
            let coll_string = serde_json::to_string(&collateral).unwrap();
            println!("[DEBUG] Attestation Runner : Collateral is : {:?}", coll_string);
            response = Response::from_string(coll_string);
        }

        request.respond(response);

    };

}
