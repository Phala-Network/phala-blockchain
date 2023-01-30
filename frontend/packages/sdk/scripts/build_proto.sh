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
rm -rf src/proto/*
npx pbjs -w es6 -t static-module -o src/proto/index.js proto/*.proto
npx pbts -o src/proto/index.d.ts src/proto/index.js

echo "Done"
