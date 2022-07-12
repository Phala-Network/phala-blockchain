#!/bin/bash

./target/release/pherry \
    --substrate-ws-endpoint ws://127.0.0.1:19944 \
    --pruntime-endpoint http://127.0.0.1:18000 \
    --no-wait \
    --use-dev-key \
    --mnemonic=//Ferdie
