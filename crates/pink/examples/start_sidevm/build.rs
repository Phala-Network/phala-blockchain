fn main() {
    let hash = sp_core::blake2_256(include_bytes!("./sideprog.wasm"));
    std::fs::write("./sideprog.wasm.hash", hash).unwrap();
}
