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
use blindai_common::BlindAIConfig;
use futures::StreamExt;
use log::*;
use num_traits::FromPrimitive;
use prost::Message;
use ring::digest;
use ring_compat::signature::Signer;
use std::{
    convert::TryInto,
    sync::Arc,
    time::{Instant, SystemTime},
    vec::Vec,
};
use tract_core::prelude::Tensor;
use tract_onnx::prelude::DatumType;

use tonic::{Request, Response, Status};

use crate::{
    auth::AuthExtension,
    identity::MyIdentity,
    model::{
        deserialize_tensor_bytes, serialize_tensor_bytes, ModelDatumType, ModelLoadContext,
        TensorFacts,
    },
    model_store::ModelStore,
    telemetry::{self, TelemetryEventProps},
};

use secured_exchange::{exchange_server::Exchange, *};

pub mod secured_exchange {
    tonic::include_proto!("securedexchange");
}

pub(crate) struct Exchanger {
    model_store: Arc<ModelStore>,
    identity: Arc<MyIdentity>,
    max_model_size: usize,
    max_input_size: usize,
    config: Arc<BlindAIConfig>,
}

impl Exchanger {
    pub fn new(
        model_store: Arc<ModelStore>,
        identity: Arc<MyIdentity>,
        max_model_size: usize,
        max_input_size: usize,
        config: Arc<BlindAIConfig>,
    ) -> Self {
        Self {
            identity,
            model_store,
            max_model_size,
            max_input_size,
            config,
        }
    }
}

