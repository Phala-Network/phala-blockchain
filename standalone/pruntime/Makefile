# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

SHELL := /bin/bash

######## SGX SDK Settings ########

SGX_SDK ?= /opt/intel/sgxsdk
SGX_MODE ?= HW
SGX_ARCH ?= x64

# Changing SGX_MODE will trigger rebuild
define DEPENDABLE_VAR
.PHONY: phony
$1: phony
	@if [[ `cat $1 2>&1` != '$($1)' ]]; then \
		echo -n $($1) > $1 ; \
	fi
endef
$(eval $(call DEPENDABLE_VAR,SGX_MODE))

TOP_DIR := .
include $(TOP_DIR)/buildenv.mk

SGX_SIGN_KEY ?= enclave/Enclave_private.dev.pem
ifdef SGX_SIGN_KEY_URL
ifneq ($(SGX_SIGN_KEY_URL),)
	SGX_SIGN_KEY := priv/Enclave_private.pem
endif
endif

SGX_ENCLAVE_CONFIG ?= enclave/Enclave.config.dev.xml
ifdef SGX_ENCLAVE_CONFIG_URL
ifneq ($(SGX_ENCLAVE_CONFIG_URL),)
	SGX_ENCLAVE_CONFIG := priv/Enclave.config.xml
endif
endif

ifeq ($(shell getconf LONG_BIT), 32)
	SGX_ARCH := x86
else ifeq ($(findstring -m32, $(CXXFLAGS)), -m32)
	SGX_ARCH := x86
endif

ifeq ($(SGX_ARCH), x86)
	SGX_COMMON_CFLAGS := -m32
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib
	SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x86/sgx_sign
	SGX_EDGER8R := $(SGX_SDK)/bin/x86/sgx_edger8r
else
	SGX_COMMON_CFLAGS := -m64
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
	SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x64/sgx_sign
	SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx_edger8r
endif

ifeq ($(SGX_DEBUG), 1)
ifeq ($(SGX_PRERELEASE), 1)
	$(error Cannot set SGX_DEBUG and SGX_PRERELEASE at the same time!!)
endif
endif

ifeq ($(SGX_DEBUG), 1)
	SGX_COMMON_CFLAGS += -O0 -g
else
	SGX_COMMON_CFLAGS += -O2
endif

SGX_COMMON_CFLAGS += -fstack-protector

######## Rust SGX SDK Settings ########

RUST_SGX_SDK := ../../teaclave-sgx-sdk
RUST_SGX_EDL_PATH := $(RUST_SGX_SDK)/edl
RUST_SGX_COMMON_PATH := $(RUST_SGX_SDK)/common

######## CUSTOM Settings ########

CUSTOM_LIBRARY_PATH := ./lib
CUSTOM_BIN_PATH := ./bin
CUSTOM_EDL_PATH := ./edl
CUSTOM_COMMON_PATH := ./common

######## EDL Settings ########

Enclave_EDL_Files := enclave/Enclave_t.c enclave/Enclave_t.h app/Enclave_u.c app/Enclave_u.h

######## APP Settings ########

App_Rust_Flags := --release
App_Include_Paths := -I ./app -I./include -I$(SGX_SDK)/include -I$(RUST_SGX_EDL_PATH) -I$(CUSTOM_EDL_PATH)
App_C_Flags := $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes $(App_Include_Paths)
App_SRC_Files := $(shell find app/ -type f -name '*.rs') $(shell find app/ -type f -name 'Cargo.toml')
App_Link_Flags := -L$(SGX_LIBRARY_PATH) -L$(CUSTOM_LIBRARY_PATH)

App_Link_Flags += -lsgx_uprotected_fs

App_Rust_Path := ./app/target/release
App_Enclave_u_Object := app/libEnclave_u.a
App_Name := bin/app

######## Enclave Settings ########

ifneq ($(SGX_MODE), HW)
	Trts_Library_Name := sgx_trts_sim
	Service_Library_Name := sgx_tservice_sim
else
	Trts_Library_Name := sgx_trts
	Service_Library_Name := sgx_tservice
endif
Crypto_Library_Name := sgx_tcrypto
KeyExchange_Library_Name := sgx_tkey_exchange
ProtectedFs_Library_Name := sgx_tprotected_fs

