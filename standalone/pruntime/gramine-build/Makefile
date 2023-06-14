SGX_SIGNER_KEY ?= ./private.dev.pem
SGX ?= 1
BUILD ?= release

include ../config.mk

ARCH_LIBDIR ?= /lib/$(shell $(CC) -dumpmachine)

CFLAGS = -Wall -Wextra

GRAMINE_LOG_LEVEL = error

USE_MUSL ?= 0

BIN_NAME = pruntime
RA_METHOD ?= epid

DATA_DIRS = data/protected_files \
            data/storage_files

GRAMINE_DIR ?= $(shell ./gramine-dir libs)
GRAMINE_LIBOS ?= $(shell ./gramine-dir libos)
GRAMINE_RUNTIME_DIR ?= $(shell ./gramine-dir runtime)

RUNTIME_DIR = cruntime
LIBOS_BASENAME ?= $(shell basename ${GRAMINE_LIBOS})
LIBOS ?= ${RUNTIME_DIR}/${LIBOS_BASENAME}
PINK_RUNTIME_DIST_DIR = ${RUNTIME_DIR}/pink-runtime

ifeq ($(USE_MUSL),1)
BIN_FILE = ../target/x86_64-unknown-linux-musl/${BUILD}/${BIN_NAME}
CARGO_ARGS = --target x86_64-unknown-linux-musl
HOST_LIBC_DIR = /lib/x86_64-linux-musl
else
BIN_FILE = ../target/${BUILD}/${BIN_NAME}
CARGO_ARGS =
HOST_LIBC_DIR = /lib/x86_64-linux-gnu
endif

ifeq ($(V),1)
CARGO_ARGS += -vv
endif

PREFIX ?= ../bin
PRUNTIME_DATA_DIR ?= data
PRUNTIME_SEAL_DIR ?= ${PRUNTIME_DATA_DIR}/protected_files
PRUNTIME_STORAGE_DIR ?= ${PRUNTIME_DATA_DIR}/storage_files

.PHONY: all
all: ${BIN_NAME} ${LIBOS}

.PHONY: signed-sgx-artifacts
signed-sgx-artifacts: ${BIN_NAME}.manifest ${BIN_NAME}.manifest.sgx ${BIN_NAME}.sig

.PHONY: ${BIN_FILE}
${BIN_FILE}:
	make BUILD=${BUILD} -C ..

${BIN_NAME}: ${BIN_FILE}
	cp ${BIN_FILE} ${BIN_NAME}
	mkdir -p ${PINK_RUNTIME_DIST_DIR}
ifeq ($(PREFER_PREBUILT_PINK_LIBS),1)
	cp ${PINK_RUNTIME_DIST} ${PINK_RUNTIME_DIST_DIR}/
	cp ../pink-runtimes/* ${PINK_RUNTIME_DIST_DIR}/
else
	cp ../pink-runtimes/* ${PINK_RUNTIME_DIST_DIR}/
	cp ${PINK_RUNTIME_DIST} ${PINK_RUNTIME_DIST_DIR}/
endif

${BIN_NAME}.manifest: ${BIN_NAME}.manifest.template
	gramine-manifest \
		-Dinstall_dir=$(INSTALL_DIR) \
		-Dlog_level=$(GRAMINE_LOG_LEVEL) \
		-Dra_client_spid=${IAS_SPID} \
		-Dseal_dir=${PRUNTIME_SEAL_DIR} \
		-Dstorage_dir=${PRUNTIME_STORAGE_DIR} \
		-Dlibdir=${RUNTIME_DIR}/lib/ \
		-Dpink_runtime_dir=${PINK_RUNTIME_DIST_DIR}/ \
		-Dlibos=${LIBOS} \
		-Dra_method=${RA_METHOD} \
		$< $@

${BIN_NAME}.manifest.sgx: ${BIN_NAME}.manifest ${BIN_NAME} ${LIBOS}
	@test -s $(SGX_SIGNER_KEY) || \
	    { echo "SGX signer private key was not found, please specify SGX_SIGNER_KEY!"; exit 1; }
	gramine-sgx-sign \
		--key $(SGX_SIGNER_KEY) \
		--manifest $< \
		--output $@

${LIBOS}:
	mkdir -p ${RUNTIME_DIR}
	rsync -r --no-links ${GRAMINE_RUNTIME_DIR}/ ${RUNTIME_DIR}/lib
ifeq ($(USE_MUSL),0)
	cp -rfL ${HOST_LIBC_DIR}/libgcc_s.so.1 ${RUNTIME_DIR}/lib/
endif
ifeq ($(SGX),1)
	cp -rfL ${GRAMINE_DIR}/sgx ${RUNTIME_DIR}/
endif
	cp -rfL ${GRAMINE_LIBOS} ${RUNTIME_DIR}/

${BIN_NAME}.sig: ${BIN_NAME}.manifest.sgx

${BIN_NAME}.token: ${BIN_NAME}.sig
	gramine-sgx-get-token \
		--output $@ --sig $<

.PHONY: token
token: ${BIN_NAME}.token

.PHONY: dirs
dirs: ${DATA_DIRS}

${DATA_DIRS}:
	mkdir -p $@

.PHONY: clean
clean:
	$(RM) *.token *.sig *.manifest.sgx *.manifest ${BIN_NAME}.o ${BIN_NAME} OUTPUT
	$(RM) -rf ../target
	$(RM) -rf data
	$(RM) -rf ${RUNTIME_DIR}

.PHONY: distclean
distclean: clean

.PHONY: pre-dist
pre-dist: all
	mkdir -p ${PREFIX}/data
	cp ${BIN_NAME} ${PREFIX}/
	cp Rocket.toml ${PREFIX}/
	cp -rfL ${RUNTIME_DIR} ${PREFIX}/

.PHONY: dist
ifeq ($(SGX),1)
dist: pre-dist ${BIN_NAME}.manifest signed-sgx-artifacts
	cp ${BIN_NAME}.manifest ${PREFIX}/
	cp ${BIN_NAME}.manifest.sgx ${PREFIX}/
	cp ${BIN_NAME}.sig ${PREFIX}/
	cp gramine-sgx ${PREFIX}/
else
dist: pre-dist ${BIN_NAME}.manifest
	cp ${BIN_NAME}.manifest ${PREFIX}/
endif

token-dist: token dist
	cp ${BIN_NAME}.token ${PREFIX}/

.PHONY: run
run: all token
	make dirs
ifeq ($(SGX),1)
	./gramine-sgx pruntime
else
	gramine-direct pruntime
endif
