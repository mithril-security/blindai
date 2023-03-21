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

use crate::model::ModelDatumType;
use crate::model_store::ModelStore;
use anyhow::{Error, Result};
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use std::io::Read;
use std::mem::size_of;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TensorInfo {
    pub fact: Vec<usize>,
    pub datum_type: ModelDatumType,
    pub node_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SerializedTensor {
    pub info: TensorInfo,
    #[serde(with = "serde_bytes")]
    pub bytes_data: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct Exchanger {
    model_store: Arc<ModelStore>,
    max_model_size: usize,
    max_input_size: usize,
}

#[derive(Deserialize)]
struct DeleteModel {
    model_id: String,
}

#[derive(Deserialize)]
pub(crate) struct RunModel {
    model_id: String,
    model_hash: String,
    pub inputs: Vec<SerializedTensor>,
}

#[derive(Deserialize)]
struct UploadModel {
    #[serde(with = "serde_bytes")]
    model: Vec<u8>,
    length: u64,
    model_name: String,
    optimize: bool,
}

#[derive(Serialize)]
pub(crate) struct SendModelReply {
    #[serde(with = "serde_bytes")]
    hash: Vec<u8>,
    model_id: String,
}

#[derive(Default, Serialize)]
pub(crate) struct RunModelReply {
    outputs: Vec<SerializedTensor>,
}

impl Exchanger {
    pub fn new(model_store: Arc<ModelStore>, max_model_size: usize, max_input_size: usize) -> Self {
        Self {
            model_store,
            max_model_size,
            max_input_size,
        }
    }

    pub fn send_model(&self, request: &rouille::Request) -> Result<SendModelReply, Error> {
        let upload_model_body: UploadModel = {
            let mut data: Vec<u8> = vec![];
            request
                .data()
                .expect("Could not get input")
                .read_to_end(&mut data)?;
            serde_cbor::from_slice(&data)?
        };

        let max_model_size = self.max_model_size;
        let mut model_size = 0usize;

        let mut model_name: std::option::Option<String> = None;

        if model_size == 0 {
            model_size = upload_model_body.length.try_into()?;
            model_name = if !upload_model_body.model_name.is_empty() {
                Some(upload_model_body.model_name)
            } else {
                None
            };
        }
        if model_size > max_model_size {
            return Err(Error::msg("Model is too big".to_string()));
        }

        if model_size == 0 {
            return Err(Error::msg("Received no data".to_string()));
        }

        let (model_id, model_hash) = self.model_store.add_model(
            &upload_model_body.model,
            model_name,
            upload_model_body.optimize,
        )?;

        // Construct the return payload
        Ok(SendModelReply {
            hash: model_hash.as_ref().to_vec(),
            model_id: model_id.to_string(),
        })
    }

    pub fn run_model(&self, request: &rouille::Request) -> Result<RunModelReply, Error> {
        let max_input_size = self.max_input_size;

        let mut data_stream = request.data().expect("Could not get the input");
        let mut data: Vec<u8> = vec![];
        data_stream.read_to_end(&mut data)?;

        let run_model_body: RunModel = serde_cbor::from_slice(&data)?;

        if run_model_body.model_id.is_empty() && run_model_body.model_hash.is_empty() {
            error!("Model_id and model_hash are empty");
            return Err(Error::msg(
                "You must provide at least one model_id or model_hash".to_string(),
            ));
        }

        if !run_model_body.model_id.is_empty() && !run_model_body.model_hash.is_empty() {
            error!("Model_id and model_hash are NOT empty, cannot pick one over the other");
            return Err(Error::msg(
                "You cannot provide a model_id and a model_hash in the same time".to_string(),
            ));
        }

        if run_model_body.inputs.len() * size_of::<u8>() > max_input_size
            || run_model_body.inputs.len() * size_of::<u8>() > max_input_size
        {
            return Err(Error::msg("Input too big".to_string()));
        }

        let uuid = if !run_model_body.model_hash.is_empty() {
            match self
                .model_store
                .get_uuid_from_hash(&run_model_body.model_hash)
            {
                Some(uuid) => uuid,
                None => {
                    error!("Hash not found");
                    return Err(Error::msg("Model doesn't exist".to_string()));
                }
            }
        } else {
            match Uuid::from_str(&run_model_body.model_id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    error!("Error in uuid");
                    return Err(Error::msg("Model doesn't exist".to_string()));
                }
            }
        };

        let res = self.model_store.use_model(uuid, |model| {
            // uncomment to run benches
            // bench(3, 50, || {
            //     model.run_inference(&mut run_model_body.inputs.clone()[..]);
            // });
            (
                model.run_inference(run_model_body.inputs.as_slice()),
                model.model_name().map(|s| s.to_string()),
            )
        });

        let res = match res {
            Some(res) => res,
            None => {
                error!("Error in model match");
                return Err(Error::msg("Model doesn't exist".to_string()));
            }
        };

        let (result, _model_name) = res;

        let outputs = match result {
            Ok(res) => res,
            Err(err) => {
                error!("Error while running inference: {}", err);
                return Err(Error::msg("Unknown error".to_string()));
            }
        };

        Ok(RunModelReply { outputs })
    }

    pub fn delete_model(&self, request: &rouille::Request) -> Result<()> {
        let mut data_stream = request.data().expect("Could not get the input");
        let mut data: Vec<u8> = vec![];
        data_stream.read_to_end(&mut data)?;

        let delete_model_body: DeleteModel = serde_cbor::from_slice(&data)?;

        if delete_model_body.model_id.is_empty() {
            return Err(Error::msg("Model doesn't exist".to_string()));
        }

        let model_id = Uuid::from_str(&delete_model_body.model_id)?;

        // Delete the model
        if self.model_store.delete_model(model_id).is_none() {
            error!("Model doesn't exist");
            return Err(Error::msg("Model doesn't exist".to_string()));
        }
        Ok(())
    }

    pub fn respond<Reply: serde::Serialize>(
        &self,
        _rq: &rouille::Request,
        reply: Result<Reply>,
    ) -> rouille::Response {
        match reply {
            Ok(reply) => rouille::Response::from_data(
                "application/cbor",
                serde_cbor::to_vec(&reply).unwrap(),
            ),
            Err(e) => rouille::Response::from_data(
                "application/cbor",
                serde_cbor::to_vec(&format!("{:?}", &e)).unwrap(),
            )
            .with_status_code(500),
        }
    }
}

#[allow(dead_code)]
pub fn bench(repeats: usize, samples: usize, f: impl Fn()) -> Result<()> {
    let mut results = vec![];
    results.reserve(samples);

    for i in 1..=samples {
        let start = Instant::now();
        for _ in 0..repeats {
            f();
        }
        let elapsed = start.elapsed().as_micros() / repeats as u128;

        info!("bench (sample {i}/{samples}): {elapsed}us/iter, {repeats} iter");

        results.push(elapsed);
    }

    let mean = results.iter().copied().sum::<u128>() as f64 / results.len() as f64;
    let variance: f64 = results
        .iter()
        .map(|res| (*res as f64 - mean).powf(2.0))
        .sum::<f64>()
        / results.len() as f64;
    let std_deviation = variance.sqrt();
    info!("Mean {}", mean / 1000.0);
    info!("Variance {}", variance / 1000.0);
    info!("Std deviation {}", std_deviation / 1000.0);

    Ok(())
}
