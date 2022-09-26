use crate::{
    model::ModelLoadContext,
    sealing::{self},
};
use std::{
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use blindai_common::{BlindAIConfig, LoadModelConfig, ModelFactsConfig};
use log::*;
use ring::digest::{self, Digest};
use uuid::Uuid;

#[cfg(not(target_env = "sgx"))]
use std::fs;
#[cfg(target_env = "sgx")]
use std::untrusted::fs;

#[cfg(not(target_env = "sgx"))]
use std::sync::RwLock;
#[cfg(target_env = "sgx")]
use std::sync::SgxRwLock as RwLock;

use std::{
    collections::{HashMap},
    sync::Arc,
};

use crate::model::{InferModel, TensorFacts};

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
        username: Option<&str>,
    ) -> Result<(String, Digest), ModelStoreError> {
        let model_id = model_id.unwrap_or_else(|| model_name.clone().unwrap_or_else(|| Uuid::new_v4().to_string()));

        let model_hash = digest::digest(&digest::SHA256, &model_bytes);
        info!("Model hash is {:?}", model_hash);

        let mut models_path = PathBuf::new();
        models_path.push(&self.config.models_path);
        let mut model_id = match key_from_id_and_username(&model_id, username, true) {
            Some(id) => id,
            None => {
                error!("Invalid model name");
                return Err(ModelStoreError::from(anyhow!("Invalid model name")));
            },
        };
        models_path.push(&model_id);
        if save_model && self.config.allow_model_sealing {
            model_id = model_id + "#" + &Uuid::new_v4().to_string();
        }

        // Sealing
        if save_model {
            info!(
                "{}",
                if self.config.allow_model_sealing {
                    "Sealing model..."
                } else {
                    "Sealing model NOT allowed, model will be saved in clear on the disk"
                }
            );
            sealing::seal(
                models_path.as_path(),
                &model_bytes,
                model_name.as_deref(),
                &model_id,
                &input_facts,
                &output_facts,
                optim,
                owner_id,
                self.config.allow_model_sealing,
            )
            .context("Sealing the model")?;
            info!(
                "{}",
                if self.config.allow_model_sealing {
                    "Model sealed"
                } else {
                    "Model saved on disk"
                }
            );
        }

        // Create an entry in the hashmap
        {
            let model = Arc::new(InferModel::load_model(
                &model_bytes,
                model_id.clone(),
                model_name,
                model_hash,
                input_facts,
                output_facts,
                optim,
                load_context,
                owner_id,
            )?);

            // take the write lock
            let mut write_guard = self.inner.write().unwrap();
            write_guard.models_by_id.insert(model_id.clone(), model);
        }

        Ok((model_id, model_hash))
    }

    // Check if model is in the already in the server or no
    pub fn in_the_server(&self, id_to_fetch: String, username: Option<&str>) -> bool {
        let read_guard = self.inner.read().unwrap();

        match key_from_id_and_username(&id_to_fetch, username, self.config.allow_model_sealing) {
            Some(key) => match read_guard.models_by_id.get(&key) {
                Some(_model) => true,
                None => false,
            }
            None => false,
        }
    }

    //Unseal function that unseal the model if we find it in the seal model, and
    // return true or false if we find it.
    pub fn unseal(&self, id_to_fetch: String, username: Option<&str>) -> Result<()> {
        if let Some(id_to_fetch) = key_from_id_and_username(&id_to_fetch, username, self.config.allow_model_sealing) {
            // take a read lock
            if let Ok(paths) = fs::read_dir(&self.config.models_path) {
                for path in paths {
                    let path = path?;
                    if let Ok(model) =
                        sealing::unseal(path.path().as_path(), self.config.allow_model_sealing)
                    {
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
                                None,
                            )
                            .map_err(|err| anyhow!("Adding model failed: {:?}", err))?;
                            info!("Model {:?} loaded", model.model_id);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn use_model<U>(&self, model_id: &str, username: Option<&str>, fun: impl FnOnce(&InferModel) -> U) -> Option<U> {
        if let Some(key) = key_from_id_and_username(model_id, username, self.config.allow_model_sealing) {
            // take a read lock
            let read_guard = self.inner.read().unwrap();

            return match read_guard.models_by_id.get(&key) {
                Some(model) => Some(fun(model)),
                None => None,
            };
        }
        None
    }

    /// If user_id is provided, it will fail if the model is not owned by this
    /// user. This will never remove startup models.
    pub fn delete_model(&self, model_id: &str, username: Option<&str>) -> Option<Arc<InferModel>> {
        if let Some(key) = key_from_id_and_username(model_id, username, self.config.allow_model_sealing) {
            let mut write_guard = self.inner.write().unwrap();
            return write_guard.models_by_id.remove(&key);
        }
        None
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
                format!("startup model - {}", model.model_id.clone()).into(),
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

pub fn key_from_id_and_username(model_id: &str, username: Option<&str>, private: bool) -> Option<String> {
    let split_model_id: Vec<&str> = model_id.split('/').collect();
    if split_model_id.len() > 2 {
        return None;
    }
    if split_model_id.len() == 2 {
        if private == true && split_model_id[0] != username.unwrap_or_default() {
            return None;
        }
        return Some(model_id.to_owned());
    }
    if let Some(username) = username {
        return Some(username.to_owned() + "/" + model_id);
    }
    Some(model_id.to_owned())
}