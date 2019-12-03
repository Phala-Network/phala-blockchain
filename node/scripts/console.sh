#!/bin/bash

BOB_RPC_PORT=30334
NODE_NAME=experimental-node

case $1 in
purge)
  rm -rf /tmp/*alice*
  rm -rf /tmp/*bob*
  rm -rf /tmp/*dev*
;;
dev)
  shift
  "./target/release/${NODE_NAME}" \
      --base-path /tmp/dev \
      --chain=dev
;;
start)
  shift
  case $1 in
  alice)
    shift
    "./target/release/${NODE_NAME}" \
        --base-path /tmp/alice \
        --chain=local \
        --alice \
        --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
        --validator "$@"
  ;;
  bob)
    shift
    "./target/release/${NODE_NAME}" \
        --base-path /tmp/bob \
        --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
        --chain=local \
        --bob \
        --port "$BOB_RPC_PORT" \
        --validator "$@"
  ;;
  *)
    echo "Can't start node '$1'"
    exit -1
  esac
;;
esac