SGX_MODE ?= SW

PINK_RUNTIME_DIR = ../../crates/pink/runtime
PINK_RUNTIME_VERSION = $(shell ./crate-version -n 2 ${PINK_RUNTIME_DIR}/Cargo.toml)
PINK_RUNTIME = ../../target/release/libpink.so
PINK_RUNTIME_DIST = bin/libpink.so.${PINK_RUNTIME_VERSION}

$(info PINK_RUNTIME_VERSION: ${PINK_RUNTIME_VERSION})

.PHONY: all clean run
ifeq ($(SGX_MODE),SW)
all: bin/Rocket.toml bin/app ${PINK_RUNTIME_DIST}

run:
	./bin/app -c 0

${PINK_RUNTIME_DIST}: ${PINK_RUNTIME}
	cp $? $@
	cp $@ target/release/
	strip --strip-all --keep-symbol=__pink_runtime_init $@

.PHONY: ${PINK_RUNTIME}
${PINK_RUNTIME}:
	cd ${PINK_RUNTIME_DIR} && cargo build --release

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
