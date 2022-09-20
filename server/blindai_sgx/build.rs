use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto/untrusted.proto");
    println!("cargo:rerun-if-changed=../proto/securedexchange.proto");
    tonic_build::compile_protos("../proto/securedexchange.proto")?;
    tonic_build::compile_protos("../proto/untrusted.proto")?;

    let is_sim = env::var("SGX_MODE").unwrap_or_else(|_| "HW".to_string());

    if is_sim == "SW" {
        println!("cargo:rustc-cfg=SGX_MODE=\"SW\"");
    } else {
        println!("cargo:rustc-cfg=SGX_MODE=\"HW\"");
    }
    Ok(())
}
