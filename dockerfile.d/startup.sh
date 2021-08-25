#!/usr/bin/env bash

nginx

echo "----------- starting blockchain ----------"
bash /root/console.sh purge

# bash /root/console.sh start alice > /root/alice.out 2>&1 &
# bash /root/console.sh start bob > /root/bob.out 2>&1 &
bash /root/console.sh dev > /root/node.out 2>&1 &

echo "----------- starting pruntime ----------"
source /opt/sgxsdk/environment
cd /root/prebuilt && ./app -c 1 > /root/pruntime.out 2>&1 &

sleep 16

echo "----------- starting pherry -------------"
cd /root/prebuilt && ./pherry --dev --pruntime-endpoint "http://localhost:8000" --substrate-ws-endpoint "ws://localhost:9944" > /root/pherry.out 2>&1 &

# tail -f /root/alice.out /root/bob.out /root/pruntime.out /root/pherry.out
tail -f /root/node.out /root/pruntime.out /root/pherry.out
