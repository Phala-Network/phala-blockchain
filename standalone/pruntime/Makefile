SGX_MODE ?= SW

.PHONY: all clean
ifeq ($(SGX_MODE),SW)
all: bin/Rocket.toml bin/app

bin/app: bin/pruntime
	cd bin && ln -sf pruntime app

bin/Rocket.toml: gramine-build/Rocket.toml
	cp $? $@

bin/pruntime: target/release/pruntime
	cp $? $@

.PHONY: target/release/pruntime
target/release/pruntime:
	cargo build --release

clean:
	rm -rf bin/*
	cargo clean
else
all:
	make -C gramine-build
clean:
	make clean -C gramine-build
endif

.PHONY: clippy
clippy:
	cargo clippy --tests
