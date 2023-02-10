fn main() {
    let rootdir = std::env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR");
    let capi_dir = format!("{rootdir}/src");
    for ver in ["v1"] {
        println!("cargo:rerun-if-changed={capi_dir}/{ver}/types.h");
        let bindings = bindgen::Builder::default()
            .header(format!("{capi_dir}/{ver}/types.h"))
            .use_core()
            .derive_default(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .layout_tests(false)
            .generate()
            .expect("Unable to generate bindings");
        let out_file = format!("{capi_dir}/{ver}/types.rs");
        bindings
            .write_to_file(out_file)
            .expect("Couldn't write bindings!");
    }
}
