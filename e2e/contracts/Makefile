MK_CONTRACTS := check_system
PURE_CONTRACTS := indeterministic_functions
ALL_CONTRACTS := $(PURE_CONTRACTS) $(MK_CONTRACTS)

PREFIX ?= $(shell pwd)
export PREFIX

.PHONY: install always-rerun clean

install: $(PREFIX) $(PURE_CONTRACTS:%=install-%) $(MK_CONTRACTS:%=mk-install-%)

$(PREFIX):
	mkdir -p $(PREFIX)

install-%: always-rerun
	cd $* && cargo contract build --release
	cp $*/target/ink/$*.contract ${PREFIX}/$*.contract

mk-install-%: always-rerun
	make -C $*
	cp $*/target/ink/$*.contract ${PREFIX}/$*.contract
	cp $*/sideprog.wasm ${PREFIX}/$*.sidevm.wasm

always-rerun:

clean: ${ALL_CONTRACTS:%=clean-%}

${ALL_CONTRACTS:%=clean-%}:
	cd ${@:clean-%=%} && cargo clean
