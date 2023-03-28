apt-get update && apt-get install -y dkms gnupg2 apt-transport-https software-properties-common
curl -fsSLo /usr/share/keyrings/intel-sgx-deb.asc https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx-deb.asc] https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -sc) main" | tee /etc/apt/sources.list.d/intel-sgx.list
apt-get update && \
apt-get install -y \
    libsgx-aesm-launch-plugin \
    libsgx-enclave-common \
    libsgx-epid \
    libsgx-launch \
    libsgx-quote-ex \
    libsgx-uae-service \
    libsgx-urts && \
mkdir /var/run/aesmd && \
rm -rf /var/lib/apt/lists/*
