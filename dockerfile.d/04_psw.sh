apt-get update && apt-get install -y dkms gnupg2 apt-transport-https software-properties-common && \
curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add - && \
add-apt-repository "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $CODENAME main" && \
apt-get update && \
apt-get install -y \
    libsgx-aesm-launch-plugin \
    libsgx-enclave-common \
    libsgx-enclave-common-dev \
    libsgx-epid \
    libsgx-epid-dev \
    libsgx-launch \
    libsgx-launch-dev \
    libsgx-quote-ex \
    libsgx-quote-ex-dev \
    libsgx-uae-service \
    libsgx-urts && \
mkdir /var/run/aesmd && \
rm -rf /var/lib/apt/lists/*
