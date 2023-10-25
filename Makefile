.PHONY: all node pruntime e2e test clippy

all: node pruntime e2e

node:
	cargo build --release
pruntime:
	make -C standalone/pruntime
e2e:
	make -C e2e/res
	cd frontend/packages/sdk && yarn && yarn build
	cd e2e && yarn && yarn build:proto
test:
	cargo test --workspace --exclude node-executor --exclude phala-node

clippy:
	cargo clippy --tests
	make clippy -C standalone/pruntime

lint:
	cargo dylint --all --workspace
