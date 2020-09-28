#!/usr/bin/env bash

# For hardware mode PRuntime
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/opt/intel/sgx-aesm-service/aesm"
cd /opt/intel/sgx-aesm-service/aesm && ./aesm_service

nginx

echo "----------- starting blockchain ----------"
bash /root/console.sh purge

# bash /root/console.sh start alice > /root/alice.out 2>&1 &
# bash /root/console.sh start bob > /root/bob.out 2>&1 &
bash /root/console.sh dev > /root/node.out 2>&1 &

echo "----------- starting pruntime ----------"
source /opt/sgxsdk/environment
cd /root/prebuilt && ./app > /root/pruntime.out 2>&1 &

sleep 6

if [ ! -f "/tmp/alice/chains/local_testnet/genesis-info.txt" ]; then
    echo "WARNING! no genesis-info.txt"
fi

echo "----------- starting phost -------------"
cd /root/prebuilt && ./phost -r --mnemonic "then prefer table fatal bus portion refuse chunk attend real horror cat" --pruntime-endpoint "http://localhost:8000" --substrate-ws-endpoint "ws://localhost:9944" > /root/phost.out 2>&1 &

# tail -f /root/alice.out /root/bob.out /root/pruntime.out /root/phost.out
tail -f /root/node.out /root/pruntime.out /root/phost.out
