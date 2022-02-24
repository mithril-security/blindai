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

extern crate env_logger;
extern crate num_derive;
extern crate num_traits;
extern crate sgx_libc;
extern crate sgx_tseal;
extern crate sgx_types;
extern crate tract_core;
extern crate tract_onnx;

use log::*;
use std::any::Any;
use std::convert::TryInto;
use std::mem::size_of;
use std::ops::DerefMut;
use std::path::Path;
use std::ptr;
use std::slice;
use std::sync::{Arc, SgxMutex as Mutex};
use std::untrusted::fs::read;
use std::untrusted::fs::{metadata, read_dir, File};
use std::vec::Vec;

use futures::{Stream, StreamExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rpc::untrusted_local_app_client::*;
use secured_exchange::exchange_server::{Exchange, ExchangeServer};
use secured_exchange::{Data, Model, ModelResult, SimpleReply};
use tokio::runtime;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::http::request;
use tonic::{
    transport::{Identity, Server},
    Request, Response, Status,
};
use tract_core::internal::*;
use tract_core::ops::matmul::lir_unary::*;
use tract_core::ops::{cnn, nn};
use tract_onnx::prelude::tract_ndarray::IxDynImpl;
use tract_onnx::prelude::*;

use crate::telemetry::{self, TelemetryEventProps};

pub mod secured_exchange {
    tonic::include_proto!("securedexchange");
}

pub type OnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

#[derive(Debug, FromPrimitive, PartialEq)]
pub enum ModelDatumType {
    F32 = 0,
    F64 = 1,
    I32 = 2,
    I64 = 3,
    U32 = 4,
    U64 = 5,
}

fn get_datum_type(datum: &Option<ModelDatumType>) -> TractResult<DatumType> {
    match *datum {
        Some(ModelDatumType::F32) => return Ok(f32::datum_type()),
        Some(ModelDatumType::F64) => return Ok(f64::datum_type()),
        Some(ModelDatumType::I32) => return Ok(i32::datum_type()),
        Some(ModelDatumType::I64) => return Ok(i64::datum_type()),
        Some(ModelDatumType::U32) => return Ok(u32::datum_type()),
        Some(ModelDatumType::U64) => return Ok(u64::datum_type()),
        None => return Err(anyhow!("Unknown type")),
    }
}

macro_rules! dispatch_numbers {
    ($($path:ident)::* ($dt:expr) ($($args:expr),*)) => { {
        use tract_onnx::prelude::DatumType;
        match $dt {
            DatumType::U32  => $($path)::*::<u32>($($args),*),
            DatumType::U64  => $($path)::*::<u64>($($args),*),
            DatumType::I32  => $($path)::*::<i32>($($args),*),
            DatumType::I64  => $($path)::*::<i64>($($args),*),
            DatumType::F32  => $($path)::*::<f32>($($args),*),
            DatumType::F64  => $($path)::*::<f64>($($args),*),
            _ => anyhow::bail!("{:?} is not a number", $dt)
        }
    } }
}

fn load_model(
    model: Vec<u8>,
    input_fact: Vec<i32>,
    datum: &Option<ModelDatumType>,
) -> TractResult<OnnxModel> {
    let mut model_slice = &model[..];
    let datum_type = get_datum_type(datum)?;
    let model_rec = tract_onnx::onnx()
        // load the model
        .model_for_read(&mut model_slice)?
        // specify input type and shape
        .with_input_fact(0, InferenceFact::dt_shape(datum_type, input_fact))?
        // optimize the model
        .into_optimized()?
        // make the model runnable and fix its inputs and outputs
        .into_runnable()?;
    Ok(model_rec)
}

fn create_tensor<'a, A: serde::de::DeserializeOwned + tract_core::prelude::Datum>(
    input: Vec<u8>,
    input_fact: &Vec<usize>,
) -> TractResult<Tensor> {
    let dim = IxDynImpl::from(input_fact.as_slice());
    let vec: Vec<A> = serde_cbor::from_slice(&input).unwrap_or(Vec::new());
    let tensor = tract_ndarray::ArrayD::from_shape_vec(dim, vec)?.into();
    Ok(tensor)
}

fn run_inference(
    model: &OnnxModel,
    input: Vec<u8>,
    input_fact: &Vec<usize>,
    datum: &Option<ModelDatumType>,
) -> TractResult<Vec<f32>> {
    let tensor = dispatch_numbers!(create_tensor(get_datum_type(datum)?)(input, &input_fact))?;
    let result = model.run(tvec!(tensor))?;
    let arr = result[0].to_array_view::<f32>()?;
    Ok(arr
        .as_slice()
        .ok_or(anyhow!("Failed to convert ArrayView to slice"))?
        .to_vec())
}

