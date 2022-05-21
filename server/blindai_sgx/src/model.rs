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

use crate::client_communication::secured_exchange::TensorInput;
use anyhow::{anyhow, Result};
use num_derive::FromPrimitive;
use std::collections::HashMap;
use tonic::Status;
use tract_onnx::prelude::{tract_ndarray::IxDynImpl, *};

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

pub type MultipleOnnxModels = HashMap<(Vec<usize>, ModelDatumType), OnnxModel>;

#[derive(Debug)]
pub struct InferenceModel {
    onnx_models: MultipleOnnxModels,
    model_name: Option<String>,
    pub tensor_inputs: Vec<TensorInput>,
    pub datum_input_output: HashMap<(Vec<usize>, ModelDatumType), ModelDatumType>,
}

impl InferenceModel {
    pub fn load_model(
        mut model_data: &[u8],
        model_name: Option<String>,
        tensor_inputs: Vec<TensorInput>,
    ) -> Result<Self> {
        let mut model_recs: MultipleOnnxModels = MultipleOnnxModels::new();
        let mut datum_input_output: HashMap<(Vec<usize>, ModelDatumType), ModelDatumType> =
            HashMap::new();

        let mut datum_input: ModelDatumType = ModelDatumType::I64; // dummy
        let mut datum_output: ModelDatumType = ModelDatumType::I64; // dummy
        for tensor_input in tensor_inputs.clone() {
            let mut input_fact: Vec<usize> = vec![];
            for x in &tensor_input.input_fact {
                input_fact.push(*x as usize);
            }
            datum_input = num_traits::FromPrimitive::from_i32(tensor_input.datum_input.clone())
                .ok_or_else(|| Status::invalid_argument("Unknown datum type".to_string()))?;

            datum_output = num_traits::FromPrimitive::from_i32(tensor_input.datum_input.clone())
                .ok_or_else(|| Status::invalid_argument("Unknown datum type".to_string()))?;
            let model_rec = tract_onnx::onnx()
                // load the model
                .model_for_read(&mut model_data)?
                // specify input type and shape
                .with_input_fact(
                    0,
                    InferenceFact::dt_shape(datum_input.get_datum_type(), tensor_input.input_fact),
                )?
                // optimize the model
                .into_optimized()?
                // make the model runnable and fix its inputs and outputs
                .into_runnable()?;

            model_recs.insert((input_fact.clone(), datum_input), model_rec);
            datum_input_output.insert(
                (input_fact.clone(), datum_input.clone()),
                datum_output.clone(),
            );
        }
        Ok(InferenceModel {
            onnx_models: model_recs.clone(),
            tensor_inputs: tensor_inputs.clone(),
            model_name,
            datum_input_output: datum_input_output.clone(),
        })
    }

    pub fn run_inference(
        &self,
        input: &[u8],
        input_fact: Vec<usize>,
        datum_type: ModelDatumType,
    ) -> Result<Vec<u8>> {
        let tensor = dispatch_numbers!(create_tensor(datum_type.get_datum_type())(
            input,
            &input_fact.clone()
        ))?;
        println!("Running inference");
        let onnx = self
            .onnx_models
            .get(&(input_fact.clone(), datum_type))
            .unwrap();
        let result = onnx.run(tvec!(tensor))?;

        let datum_output = self
            .datum_input_output
            .get(&(input_fact.clone(), datum_type))
            .unwrap();
        let arr = dispatch_numbers!(convert_tensor(&datum_output.get_datum_type())(&result[0]))?;
        Ok(arr)
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }
}
