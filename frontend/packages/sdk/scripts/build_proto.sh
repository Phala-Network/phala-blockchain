#!/bin/bash
downloadProto() {
  echo "Downloading $1.proto…"
  curl -o "proto/$1.proto" "https://raw.githubusercontent.com/Phala-Network/prpc-protos/master/$1.proto"
}

rm -rf proto
mkdir proto
downloadProto "prpc"
downloadProto "pruntime_rpc"

echo "Generating static code from proto files"
rm -rf src/pruntime/proto/*
node_modules/.bin/pbjs -w commonjs -t static-module --filter pbconfig.json -o src/pruntime/proto/index.js proto/*.proto
node_modules/.bin/pbts -o src/pruntime/proto/index.d.ts src/pruntime/proto/index.js

echo "Done"
