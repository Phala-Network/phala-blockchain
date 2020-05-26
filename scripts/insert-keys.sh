#!/bin/bash

read -s -p "Enter mnemonic: " SECRET
echo
read -p "Enter the role number: " n
echo
read -s -p "Enter port: " PORT
echo

ENDPOINT="http://localhost:${PORT}"

function get_pubkey {
  tmp=tmp.key
  subkey "$@" > "$tmp"
  awk '/Public key \(hex\): +(\w+?)/{print $4}' "$tmp"
  rm tmp.key
}

function insert_key {
  type="$1"
  suri="$2"
  if [ "$type" = 'gran' ]; then
    key_flag='-e'
  else
    key_flag='-s'
  fi

  pubkey=$(get_pubkey "$key_flag" inspect "$suri")
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

insert_key babe "${SECRET}//phat//session//${n}"
insert_key gran "${SECRET}//phat//session//${n}"
