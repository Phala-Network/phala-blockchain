#!/bin/bash

function buildProto() {
  npx pbjs -w commonjs -t static-module "../crates/phactory/api/proto/$1.proto" -o "src/proto/$1.js"
  npx pbts -o "src/proto/$1.d.ts" "src/proto/$1.js"
}

buildProto pruntime_rpc
buildProto prpc
