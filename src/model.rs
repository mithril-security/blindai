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

use crate::client_communication::{SerializedTensor, TensorInfo};
use anyhow::{anyhow, bail, Context, Result};
use core::hash::Hash;
use num_derive::FromPrimitive;
use ring::digest::Digest;
use serde_derive::{Deserialize, Serialize};
use tract_onnx::prelude::{DatumType, TVec, *};
use uuid::Uuid;

pub type OnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

#[derive(
    Debug, Default, FromPrimitive, PartialEq, Clone, Copy, Eq, Hash, Serialize, Deserialize,
)]
pub enum ModelDatumType {
    #[default]
    F32 = 0,
    F64 = 1,
    I32 = 2,
    I64 = 3,
    U32 = 4,
    U64 = 5,
    U8 = 6,
    U16 = 7,
    I8 = 8,
    I16 = 9,
    Bool = 10,
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
            ModelDatumType::U8 => u8::datum_type(),
            ModelDatumType::U16 => u16::datum_type(),
            ModelDatumType::I8 => i8::datum_type(),
            ModelDatumType::I16 => i16::datum_type(),
            ModelDatumType::Bool => bool::datum_type(),
        }
    }
}

impl TryFrom<DatumType> for ModelDatumType {
    type Error = anyhow::Error;

    fn try_from(value: DatumType) -> Result<Self, Self::Error> {
        Ok(match value {
            DatumType::F32 => ModelDatumType::F32,
            DatumType::F64 => ModelDatumType::F64,
            DatumType::I32 => ModelDatumType::I32,
            DatumType::I64 => ModelDatumType::I64,
            DatumType::U32 => ModelDatumType::U32,
            DatumType::U64 => ModelDatumType::U64,
            DatumType::U8 => ModelDatumType::U8,
            DatumType::U16 => ModelDatumType::U16,
            DatumType::I8 => ModelDatumType::I8,
            DatumType::I16 => ModelDatumType::I16,
            DatumType::Bool => ModelDatumType::Bool,
            _ => bail!("Unsupported datum type: {:?}", value),
        })
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
            DatumType::I8   => $($path)::*::<i8>($($args),*),
            DatumType::I16  => $($path)::*::<i16>($($args),*),
            DatumType::U8   => $($path)::*::<u8>($($args),*),
            DatumType::U16  => $($path)::*::<u16>($($args),*),
            DatumType::Bool => $($path)::*::<bool>($($args),*),
            _ => anyhow::bail!("{:?} is not a number", $dt)
        }
    } }
}

fn create_tensor<A: serde::de::DeserializeOwned + tract_core::prelude::Datum>(
    input: &[u8],
    input_fact: &[usize],
) -> Result<Tensor> {
    let dim = tract_ndarray::IxDynImpl::from(input_fact);
    let vec: Vec<A> = serde_cbor::from_slice(input).unwrap_or_default();
    let tensor = tract_ndarray::ArrayD::from_shape_vec(dim, vec)?.into();
    Ok(tensor)
}

fn convert_tensor<A: serde::ser::Serialize + tract_core::prelude::Datum>(
    input: &tract_core::prelude::Tensor,
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
        let mut model_rec = tract_onnx::onnx()
            .with_ignore_output_shapes(true)
            .model_for_read(&mut model_data)?;
        for (idx, (datum_input, input_fact)) in
            datum_inputs.iter().zip(input_facts.clone()).enumerate()
        {
            model_rec = model_rec.with_input_fact(
                idx,
                InferenceFact::dt_shape(datum_input.get_datum_type(), input_fact),
            )?;
        }

        Ok(InferenceModel {
            onnx: model_rec.clone().into_optimized()?.into_runnable()?.into(),
            datum_inputs,
            input_facts,
            model_name,
            model_id,
            model_hash,
            datum_outputs,
        })
    }

    pub fn run_inference(&self, inputs: &[SerializedTensor]) -> Result<Vec<SerializedTensor>> {
        let mut tensors: Vec<_> = vec![];
        for tensor in inputs {
            let tract_tensor =
                dispatch_numbers!(create_tensor(tensor.info.datum_type.get_datum_type())(
                    &tensor.bytes_data,
                    tensor.info.fact.as_slice()
                ))?;
            if let Some(node_name) = &tensor.info.node_name {
                let node_index = self.onnx.model.node_by_name(node_name)?.id;
                tensors.insert(node_index, tract_tensor);
            } else {
                tensors.push(tract_tensor);
            }
        }
        let mut result = self.onnx.run(TVec::from_vec(tensors.clone()))?;
        result = result
            .into_iter()
            .map(|tensor| {
                if tensor.datum_type() == DatumType::TDim {
                    Ok(tensor.cast_to::<i64>()?.into_owned().into())
                } else {
                    Ok(tensor)
                }
            })
            .collect::<TractResult<_>>()?;
        let mut outputs: Vec<SerializedTensor> = vec![];
        let output_names = self.get_output_names();
        for (i, tensor) in result.iter().enumerate() {
            outputs.push(SerializedTensor {
                info: TensorInfo {
                    datum_type: ModelDatumType::try_from(tensor.datum_type())?,
                    fact: tensor.shape().to_owned(),
                    node_name: Some(output_names[i].clone()),
                },
                bytes_data: dispatch_numbers!(convert_tensor(tensor.datum_type())(&tensor))?,
            });
        }
        Ok(outputs)
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

    pub fn get_output_names(&self) -> Vec<String> {
        self.onnx
            .outputs
            .iter()
            .enumerate()
            .map(|(i, outlet)| {
                self.onnx
                    .model
                    .outlet_label(*outlet)
                    .map(|e| e.to_owned())
                    .unwrap_or_else(|| format!("output_{}", i))
            })
            .collect()
    }
}
