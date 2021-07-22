fn main() {
    #[cfg(feature = "std")]
    {
        // Path #1
        prpc_build::configure()
            .out_dir("./src")
            .disable_package_emission()
            .compile(&["proto/pruntime_rpc.proto"], &["proto"])
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
