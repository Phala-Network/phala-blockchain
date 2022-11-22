#!/usr/bin/env bash
# This is script is to fix the PRB issue on 12 Nov 2022
# https://discord.com/channels/697726436211163147/891912723447832617/1041040854153957457

# Exit on error
set -e

# EDIT THESE VARIABLES
# Do not use loopback address (ie 127.0.0.1 / localhost)

PRB_MONITOR_HOST="" # "http://127.0.0.1:3000"
PRB_LFM_PEER_ID=""
KHALA_PARACHAIN_ENDPOINT="" # "ws://KHALA_NODE_IP:9945"
KHALA_RELAYCHAIN_ENDPOINT="" # "ws://KHALA_NODE_IP:9944"

############## DO NOT EDIT BELOW THIS LINE

BAD_BLOCK_FROM=2702763
BAD_BLOCK_UNTIL=2702764
FETCH_BLOCKS=4
TO_BLOCK=2702765

if [ -z "$PRB_MONITOR_HOST" ]
  then echo "Please edit PRB_MONITOR_HOST variable in the script"
  exit 1
fi

if [ -z "$PRB_LFM_PEER_ID" ]
  then echo "Please edit PRB_LFM_PEER_ID variable in the script"
  exit 1
fi

if [ -z "$KHALA_PARACHAIN_ENDPOINT" ]
  then echo "Please edit KHALA_PARACHAIN_ENDPOINT variable in the script"
  exit 1
fi

if [ -z "$KHALA_RELAYCHAIN_ENDPOINT" ]
  then echo "Please edit KHALA_RELAYCHAIN_ENDPOINT variables in the script"
  exit 1
fi

if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit 1
fi

# Template variables
PRB_MONITOR_LIST_WORKER="$PRB_MONITOR_HOST/ptp/proxy/$PRB_LFM_PEER_ID/GetWorkerStatus"

echo "Getting dependencies..."
apt-get -qq install -q -y jq curl
docker pull -q phalanetwork/phala-pherry:prb-fix

echo "Status of nodes"
NODE_COUNT=$(curl -s "$PRB_MONITOR_LIST_WORKER" | jq -c -r '.data.workerStates | length')
echo "Found ${NODE_COUNT} nodes"

NODE_COUNT_REQ_FIX=$(curl -s "$PRB_MONITOR_LIST_WORKER" | jq -c -r ".data.workerStates | .[] | select((.paraHeaderSynchedTo >= $BAD_BLOCK_FROM) and (.paraHeaderSynchedTo <= $BAD_BLOCK_UNTIL))" | wc -l)
if [ -z "$NODE_COUNT_REQ_FIX" ]
  then
  echo "Found no node stuck nodes, exiting"
  exit 0
fi

echo "Found ${NODE_COUNT_REQ_FIX} that appear stuck"

for endpoint in $(curl -s "$PRB_MONITOR_LIST_WORKER" | jq -c -r ".data.workerStates | .[] | select((.paraHeaderSynchedTo >= $BAD_BLOCK_FROM) and (.paraHeaderSynchedTo <= $BAD_BLOCK_UNTIL)) | .worker.endpoint"); do
  echo "Running fix on $endpoint"
  docker run -it --rm phalanetwork/phala-pherry:prb-fix \
    /root/pherry \
    --no-msg-submit \
    --no-register \
    --no-bind \
    --parachain \
    --substrate-ws-endpoint "$KHALA_PARACHAIN_ENDPOINT" \
    --collator-ws-endpoint "$KHALA_RELAYCHAIN_ENDPOINT" \
    --pruntime-endpoint "$endpoint" \
    --fetch-blocks $FETCH_BLOCKS \
    --to-block $TO_BLOCK \
    --no-storage-sync \
    >/dev/null
done
