fn main() {
    #[cfg(feature = "std")]
    {
        // Path #1
        use tera::{Template, Context, Tera};

        let tmpl_str = std::fs::read_to_string("proto/pruntime_rpc.proto").unwrap();
        let tmpl = Template::new("proto", None, &tmpl_str).unwrap();
        let mut tera = Tera::new("proto/*").unwrap();
        tera.templates.insert("pruntime_rpc.proto".to_owned(), tmpl);
        let render_output = std::fs::File::create("proto/pruntime_rpc_rendered.proto").unwrap();
        tera.render_to("pruntime_rpc.proto", &Context::new(), render_output).unwrap();

        prpc_build::configure()
            .out_dir("./src")
            .disable_package_emission()
            .compile(&["proto/pruntime_rpc_rendered.proto"], &["proto"])
            .unwrap();
    }

    // Workaround for enclave patched the rand crate to rand-sgx one which make the prpc_build not working.
    #[cfg(not(feature = "std"))]
    {
        // goto Path #1
        std::process::Command::new("cargo")
            .arg("check")
            .status()
            .unwrap();
    }
}
