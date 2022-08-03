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

use anyhow::{anyhow, Context, Result};
use core::hash::Hash;
use num_derive::FromPrimitive;
use ring::digest::Digest;
use tract_onnx::prelude::{tract_ndarray::IxDynImpl, DatumType, TVec, *};
use uuid::Uuid;

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

fn cbor_get_vec<A: serde::ser::Serialize>(v: Vec<A>) -> anyhow::Result<Vec<u8>> {
    serde_cbor::to_vec(&v).context("Failed to serialize inference data")
}
fn create_inputs_for_tensor<
    A: serde::ser::Serialize + tract_core::prelude::Datum + serde::de::DeserializeOwned,
>(
    input: &mut [u8],
) -> anyhow::Result<Vec<Vec<u8>>> {
    let inputs_for_tensor: Result<Vec<Vec<u8>>>;
    let input_vec: Vec<Vec<A>> = serde_cbor::from_slice::<Vec<Vec<A>>>(input)?;
    inputs_for_tensor = input_vec.into_iter().map(cbor_get_vec).collect();
    inputs_for_tensor
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
    pub datum_inputs: Vec<ModelDatumType>,
    input_facts: Vec<Vec<usize>>,
    pub datum_outputs: Vec<ModelDatumType>,
    pub onnx: Arc<OnnxModel>,
    #[allow(unused)]
    model_id: Uuid,
    model_name: Option<String>,
    model_hash: Digest,
}

impl InferenceModel {
    pub fn load_model(
        mut model_data: &[u8],
        input_facts: Vec<Vec<usize>>,
        model_id: Uuid,
        model_name: Option<String>,
        model_hash: Digest,
        datum_inputs: Vec<ModelDatumType>,
        datum_outputs: Vec<ModelDatumType>,
    ) -> Result<Self> {
        let mut model_rec = tract_onnx::onnx().with_ignore_output_shapes(true).model_for_read(&mut model_data)?;
        for (idx, (datum_input, input_fact)) in datum_inputs
            .clone()
            .iter()
            .zip(input_facts.clone())
            .enumerate()
        {
            model_rec = model_rec.with_input_fact(
                idx,
                InferenceFact::dt_shape(datum_input.get_datum_type(), &input_fact),
            )?;
        }

        Ok(InferenceModel {
            onnx: model_rec.clone().into_optimized()?.into_runnable()?.into(),
            datum_inputs: datum_inputs.clone(),
            input_facts: input_facts.clone(),
            model_name,
            model_id,
            model_hash,
            datum_outputs: datum_outputs.clone(),
        })
    }

    pub fn run_inference(&self, input: &mut [u8]) -> Result<Vec<u8>> {
        let input_datum_type = self.datum_inputs[0].get_datum_type();
        let inputs_for_tensor: Vec<Vec<u8>> =
            dispatch_numbers!(create_inputs_for_tensor(input_datum_type)(input))?;

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
        let mut result = self.onnx.run(TVec::from_vec(tensors.clone()))?;
        result = result.into_iter().map(|tensor| {
            if tensor.datum_type() == DatumType::TDim {
              Ok(tensor.cast_to::<i64>()?.into_owned().into())
            } else {
              Ok(tensor)
            }
        }).collect::<TractResult<_>>()?; 
        let datum_output = self.datum_outputs[0];
        let arr = dispatch_numbers!(convert_tensor(&datum_output.get_datum_type())(&result[0]))?;
        Ok(arr)
    }

    pub fn from_onnx_loaded(
        onnx: Arc<OnnxModel>,
        input_facts: Vec<Vec<usize>>,
        model_id: Uuid,
        model_name: Option<String>,
        model_hash: Digest,
        datum_inputs: Vec<ModelDatumType>,
        datum_outputs: Vec<ModelDatumType>,
    ) -> Self {
        InferenceModel {
            onnx,
            input_facts,
            model_id,
            model_name,
            model_hash,
            datum_inputs,
            datum_outputs,
        }
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }

    pub fn model_hash(&self) -> Digest {
        self.model_hash
    }

    pub fn datum_output(&self) -> ModelDatumType {
        self.datum_outputs[0]
    }
}
