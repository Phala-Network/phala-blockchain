#!/bin/bash

# This tool tries to make release easier

if [[ $# -ne 1 ]]; then
    echo "USAGE: $0 <output-dir>"
    exit
fi

outdir=$1
mkdir -p "${outdir}/chain"
mkdir -p "${outdir}/tee"

git status > "${outdir}/git.txt"
git diff > "${outdir}/changes.patch"

cp ./target/release/phala-node "${outdir}/chain"
cp ./target/release/phost "${outdir}/tee"
cp ./pruntime/bin/app "${outdir}/tee"
cp pruntime/bin/enclave.signed.so "${outdir}/tee"
cp pruntime/bin/Rocket.toml "${outdir}/tee"