#[tonic::async_trait]
impl Exchange for Exchanger {
    async fn send_model(
        &self,
        request: Request<tonic::Streaming<SendModelRequest>>,
    ) -> Result<Response<SendModelReply>, Status> {
        if !self.config.allow_sendmodel {
            return Err(Status::permission_denied("SendModel is disabled"));
        }

        let auth_ext = request.extensions().get::<AuthExtension>().cloned();

        if self.config.send_model_requires_auth
            && (auth_ext.is_none() || !auth_ext.as_ref().unwrap().is_logged())
        {
            return Err(Status::permission_denied("You must be logged"));
        }

        let start_time = Instant::now();

        let mut stream = request.into_inner();

        let mut model_bytes: Vec<u8> = Vec::new();
        let mut model_size = 0usize;

        let mut sign = false;
        let mut model_id = None;
        let mut model_name = None;
        let mut client_info = None;
        let mut save_model = false;

        let mut input_info_req: Vec<TensorInfo> = Vec::new();
        let mut output_info_req: Vec<TensorInfo> = Vec::new();

        // Get all model chunks from the client into a big Vec
        while let Some(model_stream) = stream.next().await {
            let mut model_proto = model_stream?;
            if model_size == 0 {
                model_size = model_proto.length.try_into().unwrap();
                model_bytes.reserve_exact(model_size);

                model_id = if !model_proto.model_id.is_empty() {
                    Some(model_proto.model_id)
                } else {
                    None
                };
                model_name = if !model_proto.model_name.is_empty() {
                    Some(model_proto.model_name)
                } else {
                    None
                };
                client_info = model_proto.client_info;
                input_info_req = model_proto.tensor_inputs;
                output_info_req = model_proto.tensor_outputs;
                sign = model_proto.sign;
                save_model = model_proto.save_model;
            }
            if model_size > self.max_model_size || model_bytes.len() > self.max_model_size {
                return Err(Status::invalid_argument("Model is too big"));
            }
            model_bytes.append(&mut model_proto.data)
        }

        if model_size == 0 {
            return Err(Status::invalid_argument("Received no data"));
        }

        let map_fn = |info: TensorInfo| {
            Ok(TensorFacts {
                datum_type: if info.datum_type >= 0 {
                    Some(
                        ModelDatumType::from_i32(info.datum_type)
                            .ok_or_else(|| anyhow!("invalid datum type: {}", info.datum_type))?,
                    )
                } else {
                    None
                },
                dims: if !info.dims.is_empty() {
                    Some(info.dims.into_iter().map(|el| el as usize).collect())
                } else {
                    None
                },
                index: if info.index >= 0 {
                    Some(info.index as usize)
                } else {
                    None
                },
                index_name: if !info.index_name.is_empty() {
                    Some(info.index_name)
                } else {
                    None
                },
            })
        };
        let input_info = input_info_req
            .iter()
            .cloned()
            .map(map_fn)
            .collect::<Result<Vec<_>>>()
            .map_err(|err| {
                error!("Error while getting input info: {:?}", err);
                Status::invalid_argument(format!("Error while getting input info: {:?}", err))
            })?;
        let output_info = output_info_req
            .iter()
            .cloned()
            .map(map_fn)
            .collect::<Result<Vec<_>>>()
            .map_err(|err| {
                error!("Error while getting output info: {:?}", err);
                Status::invalid_argument(format!("Error while getting input info: {:?}", err))
            })?;
        let (model_id, model_hash) = self
            .model_store
            .add_model(
                &model_bytes,
                model_name.clone(),
                model_id.clone(),
                &input_info,
                &output_info,
                save_model,
                true, // todo: make optim configurable
                ModelLoadContext::FromSendModel,
                auth_ext.as_ref().and_then(|ext| ext.userid()),
                auth_ext.as_ref().and_then(|ext| ext.username())
            )
            .map_err(|err| {
                error!("Error while creating model: {:?}", err);
                Status::unknown(format!("Error while creating model: {:?}", err))
            })?;

        // Construct the return payload
        let mut payload = SendModelPayload::default();
        if sign {
            payload.model_hash = model_hash.as_ref().to_vec();
            payload.tensor_inputs = input_info_req;
            payload.tensor_outputs = output_info_req;
        }
        payload.model_id = model_id;

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
        let userid = auth_ext
            .as_ref()
            .and_then(|e| e.userid())
            .map(|id| format!("{}", id));
        info!(
            "[{} {}] SendModel successful in {}ms (model={}, size={}, sign={}, userid={})",
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
            sign,
            userid.as_deref().unwrap_or("<none>"),
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
        let auth_ext = request.extensions().get::<AuthExtension>().cloned();

        let start_time = Instant::now();

        let mut input_tensors: Vec<TensorData> = Vec::new();

        let mut client_info = None;
        let mut sign = false;
        let mut model_id = "".to_string();

        // Get all the data (in chunks)

        let mut stream = request.into_inner();
        while let Some(data_stream) = stream.next().await {
            let data_proto = data_stream?;

            client_info = data_proto.client_info;
            sign = data_proto.sign;
            model_id = data_proto.model_id;

            let mut message_size = 0;
            for inp_tensor in data_proto.input_tensors {
                message_size += inp_tensor.bytes_data.len();

                if inp_tensor.info.is_none() {
                    return Err(Status::invalid_argument("No tensor info"));
                }
                if message_size > self.max_input_size {
                    return Err(Status::invalid_argument("Input too big"));
                }

                if let Some(tensor) = input_tensors.iter_mut().find(|el| {
                    let info = el.info.as_ref().unwrap();
                    let inp_tensor_info = inp_tensor.info.as_ref().unwrap();
                    // index (numeric) is same
                    info.index == inp_tensor_info.index
                    // input name is present & same
                        || (!inp_tensor_info.index_name.is_empty() && info.index_name == inp_tensor_info.index_name)
                }) {
                    tensor.bytes_data.extend(inp_tensor.bytes_data);
                } else {
                    input_tensors.push(inp_tensor);
                }
            }
        }

        let mut input_hash = vec![];
        if sign {
            let mut hash = digest::Context::new(&digest::SHA256);
            for tens in &input_tensors {
                hash.update(&tens.bytes_data)
            }
            input_hash = hash.finish().as_ref().to_vec()
        }

        let input_tensors = input_tensors
            .into_iter()
            .map(|ten| {
                let info = ten.info.ok_or_else(|| anyhow!("no tensor info provided"))?;
                deserialize_tensor_bytes(
                    FromPrimitive::from_i32(info.datum_type)
                        .ok_or_else(|| anyhow!("invalid datum type: {}", info.datum_type))?,
                    &info
                        .dims
                        .into_iter()
                        .map(|el| el as usize)
                        .collect::<Vec<_>>(),
                    &ten.bytes_data,
                )
            })
            .collect::<Result<Vec<Tensor>>>()
            .map_err(|err| {
                error!("Error while deserializing input: {:?}", err);
                Status::invalid_argument("Tensor serialization error")
            })?;

            
        let model_id_path:Vec<&str> = model_id.split('/').collect();
        if model_id.is_empty() || model_id_path.len() > 2 {
            return Err(Status::invalid_argument("Model doesn't exist"));
        }

        //if the model isn't on the server we try to load it from the disk
        if !self.model_store.in_the_server(model_id.clone()) {
            let _saved = self.model_store.unseal(model_id.clone());
        }

        let res = self.model_store.use_model(&model_id, |model| {
            (
                model.run_inference(input_tensors.into()),
                model.model_name().map(|e| e.to_string()),
                model.get_output_names(),
            )
        });

        let (results, model_name, output_names) =
            res.ok_or_else(|| Status::invalid_argument("Model doesn't exist"))?;

        let results = results.map_err(|err| {
            error!("Error while running inference: {:?}", err);
            Status::unknown(format!("{}", err))
        })?;

        let output_tensors = results
            .into_iter()
            .zip(output_names)
            .enumerate()
            .map(|(index, (mut ten, output_name))| {
                if ten.datum_type() == DatumType::TDim {
                    ten = ten.cast_to::<i64>()?.into_owned().into();
                }
                Ok(TensorData {
                    info: Some(TensorInfo {
                        dims: ten.shape().into_iter().map(|el| *el as i32).collect(),
                        datum_type: ModelDatumType::try_from(ten.datum_type())? as i32,
                        index: index as i32,
                        index_name: output_name,
                    }),
                    bytes_data: serialize_tensor_bytes(&ten)?.into(),
                })
            })
            .collect::<Result<Vec<TensorData>>>()
            .map_err(|err| {
                error!("Error while seriliazing output: {:?}", err);
                Status::unknown(format!("Error while serializing output: {:?}", err))
            })?;

        let elapsed = start_time.elapsed();

        let mut payload = RunModelPayload {
            output_tensors,
            ..Default::default()
        };
        if sign {
            payload.input_hash = input_hash;
            payload.model_id = model_id;
        }

        if self.config.send_inference_time {
            payload.inference_time = elapsed.as_millis() as u64;
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
        let userid = auth_ext
            .and_then(|e| e.userid())
            .map(|id| format!("{}", id));
        info!(
            "[{} {}] RunModel successful in {}ms (model={}, sign={}, userid={})",
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
            sign,
            userid.as_deref().unwrap_or("<none>"),
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
        let auth_ext = request.extensions().get::<AuthExtension>().cloned();
        let request = request.into_inner();
        let model_id = request.model_id;

        if model_id.is_empty() {
            return Err(Status::invalid_argument("Model doesn't exist"));
        }

        let user_id = if let Some(auth_ext) = auth_ext {
            if let Some(userid) = auth_ext.userid() {
                Some(userid)
            } else {
                return Err(Status::unauthenticated("You must provide an api key"));
            }
        } else {
            None
        };

        // Delete the model
        if self.model_store.delete_model(&model_id, user_id).is_none() {
            error!("Model doesn't exist");
            return Err(Status::invalid_argument("Model doesn't exist"));
        }
        // Construct the payload
        let reply = DeleteModelReply {};
        Ok(Response::new(reply))
    }
}
