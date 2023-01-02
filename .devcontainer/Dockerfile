# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.241.1/containers/ubuntu/.devcontainer/base.Dockerfile

# [Choice] Ubuntu version (use ubuntu-22.04 or ubuntu-18.04 on local arm64/Apple Silicon): ubuntu-22.04, ubuntu-20.04, ubuntu-18.04
ARG VARIANT="focal"

FROM mcr.microsoft.com/vscode/devcontainers/base:0-${VARIANT} AS build_valgrind
# Install valgrind patched to add support for SGX simulation
RUN apt-get update \
    && apt-get install -y autotools-dev automake \
    && git clone --depth 1 -b sgx https://github.com/mithril-security/valgrind.git \
    && cd valgrind \
    && ./autogen.sh \
    && ./configure \
    && make

## Final image
FROM mcr.microsoft.com/vscode/devcontainers/base:0-${VARIANT} AS development
# [Optional] Uncomment this section to install additional OS packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>

ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    POETRY_HOME=/home/vscode/.local/bin
ENV PATH=${CARGO_HOME}/bin:${POETRY_HOME}:${PATH}
COPY .devcontainer/library-scripts/rust-debian.sh /tmp/library-scripts/

# Enable default rustup profile instead of the minimal profile (default of the rust-debian.sh)
# The default profile includes all of components in the minimal profile, and adds rust-docs, rustfmt, and clippy. 
# This profile will be used by rustup by default, and it's the one recommended for general use.
RUN apt-get update && bash /tmp/library-scripts/rust-debian.sh "${CARGO_HOME}" "${RUSTUP_HOME}" "" "" "" "" "default"

RUN echo "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main" | tee -a /etc/apt/sources.list.d/intel-sgx.list >/dev/null \ 
    && curl -sSL "https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key" | apt-key add - \
    && apt-get update \
    && apt-get install -y sgx-aesm-service libsgx-aesm-launch-plugin

RUN rustup default nightly \
    && rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly \
    && chmod -R g+r+w "${RUSTUP_HOME}"
    
RUN apt-get update \
    && apt-get install -y pkg-config libssl-dev protobuf-compiler

# Fix #1 We need to download and install from fortanix git repo instead of crates.io becaue of this issue 
# https://github.com/fortanix/rust-sgx/issues/401
# Fix #2 To enable simulation mode we use our own fork
RUN cargo install fortanix-sgx-tools ftxsgx-simulator sgxs-tools --git https://github.com/mithril-security/rust-sgx --branch sim-mode \
    && chmod -R g+r+w "${CARGO_HOME}"

# Add valgrind
COPY --from=build_valgrind /valgrind /valgrind
RUN apt-get install -y automake \
    &&  cd /valgrind \
    && make install

# Add gdb
RUN apt-get install -y gdb

# Add just
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/bin

# Add poetry
RUN apt-get install -y python3-dev python3-distutils \
    && sudo -u "vscode" zsh -c "\
    curl -sSL https://install.python-poetry.org | python3 - \
    && mkdir ~/.oh-my-zsh/custom/plugins/poetry \
    && export PATH=${POETRY_HOME}:$PATH \
    && poetry completions zsh > ~/.oh-my-zsh/custom/plugins/poetry/_poetry \
    && sed -i -E 's/^plugins\=\(([a-zA-Z ]+)*\)/plugins=(\1 poetry)/' ~/.zshrc"

# Add github cli
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt update \
    && apt install gh -y

# Add examples dependencies
RUN apt-get install -y ffmpeg

COPY .devcontainer/start-aesmd.sh /tmp/start-aesmd.sh


# CI tests

# The goal is to handle the less likey and expensive file changes (dependency changes essentially) at the beginning of the build.
# This allow us to reuse the dependencies and ressources needed in the majority of cases

FROM development AS test

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
RUN cd tests \
    && bash setup.sh

# compile Rust sources
COPY src src
COPY host_server.pem host_server.key ./
RUN touch src/main.rs \
    && cargo build \
    && cargo build --release

# fmt and clippy
RUN cargo fmt --check \
    && cargo clippy -p tinyhttp-fortanix -- --no-deps -Dwarnings

# install python client
COPY client client
RUN cd client && poetry install --only-root

# python tests
RUN cd client && poetry run pytest

# end-to-end tests
CMD cargo run --release \
    & sleep 120 \
    && cd tests \
    && bash assert_all.sh
    
