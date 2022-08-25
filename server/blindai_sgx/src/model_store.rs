use crate::{
    model::ModelLoadContext,
    sealing::{self},
};
use std::{
    path::{Path, PathBuf},
    sync::Weak,
};

use anyhow::{anyhow, Context, Result};
use blindai_common::{BlindAIConfig, LoadModelConfig, ModelFactsConfig};
use log::*;
use ring::digest::{self, Digest};
use uuid::Uuid;
use weak_table::{weak_value_hash_map, WeakValueHashMap};

#[cfg(not(target_env = "sgx"))]
use std::fs;
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

use crate::model::{InferModel, TensorFacts, TractModel};

#[derive(Debug)]
pub enum ModelStoreError {
    NameCollision,
    Other(anyhow::Error),
}

impl From<anyhow::Error> for ModelStoreError {
    fn from(e: anyhow::Error) -> Self {
        ModelStoreError::Other(e)
    }
}

#[derive(Default)]
struct InnerModelStore {
    models_by_id: HashMap<String, Arc<InferModel>>,
    models_by_user: HashMap<usize, Arc<InferModel>>, // this should be a multimap
    onnx_by_hash: WeakValueHashMap<Vec<u8>, Weak<TractModel>>,
}

/// This is where model are stored.
pub struct ModelStore {
    inner: RwLock<InnerModelStore>,
    config: Arc<BlindAIConfig>,
}

impl ModelStore {
    pub fn new(config: Arc<BlindAIConfig>) -> Self {
        ModelStore {
            inner: RwLock::new(Default::default()),
            config,
        }
    }

