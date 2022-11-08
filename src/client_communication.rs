use crate::identity::MyIdentity;
use crate::model::ModelDatumType;
use crate::model_store::ModelStore;
use anyhow::{Error, Result};
use log::{error, info};
use ring::digest;
use ring_compat::signature::Signer;
use serde_derive::{Deserialize, Serialize};
use std::mem::size_of;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct TensorInfo {
    fact: Vec<i64>,
    datum_type: ModelDatumType,
}

#[derive(Clone)]
pub(crate) struct Exchanger {
    model_store: Arc<ModelStore>,
    identity: Arc<MyIdentity>,
    max_model_size: usize,
    max_input_size: usize,
}

#[derive(Deserialize)]
struct DeleteModel {
    model_id: String,
}

#[derive(Deserialize)]
struct RunModel {
    model_id: String,
    inputs: Vec<u8>,
    sign: bool,
}

#[derive(Deserialize)]
struct UploadModel {
    model: Vec<u8>,
    input: Vec<TensorInfo>,
    output: Vec<ModelDatumType>,
    length: u64,
    sign: bool,
    model_name: String,
}

#[derive(Default, Serialize)]
struct SendModelPayload {
    hash: Vec<u8>,
    input_fact: Vec<i32>,
    model_id: String,
}

#[derive(Default, Serialize)]
struct SendModelReply {
    payload: Vec<u8>, //sendModelPayload,
    signature: Vec<u8>,
}

#[derive(Default, Serialize)]
struct RunModelPayload {
    output: Vec<u8>,
    datum_output: i32,
    input_hash: Vec<u8>,
    model_id: String,
}

#[derive(Default, Serialize)]
struct RunModelReply {
    payload: Vec<u8>, //runModelPayload,
    signature: Vec<u8>,
}

impl Exchanger {
    pub fn new(
        model_store: Arc<ModelStore>,
        identity: Arc<MyIdentity>,
        max_model_size: usize,
        max_input_size: usize,
    ) -> Self {
        Self {
            identity,
            model_store,
            max_model_size,
            max_input_size,
        }
    }

