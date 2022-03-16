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
use std::{convert::TryInto, mem::size_of, sync::Arc, time::SystemTime, vec::Vec};

use futures::StreamExt;
use num_traits::FromPrimitive;
use prost::Message;
use ring_compat::signature::Signer;
use secured_exchange::exchange_server::Exchange;
use tonic::{Request, Response, Status};

use crate::{
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
        let mut stream = request.into_inner();
        let mut datum = ModelDatumType::I64; // dummy (changed for test)

        let mut input_fact: Vec<usize> = Vec::new();
        let mut model_bytes: Vec<u8> = Vec::new();
        let max_model_size = self.max_model_size;
        let mut model_size = 0usize;
        let mut sign = false;

        // get all model chunks from the client into a big Vec
        while let Some(model_stream) = stream.next().await {
            let mut model_proto = model_stream?;
            if model_size == 0 {
                model_size = model_proto.length.try_into().unwrap();
                model_bytes.reserve_exact(model_size);

                for x in &model_proto.input_fact {
                    input_fact.push(*x as usize);
                }

                datum = FromPrimitive::from_i32(model_proto.datum)
                    .ok_or_else(|| Status::invalid_argument(format!("Unknown datum type")))?;
                sign = model_proto.sign;
            }
            if model_size > max_model_size || model_bytes.len() > max_model_size {
                Err(Status::invalid_argument(format!("Model too big")))?;
            }
            model_bytes.append(&mut model_proto.data)
        }

        if model_size == 0 {
            Err(Status::invalid_argument(format!("Received no data")))?;
        }

        let model = InferenceModel::load_model(&model_bytes, input_fact, datum)
            .map_err(|err| {
                error!("Unknown error creating model: {}", err);
                Status::unknown(format!("Unknown error"))
            })?;

        *self.model.lock().unwrap() = Some(model);

        telemetry::add_event(TelemetryEventProps::SendModel { model_size });
        info!("Model loaded successfully");

        let mut payload = SendModelPayload::default();
        payload.model_id = "default".into();

        let payload_with_header = Payload {
            header: Some(PayloadHeader {
                issued_at: Some(SystemTime::now().into()),
            }),
            payload: Some(payload::Payload::SendModelPayload(payload)),
        };

        let mut reply = SendModelReply::default();
        reply.payload = payload_with_header.encode_to_vec();
        if sign {
            reply.signature = Some(
                self.identity
                    .signing_key
                    .sign(&reply.payload)
                    .to_bytes()
                    .to_vec(),
            );
        }

        Ok(Response::new(reply))
    }

    async fn run_model(
        &self,
        request: Request<tonic::Streaming<RunModelRequest>>,
    ) -> Result<Response<RunModelReply>, Status> {
        let mut stream = request.into_inner();

        let mut input: Vec<u8> = Vec::new();
        let mut sign = false;
        let max_input_size = self.max_input_size;

        while let Some(data_stream) = stream.next().await {
            let mut data_proto = data_stream?;
            if data_proto.input.len() * size_of::<u8>() > max_input_size.try_into().unwrap()
                || input.len() * size_of::<u8>() > max_input_size
            {
                Err(Status::invalid_argument(format!("Input too big")))?;
            }
            if input.len() == 0 {
                sign = data_proto.sign;
            }
            input.append(&mut data_proto.input);
        }

        let model_guard = self.model.lock().unwrap();

        let model = if let Some(model) = &*model_guard {
            model
        } else {
            Err(Status::invalid_argument(format!("Cannot find the model")))?
        };

        let result = model.run_inference(&input).map_err(|err| {
            error!("Unknown error running inference: {}", err);
            Status::unknown(format!("Unknown error"))
        })?;

        info!("Inference was a success");
        telemetry::add_event(TelemetryEventProps::RunModel {});

        let mut payload = RunModelPayload::default();
        payload.model_id = "default".into();
        payload.output = result;
        if sign {
            payload.input_hash = Some(digest::digest(&digest::SHA256, &input).as_ref().to_vec());
        }

        let mut reply = RunModelReply::default();
        reply.payload = payload.encode_to_vec();
        if sign {
            reply.signature = Some(
                self.identity
                    .signing_key
                    .sign(&reply.payload)
                    .to_bytes()
                    .to_vec(),
            );
        }

        Ok(Response::new(reply))
    }
}
