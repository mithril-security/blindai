use aesm_client::AesmClient;
use enclave_runner::EnclaveBuilder;
use sgxs_loaders::isgx::Device as IsgxDevice;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    thread,
};

fn usage(name: String) {
    println!("Usage: \n{name} <path_to_sgxs_file>");
}

fn parse_args() -> Result<String, ()> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        2 => Ok(args[1].to_owned()),
        _ => {
            usage(args[0].to_owned());
            Err(())
        }
    }
}

fn main() {
    // Running the remote attestation thread
    let remote_att_sgx = thread::spawn(remote_attestation_sgx::start_remote_attestation);

    // Extracting platform and uid from whoami
    let sgx_mode = if cfg!(target_env = "sgx") { "HW" } else { "SW" };
    let platform = format!("{} - {}", whoami::platform(), sgx_mode);
    let uid = {
        let mut hasher = DefaultHasher::new();
        whoami::username().hash(&mut hasher);
        whoami::hostname().hash(&mut hasher);
        platform.hash(&mut hasher);
        format!("{:X}", hasher.finish())
    };
    let custom_agent_id = std::env::var("CUSTOM_AGENT_ID").unwrap_or_default();
    // Running the enclave
    let file = parse_args().unwrap();
    let aesm_client = AesmClient::new();
    let mut device = IsgxDevice::new()
        .unwrap()
        .einittoken_provider(aesm_client)
        .build();
    let mut enclave_builder = EnclaveBuilder::new(file.as_ref());

    fn make_arg(arg_name: &str, arg_value: &str) -> Vec<u8> {
        let mut arg = arg_name.as_bytes().to_vec();
        arg.push(b'=');
        arg.extend_from_slice(arg_value.as_bytes());
        arg
    }
    enclave_builder.args([
        make_arg("--uid", &uid),
        make_arg("--platform", &platform),
        make_arg("--custom_agent_id", &custom_agent_id),
    ]);

    let enclave = enclave_builder.build(&mut device).unwrap();

    enclave
        .run()
        .map_err(|e| {
            println!("Error on exec SGX enclave \n {e}.");
            std::process::exit(1)
        })
        .unwrap();

    remote_att_sgx.join().unwrap();
}
