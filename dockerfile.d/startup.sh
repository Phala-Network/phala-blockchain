#!/usr/bin/env bash

nginx

echo "----------- starting blockchain ----------"
cd /root/phala-blockchain && bash ./scripts/console.sh purge
cd /root/phala-blockchain && bash ./scripts/console.sh start alice  > /root/alice.out 2>&1 &
cd /root/phala-blockchain && bash ./scripts/console.sh start bob > /root/bob.out 2>&1 &

echo "----------- starting pruntime ----------"
source /opt/sgxsdk/environment
cd /root/phala-blockchain/pruntime/bin && ./app > /root/pruntime.out 2>&1 &

sleep 1

if [ ! -f "/tmp/alice/chains/local_testnet/genesis-info.txt" ]; then
    echo "WARNING! no genesis-info.txt"
fi

echo "----------- starting phost -------------"
cd /root/phala-blockchain && ./target/release/phost > /root/phost.out 2>&1 &

tail -f /root/alice.out /root/bob.out /root/pruntime.out /root/phost.out
