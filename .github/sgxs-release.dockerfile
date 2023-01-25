# Minimal image to build the release version of the sgx enclave
FROM rust:1.66.1-slim-bullseye
WORKDIR blindai-preview

# Install dependencies and pre-install the rust toolchain declared via rust-toolchain.toml 
# for better caching
RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        protobuf-compiler=3.12.4-1 \
        pkg-config=0.29.2-1 \
        libssl-dev=1.1.1n-0+deb11u3 \
        gettext-base \
    && rm -rf /var/lib/apt/lists/* \
    && rustup default nightly-2023-01-11 \
    && rustup target add x86_64-fortanix-unknown-sgx

RUN cargo install --locked --git https://github.com/mithril-security/rust-sgx.git --tag fortanix-sgx-tools_v0.5.1-mithril fortanix-sgx-tools sgxs-tools

COPY Cargo.toml Cargo.lock rust-toolchain.toml  manifest.prod.template.toml ./
COPY src src
COPY tar-rs-sgx tar-rs-sgx
COPY tract tract
COPY ring-fortanix ring-fortanix
COPY tiny-http tiny-http
COPY rouille rouille

RUN cargo build --locked --release --target "x86_64-fortanix-unknown-sgx"

RUN ftxsgx-elf2sgxs target/x86_64-fortanix-unknown-sgx/release/blindai_server --heap-size 0xFBA00000 --stack-size 0x400000 --threads 20 \
    && mr_enclave=`sgxs-hash target/x86_64-fortanix-unknown-sgx/release/blindai_server.sgxs` envsubst < manifest.prod.template.toml > manifest.toml