    pub fn add_model(
        &self,
        model_bytes: &[u8],
        model_name: Option<String>,
        model_id: Option<String>,
        input_facts: &[TensorFacts],
        output_facts: &[TensorFacts],
        save_model: bool,
        optim: bool,
        load_context: ModelLoadContext,
        owner_id: Option<usize>,
    ) -> Result<(String, Digest), ModelStoreError> {
        let model_id = model_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        let model_hash = digest::digest(&digest::SHA256, &model_bytes);
        info!("Model hash is {:?}", model_hash);

        let model_hash_vec = model_hash.as_ref().to_vec();

        let mut models_path = PathBuf::new();
        models_path.push(&self.config.models_path);
        models_path.push(&model_id);

        // Sealing
        if save_model {
            info!("Sealing model...");
            sealing::seal(
                models_path.as_path(),
                &model_bytes,
                model_name.as_deref(),
                &model_id,
                &input_facts,
                &output_facts,
                optim,
                owner_id,
            )
            .context("Sealing the model")?;
            info!("Model sealed");
        }

        // Create an entry in the hashmap and in the dedup map
        {
            // take the write lock
            let mut write_guard = self.inner.write().unwrap();

            // remove a model store if the store is full (FIFO)
            let model_id_currently_store = write_guard.models_by_id.len();
            info!(
                "Max of model allow: {:?}, Current model store by id: {:?}",
                self.config.max_model_store, model_id_currently_store
            );

            // We check if the model store is full regarding the hashmap for the model
            // and we release space if necessary
            if self.config.max_model_store != 0
                && model_id_currently_store >= self.config.max_model_store
            {
                let mut first_id: String = String::new();
                for (key, model) in write_guard.models_by_id.iter() {
                    if model.load_context == ModelLoadContext::FromSendModel {
                        first_id = key;
                        break;
                    }
                }
                write_guard.models_by_id.remove(&first_id);
            }

            // HashMap entry api requires only one lookup and should be prefered than .get()
            // followed with .insert()

            // deduplication support
            let model = match write_guard.onnx_by_hash.entry(model_hash_vec.clone()) {
                weak_value_hash_map::Entry::Occupied(entry) => {
                    let tract_model = entry.get_strong();
                    info!("Reusing an existing ONNX entry for model.");
                    InferModel::from_onnx_loaded(
                        tract_model.clone(),
                        model_id.clone(),
                        model_name,
                        model_hash,
                        owner_id,
                    )
                }
                weak_value_hash_map::Entry::Vacant(entry) => {
                    info!("Creating a new ONNX entry for model.");
                    // FIXME(cchudant): this call may take a while to run, we may want to refactor
                    // this so that the lock isn't taken here
                    let inference_model = InferModel::load_model(
                        &model_bytes,
                        model_id.clone(),
                        model_name,
                        model_hash,
                        input_facts,
                        output_facts,
                        optim,
                        load_context,
                        owner_id,
                    )?;
                    entry.insert(inference_model.model.clone());
                    inference_model
                }
            };
            let model = Arc::new(model);

            // actual hashmap insertion
            match write_guard.models_by_id.entry(model_id.clone()) {
                Entry::Occupied(_) => {
                    error!(
                        "Name collision: model with name ({}) already exists.",
                        model_id
                    );
                    return Err(ModelStoreError::NameCollision)?;
                }
                Entry::Vacant(entry) => entry.insert(model.clone()),
            };

            // owner id map
            if let Some(owner_id) = owner_id {
                match write_guard.models_by_user.entry(owner_id) {
                    Entry::Occupied(mut entry) => {
                        let old_model = entry.insert(model);

                        match write_guard
                            .models_by_id
                            .entry(old_model.model_id().to_string())
                        {
                            Entry::Occupied(entry) => {
                                entry.remove();
                            }
                            _ => {}
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(model);
                    }
                }
            }
        }

        Ok((model_id, model_hash))
    }

    // Check if model is in the already in the server or no
    pub fn in_the_server(&self, id_to_fetch: String) -> bool {
        let read_guard = self.inner.read().unwrap();

        let find = match read_guard.models_by_id.get(&id_to_fetch.to_string()) {
            Some(_model) => true,
            None => false,
        };

        find
    }

    //Unseal function that unseal the model if we find it in the seal model, and
    // return true or false if we find it.
    pub fn unseal(&self, id_to_fetch: String) -> Result<()> {
        // take a read lock
        if let Ok(paths) = fs::read_dir(&self.config.models_path) {
            for path in paths {
                let path = path?;
                if let Ok(model) = sealing::unseal(path.path().as_path()) {
                    if id_to_fetch == model.model_id.clone() {
                        self.add_model(
                            &model.model_bytes,
                            model.model_name,
                            Some(model.model_id.clone()),
                            &model.input_facts,
                            &model.output_facts,
                            false,
                            model.optim,
                            ModelLoadContext::FromSendModel,
                            model.owner_id,
                        )
                        .map_err(|err| anyhow!("Adding model failed: {:?}", err))?;
                        info!("Model {:?} loaded", model.model_id);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn use_model<U>(&self, model_id: &str, fun: impl FnOnce(&InferModel) -> U) -> Option<U> {
        // take a read lock
        let read_guard = self.inner.read().unwrap();

        match read_guard.models_by_id.get(model_id) {
            Some(model) => Some(fun(model)),
            None => None,
        }
    }

    pub fn delete_model(&self, model_id: &str) -> Option<Arc<InferModel>> {
        let mut write_guard = self.inner.write().unwrap();

        let model = match write_guard.models_by_id.entry(model_id.to_string()) {
            Entry::Occupied(entry) => entry.remove(),
            Entry::Vacant(_) => return None,
        };

        if let Some(owner_id) = model.owner_id() {
            match write_guard.models_by_user.entry(owner_id) {
                Entry::Occupied(entry) => {
                    entry.remove_entry();
                }
                _ => {}
            }
        }

        Some(model)
    }

    pub fn check_seal_file_exist(&self) -> Result<()> {
        if let Ok(_paths) = fs::read_dir(&self.config.models_path) {
        } else {
            fs::create_dir(&self.config.models_path)?;
        }

        Ok(())
    }

    pub fn load_config_models(&self) -> Result<()> {
        let mut models = self.inner.write().unwrap();

        let mut load_model = |model: &LoadModelConfig| -> Result<()> {
            let model_hash = digest::digest(&digest::SHA256, b"data :)"); // FIXME

            let translate_facts = |facts: &[ModelFactsConfig]| -> Result<Vec<TensorFacts>> {
                facts
                    .into_iter()
                    .map(|fact| {
                        Ok(TensorFacts {
                            datum_type: fact.datum_type.as_deref().map_or_else::<Result<_>, _, _>(
                                || Ok(None),
                                |dt| Ok(Some(dt.parse()?)),
                            )?,
                            dims: fact.dims.clone(),
                            index: fact.index,
                            index_name: fact.index_name.clone(),
                        })
                    })
                    .collect()
            };

            let model = InferModel::load_model_path(
                Path::new(&model.path),
                model.model_id.clone(),
                None,
                model_hash,
                &translate_facts(&model.input_facts)?,
                &translate_facts(&model.output_facts)?,
                !model.no_optim,
                ModelLoadContext::FromStartupConfig,
                None,
            )?;
            models
                .models_by_id
                .insert(model.model_id().into(), model.into());

            Ok(())
        };

        for model in &self.config.load_models {
            match load_model(model) {
                Ok(()) => info!("Loaded startup model {}.", model.model_id),
                Err(err) => error!(
                    "Loading of startup model {} failed! {:?}",
                    model.model_id, err
                ),
            }
        }

        Ok(())
    }
}
