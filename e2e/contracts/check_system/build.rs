fn main() {
    println!("cargo:rerun-if-changed=sideprog.wasm");
    let hash = sp_core::blake2_256(include_bytes!("./sideprog.wasm"));
    std::fs::write("./sideprog.wasm.hash", hash).unwrap();
}
