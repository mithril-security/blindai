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
use std::collections::HashMap;
use tonic::Status;
use tract_onnx::prelude::{tract_ndarray::IxDynImpl, TVec, *};

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

fn cbor_from_vec(input: Vec<i32>) -> Vec<u8> {
    serde_cbor::to_vec(&input).unwrap_or_default()
}

fn get_vec_from_cbor(input: &[u8]) -> Vec<i32> {
    serde_cbor::from_slice(input).unwrap_or_default()
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
    pub datum_inputs: HashMap<String, ModelDatumType>,
    input_facts: HashMap<String, Vec<usize>>,
    pub datum_outputs: HashMap<String, ModelDatumType>,
    multiple_inputs: bool,
}

impl InferenceModel {
    pub fn load_model(
        mut model_data: Vec<u8>,
        model_name: Option<String>,
        tensor_inputs: HashMap<String, TensorInfo>,
        tensor_outputs: HashMap<String, TensorInfo>,
    ) -> Result<Self> {
        let convert_type = |t: i32| -> Result<_, Status> {
            num_traits::FromPrimitive::from_i32(t)
                .ok_or_else(|| Status::invalid_argument("Unknown datum type".to_string()))
        };

        // let get_index_from_string = |index: String| -> usize {
        //     return index.split("_").collect::<Vec<&str>>()[1]
        //         .parse::<usize>()
        //         .unwrap();
        // };

        let mut datum_outputs: HashMap<String, ModelDatumType> = HashMap::new();
        let mut datum_inputs: HashMap<String, ModelDatumType> = HashMap::new();
        let mut input_facts: HashMap<String, Vec<usize>> = HashMap::new();

        let mut datum_input: ModelDatumType; // dummy
        let mut datum_output: ModelDatumType; // dummy

        let model_data_copy: &mut [u8] = &mut model_data;
        let mut model_rec = tract_onnx::onnx().model_for_read(&mut &model_data_copy[..])?;
        let multiple_inputs = if tensor_inputs.clone().into_keys().len() > 1 {
            true
        } else {
            false
        };
        for it in tensor_inputs.clone().iter() {
            let (index, tensor_input): (&String, &TensorInfo) = it;
            let mut input_fact: Vec<usize> = vec![];

            for x in &tensor_input.fact {
                input_fact.push(*x as usize);
            }
            datum_input = convert_type(tensor_input.datum_type.clone())?;
            datum_output = convert_type(tensor_outputs.get(index).unwrap().datum_type.clone())?;

            datum_outputs.insert(index.clone(), datum_output.clone());
            datum_inputs.insert(index.clone(), datum_input.clone());
            input_facts.insert(index.clone(), input_fact.clone());
            let _index: usize = index.split("_").collect::<Vec<&str>>()[1]
                .parse::<usize>()
                .unwrap();
            model_rec = model_rec.with_input_fact(
                _index,
                InferenceFact::dt_shape(datum_input.get_datum_type(), &input_fact),
            )?;
        }

        Ok(InferenceModel {
            onnx: model_rec.clone().into_optimized()?.into_runnable()?,
            datum_inputs: datum_inputs.clone(),
            input_facts: input_facts.clone(),
            model_name,
            datum_outputs: datum_outputs.clone(),
            multiple_inputs,
        })
    }

    pub fn run_inference(
        &self,
        input: &mut [u8],
        input_indexes: &Vec<String>,
        output_index: &String,
    ) -> Result<Vec<u8>> {
        /*
            Reconstruct multiple inputs
        */
        let mut __input_facts: Vec<usize> = vec![0];
        let mut __inputs: Vec<Vec<u8>> = Vec::new();
        let mut sum: usize = 0;
        if self.multiple_inputs {
            let input_vec: Vec<i32> = get_vec_from_cbor(input);

            for fact in self
                .input_facts
                .clone()
                .iter()
                .filter(move |(k, _)| input_indexes.contains(k))
                .collect::<HashMap<&String, &Vec<usize>>>()
                .into_values()
            {
                let __value = fact.clone().into_iter().reduce(|a, b| (a * b)).unwrap();
                sum += __value;
                __input_facts.push(sum);
            }

            for window in __input_facts.windows(2) {
                let a = window[0];
                let b = window[1];
                let tmp: Vec<i32> = input_vec
                    .iter()
                    .enumerate()
                    .filter(move |(i, _)| i >= &a && i < &b)
                    .map(move |(_, v)| *v)
                    .collect::<Vec<i32>>();

                __inputs.push(cbor_from_vec(tmp));
            }
        } else {
            __inputs.push(input.to_owned());
        }

        let mut tensors: Vec<_> = vec![];
        println!("{}", input_indexes.len());
        for (i, tensor_index) in input_indexes.clone().iter().enumerate() {
            println!("{}, {}", i, tensor_index);
            let datum_type = self.datum_inputs.get(tensor_index).unwrap();
            let input_fact = self.input_facts.get(tensor_index).unwrap();
            let tensor = dispatch_numbers!(create_tensor(datum_type.get_datum_type())(
                &__inputs[i],
                &input_fact.clone()
            ))?;
            tensors.push(tensor);
        }

        let result = self.onnx.run(TVec::from_vec(tensors.clone()))?;
        let datum_output = self.datum_outputs.get(output_index).unwrap();
        println!("Result Length {}", result[0].len());
        println!("Result {:?}", result[0]);
        let arr = dispatch_numbers!(convert_tensor(&datum_output.get_datum_type())(&result[0]))?;
        Ok(arr)
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }
}
