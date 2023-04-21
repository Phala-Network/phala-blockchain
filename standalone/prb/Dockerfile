FROM rust:bookworm AS builder
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update
RUN apt-get install -y --no-install-recommends \
  build-essential \
  protobuf-compiler \
  llvm \
  clang \
  libclang-dev \
  libssl-dev \
  git-core \
  pkg-config \
  && apt-get clean \
  && rm -rf /tmp/* /var/tmp/*

WORKDIR /builder
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release --bin headers-cache --bin prb-wm --bin prb-config

FROM debian:bookworm-slim
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update
RUN apt-get install -y --no-install-recommends \
  libssl-dev \
  ca-certificates \
  && apt-get clean \
  && rm -rf /tmp/* /var/tmp/*

WORKDIR /app
COPY --from=builder /builder/target/release/headers-cache .
COPY --from=builder /builder/target/release/prb-wm .
COPY --from=builder /builder/target/release/prb-config .

CMD ./prb-wm