    pub fn send_model(&self, mut request: tiny_http::Request) -> Result<(), Error> {
        let start_time = Instant::now();

        let data_stream = request.as_reader();
        let mut data: Vec<u8> = vec![];
        data_stream.read_to_end(&mut data)?;

        let mut upload_model_body: UploadModel = serde_cbor::from_slice(&data).unwrap();

        let convert_type = |t: i32| -> Result<_, Error> {
            num_traits::FromPrimitive::from_i32(t)
                .ok_or_else(|| Error::msg("Unknown datum type".to_string()))
        };

        let mut tensor_inputs: Vec<TensorInfo> = Vec::new();
        let mut tensor_outputs: Vec<i32> = Vec::new();

        let mut datum_outputs: Vec<ModelDatumType> = Vec::new();
        let mut datum_inputs: Vec<ModelDatumType> = Vec::new();
        let mut input_facts: Vec<Vec<usize>> = Vec::new();
        let mut model_bytes: Vec<u8> = Vec::new();
        let max_model_size = self.max_model_size;
        let mut model_size = 0usize;
        let sign = false;

        let mut model_name: std::option::Option<String> = None;
        let client_info: std::option::Option<String> = None;

        if model_size == 0 {
            model_size = upload_model_body.length.try_into().unwrap();
            //model_size=267874659;
            model_bytes.reserve_exact(model_size);
            //model_name=None;
            model_name = if !upload_model_body.model_name.is_empty() {
                Some(upload_model_body.model_name)
            } else {
                None
            };
            println!("{:?}", model_name);
            //client_info = uploadModelBody.client_info;

            for tensor_info in &upload_model_body.input {
                tensor_inputs.push(tensor_info.clone());
            }

            for output in &upload_model_body.output {
                tensor_outputs.push((*output) as i32);
            }

            //sign = uploadModelBody.sign;
        }
        if model_size > max_model_size || model_bytes.len() > max_model_size {
            return Err(Error::msg("Model is too big".to_string()));
        }
        model_bytes.append(&mut upload_model_body.model);

        if model_size == 0 {
            return Err(Error::msg("Received no data".to_string()));
        }

        // Create datum_inputs, datum_outputs, and input_facts vector from tensor_inputs
        // and tensor_outputs
        for (_, tensor_input) in tensor_inputs.clone().iter().enumerate() {
            let mut input_fact: Vec<usize> = vec![];

            for x in &tensor_input.fact {
                input_fact.push(*x as usize);
            }
            let datum_input = convert_type(tensor_input.datum_type as i32)?; // TEMP-FIX, FIX THIS!//convert_type(tensor_input.datum_type.clone())?;
            datum_outputs = tensor_outputs
                .iter()
                .map(|v| convert_type(*v).unwrap())
                .collect();
            datum_inputs.push(datum_input);
            input_facts.push(input_fact.clone());
        }

        let (model_id, model_hash) = self
            .model_store
            .add_model(
                &model_bytes,
                input_facts.clone(),
                model_name,
                datum_inputs.clone(),
                datum_outputs,
            )
            .map_err(|err| {
                error!("Error while creating model: {}", err);
                println!("Error storing model");
            })
            .unwrap();

        // Construct the return payload

        let mut payload = SendModelPayload::default();
        if upload_model_body.sign {
            payload.hash = model_hash.as_ref().to_vec();
            payload.input_fact = input_facts
                .into_iter()
                .flatten()
                .map(|i| i as i32)
                .collect();
        }
        payload.model_id = model_id.to_string();
        /*
        let payload_with_header = Payload {
            header: Some(PayloadHeader {
                issued_at: Some(SystemTime::now().into()),
            }),
            payload: Some(payload::Payload::SendModelPayload(payload)),
        };
        */

        let mut reply = SendModelReply::default();
        /*
        {
            payload: payload_with_header.encode_to_vec(),

        };
        */

        reply.payload = serde_cbor::to_vec(&payload).unwrap();

        if upload_model_body.sign {
            reply.signature = self
                .identity
                .signing_key
                .sign(&reply.payload)
                .to_bytes()
                .to_vec();
        }

        // Logs and telemetry
        let elapsed = start_time.elapsed();
        info!(
            "Sample message" /*
                             [{} {}] SendModel successful in {}ms (model={}, size={}, sign={})",

                             client_info
                                 .as_ref()
                                 .map(|c| c.user_agent.as_ref())
                                 .unwrap_or("<unknown>"),
                             client_info
                                 .as_ref()
                                 .map(|c| c.user_agent_version.as_ref())
                                 .unwrap_or("<unknown>"),
                             elapsed.as_millis(),
                             model_name.as_deref().unwrap_or("<unknown>"),
                             model_size,
                             sign
                             */
        );
        /*
        telemetry::add_event(
            TelemetryEventProps::SendModel {
                model_size,
                model_name,
                sign,
                time_taken: elapsed.as_secs_f64(),
            },
            client_info,
        );
        */
        //Ok(Response::new(reply))
        println!("Successfully saved model");
        let serialized_reply = serde_cbor::to_vec(&reply).unwrap();

        let header =
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap();
        let response = tiny_http::Response::new(
            tiny_http::StatusCode::from(200),
            vec![header],
            serialized_reply.as_slice(),
            None,
            None,
        );

        //let response = tiny_http::Response::from_string(model_id.to_string());
        request.respond(response)?;
        Ok(())
    }

    pub fn run_model(&self, mut request: tiny_http::Request) -> Result<(), Error> {
        //Result<tiny_http::Response<std::io::Cursor<Vec<u8>>>, Error> {
        let start_time = Instant::now();

        let input: Vec<u8> = Vec::new();
        let sign = false;
        let max_input_size = self.max_input_size;
        let model_id = "".to_string();

        //let mut client_info = None;
        let data_stream = request.as_reader();
        let mut data: Vec<u8> = vec![];
        data_stream.read_to_end(&mut data)?;

        let run_model_body: RunModel = serde_cbor::from_slice(&data).unwrap();

        if run_model_body.inputs.len() * size_of::<u8>() > max_input_size
            || run_model_body.inputs.len() * size_of::<u8>() > max_input_size
        {
            return Err(Error::msg("Input too big".to_string()));
        }

        /*
        if runModelBody.inputs.is_empty() {
            sign = runModelBody.sign;
            model_id = runModelBody.modelID;
        }
        */

        //input.append(&mut data_proto.input);

        let uuid = match Uuid::from_str(&run_model_body.model_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                println!("Error in uuid");
                return Err(Error::msg("Model doesn't exist".to_string()));
            }
        };

