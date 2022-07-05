use crate::sealing::{self};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use blindai_common::NetworkConfig;
use log::*;
use ring::digest::{self, Digest};

#[cfg(target_env = "sgx")]
use std::untrusted::fs;

#[cfg(not(target_env = "sgx"))]
use std::sync::RwLock;

#[cfg(target_env = "sgx")]
use std::sync::SgxRwLock as RwLock;

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use crate::model::{InferenceModel, TensorFacts, TractModel};

struct InnerModelStore {
    models_by_id: HashMap<String, InferenceModel>,
    onnx_by_hash: HashMap<Vec<u8>, (usize, Arc<TractModel>)>,
}

/// This is where model are stored.
pub struct ModelStore {
    inner: RwLock<InnerModelStore>,
    config: Arc<NetworkConfig>,
}

impl ModelStore {
    pub fn new(config: Arc<NetworkConfig>) -> Self {
        ModelStore {
            inner: RwLock::new(InnerModelStore {
                models_by_id: HashMap::new(),
                onnx_by_hash: HashMap::new(),
            }),
            config,
        }
    }

    pub fn add_model(
        &self,
        model_bytes: &[u8],
        model_name: Option<String>,
        model_nmid: Option<String>,
        input_facts: &[TensorFacts],
        output_facts: &[TensorFacts],
        save_model: bool,
        optim: bool,
    ) -> Result<(Uuid, Digest)> {
        let model_id = model_nmid.unwrap_or_else(|| Uuid::new_v4().to_string());
        let model_hash = digest::digest(&digest::SHA256, &model_bytes);
        info!("Model hash is {:?}", model_hash);

        let model_hash_vec = model_hash.as_ref().to_vec();

        let mut models_path = PathBuf::new();
        models_path.push(&self.config.models_path);
        models_path.push(&model_id);

        // Sealing
        if save_model {
            sealing::seal(
                models_path.as_path(),
                &model_bytes,
                model_name,
                model_id,
                &input_facts,
                &output_facts,
                optim,
            )
            .context("Sealing the model")?;
            info!("Model sealed");
        }

        // Create an entry in the hashmap and in the dedup map
        {
            // take the write lock
            let mut models = self.inner.write().unwrap();

            // HashMap entry api requires only one lookup and should be prefered than .get()
            // followed with .insert()

            // deduplication support
            let model = match models.onnx_by_hash.entry(model_hash_vec.clone()) {
                Entry::Occupied(mut entry) => {
                    let (num, tract_model) = entry.get_mut();
                    *num += 1;
                    info!("Reusing an existing ONNX entry for model. (n = {})", *num);
                    InferenceModel::from_onnx_loaded(
                        tract_model.clone(),
                        model_id,
                        model_name,
                        model_hash,
                    )
                }
                Entry::Vacant(entry) => {
                    info!("Creating a new ONNX entry for model.");
                    // FIXME(cchudant): this call may take a while to run, we may want to refactor
                    // this so that the lock isn't taken here
                    let inference_model = InferenceModel::load_model(
                        &model_bytes,
                        model_id,
                        model_name,
                        model_hash,
                        input_facts,
                        output_facts,
                        optim,
                    )?;
                    entry.insert((1, inference_model.model.clone()));
                    inference_model
                }
            };
            // actual hashmap insertion
            match models.models_by_id.entry(&model_id) {
                Entry::Occupied(_) => {
                    error!(
                        "Name collision: model with name ({}) already exists.",
                        model_id
                    );
                    return Err(anyhow!("Name collision"));
                }
                Entry::Vacant(entry) => entry.insert(model),
            };
        }

        Ok((model_id, model_hash))
    }

    pub fn use_model<U>(&self, model_id: &str, fun: impl Fn(&InferenceModel) -> U) -> Option<U> {
        // take a read lock
        let read_guard = self.inner.read().unwrap();

        match read_guard.models_by_id.get(model_id) {
            Some(model) => Some(fun(model)),
            None => None,
        }
    }

    pub fn delete_model(&self, model_nmid: &str) -> Option<InferenceModel> {
        let mut write_guard = self.inner.write().unwrap();

        let model = match write_guard.models_by_id.entry(model_nmid) {
            Entry::Occupied(entry) => entry.remove(),
            Entry::Vacant(_) => return None,
        };

        match write_guard
            .onnx_by_hash
            .entry(model.model_hash().as_ref().to_vec())
        {
            Entry::Occupied(mut entry) => {
                let (i, _) = entry.get_mut();
                *i -= 1;
                if *i == 0 {
                    entry.remove();
                }
            }
            _ => {}
        }

        Some(model)
    }

    pub fn startup_unseal(&self, config: &blindai_common::NetworkConfig) -> Result<()> {
        if let Ok(paths) = fs::read_dir(&config.models_path) {
            for path in paths {
                let path = path?;
                if let Ok(model) = sealing::unseal(path.path().as_path()) {
                    self.add_model(
                        &model.model_bytes,
                        model.model_name,
                        Some(model.model_id),
                        &model.input_facts,
                        &model.output_facts,
                        false,
                        model.optim,
                    )?;
                    info!("Model {:?} loaded", model.model_id.to_string());
                } else {
                    info!("Unsealing of model {:?} failed", path.file_name());
                }
            }
        } else {
            fs::create_dir(&config.models_path)?;
        }

        Ok(())
    }
}
