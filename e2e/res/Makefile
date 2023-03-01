DRIVER_DIR=../../crates/pink-drivers/
PREFIX = $(shell pwd)

.PHONY: all drivers test-contracts clean

all: drivers test-contracts

drivers:
	make install -C $(DRIVER_DIR) PREFIX=$(PREFIX)

test-contracts:
	make install -C ../contracts PREFIX=$(PREFIX)

clean:
	make clean -C $(DRIVER_DIR)
	make clean -C ../contracts
	rm -rf *.contract
	rm -rf *.wasm
