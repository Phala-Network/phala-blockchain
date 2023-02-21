DRIVER_DIR=../../crates/pink-drivers/

CHECK_SYSTEM_BIN=check_system/target/ink/manifest.json
INDETERMINF_DIR=indeterministic_functions
INDETERMINF=${INDETERMINF_DIR}/target/ink/manifest.json

.PHONY: $(SYSTEM_CONTRACT_BIN) ${CHECK_SYSTEM_BIN} ${INDETERMINF} drivers

all: drivers ${CHECK_SYSTEM_BIN} ${INDETERMINF}

drivers:
	cd ${DRIVER_DIR} && ./build.sh
	cp ${DRIVER_DIR}/dist/* .

$(SYSTEM_CONTRACT_BIN):
	cd "${SYSTEM_CONTRACT_DIR}" && cargo contract build

${CHECK_SYSTEM_BIN}:
	make -C check_system

${INDETERMINF}:
	cd ${INDETERMINF_DIR} && cargo contract build

