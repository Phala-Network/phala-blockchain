#!/usr/bin/env bash

set -e

# You need use `sudo` to run the script
# You need edit ENV before you run the script
# It's won't run parallel, so it's slow if you have > 100 workers

PRB_MONITOR_HOST="http://IP_TO_PRB_MONITOR:3000"
PRB_LFM_PEER_ID=""
PRB_MONITOR_LIST_WORKER="$PRB_MONITOR_HOST/ptp/proxy/$PRB_LFM_PEER_ID/ListWorker"
PARACHAIN_ENDPOINT="ws://IP_TO_KHALA_NODE:9945"
RELAYCHAIN_ENDPOINT="ws://IP_TO_KHALA_NODE:9944"

echo "Preparing..."
apt-get install -y jq
docker pull phalanetwork/phala-pherry:latest
for endpoint in $(curl -s "$PRB_MONITOR_LIST_WORKER" | jq -c -r '.data.workers | .[] | .endpoint'); do
  echo "Running fix on $endpoint"
  docker run -it --rm phalanetwork/phala-pherry:latest /root/pherry --no-msg-submit --no-register --no-bind --parachain --substrate-ws-endpoint $PARACHAIN_ENDPOINT --collator-ws-endpoint $RELAYCHAIN_ENDPOINT --pruntime-endpoint $endpoint --fetch-blocks 4 --to-block 2702765
done
