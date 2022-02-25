#######################################
### Base stage: common dependencies ###
#######################################
FROM ubuntu:18.04 AS base

ENV CODENAME=bionic
ENV VERSION=2.15.101.1-bionic1
ENV DCAP_VERSION=1.12.101.1-bionic1
ENV SGX_LINUX_X64_SDK=sgx_linux_x64_sdk_2.15.101.1.bin
ENV SGX_LINUX_X64_SDK_URL="https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/"$SGX_LINUX_X64_SDK
ENV SGX_DCAP_PCCS_VERSION=1.3.101.3-bionic1
ENV GCC_VERSION=8.4.0-1ubuntu1~18.04
ENV RUST_TOOLCHAIN=nightly-2021-11-01
ENV RUST_UNTRUSTED_TOOLCHAIN=nightly-2021-11-01
ENV DCAP_PRIMITIVES_VERSION=DCAP_1.12.1
ENV DCAP_PRIMITIVES_COMMIT=4b2b8fcef71caa4da294e2171b3816e2baa3ceaf

# -- setup
ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /root

# dependencies that will be removed after build
ENV BUILD_ONLY_DEPS \
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
    software-properties-common

RUN apt update && apt install -y \
    $BUILD_ONLY_DEPS \
    # dependencies that should be kept after build
    cracklib-runtime \
    gcc-8=$GCC_VERSION

# -- custom binutils
RUN cd /root && \
    wget https://download.01.org/intel-sgx/sgx-linux/2.15.1/as.ld.objdump.r4.tar.gz && \
    tar xzf as.ld.objdump.r4.tar.gz && \
    cp -r external/toolset/$BINUTILS_DIST/* /usr/bin/ && \
    rm -rf ./external ./as.ld.objdump.r4.tar.gz

# -- sgx sdk
RUN wget $SGX_LINUX_X64_SDK_URL               && \
    chmod u+x $SGX_LINUX_X64_SDK              && \
    echo -e 'no\n/opt' | ./$SGX_LINUX_X64_SDK && \
    rm $SGX_LINUX_X64_SDK                     && \
    echo 'source /opt/sgxsdk/environment' >> /etc/environment
ENV LD_LIBRARY_PATH=/opt/sgxsdk/sdk_libs:/usr/lib:/usr/local/lib:/opt/intel/sgx-aesm-service/aesm/

# -- libsgx
RUN curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add - && \
    add-apt-repository "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $CODENAME main" && \
    apt-get install -y \
        sgx-aesm-service=$VERSION \
        libsgx-ae-epid=$VERSION \
        libsgx-ae-le=$VERSION \
        libsgx-ae-pce=$VERSION \
        libsgx-aesm-ecdsa-plugin=$VERSION \
        libsgx-aesm-epid-plugin=$VERSION \
        libsgx-aesm-launch-plugin=$VERSION \
        libsgx-aesm-pce-plugin=$VERSION \
        libsgx-aesm-quote-ex-plugin=$VERSION \
        libsgx-enclave-common=$VERSION \
        libsgx-epid=$VERSION \
        libsgx-launch=$VERSION \
        libsgx-quote-ex=$VERSION \
        libsgx-uae-service=$VERSION \
        libsgx-urts=$VERSION \
        libsgx-ae-qe3=$DCAP_VERSION \
        libsgx-ae-pce=$VERSION \
        libsgx-pce-logic=$DCAP_VERSION \
        libsgx-qe3-logic=$DCAP_VERSION \
        libsgx-ra-network=$DCAP_VERSION \
        libsgx-ra-uefi=$DCAP_VERSION \
        libsgx-dcap-ql=$DCAP_VERSION \
        libsgx-dcap-quote-verify=$DCAP_VERSION \
        libsgx-dcap-default-qpl=$DCAP_VERSION && \
    mkdir -p /var/run/aesmd && \
    ln -s /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so.1 /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so

# -- rust
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
FROM base AS hardware

# -- nodejs (needed for pccs)
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g esm pm2

# -- dcap pccs
RUN git clone https://github.com/intel/SGXDataCenterAttestationPrimitives.git -b "$DCAP_PRIMITIVES_VERSION" --depth 1 && \
    # assert commit hash
    test "$(git -C SGXDataCenterAttestationPrimitives rev-parse HEAD)" = "$DCAP_PRIMITIVES_COMMIT" && \
    make -C SGXDataCenterAttestationPrimitives/tools/PCKCertSelection/ && \
    mkdir -p SGXDataCenterAttestationPrimitives/QuoteGeneration/pccs/lib/ && \
    cp SGXDataCenterAttestationPrimitives/tools/PCKCertSelection/out/libPCKCertSelection.so SGXDataCenterAttestationPrimitives/QuoteGeneration/pccs/lib/ && \
    cp -R SGXDataCenterAttestationPrimitives/QuoteGeneration/pccs/ /opt/intel/sgx-dcap-pccs && \
    rm -rf SGXDataCenterAttestationPrimitives && \
    apt-get install -y libsgx-dcap-pccs=$SGX_DCAP_PCCS_VERSION

ADD docker/setup-pccs.sh /root
RUN /root/setup-pccs.sh && \
    rm setup-pccs.sh && \
    sed -i 's/#USE_SECURE_CERT=FALSE/USE_SECURE_CERT=FALSE/g' /etc/sgx_default_qcnl.conf

# -- build
COPY . ./server

RUN --mount=type=cache,target=/root/server/target \
    --mount=type=cache,target=/root/server/inference-server/scheduler/untrusted/target \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cargo/registry \
    make -C server SGX_MODE=HW all bin/tls/host_server.pem bin/tls/host_server.key && \
    cp -r ./server/bin/* /root && \
    cp ./server/policy.toml /root/policy.toml && \
    (rm -rf ./server || true)

# -- cleanup
RUN rustup self uninstall -y && \
    rm -rf .npm .xargo .wget-hsts && \
    apt-get remove -y $BUILD_ONLY_DEPS && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/cache/apt/archives/*

ADD docker/hardware-start.sh /root/start.sh

EXPOSE 50052
EXPOSE 50051

CMD ["/root/start.sh"]

##################################
### Software (simulation) mode ###
##################################
FROM base AS software

# -- build
COPY . ./server

# -- flag software mode
ENV SGX_MODE=SW

RUN --mount=type=cache,target=/root/server/target \
    --mount=type=cache,target=/root/server/inference-server/scheduler/untrusted/target \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cargo/registry \
    make -C server SGX_MODE=SW all bin/tls/host_server.pem bin/tls/host_server.key && \
    cp -r ./server/bin/* /root && \
    cp ./server/policy.toml /root/policy.toml && \
    (rm -rf ./server || true)

# -- cleanup
RUN rustup self uninstall -y && \
    rm -rf .npm .xargo .wget-hsts && \
    apt-get remove -y $BUILD_ONLY_DEPS && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/cache/apt/archives/*

ADD docker/software-start.sh /root/start.sh

EXPOSE 50052
EXPOSE 50051

CMD ["/root/start.sh"]

