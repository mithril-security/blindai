use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sgx_tseal::{SgxSealedData, SgxUnsealedData};
use sgx_types::{marker::ContiguousMemory, sgx_attributes_t, sgx_sealed_data_t};
#[cfg(not(target_env = "sgx"))]
use std::fs;
#[cfg(target_env = "sgx")]
use std::untrusted::fs;
use std::{path::Path, str, vec::Vec, collections::HashMap, time::SystemTime};

use crate::model::TensorFacts;

extern crate sgx_tseal;
extern crate sgx_types;

use crate::model_store::{UsersMap, ModelsMap, StoredModel};
type SerializableUsersMap = HashMap<Option<String>, Vec<(String, SystemTime)>>;

#[derive(Serialize, Debug)]
pub struct SerializableModel<'a> {
    pub model_bytes: &'a [u8],
    pub model_name: Option<&'a str>,
    pub model_id: &'a str,
    pub input_facts: &'a [TensorFacts],
    pub output_facts: &'a [TensorFacts],
    pub optim: bool,
    pub owner_id: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct DeserializableModel {
    pub model_bytes: Vec<u8>,
    pub model_name: Option<String>,
    pub model_id: String,
    pub input_facts: Vec<TensorFacts>,
    pub output_facts: Vec<TensorFacts>,
    pub optim: bool,
    pub owner_id: Option<usize>,
}

fn create_sealeddata_for_serializable<T>(
    model: T,
) -> Result<Vec<u8>> 
where T: Serialize {
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

fn recover_sealeddata_for_serializable<T>(
    mut data: Vec<u8>
) -> anyhow::Result<T> where T: DeserializeOwned {
    //recover sealed data from array
    let opt: SgxSealedData<[u8]> = from_sealed_log_for_slice::<u8>(& mut data)
        .ok_or_else(|| anyhow!("Couldn't convert the sealed data into a sealed_log"))?;

    //recover the unsealed data from the array
    let result: SgxUnsealedData<[u8]> = opt
        .unseal_data()
        .map_err(|e| anyhow!("SGX Error: {}", e.as_str()))
        .context("Couldn't recover the sealed data from the sealed_log")?;

    //decipher the sealed data
    let encoded_slice = result.get_decrypt_txt();
    let model = serde_cbor::from_slice(encoded_slice)?;
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
    owner_id: Option<usize>,
) -> anyhow::Result<()> {
    //seal data
    let model_data = create_sealeddata_for_serializable(
        SerializableModel {
            model_bytes,
            model_name,
            model_id,
            input_facts,
            output_facts,
            optim,
            owner_id,
        }
    )?;

    //write sealed data
    fs::create_dir_all(path.parent().unwrap())?;
    Ok(fs::write(path, &model_data)?)
}

pub fn unseal(path: &Path) -> anyhow::Result<DeserializableModel> {
    let buf = fs::read(path)?;
    recover_sealeddata_for_serializable(buf)
}

pub fn seal_metadata(umap: &UsersMap) -> anyhow::Result<()> {
    let mut serializable = SerializableUsersMap::new();
    for (k, v) in umap.map.iter() {
        serializable.insert(k.to_owned(), v.map.iter().map(|(k, v)| (k.to_owned(), v.last_use)).collect());
    }
    let sealed_metadata = create_sealeddata_for_serializable(serializable)?;
    Ok(fs::write("./metadata/metadata", sealed_metadata)?)
}

pub fn unseal_metadata() -> anyhow::Result<UsersMap> {
    let mut umap = UsersMap::default();
    if let Ok(buf) = fs::read("./metadata/metadata") {
        let unsealed: SerializableUsersMap = recover_sealeddata_for_serializable(buf)?;
        for (k, v) in unsealed.iter() {
            let mut model_map = ModelsMap::default();
            for (model_id, last_use) in v.iter() {
                model_map.map.insert(model_id.to_owned(), StoredModel {model: None, last_use: last_use.to_owned()});
            }
            umap.map.insert(k.to_owned(), model_map);
        }
    }
    Ok(umap)
}