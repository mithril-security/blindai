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
use anyhow::{anyhow, bail, Result};
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

macro_rules! convert_datum {
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

trait FromLeBytes :Sized {
    fn from_le_bytes(bytes: &[u8]) -> anyhow::Result<Self>;
}

trait ToLeBytes {
    fn to_le_bytes(&self) -> Vec<u8>;
}

#[test]
fn test_deserialize_array_bool() {
    assert_eq!(&Vec::<bool>::from_le_bytes(b"\x00\x01\x00").unwrap(), &[false, true, false]);
}

#[test]
fn test_deserialize_array_u16() {
    assert_eq!(&Vec::<u16>::from_le_bytes(b"\x01\x00\x02\x00\x03\x00").unwrap(), &[1u16,2,3]);
}

#[test]
fn test_deserialize_array_u32() {
    assert_eq!(&Vec::<u32>::from_le_bytes(b"\x01\x00\x00\x00\x02\x00\x00\x00\x03\x00\x00\x00").unwrap(), &[1u32,2,3]);
}

#[test]
fn test_deserialize_array_f32() {
    assert_eq!(&Vec::<f32>::from_le_bytes(b"\x00\x00\x80?\x00\x00\x00@\x00\x00@@").unwrap(), &[1.0f32,2.0,3.0]);
}

#[test]
fn test_deserialize_array_i32() {
    assert_eq!(&Vec::<i32>::from_le_bytes(b"\xce\xff\xff\xff\n\x00\x00\x00\xe8\x03\x00\x00").unwrap(), &[-50i32,10,1000]);
}

#[test]
fn test_deserialize_array_i32_corrupted() {
    let e = Vec::<i32>::from_le_bytes(b"\xce\xff\xff\xff\n\x00\x00\x00\xe8\x03\x00\x00\x03").unwrap_err();
    assert_eq!(e.to_string(), "Could not deserialize input");
}


#[test]
fn test_serialize_i32() {
    let x = [-10i32, 50, 1000].as_ref();
    assert_eq!(&Vec::<i32>::from_le_bytes(&x.to_le_bytes()).unwrap(), &x);
}

#[test]
fn test_serialize_bool() {
    let x = [true, false, true].as_ref();
    assert_eq!(&Vec::<bool>::from_le_bytes(&x.to_le_bytes()).unwrap(), &x);
}

#[test]
fn test_serialize_f32() {
    let x = [0.5, 3.14, 1000_0000.].as_ref();
    assert_eq!(&Vec::<f32>::from_le_bytes(&x.to_le_bytes()).unwrap(), &x);
}

// Macro to implement both FromLeBytes and ToLeBytes for basic numeric types
macro_rules! impl_vec_from_to_le_bytes {
    ($t:ident) => {
        impl FromLeBytes for Vec<$t> {
            fn from_le_bytes(bytes: &[u8]) -> Result<Self> {
                let mut v = Vec::<$t>::with_capacity(bytes.len()/std::mem::size_of::<$t>());
                if bytes.len() % std::mem::size_of::<$t>() != 0 {
                    bail!("Could not deserialize input");
                }
                for chunk in bytes.chunks(std::mem::size_of::<$t>()) {
                    v.push($t::from_le_bytes(chunk.try_into().unwrap()));
                }
                Ok(v)
            }
        }
        impl ToLeBytes for &[$t] {
            fn to_le_bytes(&self) -> Vec<u8> {
                self.iter().flat_map(|e| e.to_le_bytes()).collect()
            }
        }
    }
}


impl_vec_from_to_le_bytes!(u8);
impl_vec_from_to_le_bytes!(u16);
impl_vec_from_to_le_bytes!(u32);
impl_vec_from_to_le_bytes!(u64);
impl_vec_from_to_le_bytes!(i8);
impl_vec_from_to_le_bytes!(i16);
impl_vec_from_to_le_bytes!(i32);
impl_vec_from_to_le_bytes!(i64);
impl_vec_from_to_le_bytes!(f32);
impl_vec_from_to_le_bytes!(f64);

impl FromLeBytes for Vec<bool> {
    fn from_le_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bytes.into_iter().map(|b| *b!=0).collect())
    }
}

impl ToLeBytes for &[bool] {
    fn to_le_bytes(&self) -> Vec<u8> {
        self.iter().map(|e| *e as u8).collect()
    }
}



fn create_tensor<A: tract_core::prelude::Datum>(
    input: &[u8],
    input_fact: &[usize],
) -> Result<Tensor> where Vec<A>: FromLeBytes{
    let dim = tract_ndarray::IxDynImpl::from(input_fact);
    let vec  = Vec::<A>::from_le_bytes(input)?;
    let tensor = tract_ndarray::ArrayD::from_shape_vec(dim, vec)?.into();
    Ok(tensor)
}

fn convert_tensor<A: serde::ser::Serialize + tract_core::prelude::Datum>(
    input: &tract_core::prelude::Tensor,
) -> Result<Vec<u8>> where for<'a> &'a[A]: ToLeBytes {
    let arr = input.to_array_view::<A>()?;
    let slice = arr
        .as_slice()
        .ok_or_else(|| anyhow!("Failed to convert ArrayView to slice"))?;
    Ok(slice.to_le_bytes())
}

#[derive(Debug)]
pub struct InferenceModel {
    pub onnx: Arc<OnnxModel>,
    #[allow(unused)]
    model_id: Uuid,
    model_name: Option<String>,
    model_hash: Digest,
}

