#!/usr/bin/env bash
# This is script is to fix the PRB issue on 12 Nov 2022
# https://discord.com/channels/697726436211163147/891912723447832617/1041040854153957457

# Exit on error
set -e

# EDIT THESE VARIABLES
# Do not use loopback address (ie 127.0.0.1 / localhost)

PRB_IP=""
PRB_LFM_PEER_ID=""
KHALA_NODE_IP=""

############## DO NOT EDIT BELOW THIS LINE

if [ -z "$PRB_IP" ]
  then echo "Please edit variables in script"
  exit 1
fi

if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit 1
fi

# Template variables
PRB_MONITOR_HOST="http://$PRB_IP:3000"
PRB_MONITOR_LIST_WORKER="$PRB_MONITOR_HOST/ptp/proxy/$PRB_LFM_PEER_ID/GetWorkerStatus"
PARACHAIN_ENDPOINT="ws://KHALA_NODE_IP:9945"
RELAYCHAIN_ENDPOINT="ws://KHALA_NODE_IP:9944"

echo "Getting dependencies..."
apt-get -qq install -q -y jq curl
docker pull -q phalanetwork/phala-pherry:latest

echo "Status of nodes"
NODE_COUNT=$(curl -s "$PRB_MONITOR_LIST_WORKER" | jq -c -r '.data.workerStates | length')
echo "Found ${NODE_COUNT} nodes"

NODE_COUNT_REQ_FIX=$(curl -s "$PRB_MONITOR_LIST_WORKER" | jq -c -r '.data.workerStates | .[] | select(.paraHeaderSynchedTo==2702763) | select(.paraHeaderSynchedTo==2702764) | length')
if [ -z "$NODE_COUNT_REQ_FIX" ]
  then
  echo "Found no node stuck nodes, exiting"
  exit 0
fi

echo "Found ${NODE_COUNT_REQ_FIX} that appear stuck"

for endpoint in $(curl -s "$PRB_MONITOR_LIST_WORKER" | jq -c -r '.data.workerStates | .[] | select(.paraHeaderSynchedTo==2702763) | select(.paraHeaderSynchedTo==2702764) | .worker.endpoint'); do
  echo "Running fix on $endpoint"
  docker run -it --rm phalanetwork/phala-pherry:latest \
    /root/pherry \
    --no-msg-submit \
    --no-register \
    --no-bind \
    --parachain \
    --substrate-ws-endpoint $PARACHAIN_ENDPOINT \
    --collator-ws-endpoint $RELAYCHAIN_ENDPOINT \
    --pruntime-endpoint $endpoint \
    --fetch-blocks 4 \
    --to-block 2702765 \
    >/dev/null
done
