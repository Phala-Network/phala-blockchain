#!/bin/bash

# Modified based on:
# https://github.com/paritytech/polkadot/blob/master/doc/testing.md#coverage

rustup component add llvm-tools-preview
cargo install grcov miniserve

export CARGO_INCREMENTAL=0
# wasm is not happy with the instrumentation
export SKIP_BUILD_WASM=true
export BUILD_DUMMY_WASM_BINARY=true
# the actully collected coverage data (absolute path is important)
export LLVM_PROFILE_FILE="$PWD/tmp/coveragedata/llvmcoveragedata-%p-%m.profraw"
# build wasm without instrumentation
export WASM_TARGET_DIRECTORY=/tmpwasm
# required rust flags
export RUSTFLAGS="-Zinstrument-coverage"

# cleanup old coverage data
rm -rf ./tmp/coveragedata
# run tests to get coverage data
cargo +nightly test -p phala-pallets --tests

# cleanup old rendered report
rm -rf ./coverage/
# create the *html* report out of all the test binaries
# mostly useful for local inspection
grcov ./tmp/coveragedata --binary-path ./target/debug -s . -t html --branch --ignore-not-existing -o ./coverage/ \
    --keep-only 'pallets/**'
miniserve -r ./coverage

# # create a *codecov* compatible report
# grcov . --binary-path ./target/debug/ -s . -t lcov --branch --ignore-not-existing --ignore "/*" -o lcov.info
