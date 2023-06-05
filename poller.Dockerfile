FROM ubuntu:20.04 as builder

ENV DEBIAN_FRONTEND=noninteractive
ENV PATH="${PATH}:/root/.cargo/bin"

ADD dockerfile.d/01_apt.sh /root
RUN bash /root/01_apt.sh

ADD dockerfile.d/05_rust.sh /root
RUN bash /root/05_rust.sh

WORKDIR /root
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release -p phat-poller
RUN mkdir -p build-out/
RUN cp target/x86_64-unknown-linux-musl/release/phat-poller build-out/
RUN strip build-out/phat-poller

FROM scratch
WORKDIR /app
COPY --from=builder /root/build-out/phat-poller .
USER 1000:1000
ENTRYPOINT ["./phat-poller"]
