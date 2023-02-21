TARGET=wasm32-wasi
SIDE_WASM=sideprog/target/${TARGET}/release/sideprog.wasm

target/ink/log_server.contract: sideprog.wasm
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
	rm -rf e2e/node_modules/

../sidevmop/target/ink/manifest.json:
	cd ../sidevmop && cargo contract build

.PHONY: test
test: target/ink/log_server.contract ../sidevmop/target/ink/manifest.json
	cd e2e && yarn && yarn test
