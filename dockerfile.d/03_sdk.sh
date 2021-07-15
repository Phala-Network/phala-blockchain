cd /root && \
wget https://download.01.org/intel-sgx/sgx-linux/2.14/distro/ubuntu20.04-server/sgx_linux_x64_sdk_2.14.100.2.bin && \
chmod +x ./sgx_linux_x64_sdk_2.14.100.2.bin && \
echo -e 'no\n/opt' | ./sgx_linux_x64_sdk_2.14.100.2.bin && \
echo 'source /opt/sgxsdk/environment' >> /root/.bashrc && \
rm -rf ./sgx_linux_x64_sdk_2.14.100.2.bin
