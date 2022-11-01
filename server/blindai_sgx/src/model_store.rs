use crate::{
    model::ModelLoadContext,
    sealing::{self},
};
use std::path::{Path, PathBuf};

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
    time::SystemTime,
};

use crate::model::{InferModel, TensorFacts};

#[derive(Debug)]
pub enum ModelStoreError {
    Other(anyhow::Error),
}

impl From<anyhow::Error> for ModelStoreError {
    fn from(e: anyhow::Error) -> Self {
        ModelStoreError::Other(e)
    }
}

pub struct StoredModel {
    pub model: Option<InferModel>,
    pub last_use: SystemTime,
}

impl From<InferModel> for StoredModel {
    fn from(m: InferModel) -> Self {
        StoredModel {
            model: Some(m),
            last_use: SystemTime::now(),
        }
    }
}

#[derive(Default)]
pub struct ModelsMap {
    pub map: HashMap<String, StoredModel>,
    nb_loaded_models: usize,
    owner_id: usize
}

#[derive(Default)]
pub struct UsersMap {
    pub map: HashMap<Option<String>, ModelsMap>,
    nb_loaded_models: usize,
}

trait ModelClock {
    fn get_oldest_loaded(&mut self) -> Option<&mut StoredModel>;
    fn get_oldest_unloaded(&self) -> Option<(&str, Option<&str>, SystemTime)>;
}

impl ModelClock for ModelsMap {
    fn get_oldest_loaded(&mut self) -> Option<&mut StoredModel> {
        let mut oldest: Option<&mut StoredModel> = None;
        for (_, m) in self.map.iter_mut() {
            if let None = m.model {
                continue ;
            }
            if let Some(model) = &m.model {
                if model.load_context() == ModelLoadContext::FromStartupConfig {
                    continue ;
                }
            }
            if let None = oldest {
                oldest = Some(m);
            } else {
                oldest = oldest.map(|oldest| {
                    if oldest.last_use > m.last_use {
                        info!("{:?}", m.last_use);
                        m
                    } else {
                        oldest
                    }
                });
            }
        }
        return oldest;
    }

    fn get_oldest_unloaded(&self) -> Option<(&str, Option<&str>, SystemTime)> {
        let mut oldest = SystemTime::now();
        let mut oldest_id = "";
        if self.map.is_empty() {
            return None;
        }
        for (k, m) in self.map.iter() {
            if oldest > m.last_use {
                oldest = m.last_use;
                oldest_id = k;
            }
        }
        return Some((oldest_id, None, oldest));
    }
}

impl ModelClock for UsersMap {
    fn get_oldest_loaded(&mut self) -> Option<&mut StoredModel> {
        let mut oldest: Option<&mut StoredModel> = None;
        for (_, user_map) in self.map.iter_mut() {
            if let Some(o) = user_map.get_oldest_loaded() {
                if let None = oldest {
                    oldest = Some(o);
                } else {
                    oldest = oldest.map(|oldest| {
                        if oldest.last_use > o.last_use {
                            o
                        } else {
                            oldest
                        }
                    });
                }
            }
        }
        return oldest;
    }

    fn get_oldest_unloaded(&self) -> Option<(&str, Option<&str>, SystemTime)> {
        let mut oldest: Option<(&str, Option<&str>, SystemTime)> = None;
        if self.map.is_empty() {
            return None;
        }
        for (k, user_map) in self.map.iter() {
            if let None = k {
                continue ;
            }
            if let Some(o)= user_map.get_oldest_unloaded() {
                if let None = oldest {
                    oldest = Some((o.0, k.as_deref(), o.2))
                } else {
                    oldest = oldest.map(|(model_id, username, time)| {
                        if time > o.2 {
                            (o.0, k.as_deref(), o.2)
                        } else {
                            (model_id, username, time)
                        }
                    });
                }
            }
        }
        return oldest;
    }
}

