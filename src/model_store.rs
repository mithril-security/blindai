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

use anyhow::{anyhow, Result};
use log::*;
use ring::digest::{self, Digest};

use std::sync::RwLock;

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use uuid::Uuid;

use crate::model::{InferenceModel, OnnxModel};

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
        model_name: Option<String>,
        optimize: bool,
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
                        Arc::clone(onnx),
                        model_id,
                        model_name,
                        model_hash,
                    )
                }
                Entry::Vacant(entry) => {
                    info!("Creating a new ONNX entry for model.");
                    // FIXME(cchudant): this call may take a while to run, we may want to refactor
                    // this so that the lock  isn't taken here
                    let model = InferenceModel::load_model(
                        model_bytes,
                        model_id,
                        model_name,
                        model_hash,
                        optimize,
                    )?;
                    entry.insert((1, Arc::clone(&model.onnx)));
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

    pub fn get_uuid_from_hash(&self, model_hash: &str) -> Option<Uuid> {
        let read_guard = self.inner.read().unwrap();
        let digest = ring::test::from_hex(model_hash).unwrap();
        for val in read_guard.models_by_id.iter() {
            if val.1.model_hash().as_ref() == &digest[..] {
                return Some(val.0.to_owned());
            }
        }
        None
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
