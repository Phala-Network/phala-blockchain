#!/usr/bin/env bash

echo "*** Initializing WASM build environment"
/root/.cargo/bin/rustup target add wasm32-unknown-unknown --toolchain "${RUST_TOOLCHAIN}" && \
/root/.cargo/bin/rustup target add wasm32-unknown-unknown --toolchain nightly
