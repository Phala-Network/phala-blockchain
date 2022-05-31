#!/bin/sh
set -e

cargo build --manifest-path sideprog/Cargo.toml --release --target wasm32-unknown-unknown
cp sideprog/target/wasm32-unknown-unknown/release/sideprog.wasm .
wasm-strip sideprog.wasm
cargo contract build --release
