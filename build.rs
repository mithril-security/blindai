use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=DISALLOW_REMOTE_UPLOAD");
    let disallow_upload_remotely = env::var("DISALLOW_REMOTE_UPLOAD");
    if disallow_upload_remotely.is_ok() {
        println!("cargo:rustc-cfg=DISALLOW_UPLOAD_REMOTELY=\"true\"")
    }
    Ok(())
}
