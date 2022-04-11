use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/untrusted.proto")?;
    tonic_build::compile_protos("../proto/securedexchange.proto")?;

    let sdk_dir = env::var("SGX_SDK").unwrap_or_else(|_| "/opt/sgxsdk".to_string());
    let is_sim = env::var("SGX_MODE").ok().as_deref() == Some("SW");

    println!("cargo:rustc-cfg=SGX_MODE=\"{}\"", if is_sim { "SW" } else { "HW" });

    println!("cargo:rustc-link-search=native=./tmp/lib");
    println!("cargo:rustc-link-lib=static=Enclave_u");

    println!("cargo:rustc-link-search=native={}/lib64", sdk_dir);

    println!("cargo:rustc-link-lib=dylib=dcap_quoteprov");

    if is_sim {
        println!("cargo:rustc-link-lib=dylib=sgx_urts_sim");
    } else {
        println!("cargo:rustc-link-lib=dylib=sgx_urts");
    }

    Ok(())
}
