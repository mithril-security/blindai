use anyhow::{anyhow, Result};
use log::*;
use ring::digest::{self, Digest};

//#[cfg(not(target_env = "sgx"))]
use std::sync::RwLock;
//#[cfg(target_env = "sgx")]
//use std::sync::SgxRwLock as RwLock;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use uuid::Uuid;

use crate::model::{InferenceModel, ModelDatumType, OnnxModel};

struct InnerModelStore {
    models_by_id: HashMap<Uuid, InferenceModel>,
    onnx_by_hash: HashMap<Vec<u8>, (usize, Arc<OnnxModel>)>,
}

/// This is where model are stored.
pub struct ModelStore {
    inner: RwLock<InnerModelStore>,
}

impl ModelStore {
    pub fn new() -> Self {
        ModelStore {
            inner: RwLock::new(InnerModelStore {
                models_by_id: HashMap::new(),
                onnx_by_hash: HashMap::new(),
            }),
        }
    }

    pub fn add_model(
        &self,
        model_bytes: &[u8],
        input_facts: Vec<Vec<usize>>,
        model_name: Option<String>,
        datum_inputs: Vec<ModelDatumType>,
        datum_outputs: Vec<ModelDatumType>,
    ) -> Result<(Uuid, Digest)> {
        let model_id = Uuid::new_v4();
        let model_hash = digest::digest(&digest::SHA256, model_bytes);

        let model_hash_vec = model_hash.as_ref().to_vec();

        // Create an entry in the hashmap and in the dedup map
        {
            // take the write lock
            let mut models = self.inner.write().unwrap();

            // HashMap entry api requires only one lookup and should be prefered than .get()
            // followed with .insert()

            // deduplication support
            let model = match models.onnx_by_hash.entry(model_hash_vec) {
                Entry::Occupied(mut entry) => {
                    let (num, onnx) = entry.get_mut();
                    *num += 1;
                    info!("Reusing an existing ONNX entry for model. (n = {})", *num);
                    InferenceModel::from_onnx_loaded(
                        onnx.clone(),
                        input_facts,
                        model_id,
                        model_name,
                        model_hash,
                        datum_inputs,
                        datum_outputs,
                    )
                }
                Entry::Vacant(entry) => {
                    info!("Creating a new ONNX entry for model.");
                    // FIXME(cchudant): this call may take a while to run, we may want to refactor
                    // this so that the lock  isn't taken here
                    let model = InferenceModel::load_model(
                        model_bytes,
                        input_facts,
                        model_id,
                        model_name,
                        model_hash,
                        datum_inputs,
                        datum_outputs,
                    )?;
                    entry.insert((1, model.onnx.clone()));
                    model
                }
            };

            // actual hashmap insertion
            match models.models_by_id.entry(model_id) {
                Entry::Occupied(_) => {
                    error!(
                        "UUID collision: model with uuid ({}) already exists.",
                        model_id
                    );
                    return Err(anyhow!("UUID collision"));
                }
                Entry::Vacant(entry) => entry.insert(model),
            };
        }

        Ok((model_id, model_hash))
    }

    pub fn use_model<U>(&self, model_id: Uuid, fun: impl Fn(&InferenceModel) -> U) -> Option<U> {
        // take a read lock
        let read_guard = self.inner.read().unwrap();

        read_guard.models_by_id.get(&model_id).map(fun)
    }

    pub fn delete_model(&self, model_id: Uuid) -> Option<InferenceModel> {
        let mut write_guard = self.inner.write().unwrap();

        let model = match write_guard.models_by_id.entry(model_id) {
            Entry::Occupied(entry) => entry.remove(),
            Entry::Vacant(_) => return None,
        };

        if let Entry::Occupied(mut entry) = write_guard
            .onnx_by_hash
            .entry(model.model_hash().as_ref().to_vec())
        {
            let (i, _) = entry.get_mut();
            *i -= 1;
            if *i == 0 {
                entry.remove();
            }
        }

        Some(model)
    }
}
