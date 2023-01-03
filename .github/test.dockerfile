# CI tests

# To build this image you should first build the dev-env image from the dockerfile in the .devcontainer folder.

# The goal is to handle the less likey and expensive file changes (dependency changes essentially) at the beginning of the build.
# This allow us to reuse the dependencies and ressources needed in the majority of cases

FROM blindaiv2-dev

# build Rust dependencies
RUN cargo new blindaiv2
WORKDIR /blindaiv2
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY tar-rs-sgx tar-rs-sgx
COPY tract tract
COPY ring-fortanix ring-fortanix
RUN cargo build \
    && cargo build --release

# install python depencies
COPY client/pyproject.toml client/poetry.lock ./client/
RUN poetry install --directory ./client --no-root

# generate tests onnx models and inputs
COPY tests tests
RUN cd tests && bash generate_all_onnx_and_npz.sh

# compile Rust sources
COPY src src
COPY host_server.pem host_server.key ./
RUN touch src/main.rs \
    && cargo build \
    && cargo build --release

# fmt and clippy
RUN cargo fmt --check \
    && cargo clippy -p blindai_server -- --no-deps -Dwarnings

# install python client
COPY client client
RUN cd client && poetry install --only-root

# python formatting and checks
RUN cd client \
    && poetry run black --check . \
    && poetry run pytest

# end-to-end tests
CMD cargo run --release \
    & sleep 15 \
    && cd tests \
    && bash run_all_end_to_end_tests.sh
