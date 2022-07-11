# Build tree:
# base
#  \-> base-build
#       \-> build-software
#       \-> build-hardware
#  \-> software
#       * copies binaries from build-software
#  \-> hardware
#       * copies binaries from build-hardware
#  \-> hardware-dcsv3
#       * copies binaries from build-hardware
#
# Check <https://docs.mithrilsecurity.io/started/installation> for more info

#######################################
### Base stage: common dependencies ###
#######################################

### base: This image is kept minimal and optimized for size. It has the common runtime dependencies
FROM ubuntu:18.04 AS base

ARG CODENAME=bionic
ARG UBUNTU_VERSION=18.04
ARG SGX_VERSION=2.15.101.1-bionic1
ARG DCAP_VERSION=1.12.101.1-bionic1
ARG SGX_LINUX_X64_SDK=sgx_linux_x64_sdk_2.15.101.1.bin
ARG SGX_LINUX_X64_SDK_URL="https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/"$SGX_LINUX_X64_SDK

ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /root

# -- Install SGX SDK & SGX drivers
RUN \
    # Install temp dependencies
    TEMP_DEPENDENCIES="wget gnupg curl software-properties-common build-essential make" && \
    apt-get update -y && apt-get install -y $TEMP_DEPENDENCIES && \

    # Intall the SGX drivers
    curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add - && \
    add-apt-repository "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $CODENAME main" && \
    apt-get install -y \
        sgx-aesm-service=$SGX_VERSION \
        libsgx-ae-epid=$SGX_VERSION \
        libsgx-ae-le=$SGX_VERSION \
        libsgx-ae-pce=$SGX_VERSION \
        libsgx-aesm-ecdsa-plugin=$SGX_VERSION \
        libsgx-aesm-epid-plugin=$SGX_VERSION \
        libsgx-aesm-launch-plugin=$SGX_VERSION \
        libsgx-aesm-pce-plugin=$SGX_VERSION \
        libsgx-aesm-quote-ex-plugin=$SGX_VERSION \
        libsgx-enclave-common=$SGX_VERSION \
        libsgx-epid=$SGX_VERSION \
        libsgx-launch=$SGX_VERSION \
        libsgx-quote-ex=$SGX_VERSION \
        libsgx-uae-service=$SGX_VERSION \
        libsgx-urts=$SGX_VERSION \
        libsgx-ae-qe3=$DCAP_VERSION \
        libsgx-ae-pce=$SGX_VERSION \
        libsgx-pce-logic=$DCAP_VERSION \
        libsgx-qe3-logic=$DCAP_VERSION \
        libsgx-ra-network=$DCAP_VERSION \
        libsgx-ra-uefi=$DCAP_VERSION \
        libsgx-dcap-ql=$DCAP_VERSION \
        libsgx-dcap-quote-verify=$DCAP_VERSION \
        libsgx-dcap-default-qpl=$DCAP_VERSION && \
    mkdir -p /var/run/aesmd && \
    ln -s /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so.1 /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so && \

    # Intall the SGX SDK
    wget "https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/"$SGX_LINUX_X64_SDK && \
    chmod u+x $SGX_LINUX_X64_SDK  && \
    echo -e 'no\n/opt' | ./$SGX_LINUX_X64_SDK && \
    rm $SGX_LINUX_X64_SDK && \
    echo 'source /opt/sgxsdk/environment' >> /etc/environment && \

    # Remove temp dependencies
    apt-get remove -y $TEMP_DEPENDENCIES && apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* && rm -rf /var/cache/apt/archives/*

ENV LD_LIBRARY_PATH=/opt/sgxsdk/sdk_libs:/usr/lib:/usr/local/lib:/opt/intel/sgx-aesm-service/aesm/

### base-build: This image has the common build-time dependencies
FROM base AS base-build

ENV GCC_VERSION=8.4.0-1ubuntu1~18.04
ENV RUST_TOOLCHAIN=nightly-2021-11-01
ENV RUST_UNTRUSTED_TOOLCHAIN=nightly-2021-11-01

RUN apt update && apt install -y \
    unzip \
    lsb-release \
    debhelper \
    cmake \
    reprepro \
    autoconf \
    automake \
    bison \
    build-essential \
    curl \
    dpkg-dev \
    expect \
    flex \
    gdb \
    git \
    git-core \
    gnupg \
    kmod \
    libboost-system-dev \
    libboost-thread-dev \
    libcurl4-openssl-dev \
    libiptcdata0-dev \
    libjsoncpp-dev \
    liblog4cpp5-dev \
    libprotobuf-dev \
    libssl-dev \
    libtool \
    libxml2-dev \
    ocaml \
    ocamlbuild \
    pkg-config \
    protobuf-compiler \
    python \
    texinfo \
    uuid-dev \
    wget \
    zip \
    software-properties-common \
    cracklib-runtime \
    gcc-8=$GCC_VERSION \
 && rm -rf /var/lib/apt/lists/*

# -- Custom binutils
RUN cd /root && \
    wget https://download.01.org/intel-sgx/sgx-linux/2.15.1/as.ld.objdump.r4.tar.gz && \
    tar xzf as.ld.objdump.r4.tar.gz && \
    cp -r external/toolset/$BINUTILS_DIST/* /usr/bin/ && \
    rm -rf ./external ./as.ld.objdump.r4.tar.gz

# -- Rust
RUN cd /root && \
    curl 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' --output /root/rustup-init && \
    chmod +x /root/rustup-init && \
    echo '1' | /root/rustup-init --default-toolchain $RUST_TOOLCHAIN && \
    echo 'source /root/.cargo/env' >> /root/.bashrc && \
    /root/.cargo/bin/rustup toolchain install $RUST_UNTRUSTED_TOOLCHAIN && \
    /root/.cargo/bin/rustup component add cargo clippy rust-docs rust-src rust-std rustc rustfmt && \
    /root/.cargo/bin/rustup component add --toolchain $RUST_UNTRUSTED_TOOLCHAIN cargo clippy rust-docs rust-src rust-std rustc rustfmt && \
    /root/.cargo/bin/cargo install xargo && \
    rm /root/rustup-init
ENV PATH="/root/.cargo/bin:$PATH"

##################################
### Hardware (production) mode ###
##################################

### build-hardware: This image is used for building the app
FROM base-build AS build-hardware

COPY . ./server

ENV SGX_MODE=HW

RUN --mount=type=cache,id=HW-/root/server/target,target=/root/server/target \
    --mount=type=cache,id=HW-/root/server/tmp,target=/root/server/tmp \
    --mount=type=cache,target=/root/.xargo \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cargo/registry \
    make -C server SGX_MODE=HW all bin/tls/host_server.pem bin/tls/host_server.key && \
    cp -r ./server/bin/* /root && \
    cp ./server/policy.toml /root/policy.toml

### hardware: This image is used for running the app. It is kept minimal and optimized for size
FROM base AS hardware

ARG DCAP_PRIMITIVES_VERSION=DCAP_1.12.1
ARG DCAP_PRIMITIVES_COMMIT=4b2b8fcef71caa4da294e2171b3816e2baa3ceaf
ARG SGX_DCAP_PCCS_VERSION=1.3.101.3-bionic1

# -- Install DCAP PCCS
RUN \
    --mount=type=bind,source=docker/setup-pccs.sh,target=/root/setup-pccs.sh \

    # Install temp dependencies
    TEMP_DEPENDENCIES="git curl make build-essential gcc g++" && \
    apt-get update -y && apt-get install -y $TEMP_DEPENDENCIES && \

    # Install nodejs (needed for PCCS)
    curl -sL https://deb.nodesource.com/setup_14.x | bash - && \
    apt-get install -y nodejs cracklib-runtime && \

    # Install DCAP PCCS
    git clone https://github.com/intel/SGXDataCenterAttestationPrimitives.git -b "$DCAP_PRIMITIVES_VERSION" --depth 1 && \
    # assert commit hash
    test "$(git -C SGXDataCenterAttestationPrimitives rev-parse HEAD)" = "$DCAP_PRIMITIVES_COMMIT" && \
    make -C SGXDataCenterAttestationPrimitives/tools/PCKCertSelection/ && \
    mkdir -p SGXDataCenterAttestationPrimitives/QuoteGeneration/pccs/lib/ && \
    cp SGXDataCenterAttestationPrimitives/tools/PCKCertSelection/out/libPCKCertSelection.so SGXDataCenterAttestationPrimitives/QuoteGeneration/pccs/lib/ && \
    cp -R SGXDataCenterAttestationPrimitives/QuoteGeneration/pccs/ /opt/intel/sgx-dcap-pccs && \
    rm -rf SGXDataCenterAttestationPrimitives && \
    apt-get install -y libsgx-dcap-pccs=$SGX_DCAP_PCCS_VERSION && \

    # Install needed nodejs packages
    npm install -g esm pm2 && \
    sed -i 's/#USE_SECURE_CERT=FALSE/USE_SECURE_CERT=FALSE/g' /etc/sgx_default_qcnl.conf && \

    # Run setup script
    /root/setup-pccs.sh && \

    # Remove temp dependencies
    apt-get remove -y $TEMP_DEPENDENCIES && apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* && rm -rf /var/cache/apt/archives/*

COPY docker/hardware-start.sh /root/start.sh

# -- Copy built files from the build image
COPY --from=build-hardware \
    /root/enclave.signed.so \
    /root/blindai_app \
    /root/config.toml \
    /root/policy.toml \
    /root
COPY --from=build-hardware \
    /root/tls/ \
    /root/tls/

EXPOSE 50052
EXPOSE 50051

CMD ./start.sh

#################################################
### Hardware (production) mode - Azure DCs_v3 ###
#################################################

### hardware-dcsv3: This image is used for running the app. It is kept minimal and optimized for size
FROM base AS hardware-dcsv3

# -- Install azure_dcap_client
RUN \
    # Install temp dependencies
    TEMP_DEPENDENCIES="curl gnupg software-properties-common" && \
    apt-get update -y && apt-get install -y $TEMP_DEPENDENCIES && \

    # We need to remove the default quote providing library in order to avoid conflicts
    apt-get remove -y libsgx-dcap-default-qpl && \
    
    # Install azure_dcap_client
    curl -sSL https://packages.microsoft.com/keys/microsoft.asc | apt-key add - && \
    add-apt-repository "https://packages.microsoft.com/ubuntu/"$UBUNTU_VERSION"/prod" && \
    apt-get update && apt-get install -y az-dcap-client && \
    ln -s /usr/lib/libdcap_quoteprov.so /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so.1 && \

    # Remove temp dependencies
    apt-get remove -y $TEMP_DEPENDENCIES && apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* && rm -rf /var/cache/apt/archives/*

COPY docker/hardware-dcsv3.sh /root/start.sh

# -- Copy built files from the build image
COPY --from=build-hardware \
    /root/enclave.signed.so \
    /root/blindai_app \
    /root/config.toml \
    /root/policy.toml \
    /root
COPY --from=build-hardware \
    /root/tls/ \
    /root/tls/

# -- Flag Azure DCs_v3 mode
ENV BLINDAI_AZURE_DCSV3_PATCH=1

EXPOSE 50052
EXPOSE 50051

CMD ./start.sh

##################################
### Software (simulation) mode ###
##################################

### build-software: This image is used for building the app
FROM base-build AS build-software

COPY . ./server

ENV SGX_MODE=SW

RUN --mount=type=cache,id=SW-/root/server/target,target=/root/server/target \
    --mount=type=cache,id=SW-/root/server/tmp,target=/root/server/tmp \
    --mount=type=cache,target=/root/.xargo \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cargo/registry \
    make -C server SGX_MODE=SW all bin/tls/host_server.pem bin/tls/host_server.key && \
    cp -r ./server/bin/* /root && \
    cp ./server/policy.toml /root/policy.toml

### software: This image is used for running the app. It is kept minimal and optimized for size
FROM base AS software

# -- Copy built files from the build image
COPY --from=build-software \
    /root/enclave.signed.so \
    /root/blindai_app \
    /root/config.toml \
    /root/policy.toml \
    /root
COPY --from=build-software \
    /root/tls/ \
    /root/tls/

ENV SGX_MODE=SW

EXPOSE 50052
EXPOSE 50051

CMD ./blindai_app

### vscode-dev-env: This image is used for developers to work on blindai with vscode remote containers extension

FROM base-build AS dev-env

# Options for setup script
ARG INSTALL_ZSH="true"
ARG UPGRADE_PACKAGES="false"
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

ENV SGX_MODE=SW
ENV BLINDAI_DISABLE_TELEMETRY=1

# run VS Code dev container setup script
COPY common-dev.sh /tmp/library-scripts/
RUN bash /tmp/library-scripts/common-dev.sh "${INSTALL_ZSH}" "${USERNAME}" "${USER_UID}" "${USER_GID}" "${UPGRADE_PACKAGES}" "true" "false" \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/*  /tmp/library-scripts

USER $USERNAME

# install rustup and cargo for vscode user
RUN cd ~ && \
    curl 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' --output ~/rustup-init && \
    chmod +x ~/rustup-init && \
    echo '1' | ~/rustup-init --default-toolchain $RUST_TOOLCHAIN && \
    . ~/.cargo/env && \
    echo 'source ~/.cargo/env' >> ~/.bashrc && \
    rustup toolchain install $RUST_UNTRUSTED_TOOLCHAIN && \
    rustup component add cargo clippy rust-docs rust-src rust-std rustc rustfmt && \
    rustup component add --toolchain $RUST_UNTRUSTED_TOOLCHAIN cargo clippy rust-docs rust-src rust-std rustc rustfmt && \
    cargo install xargo && \
    rm ~/rustup-init

USER root

# install and configure python and pip
RUN \
    apt-get install -y software-properties-common && \
    add-apt-repository ppa:deadsnakes/ppa  && \
    apt-get update && \
    apt-get install -y python3.9-dev python3.9-distutils libgl1-mesa-glx && \
    curl https://bootstrap.pypa.io/get-pip.py -o get-pip.py && python3.9 get-pip.py && rm get-pip.py && \
    update-alternatives --install /usr/bin/python python /usr/bin/python3.9 1 && \
    pip install virtualenv
