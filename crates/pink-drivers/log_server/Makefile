TARGET=wasm32-wasi
SIDE_WASM=sideprog/target/${TARGET}/release/sideprog.wasm
CONTRACT=target/ink/log_server.contract
PREFIX ?= $(shell realpath ../dist)

${CONTRACT}: sideprog.wasm
	cargo contract build

sideprog.wasm: ${SIDE_WASM}
	cp ${SIDE_WASM} .

.PHONY: ${SIDE_WASM}

${SIDE_WASM}:
	cargo build --manifest-path sideprog/Cargo.toml --release --target ${TARGET}

.PHONY: clean
clean:
	rm -rf sideprog.wasm sideprog.wasm.hash
	rm -rf target/ sideprog/target/

install: ${CONTRACT}
	cp ${CONTRACT} ${PREFIX}/
	cp ${SIDE_WASM} ${PREFIX}/log_server.sidevm.wasm
