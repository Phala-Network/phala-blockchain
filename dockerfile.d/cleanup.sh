#!/bin/bash

# Cargo
rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

# APT
rm -rf /var/lib/apt/lists/*
rm -rf /var/cache/apt/archives/*
