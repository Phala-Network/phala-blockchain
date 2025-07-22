set -e
# Workaround for https://github.com/orgs/community/discussions/47863
apt-mark hold grub-efi-amd64-signed
apt update
apt upgrade -y
apt install -y apt-utils \
 apt-transport-https \
 software-properties-common \
 readline-common \
 curl \
 vim \
 wget \
 gnupg \
 gnupg2 \
 gnupg-agent \
 ca-certificates \
 unzip \
 lsb-release \
 debhelper \
 gettext \
 cmake \
 reprepro \
 autoconf \
 automake \
 bison \
 build-essential \
 curl \
 dpkg-dev \
 expect \
 flex \
 gcc \
 gdb \
 git \
 git-core \
 gnupg \
 kmod \
 libboost-system-dev \
 libboost-thread-dev \
 libcurl4-openssl-dev \
 libiptcdata0-dev \
 libjsoncpp-dev \
 liblog4cpp5-dev \
 libprotobuf-dev \
 libssl-dev \
 libtool \
 libxml2-dev \
 uuid-dev \
 ocaml \
 ocamlbuild \
 pkg-config \
 protobuf-compiler \
 python-is-python3 \
 texinfo \
 llvm \
 clang \
 libclang-dev \
 nginx \
 python3-pip \
 musl-tools \

apt autoremove -y
rm -rf /var/lib/apt/lists/*