impl InferenceModel {
    #[allow(clippy::too_many_arguments)]
    pub fn load_model(
        mut model_data: &[u8],
        model_id: Uuid,
        model_name: Option<String>,
        model_hash: Digest,
        optimize: bool,
    ) -> Result<Self> {
        let model_rec = tract_onnx::onnx()
            .with_ignore_output_shapes(true)
            .model_for_read(&mut model_data)?;
        let onnx = match optimize {
            true => model_rec.into_optimized()?,
            false => model_rec.into_typed()?,
        };

        Ok(InferenceModel {
            onnx: onnx.into_runnable()?.into(),
            model_name,
            model_id,
            model_hash,
        })
    }

    pub fn run_inference(&self, inputs: &[SerializedTensor]) -> Result<Vec<SerializedTensor>> {
        let mut tensors: Vec<_> = vec![];
        let outlets = self.onnx.model.input_outlets()?;
        for tensor in inputs {
            let tract_tensor = convert_datum!(create_tensor(
                tensor.info.datum_type.get_datum_type()
            )(
                &tensor.bytes_data, tensor.info.fact.as_slice()
            ))?;
            if let Some(node_name) = &tensor.info.node_name {
                let node_id = self.onnx.model.node_id_by_name(node_name)?;
                let rank = outlets
                    .iter()
                    .position(|&outlet| outlet.node == node_id)
                    .ok_or_else(|| anyhow!("no node with name {}", node_name))?;
                tensors.insert(rank, tract_tensor);
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
                bytes_data: convert_datum!(convert_tensor(tensor.datum_type())(tensor))?,
            });
        }
        Ok(outputs)
    }

    pub fn from_onnx_loaded(
        onnx: Arc<OnnxModel>,
        model_id: Uuid,
        model_name: Option<String>,
        model_hash: Digest,
    ) -> Self {
        InferenceModel {
            onnx,
            model_id,
            model_name,
            model_hash,
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
                    .unwrap_or_else(|| format!("output_{i}"))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_store::ModelStore;
    use anyhow::Result;

    use std::str::FromStr;
    use std::{collections::HashMap, sync::Mutex};

    use lazy_static::lazy_static;

    static MOBILENET: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/mobilenet/mobilenetv2-7.onnx"
    ));
    static GRACE_HOPPER_JPG: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/mobilenet/grace_hopper.jpg"
    ));

    lazy_static! {
        static ref MODELSTORE: Mutex<ModelStore> = Mutex::new(ModelStore::new());
    }

    lazy_static! {
        static ref UUID_HASHMAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    }

    fn get_uuid(name: String) -> String {
        let mutex_guard = UUID_HASHMAP.lock().unwrap();
        let uuid_res = mutex_guard.get(&name);
        match uuid_res {
            None => "".into(),
            Some(uuid) => uuid.into(),
        }
    }

    fn add_uuid(name: String, uuid: String) {
        UUID_HASHMAP.lock().unwrap().insert(name, uuid);
    }

    fn add_model(model_bytes: &[u8], model_name: String, optimize: bool) -> Result<(Uuid, Digest)> {
        MODELSTORE
            .lock()
            .unwrap()
            .add_model(model_bytes, Some(model_name), optimize)
    }

    #[test]
    fn load_mobilenet_optimized() {
        let res = add_model(MOBILENET, "optimized".into(), true);
        assert_eq!(res.is_ok(), true);
        add_uuid("optimized".into(), res.unwrap().0.to_string())
    }

    #[test]
    fn load_mobilenet_non_optimized() {
        let res = add_model(MOBILENET, "non_optimized".into(), false);
        assert_eq!(res.is_ok(), true);
        add_uuid("non_optimized".into(), res.unwrap().0.to_string())
    }

    #[test]
    fn run_mobilenet_optimized() {
        let uuid = get_uuid("optimized".into());
        common_runmodel(uuid)
    }

    #[test]
    fn run_mobilenet_non_optimized() {
        let uuid = get_uuid("non_optimized".into());
        common_runmodel(uuid)
    }

    fn common_runmodel(uuid: String) {
        // taken straight from tract example, will prepare a jpg for the inference
        let image = image::load_from_memory(GRACE_HOPPER_JPG).unwrap().to_rgb8();
        let resized =
            image::imageops::resize(&image, 224, 224, ::image::imageops::FilterType::Triangle);
        let image = tract_ndarray::Array4::from_shape_fn((1, 3, 224, 224), |(_, c, y, x)| {
            let mean = [0.485, 0.456, 0.406][c];
            let std = [0.229, 0.224, 0.225][c];
            (resized[(x as _, y as _)][c] as f32 / 255.0 - mean) / std
        });

        // For tests purpose, the SerializedTensor object is created by hand
        let image = serde_cbor::to_vec(&image.as_slice().unwrap()).unwrap();
        let info = TensorInfo {
            fact: vec![1, 3, 224, 224],
            datum_type: ModelDatumType::F32,
            node_name: None,
        };
        let tensor = SerializedTensor {
            info: info,
            bytes_data: image,
        };

        let res = MODELSTORE
            .lock()
            .unwrap()
            .use_model(Uuid::from_str(&uuid).unwrap(), |model| {
                (model.run_inference(vec![tensor.clone()].as_slice()),)
            });
        if let Some(tensor) = res {
            let result = &tensor.0.expect("Failed to run inference")[0];
            let tract_tensor =
                create_tensor::<f32>(&result.bytes_data, result.info.fact.as_slice()).unwrap();
            let arr = tract_tensor
                .to_array_view::<f32>()
                .unwrap()
                .iter()
                .cloned()
                .zip(2..)
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            if let Some(final_result) = arr {
                // Uses result got from onnx-mobile example from tract
                let diff = f32::abs(final_result.0 - 12.316545);
                assert_eq!(diff < 0.001, true); // Reproduce (or at least try to) assertLess from Python
                assert_eq!(final_result.1, 654)
            } else {
                panic!("Inference failed");
            }
        } else {
            panic!("Inference failed");
        }
    }
}
