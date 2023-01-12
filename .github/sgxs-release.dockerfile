# Minimal image to build the release version of the sgx enclave
FROM rust:1.66-slim-bullseye
WORKDIR blindai-preview
RUN apt-get update && apt install protobuf-compiler=3.12.4-1 pkg-config=0.29.2-1 libssl-dev=1.1.1n-0+deb11u3 -y && \
    rustup default nightly-2023-01-11 && rustup target add x86_64-fortanix-unknown-sgx
COPY src src
COPY Cargo.toml Cargo.lock ./
COPY tar-rs-sgx tar-rs-sgx
COPY tract tract
COPY ring-fortanix ring-fortanix
COPY host_server.key host_server.pem ./
RUN cargo install fortanix-sgx-tools sgxs-tools && cargo build --release --target "x86_64-fortanix-unknown-sgx" && \
    ftxsgx-elf2sgxs target/x86_64-fortanix-unknown-sgx/release/blindai_server --heap-size 0xFBA00000 --stack-size 0x400000 --threads 20
CMD sleep 20
