#!/bin/bash

git submodule update --init --recursive
cargo build --release
