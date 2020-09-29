FROM ubuntu:18.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt upgrade -y && apt install -y autoconf automake bison build-essential cmake curl dpkg-dev expect flex gcc-8 gdb git git-core gnupg kmod libboost-system-dev libboost-thread-dev libcurl4-openssl-dev libiptcdata0-dev libjsoncpp-dev liblog4cpp5-dev libprotobuf-c0-dev libprotobuf-dev libssl-dev libtool libxml2-dev ocaml ocamlbuild pkg-config protobuf-compiler python sudo systemd-sysv texinfo uuid-dev vim wget software-properties-common lsb-release apt-utils binutils-dev nginx && apt autoremove -y

ADD ./dockerfile.d/01_llvm_10.sh /root
RUN bash /root/01_llvm_10.sh

ENV BINUTILS_PREFIX=/usr

ADD ./dockerfile.d/03_sdk.sh /root
RUN bash /root/03_sdk.sh

# Sixth, PSW

ENV CODENAME        bionic
ENV VERSION         2.11.100.2-bionic1

ADD ./dockerfile.d/04_psw.sh /root
RUN bash /root/04_psw.sh

# Seventh, Rust

ENV rust_toolchain  nightly
ADD ./dockerfile.d/05_rust.sh /root
RUN bash /root/05_rust.sh
ADD ./dockerfile.d/06_wasm.sh /root
RUN bash /root/06_wasm.sh

ENV DEBIAN_FRONTEND=''
ENV CODENAME=''
ENV VERSION=''

WORKDIR /root

# ====== download Phala ======

# RUN git clone --recursive https://github.com/Phala-Network/phala-blockchain.git

# ====== download Phala ======
RUN mkdir phala-blockchain
ADD . phala-blockchain

# ====== build phala ======

ARG SGX_SPID
ARG SGX_IAS_API_KEY

RUN cd phala-blockchain && PATH="$PATH:$HOME/.cargo/bin" cargo build --release
RUN cd phala-blockchain/pruntime && PATH="$PATH:$HOME/.cargo/bin" SGX_SDK="/opt/sgxsdk" SGX_MODE=HW make

# ====== copy compiled ======

RUN mkdir prebuilt
RUN cp phala-blockchain/target/release/phost prebuilt
RUN cp phala-blockchain/target/release/phala-node prebuilt
RUN cp phala-blockchain/pruntime/bin/app prebuilt
RUN cp phala-blockchain/pruntime/bin/enclave.signed.so prebuilt
RUN cp phala-blockchain/pruntime/bin/Rocket.toml prebuilt

# ====== clean up ======

ADD dockerfile.d/cleanup.sh .
RUN bash cleanup.sh
RUN rm -rf phala-blockchain

# ====== start phala ======
ADD dockerfile.d/console.sh ./console.sh
ADD dockerfile.d/startup.hw.sh ./startup.sh
ADD dockerfile.d/api.nginx.conf /etc/nginx/sites-enabled/default
CMD bash ./startup.sh

EXPOSE 8080
EXPOSE 9933
EXPOSE 9944
EXPOSE 30333
