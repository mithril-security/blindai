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

use log::*;
use ring::digest;
#[cfg(not(target_env = "sgx"))]
use std::sync::Mutex;
#[cfg(target_env = "sgx")]
use std::sync::SgxMutex as Mutex;
use std::{
    convert::TryInto,
    mem::size_of,
    sync::Arc,
    time::{Instant, SystemTime},
    vec::Vec,
};

use futures::StreamExt;
use prost::Message;
use ring_compat::signature::Signer;
use secured_exchange::exchange_server::Exchange;
use std::collections::HashMap;
use tonic::{Request, Response, Status};

use crate::{
    client_communication::secured_exchange::TensorInfo,
    identity::MyIdentity,
    model::{InferenceModel, ModelDatumType},
    telemetry::{self, TelemetryEventProps},
};
use secured_exchange::*;

pub mod secured_exchange {
    tonic::include_proto!("securedexchange");
}

pub(crate) struct Exchanger {
    model: std::sync::Arc<Mutex<Option<InferenceModel>>>,
    identity: Arc<MyIdentity>,
    max_model_size: usize,
    max_input_size: usize,
}

impl Exchanger {
    pub fn new(identity: Arc<MyIdentity>, max_model_size: usize, max_input_size: usize) -> Self {
        Self {
            identity,
            model: Arc::new(Mutex::new(None)),
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

        let mut stream = request.into_inner();
        let mut tensor_inputs: HashMap<String, TensorInfo> = HashMap::new();
        let mut tensor_outputs: HashMap<String, TensorInfo> = HashMap::new();
        let input_facts: Vec<Vec<usize>> = Vec::new();
        let mut model_bytes: Vec<u8> = Vec::new();
        let max_model_size = self.max_model_size;
        let mut model_size = 0usize;
        let mut sign = false;

        let mut model_name = None;
        let mut client_info = None;

        // get all model chunks from the client into a big Vec
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

                for pair in &model_proto.tensor_inputs {
                    let key = &pair.index;
                    let value = pair.info.as_ref().unwrap();
                    tensor_inputs.insert(key.to_string(), value.clone());
                }

                for pair in &model_proto.tensor_outputs {
                    let key = &pair.index;
                    let value = pair.info.as_ref().unwrap();
                    tensor_outputs.insert(key.to_string(), value.clone());
                }

                sign = model_proto.sign;
            }
            if model_size > max_model_size || model_bytes.len() > max_model_size {
                return Err(Status::invalid_argument("Model too big".to_string()));
            }
            model_bytes.append(&mut model_proto.data)
        }

        if model_size == 0 {
            return Err(Status::invalid_argument("Received no data".to_string()));
        }

        let model = InferenceModel::load_model(
            model_bytes.clone(),
            model_name.clone(),
            tensor_inputs.clone(),
            tensor_outputs.clone(),
        )
        .map_err(|err| {
            error!("Unknown error creating model: {}", err);
            Status::unknown("Unknown error".to_string())
        })?;

        *self.model.lock().unwrap() = Some(model);

        let mut payload = SendModelPayload::default();
        // payload.model_id = "default".into();
        if sign {
            payload.model_hash = digest::digest(&digest::SHA256, &model_bytes)
                .as_ref()
                .to_vec();
            payload.input_fact = input_facts
                .into_iter()
                .flatten()
                .map(|i| i as i32)
                .collect();
        }

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

        let mut client_info = None;
        let mut tensor_index = String::default();

        while let Some(data_stream) = stream.next().await {
            let mut data_proto = data_stream?;

            client_info = data_proto.client_info;
            tensor_index = data_proto.tensor_index;

            if data_proto.input.len() * size_of::<u8>() > max_input_size
                || input.len() * size_of::<u8>() > max_input_size
            {
                return Err(Status::invalid_argument("Input too big".to_string()));
            }
            if input.is_empty() {
                sign = data_proto.sign;
            }
            input.append(&mut data_proto.input);
        }
        let model_guard = self.model.lock().unwrap();
        let model = if let Some(model) = &*model_guard {
            model
        } else {
            return Err(Status::invalid_argument(
                "Cannot find the model".to_string(),
            ));
        };
        let result = model
            .run_inference(&input.clone(), &tensor_index)
            .map_err(|err| {
                error!("Unknown error running inference: {}", err);
                Status::unknown("Unknown error".to_string())
            })?;

        let datum_out: ModelDatumType = *model.datum_outputs.get(&tensor_index).unwrap();
        let mut payload = RunModelPayload {
            output: result,
            datum_output: datum_out as i32,
            ..Default::default()
        };
        // payload.model_id = "default".into();
        if sign {
            payload.input_hash = digest::digest(&digest::SHA256, &input).as_ref().to_vec();
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
            model.model_name().unwrap_or("<unknown>"),
            sign
        );
        telemetry::add_event(
            TelemetryEventProps::RunModel {
                model_name: model.model_name().map(|e| e.to_string()),
                sign,
                time_taken: elapsed.as_secs_f64(),
            },
            client_info,
        );

        Ok(Response::new(reply))
    }
}
