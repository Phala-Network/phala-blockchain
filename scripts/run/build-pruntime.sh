#!/bin/bash

cd standalone/pruntime
cargo build --release
cp ./target/release/pruntime ./bin/
