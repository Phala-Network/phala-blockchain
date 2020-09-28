#!/bin/bash

# Cargo
rm /root/rustup-init && rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

# APT
rm -rf /var/lib/apt/lists/*
