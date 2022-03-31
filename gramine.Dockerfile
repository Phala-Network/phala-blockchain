FROM ubuntu:20.04

ARG DEBIAN_FRONTEND='noninteractive'

ADD dockerfile.d/01_apt_gramine.sh /root
RUN bash /root/01_apt_gramine.sh

ADD dockerfile.d/02_pip.sh /root
RUN bash /root/02_pip.sh

ADD ./dockerfile.d/03_sdk.sh /root
RUN bash /root/03_sdk.sh

ARG CODENAME='focal'
ADD ./dockerfile.d/04_psw.sh /root
RUN bash /root/04_psw.sh

ARG RUST_TOOLCHAIN='nightly-2022-02-23'
ADD ./dockerfile.d/05_rust.sh /root
RUN bash /root/05_rust.sh

WORKDIR /root

# ====== build pruntime ======

RUN mkdir phala-blockchain
ADD . phala-blockchain

RUN mkdir prebuilt

RUN cd phala-blockchain/standalone/pruntime/pruntime/gramine-build && \
    PATH=$PATH:/root/.cargo/bin make dist PREFIX=/root/prebuilt && \
    make clean && \
    rm -rf /root/.cargo/registry && \
    rm -rf /root/.cargo/git

# ====== clean up ======

RUN rm -rf phala-blockchain
ADD dockerfile.d/cleanup.sh .
RUN bash cleanup.sh

# ====== start phala ======

ADD dockerfile.d/startup-gramine.sh ./startup.sh
CMD bash ./startup.sh

EXPOSE 8000
