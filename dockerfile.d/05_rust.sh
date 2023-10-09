RUST_TOOLCHAIN=1.73.0
cd /root && \
curl 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' --output /root/rustup-init && \
chmod +x /root/rustup-init && \
echo '1' | /root/rustup-init --default-toolchain "${RUST_TOOLCHAIN}" && \
echo 'source /root/.cargo/env' >> /root/.bashrc && \
/root/.cargo/bin/rustup component add rust-src rust-analysis clippy && \
/root/.cargo/bin/rustup target add wasm32-unknown-unknown && \
rm /root/rustup-init && rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git