        let res = self.model_store.use_model(uuid, |model| {
            (
                model.run_inference(&mut run_model_body.inputs.clone()[..]),
                model.model_name().map(|s| s.to_string()),
                model.datum_output(),
            )
        });

        let res = match res {
            Some(res) => res,
            None => {
                println!("Error in model match");
                return Err(Error::msg("Model doesn't exist".to_string()));
            }
        };

        let (result, model_name, datum_output) = res;

        let result = match result {
            Ok(res) => res,
            Err(err) => {
                error!("Error while running inference: {}", err);
                println!("Error in result");
                return Err(Error::msg("Unknown error".to_string()));
            }
        };

        //let mut ret_payload:runModelReturn = {

        //};

        let mut payload = RunModelPayload::default();
        payload.output = result;
        payload.datum_output = datum_output as i32;

        /*
        runModelPayload {
            output: result,
            datum_output: datum_output as i32,
            ..Default::default()
        };
        */

        if run_model_body.sign {
            payload.input_hash = digest::digest(&digest::SHA256, &input).as_ref().to_vec();
            payload.model_id = model_id;
        }

        /*
        let payload_with_header = Payload {
            header: Some(PayloadHeader {
                issued_at: Some(SystemTime::now().into()),
            }),
            payload: Some(payload::Payload::RunModelPayload(payload)),
        };
        */

        let mut reply = RunModelReply::default();
        reply.payload = serde_cbor::to_vec(&payload).unwrap();

        /*
        RunModelReply {
            payload: payload_with_header.encode_to_vec(),
            ..Default::default()
        };
        */

        if sign {
            reply.signature = self
                .identity
                .signing_key
                .sign(&reply.payload)
                .to_bytes()
                .to_vec();
        }

        /*
        // Log and telemetry
        let elapsed = start_time.elapsed();
        info!(
            "[{} {}] RunModel successful in {}ms (model={}, sign={})",
            client_info
                .as_ref()
                .map(|c| c.user_agent.as_ref())
                .unwrap_or("<unknown>"),
            client_info
                .as_ref()
                .map(|c| c.user_agent_version.as_ref())
                .unwrap_or("<unknown>"),
            elapsed.as_millis(),
            model_name.as_deref().unwrap_or("<unknown>"),
            sign
        );
        telemetry::add_event(
            TelemetryEventProps::RunModel {
                model_name: model_name.map(|e| e.to_string()),
                sign,
                time_taken: elapsed.as_secs_f64(),
            },
            client_info,
        );
        */
        println!("Successfully ran model");

        let serialized_reply = serde_cbor::to_vec(&reply).unwrap();

        let header =
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap();
        let response = tiny_http::Response::new(
            tiny_http::StatusCode::from(200),
            vec![header],
            serialized_reply.as_slice(),
            None,
            None,
        );

        //let response = tiny_http::Response::from_string(model_id);
        request.respond(response)?;
        Ok(())
    }

    pub fn delete_model(&self, mut request: tiny_http::Request) -> Result<(), Error> {
        let data_stream = request.as_reader();
        let mut data: Vec<u8> = vec![];
        data_stream.read_to_end(&mut data)?;

        let delete_model_body: DeleteModel = serde_cbor::from_slice(&data).unwrap();

        if delete_model_body.model_id.is_empty() {
            return Err(Error::msg("Model doesn't exist".to_string()));
        }

        let model_id = Uuid::from_str(&delete_model_body.model_id).unwrap();

        // Delete the model
        if self.model_store.delete_model(model_id).is_none() {
            error!("Model doesn't exist");
            return Err(Error::msg("Model doesn't exist".to_string()));
        }

        println!("Deleted model successfully");
        let response = tiny_http::Response::from_string("Deleted".to_string());
        request.respond(response)?;
        Ok(())
    }
}
