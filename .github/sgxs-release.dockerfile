# Minimal image to build the release version of the sgx enclave
FROM rust:1.66.0-alpine3.17
WORKDIR blindai-preview
RUN rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly
COPY src src
COPY Cargo.toml Cargo.lock ./
COPY tar-rs-sgx tar-rs-sgx
COPY tract tract
COPY ring-fortanix ring-fortanix
RUN cargo build --release --target "x86_64-fortanix-unknown-sgx"