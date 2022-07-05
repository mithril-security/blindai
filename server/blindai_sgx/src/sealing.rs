use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sgx_tseal::SgxSealedData;
use sgx_types::{marker::ContiguousMemory, sgx_attributes_t, sgx_sealed_data_t};
#[cfg(not(target_env = "sgx"))]
use std::fs;
#[cfg(target_env = "sgx")]
use std::untrusted::fs;
use std::{path::Path, str, vec::Vec};
use uuid::Uuid;

use crate::model::TensorFacts;

extern crate sgx_tseal;
extern crate sgx_types;

#[derive(Serialize, Debug)]
pub struct SerializableModel<'a> {
    pub model_bytes: &'a [u8],
    pub model_name: Option<&'a str>,
    pub model_id: &'a str,
    pub input_facts: &'a [TensorFacts],
    pub output_facts: &'a [TensorFacts],
    pub optim: bool,
}

#[derive(Deserialize, Debug)]
pub struct DeserializableModel {
    pub model_bytes: Vec<u8>,
    pub model_name: Option<String>,
    pub model_id: String,
    pub input_facts: Vec<TensorFacts>,
    pub output_facts: Vec<TensorFacts>,
    pub optim: bool,
}

fn create_sealeddata_for_serializable(model: SerializableModel) -> Result<Vec<u8>> {
    let encoded_vec = serde_cbor::to_vec(&model)?;
    let encoded_slice = encoded_vec.as_slice();

    let attr = sgx_attributes_t {
        flags: sgx_types::TSEAL_DEFAULT_FLAGSMASK,
        xfrm: 0,
    };
    let sealed_data = SgxSealedData::<[u8]>::seal_data_ex(
        sgx_types::SGX_KEYPOLICY_MRENCLAVE,
        attr,
        0,
        &[],
        encoded_slice,
    )
    .map_err(|e| anyhow!("SGX Error: {}", e.as_str()))
    .context("Couldn't seal data")?;

    //calculate the size of the sealed data
    let size_seal = SgxSealedData::<[u8]>::calc_raw_sealed_data_size(
        sealed_data.get_add_mac_txt_len(),
        sealed_data.get_encrypt_txt_len(),
    );

    let mut sealed_log_arr: Vec<u8> = vec![0; size_seal as usize];

    //write sealed data to array
    to_sealed_log_for_slice(&sealed_data, &mut sealed_log_arr)
        .ok_or_else(|| anyhow!("sealing failed"))?;
    Ok(sealed_log_arr)
}

fn recover_sealeddata_for_serializable(mut data: Vec<u8>) -> anyhow::Result<DeserializableModel> {
    //recover sealed data from array
    let opt = from_sealed_log_for_slice::<u8>(&mut data)
        .ok_or_else(|| anyhow!("Couldn't convert the sealed data into a sealed_log"))?;

    //recover the unsealed data from the array
    let result = opt
        .unseal_data()
        .map_err(|e| anyhow!("SGX Error: {}", e.as_str()))
        .context("Couldn't recover the sealed data from the sealed_log")?;

    //decipher the sealed data
    let encoded_slice = result.get_decrypt_txt();
    let model: DeserializableModel = serde_cbor::from_slice(encoded_slice)?;

    Ok(model)
}

fn to_sealed_log_for_slice<T: Copy + ContiguousMemory>(
    sealed_data: &SgxSealedData<[T]>,
    sealed_log: &mut [u8],
) -> Option<*mut sgx_sealed_data_t> {
    // Safety:
    // sealed_log is a slice and therefore has a valid pointer and length
    unsafe {
        sealed_data.to_raw_sealed_data_t(
            sealed_log.as_mut_ptr() as *mut sgx_sealed_data_t,
            sealed_log.len() as u32,
        )
    }
}

fn from_sealed_log_for_slice<'a, T: Copy + ContiguousMemory>(
    sealed_log: &mut [u8],
) -> Option<SgxSealedData<'a, [T]>> {
    // Safety:
    // sealed_log is a slice and therefore has a valid pointer and length
    unsafe {
        SgxSealedData::<[T]>::from_raw_sealed_data_t(
            sealed_log.as_mut_ptr() as *mut sgx_sealed_data_t,
            sealed_log.len() as u32,
        )
    }
}

pub fn seal(
    path: &Path,
    model_bytes: &[u8],
    model_name: Option<&str>,
    model_id: &str,
    input_facts: &[TensorFacts],
    output_facts: &[TensorFacts],
    optim: bool,
) -> anyhow::Result<()> {
    //seal data
    let sealed = create_sealeddata_for_serializable(SerializableModel {
        model_bytes,
        model_name,
        model_id,
        input_facts,
        output_facts,
        optim,
    })?;

    //write sealed data
    Ok(fs::write(path, &sealed)?)
}

pub fn unseal(path: &Path) -> anyhow::Result<DeserializableModel> {
    let buf = fs::read(path)?;
    recover_sealeddata_for_serializable(buf)
}
