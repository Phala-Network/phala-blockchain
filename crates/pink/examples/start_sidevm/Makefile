SIDE_WASM=sideprog/target/wasm32-unknown-unknown/release/sideprog.wasm

target/ink/start_sidevm.contract: sideprog.wasm
	cargo contract build --release

sideprog.wasm: ${SIDE_WASM}
	cp ${SIDE_WASM} .
	wasm-strip sideprog.wasm

.PHONY: ${SIDE_WASM}

${SIDE_WASM}:
	cargo build --manifest-path sideprog/Cargo.toml --release --target wasm32-unknown-unknown

.PHONY: clean
clean:
	rm -rf sideprog.wasm
	rm -rf target/ sideprog/target/

