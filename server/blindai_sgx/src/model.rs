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

use std::ops::RangeBounds;
use std::path::Path;
use std::str::FromStr;
use std::{borrow::Cow, convert::TryFrom, vec::Vec};

use anyhow::{anyhow, bail, Error, Result};
use core::hash::Hash;
use log::*;
use num_derive::FromPrimitive;
use ring::digest::Digest;
use serde::{Deserialize, Serialize};
use tract_onnx::prelude::*;
use tract_onnx::tract_hir::infer::InferenceOp;

pub use tract_onnx::prelude::DatumType as TractDatumType;

pub fn deserialize_tensor_bytes(
    dt: ModelDatumType,
    shape: &[usize],
    data: &[u8],
) -> Result<Tensor> {
    unsafe {
        match dt {
            ModelDatumType::U8 => Tensor::from_raw::<u8>(shape, data),
            ModelDatumType::U16 => Tensor::from_raw::<u16>(shape, data),
            ModelDatumType::U32 => Tensor::from_raw::<u32>(shape, data),
            ModelDatumType::U64 => Tensor::from_raw::<u64>(shape, data),
            ModelDatumType::I8 => Tensor::from_raw::<i8>(shape, data),
            ModelDatumType::I16 => Tensor::from_raw::<i16>(shape, data),
            ModelDatumType::I32 => Tensor::from_raw::<i32>(shape, data),
            ModelDatumType::I64 => Tensor::from_raw::<i64>(shape, data),
            ModelDatumType::F32 => Tensor::from_raw::<f32>(shape, data),
            ModelDatumType::F64 => Tensor::from_raw::<f64>(shape, data),
            ModelDatumType::Bool => Ok(Tensor::from_raw::<u8>(shape, data)?
                .into_array::<u8>()?
                .mapv(|x| x != 0)
                .into()),
            // _ => bail!("Cannot parse tensor with datatype {:?} from bytes", dt),
        }
    }
}

pub fn serialize_tensor_bytes(tensor: &Tensor) -> Result<Cow<'_, [u8]>> {
    Ok(match tensor.datum_type().try_into()? {
        ModelDatumType::U8
        | ModelDatumType::U16
        | ModelDatumType::U32
        | ModelDatumType::U64
        | ModelDatumType::I8
        | ModelDatumType::I16
        | ModelDatumType::I32
        | ModelDatumType::I64
        | ModelDatumType::F32
        | ModelDatumType::F64 => unsafe { tensor.as_bytes() }.into(),
        ModelDatumType::Bool => tensor
            .to_array_view::<bool>()?
            .mapv(|x| x as u8)
            .into_raw_vec()
            .into(),
    })
}

#[derive(Debug, FromPrimitive, PartialEq, Clone, Copy, Eq, Hash, Serialize, Deserialize)]
pub enum ModelDatumType {
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

impl TryFrom<TractDatumType> for ModelDatumType {
    type Error = Error;

    fn try_from(value: TractDatumType) -> Result<Self, Self::Error> {
        Ok(match value {
            TractDatumType::F32 => ModelDatumType::F32,
            TractDatumType::F64 => ModelDatumType::F64,
            TractDatumType::I32 => ModelDatumType::I32,
            TractDatumType::I64 => ModelDatumType::I64,
            TractDatumType::U32 => ModelDatumType::U32,
            TractDatumType::U64 => ModelDatumType::U64,
            TractDatumType::U8 => ModelDatumType::U8,
            TractDatumType::U16 => ModelDatumType::U16,
            TractDatumType::I8 => ModelDatumType::I8,
            TractDatumType::I16 => ModelDatumType::I16,
            TractDatumType::Bool => ModelDatumType::Bool,
            _ => bail!("Unsupported datum type: {:?}", value),
        })
    }
}

impl Into<TractDatumType> for ModelDatumType {
    fn into(self) -> TractDatumType {
        match self {
            ModelDatumType::F32 => TractDatumType::F32,
            ModelDatumType::F64 => TractDatumType::F64,
            ModelDatumType::I32 => TractDatumType::I32,
            ModelDatumType::I64 => TractDatumType::I64,
            ModelDatumType::U32 => TractDatumType::U32,
            ModelDatumType::U64 => TractDatumType::U64,
            ModelDatumType::U8 => TractDatumType::U8,
            ModelDatumType::U16 => TractDatumType::U16,
            ModelDatumType::I8 => TractDatumType::I8,
            ModelDatumType::I16 => TractDatumType::I16,
            ModelDatumType::Bool => TractDatumType::Bool,
        }
    }
}

impl FromStr for ModelDatumType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "F32" | "f32" => ModelDatumType::F32,
            "F64" | "f64" => ModelDatumType::F64,
            "I32" | "i32" => ModelDatumType::I32,
            "I64" | "i64" => ModelDatumType::I64,
            "U32" | "u32" => ModelDatumType::U32,
            "U64" | "u64" => ModelDatumType::U64,
            "U8" | "u8" => ModelDatumType::U8,
            "U16" | "u16" => ModelDatumType::U16,
            "I8" | "i8" => ModelDatumType::I8,
            "I16" | "i16" => ModelDatumType::I16,
            "Bool" | "bool" => ModelDatumType::Bool,
            s => bail!("invalid model datum type: {:?}", s),
        })
    }
}

