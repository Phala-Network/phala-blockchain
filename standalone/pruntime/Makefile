SGX_MODE ?= SW
TARGET=x86_64-unknown-linux-musl
BUILT_PRUNTIME=target/${TARGET}/release/pruntime

.PHONY: all clean
ifeq ($(SGX_MODE),SW)
all: bin/Rocket.toml bin/app

bin/app: bin/pruntime
	cd bin && ln -sf pruntime app

bin/Rocket.toml: gramine-build/Rocket.toml
	cp $? $@

bin/pruntime: ${BUILT_PRUNTIME}
	cp $? $@

.PHONY: ${BUILT_PRUNTIME}
${BUILT_PRUNTIME}:
	cargo build --release --target ${TARGET}

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
