# CI tests

# To build this image you should first build the dev-env image from the dockerfile in the .devcontainer folder.

# The goal is to handle the less likey and expensive file changes (dependency changes essentially) at the beginning of the build.
# This allow us to reuse the dependencies and ressources needed in the majority of cases

FROM blindaiv2-dev

# build Rust dependencies
RUN cargo new blindaiv2
WORKDIR /blindaiv2
COPY justfile Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY tar-rs-sgx tar-rs-sgx
COPY tract tract
COPY ring-fortanix ring-fortanix
COPY rouille rouille
COPY tiny-http tiny-http
RUN cargo build --target x86_64-fortanix-unknown-sgx\
    && cargo build --target x86_64-fortanix-unknown-sgx --release

# install python depencies
COPY client/pyproject.toml client/poetry.lock ./client/
RUN poetry install --directory ./client --no-root

# generate tests onnx models and inputs
COPY tests tests
RUN cd tests && bash generate_all_onnx_and_npz.sh

# compile Rust sources for the enclave
COPY src src
RUN touch src/main.rs \
    && cargo build --target x86_64-fortanix-unknown-sgx \
    && cargo build --target x86_64-fortanix-unknown-sgx --release

# compile Rust sources for the runner
COPY runner runner
RUN cd runner \
    && cargo check \
    && cargo build --release

# cargo fmt, clippy and audit
RUN cargo fmt --check \
    && cargo clippy --target x86_64-fortanix-unknown-sgx -p blindai_server -- --no-deps -Dwarnings \
    && cargo audit

# install python client and type stubs
COPY client client
RUN cd client \
    && poetry install

# python formatting checks and unit tests
RUN cd client \
    && poetry run black --check . \
    && poetry run mypy --install-types --non-interactive --ignore-missing-imports --follow-imports=skip \
    && poetry run pytest --ignore=tests/integration_test.py 

COPY manifest.dev.template.toml manifest.prod.template.toml ./

# end-to-end tests
CMD ( cd /opt/intel/sgx-dcap-pccs && npm start pm2 ) & \
    just run --release & \ 
    sleep 10 \
    && cd tests \
    && bash run_all_end_to_end_tests.sh
