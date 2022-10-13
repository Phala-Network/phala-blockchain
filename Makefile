.PHONY: all node pruntime e2e test

all: node pruntime e2e

node:
	cargo build --release
pruntime:
	make -C standalone/pruntime
e2e:
	make -C e2e/res
	cd e2e && yarn build:proto
test:
	cargo test --workspace --exclude node-executor --exclude phala-node

