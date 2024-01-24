#!/bin/sh
set -e

cargo llvm-cov \
    --html \
    -p pink \
    -p pink-extension-macro \
    -p pink-extension-runtime \
    -p pink-runtime \
    -p pink-runtime-macro \
    -p pink-capi \
    -p pink-types \
    -p phala-git-revision \
    -p phala-serde-more \
    -p phala-crypto \
    -p phala-mq \
    -p phala-sanitized-logger \
    -p phala-types \
    -p phala-wasm-checker \
    -p reqwest-env-proxy \
