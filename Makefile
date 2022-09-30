.PHONY: all node pruntime e2e

all: node pruntime e2e

node:
	cargo build --release
pruntime:
	make -C standalone/pruntime
e2e:
	make -C e2e/res

