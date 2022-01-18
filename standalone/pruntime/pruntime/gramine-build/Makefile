SGX_SIGNER_KEY ?= ~/.config/gramine/enclave-key.pem
SGX ?= 1

ARCH_LIBDIR ?= /lib/$(shell $(CC) -dumpmachine)

CFLAGS = -Wall -Wextra

GRAMINE_LOG_LEVEL = error

BIN_NAME=pruntime
BIN_FILE=../target/release/${BIN_NAME}

.PHONY: all
all: ${BIN_NAME} ${BIN_NAME}.manifest
ifeq ($(SGX),1)
all: ${BIN_NAME}.manifest.sgx ${BIN_NAME}.sig ${BIN_NAME}.token
endif

${BIN_FILE}:
	cargo build --release

${BIN_NAME}: ${BIN_FILE}
	cp ${BIN_FILE} ${BIN_NAME}

${BIN_NAME}.manifest: ${BIN_NAME}.manifest.template
	gramine-manifest \
		-Dinstall_dir=$(INSTALL_DIR) \
		-Dlog_level=$(GRAMINE_LOG_LEVEL) \
		$< $@

${BIN_NAME}.manifest.sgx: ${BIN_NAME}.manifest ${BIN_FILE}
	@test -s $(SGX_SIGNER_KEY) || \
	    { echo "SGX signer private key was not found, please specify SGX_SIGNER_KEY!"; exit 1; }
	gramine-sgx-sign \
		--key $(SGX_SIGNER_KEY) \
		--manifest $< \
		--output $@

${BIN_NAME}.sig: ${BIN_NAME}.manifest.sgx

${BIN_NAME}.token: ${BIN_NAME}.sig
	gramine-sgx-get-token \
		--output $@ --sig $<

.PHONY: clean
clean:
	$(RM) *.token *.sig *.manifest.sgx *.manifest ${BIN_NAME}.o ${BIN_NAME} OUTPUT
	$(RM) -rf ../target

.PHONY: distclean
distclean: clean
