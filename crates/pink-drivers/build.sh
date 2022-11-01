#!/bin/sh

set -e

mkdir -p dist

build() {
    NAME=$1
    if [ x$2 = x1 ]; then
        (cd $NAME && make)
        cp $NAME/sideprog.wasm dist/$NAME.sidevm.wasm
    else
        (cd $NAME && cargo contract build --release)
    fi
    cp $NAME/target/ink/$NAME.contract dist/
}

build system
build tokenomic
build sidevm_deployer
build log_server 1
