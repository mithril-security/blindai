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

use crate::identity::MyIdentity;
use crate::model::ModelDatumType;
use crate::model_store::ModelStore;
use anyhow::{Error, Result};
use log::error;
use ring::digest;
use ring_compat::signature::Signer;
use serde_derive::{Deserialize, Serialize};
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
    pub bytes_data: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct Exchanger {
    model_store: Arc<ModelStore>,
    identity: Arc<MyIdentity>,
    max_model_size: usize,
    max_input_size: usize,
}

#[derive(Deserialize)]
struct DeleteModel {
    model_id: String,
}

#[derive(Deserialize)]
pub struct RunModel {
    model_id: String,
    pub inputs: Vec<SerializedTensor>,
    sign: bool,
}

#[derive(Deserialize)]
struct UploadModel {
    model: Vec<u8>,
    input: Vec<TensorInfo>,
    output: Vec<ModelDatumType>,
    length: u64,
    sign: bool,
    model_name: String,
    optimize: bool,
}

#[derive(Default, Serialize)]
struct SendModelPayload {
    hash: Vec<u8>,
    input_fact: Vec<i32>,
    model_id: String,
}

#[derive(Default, Serialize)]
pub struct SendModelReply {
    payload: Vec<u8>, //sendModelPayload,
    signature: Vec<u8>,
}

#[derive(Default, Serialize)]
struct RunModelPayload {
    outputs: Vec<SerializedTensor>,
    input_hash: Vec<u8>,
    model_id: String,
}

#[derive(Default, Serialize)]
pub struct RunModelReply {
    payload: Vec<u8>, //runModelPayload,
    signature: Vec<u8>,
}

impl Exchanger {
    pub fn new(
        model_store: Arc<ModelStore>,
        identity: Arc<MyIdentity>,
        max_model_size: usize,
        max_input_size: usize,
    ) -> Self {
        Self {
            identity,
            model_store,
            max_model_size,
            max_input_size,
        }
    }

    pub fn send_model(&self, request: &mut tiny_http::Request) -> Result<SendModelReply, Error> {
        let upload_model_body: UploadModel = {
            let mut data: Vec<u8> = vec![];
            request.as_reader().read_to_end(&mut data)?;
            serde_cbor::from_slice(&data)?
        };

        let convert_type = |t: i32| -> Result<_, Error> {
            num_traits::FromPrimitive::from_i32(t)
                .ok_or_else(|| Error::msg("Unknown datum type".to_string()))
        };

        let mut tensor_inputs: Vec<TensorInfo> = Vec::new();
        let mut tensor_outputs: Vec<i32> = Vec::new();

        let mut datum_outputs: Vec<ModelDatumType> = Vec::new();
        let mut datum_inputs: Vec<ModelDatumType> = Vec::new();
        let mut input_facts: Vec<Vec<usize>> = Vec::new();
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

            for tensor_info in &upload_model_body.input {
                tensor_inputs.push(tensor_info.clone());
            }

            for output in &upload_model_body.output {
                tensor_outputs.push((*output) as i32);
            }
        }
        if model_size > max_model_size {
            return Err(Error::msg("Model is too big".to_string()));
        }

        if model_size == 0 {
            return Err(Error::msg("Received no data".to_string()));
        }

        // Create datum_inputs, datum_outputs, and input_facts vector from tensor_inputs
        // and tensor_outputs
        for (_, tensor_input) in tensor_inputs.clone().iter().enumerate() {
            let mut input_fact: Vec<usize> = vec![];

            for x in &tensor_input.fact {
                input_fact.push(*x);
            }
            let datum_input = convert_type(tensor_input.datum_type as i32)?; // TEMP-FIX, FIX THIS!//convert_type(tensor_input.datum_type.clone())?;
            datum_outputs = tensor_outputs
                .iter()
                .map(|v| convert_type(*v).unwrap())
                .collect();
            datum_inputs.push(datum_input);
            input_facts.push(input_fact.clone());
        }

