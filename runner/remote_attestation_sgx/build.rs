fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rustc-link-lib=dylib=dcap_quoteprov");
    Ok(())
}
