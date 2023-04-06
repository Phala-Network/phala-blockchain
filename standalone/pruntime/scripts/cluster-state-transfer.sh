#!/bin/sh
set -e

WORKER0=http://localhost:8000
WORKER1=http://localhost:8001
WORKER1_DATA_DIR=data/storage_files/

echo "Starting"

REQUEST=$(curl -s $WORKER1/prpc/PhactoryAPI.GenerateClusterStateRequest)
echo "REQUEST: $REQUEST"

STATE_INFO=$(curl -s -d $REQUEST $WORKER0/prpc/PhactoryAPI.SaveClusterState?json)
echo "STATE_INFO: $STATE_INFO"

FILENAME=$(echo $STATE_INFO | jq -r .filename)
echo "FILENAME: $FILENAME"

URL=$WORKER0/download/$FILENAME
DST=$WORKER1_DATA_DIR/$FILENAME
echo "Downloading $URL to $DST"
curl -s $URL -o $DST

echo "Loading state"
curl -s -d $STATE_INFO $WORKER1/prpc/PhactoryAPI.LoadClusterState?json
echo Done

