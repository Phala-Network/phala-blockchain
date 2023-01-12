#!/bin/bash

./target/release/phala-node \
    --dev --tmp \
    --rpc-methods=Unsafe \
    --rpc-cors=all \
    --ws-port 19944 \
    --ws-external \
    --rpc-port 19933 \
    --rpc-external \
