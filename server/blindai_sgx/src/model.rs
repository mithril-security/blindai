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

use std::vec::Vec;

use crate::client_communication::secured_exchange::TensorInfo;
use anyhow::{anyhow, Result};
use core::hash::Hash;
use num_derive::FromPrimitive;
use tonic::Status;
use tract_onnx::prelude::{tract_ndarray::IxDynImpl, DatumType, TVec, *};

pub type OnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

#[derive(Debug, FromPrimitive, PartialEq, Clone, Copy, Eq, Hash)]
pub enum ModelDatumType {
    F32 = 0,
    F64 = 1,
    I32 = 2,
    I64 = 3,
    U32 = 4,
    U64 = 5,
}

impl ModelDatumType {
    fn get_datum_type(self) -> DatumType {
        match self {
            ModelDatumType::F32 => f32::datum_type(),
            ModelDatumType::F64 => f64::datum_type(),
            ModelDatumType::I32 => i32::datum_type(),
            ModelDatumType::I64 => i64::datum_type(),
            ModelDatumType::U32 => u32::datum_type(),
            ModelDatumType::U64 => u64::datum_type(),
        }
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

fn cbor_from_vec<A: serde::ser::Serialize>(input: Vec<A>) -> Vec<u8> {
    serde_cbor::to_vec(&input).unwrap_or_default()
}

fn get_vec_from_cbor<'a, A: serde::de::DeserializeOwned>(input: &[u8]) -> Vec<Vec<A>> {
    serde_cbor::from_slice::<Vec<Vec<A>>>(input).unwrap_or_default()
}

fn create_tensor<A: serde::de::DeserializeOwned + tract_core::prelude::Datum>(
    input: &[u8],
    input_fact: &[usize],
) -> Result<Tensor> {
    let dim = IxDynImpl::from(input_fact);
    let vec: Vec<A> = serde_cbor::from_slice(input).unwrap_or_default();
    let tensor = tract_ndarray::ArrayD::from_shape_vec(dim, vec)?.into();
    Ok(tensor)
}

fn convert_tensor<A: serde::ser::Serialize + tract_core::prelude::Datum>(
    input: &tract_onnx::prelude::Tensor,
) -> Result<Vec<u8>> {
    let arr = input.to_array_view::<A>()?;
    let slice = arr
        .as_slice()
        .ok_or_else(|| anyhow!("Failed to convert ArrayView to slice"))?;
    Ok(serde_cbor::to_vec(&slice)?)
}

#[derive(Debug)]
pub struct InferenceModel {
    onnx: OnnxModel,
    model_name: Option<String>,
    pub datum_inputs: Vec<ModelDatumType>,
    input_facts: Vec<Vec<usize>>,
    pub datum_outputs: Vec<ModelDatumType>,
}

impl InferenceModel {
    pub fn load_model(
        mut model_data: Vec<u8>,
        model_name: Option<String>,
        tensor_inputs: Vec<TensorInfo>,
        tensor_outputs: Vec<i32>,
    ) -> Result<Self> {
        let convert_type = |t: i32| -> Result<_, Status> {
            num_traits::FromPrimitive::from_i32(t)
                .ok_or_else(|| Status::invalid_argument("Unknown datum type".to_string()))
        };

        let mut datum_outputs: Vec<ModelDatumType> = Vec::new();
        let mut datum_inputs: Vec<ModelDatumType> = Vec::new();
        let mut input_facts: Vec<Vec<usize>> = Vec::new();

        let mut datum_input: ModelDatumType; // dummy

        let model_data_copy: &mut [u8] = &mut model_data;
        let mut model_rec = tract_onnx::onnx().model_for_read(&mut &model_data_copy[..])?;
        for (idx, tensor_input) in tensor_inputs.clone().iter().enumerate() {
            let mut input_fact: Vec<usize> = vec![];

            for x in &tensor_input.fact {
                input_fact.push(*x as usize);
            }
            datum_input = convert_type(tensor_input.datum_type.clone())?;
            datum_outputs = tensor_outputs
                .iter()
                .map(|v| convert_type(*v).unwrap())
                .collect();
            datum_inputs.push(datum_input.clone());
            input_facts.push(input_fact.clone());
            model_rec = model_rec.with_input_fact(
                idx,
                InferenceFact::dt_shape(datum_input.get_datum_type(), &input_fact),
            )?;
        }

        Ok(InferenceModel {
            onnx: model_rec.clone().into_optimized()?.into_runnable()?,
            datum_inputs: datum_inputs.clone(),
            input_facts: input_facts.clone(),
            model_name,
            datum_outputs: datum_outputs.clone(),
        })
    }

    pub fn run_inference(&self, input: &mut [u8]) -> Result<Vec<u8>> {
        let inputs_for_tensor: Vec<Vec<u8>>;
        let input_datum_type = self
            .datum_inputs
            .clone()
            .into_iter()
            .nth(0)
            .unwrap()
            .get_datum_type();

        match input_datum_type {
            DatumType::U32 => {
                let input_vec: Vec<Vec<u32>> = get_vec_from_cbor::<u32>(input);
                inputs_for_tensor = input_vec.into_iter().map(|v| cbor_from_vec(v)).collect();
            }
            DatumType::U64 => {
                let input_vec: Vec<Vec<u64>> = get_vec_from_cbor::<u64>(input);
                inputs_for_tensor = input_vec.into_iter().map(|v| cbor_from_vec(v)).collect();
            }
            DatumType::I32 => {
                let input_vec: Vec<Vec<i32>> = get_vec_from_cbor::<i32>(input);
                inputs_for_tensor = input_vec.into_iter().map(|v| cbor_from_vec(v)).collect();
            }
            DatumType::I64 => {
                let input_vec: Vec<Vec<i64>> = get_vec_from_cbor::<i64>(input);
                inputs_for_tensor = input_vec.into_iter().map(|v| cbor_from_vec(v)).collect();
            }
            DatumType::F32 => {
                let input_vec: Vec<Vec<f32>> = get_vec_from_cbor::<f32>(input);
                inputs_for_tensor = input_vec.into_iter().map(|v| cbor_from_vec(v)).collect();
            }
            DatumType::F64 => {
                let input_vec: Vec<Vec<f64>> = get_vec_from_cbor::<f64>(input);
                inputs_for_tensor = input_vec.into_iter().map(|v| cbor_from_vec(v)).collect();
            }
            _ => anyhow::bail!("{:?} is not a number", input_datum_type),
        }

        let mut tensors: Vec<_> = vec![];
        for (i, (datum_type, input_fact)) in self
            .datum_inputs
            .clone()
            .into_iter()
            .zip(self.input_facts.clone())
            .enumerate()
        {
            let tensor = dispatch_numbers!(create_tensor(datum_type.get_datum_type())(
                &inputs_for_tensor[i],
                &input_fact.clone()
            ))?;
            tensors.push(tensor);
        }

        let result = self.onnx.run(TVec::from_vec(tensors.clone()))?;
        let datum_output = self.datum_outputs[0];
        let arr = dispatch_numbers!(convert_tensor(&datum_output.get_datum_type())(&result[0]))?;
        Ok(arr)
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }
}
