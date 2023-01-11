#[macro_use]
mod quote_generation;
mod quote_verification_collateral;

use anyhow::{anyhow, Result};
use quote_generation::RunnerContext;
use quote_verification_collateral::get_collateral_from_quote;

use std::{thread, time};

use std::sync::{Arc, Mutex};
use rouille;
use std::io::Read;
use log::debug;

// No tls connection
pub fn run_remote_attestation() -> Result<()> {
    let mut runner_context = Arc::new(Mutex::new(RunnerContext::init().unwrap()));
    // Binding the runner
    let mut run_ctx_unlocked = runner_context.clone();
    debug!("[DEBUG] Running on localhost:11000 ");
    rouille::start_server("localhost:11000", move |request| {
        let mut run_ctx = run_ctx_unlocked.lock().unwrap();
        rouille::router!(request, 
            (GET)(/target-info) => {
                let target_info = &run_ctx.target_info.clone();
                rouille::Response::from_data("application/octet-stream", &**target_info)
            },

            (POST)(/report) => {
                let mut data_report = request.data().expect("Oops, body already retrieved, problem in the server");
                let mut report = Vec::new();
                data_report.read_to_end(&mut report);
                run_ctx.update_report(report);
                debug!(
                    "[DEBUG] Attestation Runner : report is : {:?}",
                    run_ctx.report_slice
                );
                rouille::Response::from_data("application/octet-stream", run_ctx.report_slice.clone())
            }, 

            (GET)(/get-quote) => {
                debug!(
                    "[DEBUG] Attestation Runner : report is : {:?}",
                    run_ctx.get_report()
                );
                let raw_quote = run_ctx.get_quote().unwrap();
                debug!("[DEBUG] Attestation Runner : quote is : {:?}", raw_quote);
                rouille::Response::from_data("application/octet-stream", raw_quote)
            }, 

            (GET)(/get-collateral) => {
                debug!("quote slice is : {:?}", run_ctx.quote_slice);
                let collateral = get_collateral_from_quote(&run_ctx.quote_slice).unwrap();
                debug!("collateral is : {:?}", collateral);
                let coll_string = serde_json::to_string(&collateral).unwrap();
                debug!(
                    "[DEBUG] Attestation Runner : Collateral is : {:?}",
                    coll_string
                );
                rouille::Response::json(&coll_string)
            }, 

            _ => {
                rouille::Response::empty_404()
            }
        )
    })
}