#[allow(unused_macros)]
/// This macro dispatches a generic function call from a ModelDatumType
/// instance.
///
/// # Example
/// ```rs
/// fn generic_datum_fn<D: Datum>(_dummy_arg: i32) {
///   // this function uses the datum type as a type parameter
/// }
///
/// let dt = ModelDatumType::F32; // we have a ModelDatumType enum instance here
/// dispatch_model_datum!(generic_datum_fn(dt)(0i32))
/// ```
macro_rules! dispatch_model_datum {
    ($($path:ident)::* ($dt:expr) ($($args:expr),*)) => { {
        use $crate::model::ModelDatumType;
        match $dt {
            ModelDatumType::F32 => $($path)::*::<f32>($($args),*),
            ModelDatumType::F64 => $($path)::*::<f64>($($args),*),
            ModelDatumType::I32 => $($path)::*::<i32>($($args),*),
            ModelDatumType::I64 => $($path)::*::<i64>($($args),*),
            ModelDatumType::U32 => $($path)::*::<u32>($($args),*),
            ModelDatumType::U64 => $($path)::*::<u64>($($args),*),
            ModelDatumType::U8 => $($path)::*::<u8>($($args),*),
            ModelDatumType::U16 => $($path)::*::<u16>($($args),*),
            ModelDatumType::I8 => $($path)::*::<i8>($($args),*),
            ModelDatumType::I16 => $($path)::*::<i16>($($args),*),
            ModelDatumType::Bool => $($path)::*::<bool>($($args),*),
        }
    } }
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct TensorFacts {
    pub datum_type: Option<ModelDatumType>,
    pub dims: Option<Vec<usize>>,
    pub index: Option<usize>,
    pub index_name: Option<String>,
}

pub type OptimizedOnnx =
    SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;
pub type UnoptimizedOnnx =
    SimplePlan<InferenceFact, Box<dyn InferenceOp>, Graph<InferenceFact, Box<dyn InferenceOp>>>;

fn add_model_facts(
    graph: &mut Graph<InferenceFact, Box<dyn InferenceOp>>,
    input_facts: &[TensorFacts],
    output_facts: &[TensorFacts],
) -> Result<()> {
    // add facts
    for (facts, is_input) in [(input_facts, true), (output_facts, false)].into_iter() {
        for (index, fact) in facts.into_iter().enumerate() {
            let mut inference_fact = InferenceFact::new();
            if let Some(datum_type) = fact.datum_type {
                inference_fact = inference_fact.with_datum_type(datum_type.into());
            }
            if let Some(dims) = fact.dims.as_ref() {
                inference_fact = inference_fact.with_shape(dims);
            }

            let index = if let Some(i) = fact.index {
                i
            } else if let Some(name) = fact.index_name.as_ref() {
                // find output / input by outlet name

                let outlet = graph
                    .find_outlet_label(name)
                    .ok_or_else(|| anyhow!("Cannot find input/output named {}", name))?;

                let outlets = if is_input {
                    graph.input_outlets()?
                } else {
                    graph.output_outlets()?
                };
                outlets
                    .into_iter()
                    .position(|el| *el == outlet)
                    .ok_or_else(|| anyhow!("Cannot find input/output named {}", name))?
            } else {
                index // default to using the index of the fact in the array
            };

            if is_input {
                graph.set_input_fact(index, inference_fact)?;
            } else {
                graph.set_output_fact(index, inference_fact)?;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Hash, Clone)]
pub enum TractModel {
    OptimizedOnnx(OptimizedOnnx),
    UnoptimizedOnnx(UnoptimizedOnnx),
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub enum ModelLoadContext {
    FromSendModel,
    FromStartupConfig,
}

#[derive(Debug)]
pub struct InferModel {
    pub model: Arc<TractModel>,
    model_id: String,
    model_name: Option<String>,
    model_hash: Digest,
    #[allow(unused)]
    load_context: ModelLoadContext,
    owner_id: Option<usize>,
}

impl InferModel {
    pub fn load_model_path(
        model_path: &Path,
        model_id: String,
        model_name: Option<String>,
        model_hash: Digest,
        input_facts: &[TensorFacts],
        output_facts: &[TensorFacts],
        optim: bool,
        load_context: ModelLoadContext,
        owner_id: Option<usize>,
    ) -> Result<Self> {
        let mut graph = tract_onnx::onnx().model_for_path(model_path)?;

        trace!(
            "Loading model from path with input_facts {:?} output_facts {:?} optim {:?}",
            input_facts,
            output_facts,
            optim
        );

        add_model_facts(&mut graph, input_facts, output_facts)?;

        let model = if optim {
            trace!("Optimizing model...");
            TractModel::OptimizedOnnx(graph.into_optimized()?.into_runnable()?)
        } else {
            TractModel::UnoptimizedOnnx(graph.into_runnable()?)
        };

        Ok(InferModel {
            model: model.into(),
            model_name,
            model_id,
            model_hash,
            load_context,
            owner_id,
        })
    }

    pub fn load_model(
        mut model_data: &[u8],
        model_id: String,
        model_name: Option<String>,
        model_hash: Digest,
        input_facts: &[TensorFacts],
        output_facts: &[TensorFacts],
        optim: bool,
        load_context: ModelLoadContext,
        owner_id: Option<usize>,
    ) -> Result<Self> {
        let mut graph = tract_onnx::onnx().model_for_read(&mut model_data)?;

        trace!(
            "Loading model from bytes with input_facts {:?} output_facts {:?} optim {:?}",
            input_facts,
            output_facts,
            optim
        );

        add_model_facts(&mut graph, input_facts, output_facts)?;

        let model = if optim {
            trace!("Optimizing model...");
            TractModel::OptimizedOnnx(graph.into_optimized()?.into_runnable()?)
        } else {
            TractModel::UnoptimizedOnnx(graph.into_runnable()?)
        };

        Ok(InferModel {
            model: model.into(),
            model_name,
            model_id,
            model_hash,
            load_context,
            owner_id,
        })
    }

    pub fn run_inference(&self, inputs: TVec<Tensor>) -> Result<TVec<Arc<Tensor>>> {
        trace!(
            "Running inference for model {}: {:?}.",
            self.model_id,
            inputs
        );
        match self.model.as_ref() {
            TractModel::OptimizedOnnx(model) => model.run(inputs),
            TractModel::UnoptimizedOnnx(model) => model.run(inputs),
        }
    }

    pub fn from_onnx_loaded(
        onnx: Arc<TractModel>,
        model_id: String,
        model_name: Option<String>,
        model_hash: Digest,
        owner_id: Option<usize>,
    ) -> Self {
        InferModel {
            model: onnx,
            model_id,
            model_name,
            model_hash,
            load_context: ModelLoadContext::FromSendModel,
            owner_id,
        }
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn owner_id(&self) -> Option<usize> {
        self.owner_id
    }

    pub fn model_hash(&self) -> Digest {
        self.model_hash
    }

    pub fn get_output_names(&self) -> Vec<String> {
        match self.model.as_ref() {
            TractModel::OptimizedOnnx(model) => model
                .outputs
                .iter()
                .enumerate()
                .map(|(i, outlet)| {
                    model
                        .model
                        .outlet_label(*outlet)
                        .map(|e| e.to_owned())
                        .unwrap_or_else(|| format!("output_{}", i))
                })
                .collect(),
            TractModel::UnoptimizedOnnx(model) => model
                .outputs
                .iter()
                .enumerate()
                .map(|(i, outlet)| {
                    model
                        .model
                        .outlet_label(*outlet)
                        .map(|e| e.to_owned())
                        .unwrap_or_else(|| format!("output_{}", i))
                })
                .collect(),
        }
    }
}
