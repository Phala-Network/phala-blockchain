SYSTEM_CONTRACT_DIR=../../crates/pink-drivers/system
SYSTEM_CONTRACT_BIN=${SYSTEM_CONTRACT_DIR}/target/ink/pink_system.contract
CHECK_SYSTEM_BIN=check_system/target/ink/manifest.json

.PHONY: $(SYSTEM_CONTRACT_BIN) ${CHECK_SYSTEM_BIN}

all: pink_system.contract ${CHECK_SYSTEM_BIN}

pink_system.contract: ${SYSTEM_CONTRACT_BIN}
	cp $? $@

$(SYSTEM_CONTRACT_BIN):
	cd "${SYSTEM_CONTRACT_DIR}" && cargo contract build --release

${CHECK_SYSTEM_BIN}:
	make -C check_system

