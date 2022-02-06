FROM ubuntu:18.04

ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /root
ENV CODENAME bionic
ENV VERSION 2.15.101.1-bionic1
ENV DCAP_VERSION 1.12.101.1-bionic1
ENV SGX_LINUX_X64_SDK sgx_linux_x64_sdk_2.15.101.1.bin
ENV SGX_LINUX_X64_SDK_URL "https://download.01.org/intel-sgx/sgx-linux/2.15.1/distro/ubuntu18.04-server/"$SGX_LINUX_X64_SDK

RUN apt-get update && apt-get install -q -y \
    autoconf automake bison build-essential cmake curl cracklib-runtime \
    dpkg-dev expect flex gcc-8 git git-core gnupg kmod sqlite3\
    libboost-system-dev libboost-thread-dev libcurl4-openssl-dev \
    libiptcdata0-dev libjsoncpp-dev liblog4cpp5-dev libprotobuf-c0-dev \
    libprotobuf-dev libssl-dev libtool libxml2-dev ocaml ocamlbuild \
    pkg-config protobuf-compiler python texinfo uuid-dev vim wget dkms gnupg2 \
    apt-transport-https software-properties-common systemd net-tools mlocate && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/cache/apt/archives/* && \
    apt-get -y -q upgrade

RUN wget $SGX_LINUX_X64_SDK_URL               && \
    chmod u+x $SGX_LINUX_X64_SDK              && \
    echo -e 'no\n/opt' | ./$SGX_LINUX_X64_SDK && \
    rm $SGX_LINUX_X64_SDK                     && \
    echo 'source /opt/sgxsdk/environment' >> /etc/environment
ENV LD_LIBRARY_PATH=/opt/sgxsdk/sdk_libs:/usr/lib:/usr/local/lib:/opt/intel/sgx-aesm-service/aesm/

RUN curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add -
RUN add-apt-repository "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $CODENAME main" && \
    apt-get update -y && \
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
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/cache/apt/archives/* 

EXPOSE 50052
EXPOSE 50051

RUN echo 'SGX_MODE=SW' >> /etc/environment

COPY bin/ /root/
ADD docker/software/start.sh /root
ENTRYPOINT ["/root/start.sh"]