#[derive(Default)]
pub struct InnerModelStore {
    models: UsersMap,
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
        user_id: Option<&str>,
        username: Option<&str>,
    ) -> Result<(String, Digest), ModelStoreError> {
        let model_id = model_id.unwrap_or_else(|| model_name.clone().unwrap_or_else(|| Uuid::new_v4().to_string()));

        let model_hash = digest::digest(&digest::SHA256, &model_bytes);
        info!("Model hash is {:?}", model_hash);

        let mut models_path = PathBuf::new();
        models_path.push(&self.config.models_path);
        let model_id = match key_from_id_and_username(&model_id, username) {
            Some(id) => id,
            None => {
                error!("Invalid model name");
                return Err(ModelStoreError::from(anyhow!("Invalid model name")));
            },
        };
        models_path.push(&model_id);

        if models_path.as_path().exists() && load_context != ModelLoadContext::FromLoadingFromDisk {
            return Err(ModelStoreError::from(anyhow!("A model with the same name already exists, delete it first if you want to replace it")));
        }

        // Sealing
        let owner_id = user_id.map(|id| id.parse::<usize>().unwrap());
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

        let model = InferModel::load_model(
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

        {
            // take the write lock
            let mut write_guard = self.inner.write().unwrap(); // take the write lock

            let usermap = &mut write_guard.models;
            let d = if usermap.nb_loaded_models == self.config.max_model_store.unwrap_or(usize::max_value()) {
                info!("Global loaded model limit reached. The oldest model used will be unloaded...");
                usermap.get_oldest_loaded().unwrap().model.take(); // drop model
                usermap.nb_loaded_models -= 1;
                true
            } else {
                false
            };

            let models = usermap.map.entry(username.map(str::to_string))
            .or_insert(ModelsMap::default());
            if models.nb_loaded_models >= self.config.max_loaded_model_per_user.unwrap_or(usize::max_value()) {
                info!("User loaded model limit reached. The oldest model used will be unloaded...");
                if !d {
                    models.get_oldest_loaded().unwrap().model.take(); // drop model
                }
                models.nb_loaded_models -= 1;
            }
            if save_model {
                if models.map.len() >= self.config.max_sealed_model_per_user.unwrap_or(usize::max_value()) {
                    info!("User sealed model limit reached. The oldest model used will be deleted...");
                    let (oldest_id, _, _) = models.get_oldest_unloaded().unwrap();
                    let oldest_id = oldest_id.to_owned();
                    if let Some(path) = path_from_key(&self.config.models_path, &oldest_id) {
                        let _ = fs::remove_file(path);
                    }
                    models.map.remove(&oldest_id);
                }
            }
            models.map.insert(model_id.clone(), model.into());
            models.owner_id = owner_id.unwrap_or(0);
            models.nb_loaded_models += 1;
            write_guard.models.nb_loaded_models += 1;
            crate::sealing::seal_metadata(&write_guard.models)?;
            info!("Metadata sealed");
        }
        Ok((model_id, model_hash))
    }

    //Unseal function that unseal the model if we find it in the seal model
    pub fn unseal(&self, id_to_fetch: &str, user_id: Option<&str>, username: Option<&str>, path: &Path) -> Result<()> {
        if let Some(id_to_fetch) = key_from_id_and_username(id_to_fetch, username) {
            if let Ok(model) = sealing::unseal(path)
            {
                if id_to_fetch == model.model_id {
                    self.add_model(
                        &model.model_bytes,
                        model.model_name,
                        Some(model.model_id.clone()),
                        &model.input_facts,
                        &model.output_facts,
                        false,
                        model.optim,
                        ModelLoadContext::FromLoadingFromDisk,
                        user_id,
                        username,
                    )
                    .map_err(|err| anyhow!("Adding model failed: {:?}", err))?;
                    return Ok(());
                }
            }
        }
        Err(anyhow!("Sealed model not found"))
    }

    pub fn use_model<U>(&self, model_id: &str, user_id: Option<&str>, username: Option<&str>, disable_ownership_check: bool, fun: impl FnOnce(&InferModel) -> U) -> Option<U> {
        if let Some(key) = key_from_id_and_username(model_id, username) {
            // take a write lock
            let mut write_guard = self.inner.write().unwrap();
            if let Some(models) = write_guard.models.map.get_mut(&username.map(str::to_string)) {
                let owner = user_id.map(|id| id.parse::<usize>().unwrap());
                if let Some(user) = owner {
                    if models.owner_id != user && !disable_ownership_check {
                        drop(write_guard);
                        info!("Attempt at accessing a model from unauthorized namespace {:?} userid:{:?}", model_id, user);
                        return None;
                    }
                }
                else {
                    if models.owner_id != 0 {
                        drop(write_guard);
                        info!("Attempt from a guest at accessing a model from unauthorized namespace {:?}", model_id);
                        return None;
                    }
                }
                if let Some(mut model) = models.map.get_mut(&key) {
                    if let None = model.model {
                        drop(write_guard);
                        if let Some(path) = path_from_key(&self.config.models_path, &key) {
                            if let Err(e) = self.unseal(&key, user_id, username, &Path::new(&path)) {
                                info!("{:?}", e);
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        model.last_use = SystemTime::now(); //record date of use
                        drop(write_guard);
                    }
                    let read_guard = self.inner.read().unwrap(); //switch to read guard for inference
                    let model = read_guard.models.map.get(&username.map(str::to_string)).unwrap().map.get(&key).unwrap();
                    return Some(fun(model.model.as_ref().unwrap()));
                }
                else if let Some(_) = username {
                    drop(write_guard);
                    return self.use_model(model_id, None, None, false, fun); // if model not found for user, search again in public namespace
                }
            }
            else if let Some(_) = username {
                drop(write_guard);
                return self.use_model(model_id, None, None, false, fun); // if model not found for user, search again in public namespace
            }
        }
        None
    }

    /// If user_id is provided, it will fail if the model is not owned by this
    /// user. This will never remove startup models.
    pub fn delete_model(&self, model_id: &str, username: Option<&str>) -> Option<()> {
        if let Some(key) = key_from_id_and_username(model_id, username) {
            let mut write_guard = self.inner.write().unwrap();
            if let Some(map) = write_guard.models.map.get_mut(&username.map(str::to_string)) {
                if let Some(model) = map.map.get(&key).and_then(|m| m.model.as_ref()) {
                    if model.load_context() == ModelLoadContext::FromStartupConfig {
                        return None;
                    }                }
                if let Some(_) = map.map.remove(&key).and_then(|m| m.model) {
                    map.nb_loaded_models -= 1;
                    write_guard.models.nb_loaded_models -= 1;
                }
            }
            if let Some(path) = path_from_key(&self.config.models_path, &key) {
                let _ = fs::remove_file(&path);
                return Some(());
            }
        }
        None
    }

    pub fn prune_old_models(&self) -> usize {
        let mut nb = 0;
        loop {
            let mut model = None;
            {
                let read_guard = self.inner.read().unwrap(); // take read lock
                if let Some((model_id, username, last_use)) = read_guard.models.get_oldest_unloaded() {
                    if SystemTime::now() - std::time::Duration::from_secs((self.config.daily_model_cleanup.unwrap() * 60 * 60 * 24).try_into().unwrap()) >= last_use {
                        model = Some((model_id.to_string(), username.map(|s| s.to_string())));
                        nb += 1
                    }
                }
            }
            if let Some((model_id, username)) = model {
                self.delete_model(&model_id, username.as_deref());
                info!("model {} owned by user {:?} automatically removed", &model_id, username.as_deref());
            } else {
                break nb
            }
        }
    }

    pub fn check_seal_file_exist(&self) -> Result<()> {
        if let Ok(_paths) = fs::read_dir(&self.config.models_path) {
        } else {
            fs::create_dir(&self.config.models_path)?;
        }
        Ok(())
    }

    pub fn load_config_models(&self) -> Result<()> {
        let mut write_guard = self.inner.write().unwrap();

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
            let h = write_guard
                .models
                .map
                .entry(None)
                .or_insert(ModelsMap::default());
            h.map.insert(model.model_id().into(), model.into());
            write_guard.models.nb_loaded_models += 1;
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

    pub fn load_metadata(&self) -> anyhow::Result<()> {
        let meta = crate::sealing::unseal_metadata();
        match meta {
            Ok(users_map) => {
                self.inner.write().unwrap().models = users_map;
            }
            Err(err) => {
                return Err(err);
            }
        }
        Ok(())
    }

}

pub fn key_from_id_and_username(model_id: &str, username: Option<&str>) -> Option<String> {
    let split_model_id: Vec<&str> = model_id.split('/').collect();
    if let Some(username) = username {
        if split_model_id.len() > 2 {
            return None;
        }
        if split_model_id.len() == 2 {
            if split_model_id[0] != username {
                return None;
            }
            return Some(model_id.to_owned());
        }
        return Some(username.to_owned() + "/" + model_id);
    }
    Some(model_id.to_owned())
}

pub fn path_from_key(model_path: &str, key: &str) -> Option<String> {
    Some(model_path.to_string() + "/" + key)
}