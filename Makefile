.PHONY: all node pruntime e2e test clippy dist

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

dist:
	rm -rf dist && mkdir dist
	cp standalone/pruntime/bin/libpink* dist/
	cp standalone/pruntime/bin/pruntime dist/
	cp target/release/pherry dist/
	cp target/release/phala-node dist/
	cp e2e/res/*.contract dist/
	cp e2e/res/*.wasm dist/
	rm -rf dist/check_system*
	rm -rf dist/indeterministic_functions.contract

