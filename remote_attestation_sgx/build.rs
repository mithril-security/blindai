
fn main() -> Result<(), Box<dyn std::error::Error>> {

    // add of the dcap_quoteprov shared library
    println!("cargo:rustc-link-lib=dylib=dcap_quoteprov");
    Ok(())
}
