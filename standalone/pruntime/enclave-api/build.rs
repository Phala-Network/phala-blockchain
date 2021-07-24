fn main() {
    #[cfg(feature = "std")]
    {
        // Path #1
        use tera::{Context, Tera};

        let tera = Tera::new("proto/*.proto").unwrap();

        let tmpdir = tempdir::TempDir::new("rendered_proto").unwrap();
        let render_dir = tmpdir.path();

        for tmpl in tera.templates.keys() {
            let render_output = std::fs::File::create(render_dir.join(tmpl)).unwrap();
            tera.render_to(tmpl, &Context::new(), render_output)
                .unwrap();
        }

        let out_dir = "./src/proto_generated";

        prpc_build::configure()
            .out_dir(out_dir)
            .mod_prefix("crate::prpc::")
            .disable_package_emission()
            .compile(&["pruntime_rpc.proto"], &[render_dir])
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
