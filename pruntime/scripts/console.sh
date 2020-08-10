#!/bin/bash

# Require jq installed:
#  brew install jq
#  apt install jq

ENDPOINT='http://127.0.0.1:8000'

function req {
  rand_number=$((RANDOM))
  data="${2}"
  if [ -z "${data}" ]; then
    data='{}'
  fi
  curl -sgX POST -H "Content-Type: application/json" "${ENDPOINT}/${1}" \
       -d "{\"input\":${data}, \"nonce\": {\"id\": ${rand_number}}}" \
       | tee /tmp/req_result.json | jq '.payload|fromjson'
  echo
}

function query {
	contract_id="${1}"
	payload="${2}"
	rand_number=$((RANDOM))

	query_body_json=$(echo "{\"contract_id\":${contract_id},\"nonce\":${rand_number},\"request\":${payload}}" | jq '. | tojson' -c)
	query_payload_json=$(echo "{\"Plain\":${query_body_json}}" | jq '. | tojson')
	query_data="{\"query_payload\":${query_payload_json}}"
	req 'query' "${query_data}"
}

function req_set {
  id=$1
  path=$2
  file_b64=$(cat "${path}"| base64 -w 0)
  req set "{\"path\": \"${id}\", \"data\": \"${file_b64}\"}"
}

function set_dataset {
  req_set "/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG" ./scripts/dataset.csv
}

function set_query {
  req_set "/ipfs/QmY6yj1GsermExDXoosVE3aSPxdMNYr6aKuw3nA8LoWPRS" ./scripts/query.csv
}

function get_result {
  path="/order/0"
  req get "{\"path\": \"${path}\"}"
  cat /tmp/req_result.json | jq '.payload|fromjson|.value' -r | base64 -d
}

function init {
  req init_runtime "{\"skip_ra\": true, \"bridge_genesis_info_b64\": \"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAh9rd6Uku4dTja+JQVMLsOZ5GtS4nU0cdpuvgchlapeMDFwoudZe3t+PYTAU5HROaYrFX54eG2MCC8p3PTBETFAAIiNw0F9UFjsS0UD4MEuoaCom+IA/piSJCPUM0AU+msO4BAAAAAAAAANF8LXgj6/Jg/ROPLX4n0RTAFF2Wi1/1AGEl8kFPra5pAQAAAAAAAAAMnQFkcmFuZHBhX2F1dGhvcml0aWVzSQEBCIjcNBfVBY7EtFA+DBLqGgqJviAP6YkiQj1DNAFPprDuAQAAAAAAAADRfC14I+vyYP0Tjy1+J9EUwBRdlotf9QBhJfJBT62uaQEAAAAAAAAAbQGCpqgAgEqiIYyEDJYvjKZbaiGZXNhVYVwnf5vRepPUObtd8VNyUFx4dHJpbnNpY19pbmRleBAAAAAAgHNMr3LFImHJmuZEj2SsazMW8g7n4rDnzJiLF9Ww8Xx3oQKALhCA33I0FGiDoLZ6HBWl1uCIt+sLgUbPlfMJqUk/gukhEt6AviHkl5KFGndUmA+ClBT2kPSvmBOvZTWowWjNYfynHU6AOFjSvKwU3s/vvRg7QOrJeehLgo9nGfN91yHXkHcWLkuAUJegqkIzp2A6LPkZouRRsKgiY4Wu92V8JXrn3aSXrw2AXDYZ0c8CICTMvasQ+rEpErmfEmg+BzH19s/zJX4LP8Y=\"}"
}

case $1 in
run)
  make && cd bin && ./app
;;
init)
  init
;;
set-dataset)
  set_dataset
;;
set-query)
  set_query
;;
set-all)
  set_dataset
  set_query
;;
get-result)
  get_result
;;
query)
  shift 1
  query "$@"
;;
*)
  req "$@"
;;
esac
