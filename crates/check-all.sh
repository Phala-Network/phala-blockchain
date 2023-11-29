#!/bin/sh

# Make sure all crates can be compiled alone

for m in $(find . -name Cargo.toml); do
    D=$(dirname $m)
    echo "entering $D"
    (cd $D && cargo check)
done