        let (model_id, model_hash) = self.model_store.add_model(
            &upload_model_body.model,
            input_facts.clone(),
            model_name,
            datum_inputs.clone(),
            datum_outputs,
            upload_model_body.optimize,
        )?;

        // Construct the return payload

        let mut payload = SendModelPayload::default();
        if upload_model_body.sign {
            payload.hash = model_hash.as_ref().to_vec();
            payload.input_fact = input_facts
                .into_iter()
                .flatten()
                .map(|i| i as i32)
                .collect();
        }
        payload.model_id = model_id.to_string();

        let mut reply = SendModelReply {
            payload: serde_cbor::to_vec(&payload)?,
            ..Default::default()
        };

        if upload_model_body.sign {
            reply.signature = self
                .identity
                .signing_key
                .sign(&reply.payload)
                .to_bytes()
                .to_vec();
        }

        Ok(reply)
    }

    pub fn run_model(&self, request: &mut tiny_http::Request) -> Result<RunModelReply, Error> {
        let input: Vec<u8> = Vec::new();
        let sign = false;
        let max_input_size = self.max_input_size;
        let model_id = "".to_string();

        let data_stream = request.as_reader();
        let mut data: Vec<u8> = vec![];
        data_stream.read_to_end(&mut data)?;

        let run_model_body: RunModel = serde_cbor::from_slice(&data)?;

        if run_model_body.inputs.len() * size_of::<u8>() > max_input_size
            || run_model_body.inputs.len() * size_of::<u8>() > max_input_size
        {
            return Err(Error::msg("Input too big".to_string()));
        }

        let uuid = match Uuid::from_str(&run_model_body.model_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                println!("Error in uuid");
                return Err(Error::msg("Model doesn't exist".to_string()));
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
                println!("Error in model match");
                return Err(Error::msg("Model doesn't exist".to_string()));
            }
        };

        let (result, _model_name) = res;

        let outputs = match result {
            Ok(res) => res,
            Err(err) => {
                error!("Error while running inference: {}", err);
                println!("Error in result");
                return Err(Error::msg("Unknown error".to_string()));
            }
        };

        let mut payload = RunModelPayload {
            outputs,
            ..Default::default()
        };

        if run_model_body.sign {
            payload.input_hash = digest::digest(&digest::SHA256, &input).as_ref().to_vec();
            payload.model_id = model_id;
        }

        let mut reply = RunModelReply {
            payload: serde_cbor::to_vec(&payload)?,
            ..Default::default()
        };

        if sign {
            reply.signature = self
                .identity
                .signing_key
                .sign(&reply.payload)
                .to_bytes()
                .to_vec();
        }

        Ok(reply)
    }

    pub fn delete_model(&self, request: &mut tiny_http::Request) -> Result<()> {
        let data_stream = request.as_reader();
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

    pub fn respond<Reply: serde::Serialize>(&self, rq: tiny_http::Request, reply: Result<Reply>) {
        let (serialized_reply, code) = match reply {
            Ok(reply) => (serde_cbor::to_vec(&reply).unwrap(), 200),
            Err(e) => (serde_cbor::to_vec(&format!("{:?}", &e)).unwrap(), 400),
        };
        let header =
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap();
        let response = tiny_http::Response::new(
            tiny_http::StatusCode::from(code),
            vec![header],
            serialized_reply.as_slice(),
            None,
            None,
        );
        rq.respond(response).unwrap();
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

        println!("bench (sample {i}/{samples}): {elapsed}us/iter, {repeats} iter");

        results.push(elapsed);
    }

    let mean = results.iter().copied().sum::<u128>() as f64 / results.len() as f64;
    let variance: f64 = results
        .iter()
        .map(|res| (*res as f64 - mean).powf(2.0))
        .sum::<f64>()
        / results.len() as f64;
    let std_deviation = variance.sqrt();
    println!("Mean {}", mean / 1000.0);
    println!("Variance {}", variance / 1000.0);
    println!("Std deviation {}", std_deviation / 1000.0);

    Ok(())
}
