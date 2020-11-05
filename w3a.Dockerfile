###
### W3a docker file
### docker build --network=host -f w3a.Dockerfile -t w3a .
### docker run -it -p 7000:7000 -p 9000:9000 --env DOCKER_IP=xxx.xxx.xxx.xxx w3a
###

FROM ubuntu:18.04

ARG DEBIAN_FRONTEND='noninteractive'

ADD dockerfile.d/01_apt.sh /root
RUN bash /root/01_apt.sh

ADD dockerfile.d/02_llvm.sh /root
RUN bash /root/02_llvm.sh

ADD ./dockerfile.d/03_sdk.sh /root
RUN bash /root/03_sdk.sh

ARG RUST_TOOLCHAIN='nightly-2020-11-01'
ADD ./dockerfile.d/05_rust.sh /root
RUN bash /root/05_rust.sh

WORKDIR /root

# ====== build TEE ======

RUN mkdir phala-blockchain
ADD . phala-blockchain

RUN mkdir prebuilt

RUN cd phala-blockchain/pruntime && \
    PATH="$PATH:$HOME/.cargo/bin" SGX_SDK="/opt/sgxsdk" SGX_MODE=SW make && \
    cp ./bin/app /root/prebuilt && \
    cp ./bin/enclave.signed.so /root/prebuilt && \
    cp ./bin/Rocket.toml /root/prebuilt && \
    PATH="$PATH:$HOME/.cargo/bin" make clean && \
    rm -rf /root/.cargo/registry && \
    rm -rf /root/.cargo/git

# ====== clean up ======

RUN rm -rf phala-blockchain
ADD dockerfile.d/cleanup.sh .
RUN bash cleanup.sh

# ===== build gateway =====

WORKDIR /root
RUN curl -sSL https://rvm.io/mpapis.asc | gpg --import -
RUN curl -sSL https://rvm.io/pkuczynski.asc | gpg --import -
RUN curl -sSL https://get.rvm.io | bash -s stable

ENV PATH /usr/local/rvm/gems/ruby-2.6.5/bin:/usr/local/rvm/rubies/ruby-2.6.5@global/bin:/usr/local/rvm/rubies/ruby-2.6.5/bin:/usr/local/rvm/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
RUN echo "export rvm_max_time_flag=20" >> .rvmrc
RUN rvm requirements
RUN rvm install 2.6.5
ENV GEM_HOME /usr/local/rvm/gems/ruby-2.6.5
ENV GEM_PATH /usr/local/rvm/gems/ruby-2.6.5:/usr/local/rvm/gems/ruby-2.6.5@global
RUN gem install bundler rails pry
RUN apt install -y libsecp256k1-dev

RUN git clone https://github.com/Phala-Network/w3a-gateway.git
WORKDIR /root/w3a-gateway
RUN bundle install
RUN cp config/credentials.yml.example config/credentials.yml
RUN cp config/database.yml.sqlite3 config/database.yml
RUN rails db:migrate
RUN rails db:seed

# ===== build backend =====

WORKDIR /root
RUN apt update
RUN curl -sL https://deb.nodesource.com/setup_12.x | sudo -E bash -
RUN apt install -y nodejs

RUN git clone https://github.com/Phala-Network/w3a-backend.git

WORKDIR /root/w3a-backend
RUN npm install
RUN node main.js init

WORKDIR /root/w3a-backend/web_server
RUN npm install

WORKDIR /root
RUN cp w3a-backend/dockerfile.script/*.sh .
RUN chmod 700 *.sh
CMD bash ./w3a_start.sh

EXPOSE 7000
EXPOSE 9000
