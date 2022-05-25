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

use anyhow::{anyhow, Result};
use num_derive::FromPrimitive;
use ring::digest::Digest;
use tract_onnx::prelude::{tract_ndarray::IxDynImpl, *};
use uuid::Uuid;

pub type OnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

#[derive(Debug, FromPrimitive, PartialEq, Clone, Copy)]
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

#[derive(Debug)]
pub struct InferenceModel {
    pub onnx: Arc<OnnxModel>,
    datum_type: ModelDatumType,
    input_fact: Vec<usize>,
    #[allow(unused)]
    model_id: Uuid,
    model_name: Option<String>,
    model_hash: Digest,
    datum_output: ModelDatumType,
}

impl InferenceModel {
    pub fn load_model(
        mut model_data: &[u8],
        input_fact: Vec<usize>,
        datum_type: ModelDatumType,
        model_id: Uuid,
        model_name: Option<String>,
        model_hash: Digest,
        datum_output: ModelDatumType,
    ) -> Result<Self> {
        let model_rec = tract_onnx::onnx()
            // load the model
            .model_for_read(&mut model_data)?
            // specify input type and shape
            .with_input_fact(
                0,
                InferenceFact::dt_shape(datum_type.get_datum_type(), &input_fact),
            )?
            // optimize the model
            .into_optimized()?
            // make the model runnable and fix its inputs and outputs
            .into_runnable()?;
        Ok(InferenceModel {
            onnx: model_rec.into(),
            datum_type,
            input_fact,
            model_id,
            model_name,
            model_hash,
            datum_output,
        })
    }

    pub fn from_onnx_loaded(
        onnx: Arc<OnnxModel>,
        input_fact: Vec<usize>,
        datum_type: ModelDatumType,
        model_id: Uuid,
        model_name: Option<String>,
        model_hash: Digest,
        datum_output: ModelDatumType,
    ) -> Self {
        InferenceModel {
            onnx,
            datum_type,
            input_fact,
            model_id,
            model_name,
            model_hash,
            datum_output,
        }
    }

    pub fn run_inference(&self, input: &[u8]) -> Result<Vec<u8>> {
        let tensor = dispatch_numbers!(create_tensor(self.datum_type.get_datum_type())(
            input,
            &self.input_fact
        ))?;
        let result = self.onnx.run(tvec!(tensor))?;
        let arr = dispatch_numbers!(convert_tensor(&self.datum_output.get_datum_type())(
            &result[0]
        ))?;
       Ok(arr)
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }

    pub fn model_hash(&self) -> Digest {
        self.model_hash
    }

    pub fn datum_output(&self) -> ModelDatumType {
        self.datum_output
    }
}
