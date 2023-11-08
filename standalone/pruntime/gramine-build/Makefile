SGX_SIGNER_KEY ?= ./private.dev.pem
SGX ?= 1
RA_TYPE ?= epid
BUILD ?= release

include ../config.mk

ARCH_LIBDIR ?= /lib/$(shell $(CC) -dumpmachine)

CFLAGS = -Wall -Wextra

GRAMINE_LOG_LEVEL ?= error

USE_MUSL ?= 0

BIN_NAME = pruntime

DATA_DIRS = data/protected_files \
            data/storage_files

GRAMINE_DIR ?= $(shell ./gramine-dir libs)
GRAMINE_LIBOS ?= $(shell ./gramine-dir libos)
GRAMINE_RUNTIME_DIR ?= $(shell ./gramine-dir runtime)

RUNTIME_DIR = cruntime
LIBOS_BASENAME ?= $(shell basename ${GRAMINE_LIBOS})
LIBOS ?= ${RUNTIME_DIR}/${LIBOS_BASENAME}
PINK_RUNTIME_DIST_DIR = ${RUNTIME_DIR}/pink-runtime
PREBUILT_PINK_RUNTIMES_PROFILE ?= development

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
	make BUILD=${BUILD} PREBUILT_PINK_RUNTIMES_PROFILE=${PREBUILT_PINK_RUNTIMES_PROFILE} -C ..

${BIN_NAME}: ${BIN_FILE}
	cp ${BIN_FILE} ${BIN_NAME}
	mkdir -p ${PINK_RUNTIME_DIST_DIR}
ifeq ($(PREFER_PREBUILT_PINK_LIBS),1)
	cp ${PINK_RUNTIME_DIST} ${PINK_RUNTIME_DIST_DIR}/
	cp ../pink-runtimes/${PREBUILT_PINK_RUNTIMES_PROFILE}/* ${PINK_RUNTIME_DIST_DIR}/
else
	cp ../pink-runtimes/${PREBUILT_PINK_RUNTIMES_PROFILE}/* ${PINK_RUNTIME_DIST_DIR}/
	cp ${PINK_RUNTIME_DIST} ${PINK_RUNTIME_DIST_DIR}/
endif

${BIN_NAME}.manifest: ${BIN_NAME}.manifest.template
	gramine-manifest \
		-Dinstall_dir=$(INSTALL_DIR) \
		-Dlog_level=$(GRAMINE_LOG_LEVEL) \
		-Dra_type=${RA_TYPE} \
		-Dra_client_spid=${IAS_SPID} \
		-Dra_client_linkable=${IAS_LINKABLE} \
		-Dseal_dir=${PRUNTIME_SEAL_DIR} \
		-Dstorage_dir=${PRUNTIME_STORAGE_DIR} \
		-Dlibdir=${RUNTIME_DIR}/lib/ \
		-Dpink_runtime_dir=${PINK_RUNTIME_DIST_DIR}/ \
		-Dlibos=${LIBOS} \
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
	cp -rfL ${HOST_LIBC_DIR}/libstdc++.so.6 ${RUNTIME_DIR}/lib/
endif
ifeq ($(SGX),1)
	cp -rfL ${GRAMINE_DIR}/sgx ${RUNTIME_DIR}/
	cp -rfL ${ARCH_LIBDIR}/libsgx_dcap_quoteverify.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libdcap_quoteprov.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libsgx_urts.so.2 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libsgx_default_qcnl_wrapper.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libcurl.so.4 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libnghttp2.so.14 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libidn2.so.0 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libunistring.so.2 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libgnutls.so.30 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libhogweed.so.6 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libnettle.so.8 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libgmp.so.10 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libkrb5.so.3 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libk5crypto.so.3 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libkrb5support.so.0 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libbrotlicommon.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libp11-kit.so.0 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libkeyutils.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libtasn1.so.6 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libsasl2.so.2 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libcom_err.so.2 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/librtmp.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libssh.so.4 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libpsl.so.5 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libssl.so.3 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libcrypto.so.3 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libgssapi_krb5.so.2 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libldap-2.5.so.0 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/liblber-2.5.so.0 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libzstd.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libbrotlidec.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libz.so.1 ${RUNTIME_DIR}/lib/
	cp -rfL ${ARCH_LIBDIR}/libffi.so.8 ${RUNTIME_DIR}/lib/
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
	mkdir -p ${PREFIX}/data/protected_files
	mkdir -p ${PREFIX}/data/storage_files
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
	gramine-sgx-sigstruct-view ${BIN_NAME}.sig
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
	./gramine-sgx ${BIN_NAME}
else
	gramine-direct ${BIN_NAME}
endif
