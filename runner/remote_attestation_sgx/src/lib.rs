mod quote_generation;
mod quote_verification_collateral;

use anyhow::{anyhow, Result};
use quote_generation::RunnerContext;
use quote_verification_collateral::get_collateral_from_quote;

use std::{thread, time};

use std::sync::Arc;
use tiny_http::{Response, Server};

use log::debug;

// No tls connection
pub fn run_remote_attestation() -> Result<()> {
    let runner_context = RunnerContext::init().unwrap();
    // Binding the runner

    let ti = &runner_context.target_info.clone();

    let server_runner = Server::http("127.0.0.1:11000").map_err(|e| anyhow!(e))?;
    let server = Arc::new(server_runner);
    let mut server_handler = Vec::new();

    let target_info = ti.clone();
    let server_run = server;
    let mut run_ctx = runner_context;

    server_handler.push(thread::spawn(move || {
        for mut request in server_run.incoming_requests() {
            let mut response = Response::from_string("");

            if request.url() == "/target-info" && request.method() == &tiny_http::Method::Get {
                response = Response::from_data(&*target_info);
            } else if request.url() == "/report" && request.method() == &tiny_http::Method::Post {
                let mut report: Vec<u8> = Vec::new();
                request.as_reader().read_to_end(&mut report).unwrap();
                // println!("[DEBUG] Attestation Runner : report is : {:?}", report);
                run_ctx.update_report(report);
                debug!(
                    "[DEBUG] Attestation Runner : report is : {:?}",
                    run_ctx.report_slice
                );

                response = Response::from_data(run_ctx.report_slice.clone());
            } else if request.url() == "/get-quote" && request.method() == &tiny_http::Method::Get {
                debug!(
                    "[DEBUG] Attestation Runner : report is : {:?}",
                    run_ctx.report_slice
                );

                let raw_quote = run_ctx.get_quote().unwrap();
                debug!("[DEBUG] Attestation Runner : quote is : {:?}", raw_quote);
                response = Response::from_data(raw_quote);
                thread::sleep(time::Duration::from_millis(5000))
            } else if request.url() == "/getcollateral"
                && request.method() == &tiny_http::Method::Get
            {
                debug!("running getcollateral");
                debug!("quote slice is : {:?}", run_ctx.quote_slice);
                let collateral = get_collateral_from_quote(&run_ctx.quote_slice).unwrap();
                thread::sleep(time::Duration::from_millis(2000));
                debug!("collateral is : {:?}", collateral);
                let coll_string = serde_json::to_string(&collateral).unwrap();
                debug!(
                    "[DEBUG] Attestation Runner : Collateral is : {:?}",
                    coll_string
                );
                response = Response::from_string(coll_string);
            }

            request.respond(response).unwrap();
        }
    }));

    for u in server_handler {
        u.join().unwrap();
    }
    Ok(())
}
