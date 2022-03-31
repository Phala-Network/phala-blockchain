GX_SIGNER_KEY ?= ./sgx_private.pem
SGX ?= 1
USE_MUSL ?= 0 
BIN_NAME = gramine

ifeq ($(USE_MUSL), 1) 
    BIN_FILE = target/x86_64-unknown-linux-musl/release/${BIN_NAME}
    CARGO_ARGS = --target x86_64-unknown-linux-musl
else
    BIN_FILE = target/release/${BIN_NAME}
    CARGO_ARGS =
endif

.PHONY: all
all: ${BIN_NAME} ${BIN_NAME}.manifest
ifeq ($(SGX), 1)
all: ${BIN_NAME}.manifest.sgx ${BIN_NAME}.token
endif 

.PHONY: ${BIN_FILE}
${BIN_FILE}:
	cargo build --release ${CARGO_ARGS}

${BIN_NAME}: ${BIN_FILE}
	cp ../../${BIN_FILE} ${BIN_NAME}

${BIN_NAME}.manifest: ${BIN_NAME}.manifest.template
	mkdir -p data
	mkdir -p allowed
	gramine-manifest \
	    -Dlog_level=all \
	    $< $@

${BIN_NAME}.manifest.sgx: ${BIN_NAME}.manifest ${BIN_FILE}
	gramine-sgx-sign \
	    --manifest $< \
	    --output $@

${BIN_NAME}.sig: ${BIN_NAME}.manifest.sgx

${BIN_NAME}.token: ${BIN_NAME}.sig
	gramine-sgx-get-token \
	    --output $@ --sig $<

.PHONY: clean
clean: 
	$(RM) *.token *.sig *.manifest.sgx *.manifest ${BIN_NAME}.o ${BIN_NAME} OUTPUT
	# no use for remove the target should cargo clean again 
	rm -rf data
	rm -rf allowed

# I think this is no use for the latest version gramine
.PHONY: sgx_gen_private_key
sgx_gen_private_key:
	openssl genrsa -out sgx_private.pem -3 3072

