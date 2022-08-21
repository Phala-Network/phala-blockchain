#!/bin/bash

export ROCKET_PORT=18000
export RUST_LOG=info,pruntime=trace,pink=trace,contracts=trace
cd standalone/pruntime/bin
mkdir data
./pruntime -c 0 --address 0.0.0.0 --port 18000 |& tee pruntime.log
