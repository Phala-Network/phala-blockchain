#!/usr/bin/env bash

echo "*** Initializing WASM build environment"
if [ -z ${RUST_TOOLCHAIN+x} ]
then
  /root/.cargo/bin/rustup target add wasm32-unknown-unknown --toolchain "${RUST_TOOLCHAIN}"
else
  /root/.cargo/bin/rustup target add wasm32-unknown-unknown --toolchain nightly-2020-09-27
fi