#[derive(Debug, Default)]
pub(crate) struct Exchanger {
    pub model: std::sync::Arc<Mutex<Option<OnnxModel>>>,
    pub input_fact: std::sync::Arc<Mutex<Vec<usize>>>,
    pub max_model_size: usize,
    pub max_input_size: usize,
    pub datum_type: std::sync::Arc<Mutex<Option<ModelDatumType>>>,
}

impl Exchanger {
    pub fn new(max_model_size: usize, max_input_size: usize) -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            input_fact: Arc::new(Mutex::new(Vec::new())),
            max_model_size: max_model_size,
            max_input_size: max_input_size,
            datum_type: Arc::new(Mutex::new(None)),
        }
    }
}

#[tonic::async_trait]
impl Exchange for Exchanger {
    async fn send_model(
        &self,
        request: Request<tonic::Streaming<Model>>,
    ) -> Result<Response<SimpleReply>, Status> {
        let mut reply = SimpleReply::default();
        let mut stream = request.into_inner();
        let mut model_proto = Model::default();

        let mut input_fact: Vec<usize> = Vec::new();
        let mut model_bytes: Vec<u8> = Vec::new();
        let max_model_size = self.max_model_size;
        let mut model_size: usize = 0;

        while let Some(model_stream) = stream.next().await {
            model_proto = model_stream?;
            if model_size == 0 {
                model_size = model_proto.length.try_into().unwrap();
            }
            if input_fact.len() == 0 {
                for x in &model_proto.input_fact {
                    input_fact.push(*x as usize);
                }
            }
            if model_size > max_model_size || model_bytes.len() > max_model_size {
                error!("Incoming model is too big");
                return Err(Status::invalid_argument(format!("Model too big")));
            }
            model_bytes.append(&mut model_proto.data);
        }

        telemetry::add_event(TelemetryEventProps::SendModel {
            model_size: model_bytes.len(),
        });

        let datum = FromPrimitive::from_i32(model_proto.datum);
        match load_model(model_bytes, model_proto.input_fact.clone(), &datum) {
            Ok(model_rec) => {
                *self.model.lock().unwrap() = Some(model_rec);
                let input = &mut *self.input_fact.lock().unwrap();
                input.clear();
                input.append(&mut input_fact);
                *self.datum_type.lock().unwrap() = datum;
                reply.ok = true;
                reply.msg = format!("OK");
                info!("Model loaded successfully");
            }
            Err(_x) => {
                reply.ok = false;
                reply.msg = format!(
                    "Failed to load model, the model or the input format are perhaps invalid"
                );
                error!("Failed to load model, the model or the input format are perhaps invalid");
            }
        }
        Ok(Response::new(reply))
    }

    async fn run_model(
        &self,
        request: Request<tonic::Streaming<Data>>,
    ) -> Result<Response<ModelResult>, Status> {
        let mut reply = ModelResult::default();
        let mut stream = request.into_inner();
        let mut data_proto = Data::default();

        let mut input: Vec<u8> = Vec::new();
        let max_input_size = self.max_input_size;

        telemetry::add_event(TelemetryEventProps::RunModel {});

        while let Some(data_stream) = stream.next().await {
            data_proto = data_stream?;
            if data_proto.input.len() * size_of::<u8>() > max_input_size.try_into().unwrap()
                || input.len() * size_of::<u8>() > max_input_size
            {
                error!("Incoming input is too big");
                return Err(Status::invalid_argument(format!("Input too big")));
            }
            input.append(&mut data_proto.input);
        }

        let input_fact = &*self.input_fact.lock().unwrap();
        let datum = &*self.datum_type.lock().unwrap();
        if let Some(model) = &*self.model.lock().unwrap() {
            match run_inference(&model, input, &input_fact, &datum) {
                Ok(output) => {
                    reply.output = output;
                    reply.ok = true;
                    reply.msg = String::from("OK");
                    info!("Inference done successfully, sending encrypted result to the client");
                }
                Err(_) => {
                    reply.ok = false;
                    reply.msg = String::from("Error while running the model");
                    error!("Error while running the inference");
                }
            }
        } else {
            reply.ok = false;
            reply.msg = String::from("Model not loaded, cannot continue");
            error!("Model not loaded, cannot run inference");
            return Ok(Response::new(reply));
        }

        Ok(Response::new(reply))
    }
}
