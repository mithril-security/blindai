#[cfg(target_env = "sgx")]
pub mod platform;

#[cfg(not(target_env = "sgx"))]
mod ocall;
