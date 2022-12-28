#!/bin/sh

URI=http://localhost:9933

rpc() {
    curl $URI -H "Content-Type:application/json;charset=utf-8" -d '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"'"$1"'",
        "params": '"$2"'
    }' 2>/dev/null
}

get_hash(){
    rpc chain_getBlockHash "[$1]" | jq -r .result
}

get_changes() {
    rpc pha_getStorageChanges "[\"$1\", \"$1\"]"
}

get_changes_rng() {
    rpc pha_getStorageChanges "[\"$1\", \"$2\"]"
}

get_pairs() {
    echo rpc state_getPairs "[\"0x\",\"$1\"]"
    rpc state_getPairs "[\"0x\",\"$1\"]" > pairs.json
}

HASH=`get_hash`
get_pairs $HASH
