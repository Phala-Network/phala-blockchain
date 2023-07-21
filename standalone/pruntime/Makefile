SGX_MODE ?= SW
BUILD?=release
ifeq ($(BUILD),release)
	XARGS=--release
endif
PREBUILT_PINK_RUNTIMES_PROFILE ?= development

include config.mk

.PHONY: all clean run app runtime

all: app runtime
app: bin/app
runtime: ${PINK_RUNTIME_DIST}
run:
	./bin/app -c 0

${PINK_RUNTIME_DIST}: ${PINK_RUNTIME}
	cp pink-runtimes/${PREBUILT_PINK_RUNTIMES_PROFILE}/* bin/
	cp $? $@
	cp $@ target/${BUILD}/
	strip --strip-all --keep-symbol=__pink_runtime_init $@

.PHONY: ${PINK_RUNTIME}
${PINK_RUNTIME}:
	cd ${PINK_RUNTIME_DIR} && cargo build ${XARGS}

bin/app: bin/pruntime bin/Rocket.toml
	cd bin && ln -sf pruntime app

bin/Rocket.toml: gramine-build/Rocket.toml
	cp $? $@

bin/pruntime: target/${BUILD}/pruntime
	cp $? $@

.PHONY: target/${BUILD}/pruntime
target/${BUILD}/pruntime:
	cargo build ${XARGS}

clean:
	rm -rf bin/*
	cargo clean

.PHONY: clippy
clippy:
	cargo clippy --tests
