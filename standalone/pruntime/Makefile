SGX_MODE ?= SW

include config.mk

.PHONY: all clean run

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

.PHONY: clippy
clippy:
	cargo clippy --tests
