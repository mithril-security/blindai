use aesm_client::AesmClient;
use enclave_runner::EnclaveBuilder;
use env_logger::Env;
use sgxs_loaders::isgx::Device as IsgxDevice;
use std::thread;

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
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    // Running the remote attestation thread
    let remote_att_sgx = thread::spawn(|| remote_attestation_sgx::start_remote_attestation());

    // Running the enclave
    let file = parse_args().unwrap();
    let aesm_client = AesmClient::new();
    let mut device = IsgxDevice::new()
        .unwrap()
        .einittoken_provider(aesm_client)
        .build();
    let enclave_builder = EnclaveBuilder::new(file.as_ref());

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
