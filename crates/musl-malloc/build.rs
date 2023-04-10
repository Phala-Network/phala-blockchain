use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=csrc/malloc.c");
    println!("cargo:rerun-if-changed=csrc/malloc.h");

    cc::Build::new()
        .file("csrc/malloc.c")
        .flag("-Wno-parentheses")
        .flag("-Wno-unused-value")
        .compile("libmusl_malloc.a");

    let bindings = bindgen::Builder::default()
        .header("csrc/malloc.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