RustEnclave_C_Files := $(wildcard ./enclave/*.c)
RustEnclave_C_Objects := $(RustEnclave_C_Files:.c=.o)
RustEnclave_Include_Paths := -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH) -I$(RUST_SGX_COMMON_PATH)/inc -I$(RUST_SGX_EDL_PATH) -I$(SGX_SDK)/include -I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid -I ./enclave -I./include

RustEnclave_Link_Libs := -L$(CUSTOM_LIBRARY_PATH) -lenclave -lsgx_pthread -lsgx_tstdc
RustEnclave_Compile_Flags := $(SGX_COMMON_CFLAGS) $(ENCLAVE_CFLAGS) $(RustEnclave_Include_Paths)
RustEnclave_Link_Flags := -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) \
	-Wl,--whole-archive -l$(Trts_Library_Name) -Wl,--no-whole-archive \
	-Wl,--start-group -lsgx_tstdc -lsgx_tcxx -l$(Service_Library_Name) -l$(Crypto_Library_Name) -l$(ProtectedFs_Library_Name) $(RustEnclave_Link_Libs) -Wl,--end-group \
	-Wl,--version-script=enclave/Enclave.lds \
	$(ENCLAVE_LDFLAGS)

RustEnclave_Name := enclave/enclave.so
Signed_RustEnclave_Name := bin/enclave.signed.so
Dumped_Signed_RustEnclave_Name := bin/enclave.dump

.PHONY: all
all: $(App_Name) $(Signed_RustEnclave_Name)

######## EDL Objects ########

$(Enclave_EDL_Files): $(SGX_EDGER8R) enclave/Enclave.edl
	$(SGX_EDGER8R) --trusted enclave/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(RUST_SGX_EDL_PATH) --search-path $(CUSTOM_EDL_PATH) --trusted-dir enclave
	$(SGX_EDGER8R) --untrusted enclave/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(RUST_SGX_EDL_PATH) --search-path $(CUSTOM_EDL_PATH)  --untrusted-dir app
	@echo "GEN  =>  $(Enclave_EDL_Files)"

######## App Objects ########

app/Enclave_u.o: $(Enclave_EDL_Files)
	@$(CC) $(App_C_Flags) -c app/Enclave_u.c -o $@
	@echo "CC   <=  $<"

$(App_Enclave_u_Object): app/Enclave_u.o
	$(AR) rcsD $@ $^
	cp $(App_Enclave_u_Object) ./lib

$(App_Name): $(App_Enclave_u_Object) $(App_SRC_Files) SGX_MODE
	@cd app && SGX_SDK=$(SGX_SDK) cargo build $(App_Rust_Flags)
	@echo "Cargo  =>  $@"
	mkdir -p bin
	cp $(App_Rust_Path)/app ./bin

######## Enclave Objects ########

enclave/Enclave_t.o: $(Enclave_EDL_Files)
	$(CC) $(RustEnclave_Compile_Flags) -c enclave/Enclave_t.c -o $@
	@echo "CC   <=  $<"

$(RustEnclave_Name): enclave enclave/Enclave_t.o
	$(CXX) enclave/Enclave_t.o -o $@ $(RustEnclave_Link_Flags)
	@echo "LINK =>  $@"

$(Signed_RustEnclave_Name): $(RustEnclave_Name)
	@mkdir -p priv
	@if [ ! -z "$(SGX_SIGN_KEY_URL)" ]; then wget $(SGX_SIGN_KEY_URL) -q -t 3 -O $(SGX_SIGN_KEY); fi
	@if [ ! -z "$(SGX_ENCLAVE_CONFIG_URL)" ]; then wget $(SGX_ENCLAVE_CONFIG_URL) -q -t 3 -O $(SGX_ENCLAVE_CONFIG); fi
	@mkdir -p bin
	$(SGX_ENCLAVE_SIGNER) sign -key $(SGX_SIGN_KEY) -enclave $(RustEnclave_Name) -out $@ -config $(SGX_ENCLAVE_CONFIG)
	@rm -rf priv
	@echo "SIGN =>  $@"

.PHONY: enclave
enclave:
	$(MAKE) -C ./enclave/

.PHONY: dump
dump:
	$(SGX_ENCLAVE_SIGNER) dump -enclave $(Signed_RustEnclave_Name) -dumpfile $(Dumped_Signed_RustEnclave_Name)
	@cat $(Dumped_Signed_RustEnclave_Name)

.PHONY: clean
clean:
	@rm -f $(App_Name) $(RustEnclave_Name) $(Signed_RustEnclave_Name) $(Dumped_Signed_RustEnclave_Name) enclave/*_t.* app/*_u.* lib/*.a
	@cd enclave && cargo clean && rm -f Cargo.lock
	@cd app && cargo clean && rm -f Cargo.lock
