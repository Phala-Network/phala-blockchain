BUILD_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

PINK_RUNTIME_DIR = ${BUILD_DIR}/../../crates/pink/runtime
PINK_RUNTIME_VERSION = $(shell make -C ${BUILD_DIR}/ -f crate-version.mk 1>&2 && ${BUILD_DIR}/crate-version -n 2 ${PINK_RUNTIME_DIR}/Cargo.toml)
PINK_RUNTIME = ${BUILD_DIR}/../../target/release/libpink.so
PINK_RUNTIME_DIST = ${BUILD_DIR}/bin/libpink.so.${PINK_RUNTIME_VERSION}

$(info PINK_RUNTIME_VERSION: ${PINK_RUNTIME_VERSION})
$(info PINK_RUNTIME_DIST: ${PINK_RUNTIME_DIST})
