MK_CONTRACTS := log_server
PURE_CONTRACTS := system tokenomic sidevm_deployer
ALL_CONTRACTS := $(PURE_CONTRACTS) $(MK_CONTRACTS)
PREFIX ?= $(shell realpath ./dist)
export PREFIX


.PHONY: all always-rerun clean
all:

$(PURE_CONTRACTS): always-rerun
	make -f ../build-contract.mk CONTRACT_NAME=$@ -C $@

install: ${PREFIX} $(PURE_CONTRACTS:%=install-%) $(MK_CONTRACTS:%=mk-install-%)

install-%: always-rerun
	make -f ../build-contract.mk CONTRACT_NAME=$* -C $* install

mk-install-%: always-rerun
	make install -C $*

${PREFIX}:
	mkdir -p ${PREFIX}

always-rerun:

clean: ${ALL_CONTRACTS:%=clean-%}

clean-%:
	make clean -C $* -f ../build-contract.mk