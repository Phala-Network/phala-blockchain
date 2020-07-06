#!/bin/bash

rm -rf /tmp/rmbak
mkdir /tmp/rmbak
cp ./phala-blockchain/target/release/phala-node /tmp/rmbak
cp ./phala-blockchain/target/release/phost /tmp/rmbak
rm -rf ./phala-blockchain/target/release
mv /tmp/rmbak ./phala-blockchain/target/release

# pRuntime
rm -rf ./phala-blockchain/pruntime/enclave/target
rm -rf ./phala-blockchain/pruntime/app/target

# Cargo
rm /root/rustup-init && rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

# APT
rm -rf /var/lib/apt/lists/*
