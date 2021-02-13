fn main() {
    println!("cargo:rustc-env=SKIP_WASM_BUILD=1");
    println!("cargo:rerun-if-changed=build.rs");
}
