
use ring::{digest, digest::Digest};



use std::io::{self, Write};
use remote_attestation_sgx::{get_target_info, get_attestation_key};
use sgx_isa::{Targetinfo, Report, Keyrequest};
use quote_generation::{RunnerContext};


pub(crate) struct DcapQuoteProvider {
    hash: Digest, 
    pub enclave_held_data: Vec<u8>,
}


impl DcapQuoteProvider {
    pub fn get_quote(&self) -> anyhow::Result<<Vec<u8>> {
        get_quote_with_data(self.hash.as_ref())
    }
    pub fn new(enclave_held_data: &[u8]) -> Self {
        DcapQuoteProvider {
            hash: digest::digest(&digest::SHA256, enclave_held_data),
            enclave_held_data: enclave_held_data.to_vec(),
        }
    }
}

// get_quote_with_data takes the targetinfo returned by the library 
pub fn get_quote_with_data(data_hash: &[u8]) -> anyhow::Result<Vec<u8>> {

    let runner : RunnerContext = RunnerContext.init()?;

    // Sending the targetinfo to the enclave to get the report
    //-------------------------------------------------------------
    //Code in the enclave
    // let targetinfo_bytes = get_target_info(); 

    // // try copy from for targetinfo
    // let targetinfo = Targetinfo::try_copy_from(targetinfo_bytes); 

    // // Moving on to report generation
    // let report = Report::for_target(&targetinfo.unwrap(), &data_hash); 
    //-------------------------------------------------------------
    let report_slice = getReport_from_enclave();
    let quote = runner.get_quote(report_slice)?;
    Ok(quote)
}