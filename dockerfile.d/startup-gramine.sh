#!/usr/bin/env bash
echo "----------- starting aesmd ----------"
LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm /opt/intel/sgx-aesm-service/aesm/aesm_service &
echo "----------- starting pruntime ----------"
#source /opt/sgxsdk/environment
cd /root/prebuilt && gramine-sgx pruntime -c 1
