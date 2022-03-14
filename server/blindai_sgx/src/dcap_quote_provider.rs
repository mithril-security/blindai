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

use ring::{digest, digest::Digest};
use sgx_types::sgx_ql_att_key_id_t;

use blindai_sgx_attestation::platform;

pub(crate) struct DcapQuoteProvider {
    hash: Digest,
    pub enclave_held_data: Vec<u8>,
}

impl DcapQuoteProvider {
    pub fn get_quote(self: &Self) -> anyhow::Result<Vec<u8>> {
        get_quote_with_data(self.hash.as_ref())
    }
    pub fn new(enclave_held_data: &[u8]) -> Self {
        DcapQuoteProvider {
            hash: digest::digest(&digest::SHA256, &enclave_held_data),
            enclave_held_data: enclave_held_data.to_vec(),
        }
    }
}

pub fn get_quote_with_data(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let (mut ak_id, qe_target_info) = platform::init_sgx_quote()?;

    // For DCAP-based attestation, SPID should be 0
    const SPID_OFFSET: usize = std::mem::size_of::<sgx_ql_att_key_id_t>();
    ak_id.att_key_id[SPID_OFFSET..(SPID_OFFSET + sgx_types::sgx_spid_t::default().id.len())]
        .fill(0);

    let sgx_report = platform::create_sgx_isv_enclave_report(data, qe_target_info)?;
    let quote = platform::get_sgx_quote(&ak_id, sgx_report)?;

    Ok(quote)
}
