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

use futures::StreamExt;
use log::*;
use prost::Message;
use ring::digest::{self, Digest};
use ring_compat::signature::Signer;
use std::{
    convert::TryInto,
    mem::size_of,
    str::FromStr,
    sync::Arc,
    time::{Instant, SystemTime},
    vec::Vec,
};

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    client_communication::secured_exchange::TensorInfo,
    identity::MyIdentity,
    model::ModelDatumType,
    model_store::ModelStore,
    telemetry::{self, TelemetryEventProps},
};

use secured_exchange::{exchange_server::Exchange, *};

pub mod secured_exchange {
    tonic::include_proto!("securedexchange");
}

pub(crate) struct Exchanger {
    pub model_store: Arc<ModelStore>,
    identity: Arc<MyIdentity>,
    max_model_size: usize,
    max_input_size: usize,
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
}

#[tonic::async_trait]
impl Exchange for Exchanger {
    async fn send_model(
        &self,
        request: Request<tonic::Streaming<SendModelRequest>>,
    ) -> Result<Response<SendModelReply>, Status> {
        let start_time = Instant::now();

        let convert_type = |t: i32| -> Result<_, Status> {
            num_traits::FromPrimitive::from_i32(t)
                .ok_or_else(|| Status::invalid_argument("Unknown datum type".to_string()))
        };

        let mut stream = request.into_inner();
        let mut tensor_inputs: Vec<TensorInfo> = Vec::new();
        let mut tensor_outputs: Vec<i32> = Vec::new();

        let mut datum_outputs: Vec<ModelDatumType> = Vec::new();
        let mut datum_inputs: Vec<ModelDatumType> = Vec::new();
        let mut input_facts: Vec<Vec<usize>> = Vec::new();
        let mut model_bytes: Vec<u8> = Vec::new();
        let max_model_size = self.max_model_size;
        let mut model_size = 0usize;
        let mut sign = false;
        let mut sealed = false;

        let mut model_name = None;
        let mut client_info = None;

        // Get all model chunks from the client into a big Vec
        while let Some(model_stream) = stream.next().await {
            let mut model_proto = model_stream?;
            if model_size == 0 {
                model_size = model_proto.length.try_into().unwrap();
                model_bytes.reserve_exact(model_size);

                model_name = if !model_proto.model_name.is_empty() {
                    Some(model_proto.model_name)
                } else {
                    None
                };
                client_info = model_proto.client_info;

                for tensor_info in &model_proto.tensor_inputs {
                    tensor_inputs.push(tensor_info.clone());
                }

                for output in &model_proto.tensor_outputs {
                    tensor_outputs.push(*output);
                }

                sign = model_proto.sign;
                sealed = model_proto.sealed;
            }
            if model_size > max_model_size || model_bytes.len() > max_model_size {
                return Err(Status::invalid_argument("Model is too big".to_string()));
            }
            model_bytes.append(&mut model_proto.data)
        }
        if model_size == 0 {
            return Err(Status::invalid_argument("Received no data".to_string()));
        }

        // Create datum_inputs, datum_outputs, and input_facts vector from tensor_inputs
        // and tensor_outputs
        for (_, tensor_input) in tensor_inputs.clone().iter().enumerate() {
            let mut input_fact: Vec<usize> = vec![];

            for x in &tensor_input.fact {
                input_fact.push(*x as usize);
            }
            let datum_input = convert_type(tensor_input.datum_type.clone())?;
            datum_outputs = tensor_outputs
                .iter()
                .map(|v| convert_type(*v).unwrap())
                .collect();
            datum_inputs.push(datum_input.clone());
            input_facts.push(input_fact.clone());
        }


        // Optimize, save and seal
        let mut model_id = Uuid::new_v4();
        let mut models_path = std::env::current_dir()?;
        models_path.push("models");
        models_path.push(model_id.to_string());
        let mut model_hash: Digest =digest::digest(&digest::SHA256, &model_bytes);
        if sealed {
        let (model_id_new, model_hash_new) = self
            .model_store
            .add_model(
                models_path.as_path(),
                &model_bytes,
                input_facts.clone(),
                model_name.clone(),
                model_id,
                datum_inputs.clone(),
                datum_outputs.clone(),
            )
            .map_err(|err| {
                error!("Error while creating model: {}", err);
                Status::unknown("Unknown error".to_string())
            })?;
        model_id=model_id_new;
        model_hash=model_hash_new
        }
        // Construct the return payload
        let mut payload = SendModelPayload::default();
        if sign {
            payload.model_hash = model_hash.as_ref().to_vec();
            payload.input_fact = input_facts
                .into_iter()
                .flatten()
                .map(|i| i as i32)
                .collect();
        }
        payload.model_id = model_id.to_string();
        let payload_with_header = Payload {
            header: Some(PayloadHeader {
                issued_at: Some(SystemTime::now().into()),
            }),
            payload: Some(payload::Payload::SendModelPayload(payload)),
        };

        let mut reply = SendModelReply {
            payload: payload_with_header.encode_to_vec(),
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

        // Logs and telemetry
        let elapsed = start_time.elapsed();
        info!(
            "[{} {}] SendModel successful in {}ms (model={}, size={}, sign={})",
            client_info
                .as_ref()
                .map(|c| c.user_agent.as_ref())
                .unwrap_or("<unknown>"),
            client_info
                .as_ref()
                .map(|c| c.user_agent_version.as_ref())
                .unwrap_or("<unknown>"),
            elapsed.as_millis(),
            model_name.as_deref().unwrap_or("<unknown>"),
            model_size,
            sign
        );
        telemetry::add_event(
            TelemetryEventProps::SendModel {
                model_size,
                model_name,
                sign,
                time_taken: elapsed.as_secs_f64(),
            },
            client_info,
        );

        Ok(Response::new(reply))
    }

    async fn run_model(
        &self,
        request: Request<tonic::Streaming<RunModelRequest>>,
    ) -> Result<Response<RunModelReply>, Status> {

        let start_time = Instant::now();

        let mut stream = request.into_inner();

        let mut input: Vec<u8> = Vec::new();
        let mut sign = false;
        let max_input_size = self.max_input_size;
        let mut model_id = "".to_string();

        let mut client_info = None;

        // Get all the data and put it in a Vec
        while let Some(data_stream) = stream.next().await {
            let mut data_proto = data_stream?;

            client_info = data_proto.client_info;

            if data_proto.input.len() * size_of::<u8>() > max_input_size
                || input.len() * size_of::<u8>() > max_input_size
            {
                return Err(Status::invalid_argument("Input too big".to_string()));
            }
            if input.is_empty() {
                sign = data_proto.sign;
                model_id = data_proto.model_id;
            }
            input.append(&mut data_proto.input);
        }

        // Find the model with model_id
        let uuid = Uuid::from_str(&model_id).map_err(|_err| Status::invalid_argument("Model doesn't exist"))?;

        let res = self.model_store.use_model(uuid, |model| {
            (
                model.run_inference(&input),
                model.model_name().map(|e| e.to_string()),
                model.datum_outputs().to_vec(),
            )
        });
        let (results, model_name, datum_outputs) = res.ok_or_else(|| Status::invalid_argument("Model doesn't exist"))?;

        let results = results.map_err(|err| {
            error!("Error while running inference: {}", err);
            return Status::unknown("Unknown error".to_string());
        })?;

        let mut payload = RunModelPayload {
            output: results,
            output_tensors: datum_outputs.into_iter().map(|dt| TensorInfo { datum_type: dt as i32, fact: vec![] }).collect(),
            ..Default::default()
        };
        if sign {
            payload.input_hash = digest::digest(&digest::SHA256, &input).as_ref().to_vec();
            payload.model_id = model_id;
        }

        let payload_with_header = Payload {
            header: Some(PayloadHeader {
                issued_at: Some(SystemTime::now().into()),
            }),
            payload: Some(payload::Payload::RunModelPayload(payload)),
        };

        let mut reply = RunModelReply {
            payload: payload_with_header.encode_to_vec(),
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

        // Log and telemetry
        let elapsed = start_time.elapsed();
        info!(
            "[{} {}] RunModel successful in {}ms (model={}, sign={})",
            client_info
                .as_ref()
                .map(|c| c.user_agent.as_ref())
                .unwrap_or("<unknown>"),
            client_info
                .as_ref()
                .map(|c| c.user_agent_version.as_ref())
                .unwrap_or("<unknown>"),
            elapsed.as_millis(),
            model_name.as_deref().unwrap_or("<unknown>"),
            sign
        );
        telemetry::add_event(
            TelemetryEventProps::RunModel {
                model_name: model_name.map(|e| e.to_string()),
                sign,
                time_taken: elapsed.as_secs_f64(),
            },
            client_info,
        );

        Ok(Response::new(reply))
    }

    async fn delete_model(
        &self,
        request: Request<DeleteModelRequest>,
    ) -> Result<Response<DeleteModelReply>, Status> {
        let request = request.into_inner();
        let model_id = Uuid::from_str(&request.model_id)
            .map_err(|_| Status::invalid_argument("Model doesn't exist"))?;

        // Delete the model
        if self.model_store.delete_model(model_id).is_none() {
            error!("Model doesn't exist");
            return Err(Status::invalid_argument("Model doesn't exist"));
        }

        // Construct the payload
        let reply = DeleteModelReply {};
        Ok(Response::new(reply))
    }
}