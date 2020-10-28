#!/bin/bash

read -s -p "Enter mnemonic: " SECRET
echo
read -p "Enter the role number: " n
echo
read -s -p "Enter port: " PORT
echo

ENDPOINT="http://localhost:${PORT}"
NETWORK="phat3"

function get_pubkey {
  tmp=tmp.key
  ./phala-node "$@" > "$tmp"
  awk '/Public key \(hex\): +(\w+?)/{print $4}' "$tmp"
  rm tmp.key
}

function insert_key {
  type="$1"
  suri="$2"

  if [ "$type" = 'gran' ]; then
    pubkey=$(get_pubkey key inspect-key -n phala --scheme Ed25519 "$suri")
  else
    pubkey=$(get_pubkey key inspect-key -n phala --scheme Sr25519 "$suri")
  fi

  curl "$ENDPOINT" -H "Content-Type:application/json;charset=utf-8" -d \
    """{
      \"jsonrpc\":\"2.0\",
      \"id\":1,
      \"method\":\"author_insertKey\",
      \"params\": [
        \"${type}\",
        \"${suri}\",
        \"${pubkey}\"
      ]
    }"""
}

insert_key babe "${SECRET}//${NETWORK}//session//${n}"
insert_key gran "${SECRET}//${NETWORK}//session//${n}"
