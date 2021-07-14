use std::env;

fn main() {
    let sdk_dir = env::var("SGX_SDK").unwrap_or_else(|_| "/opt/intel/sgxsdk".to_string());
    println!("cargo:rustc-link-search=native={}/lib64", sdk_dir);
    println!("cargo:rustc-link-lib=static=sgx_tprotected_fs");

    let ias_env = env::var("IAS_ENV").unwrap_or_else(|_| "DEV".to_string());
    match ias_env.as_ref() {
        "PROD" => {
            println!("cargo:rustc-env=IAS_HOST=api.trustedservices.intel.com");
            println!("cargo:rustc-env=IAS_SIGRL_ENDPOINT=/sgx/attestation/v4/sigrl/");
            println!("cargo:rustc-env=IAS_REPORT_ENDPOINT=/sgx/attestation/v4/report");
        }
        _ => {
            // DEV by default
            println!("cargo:rustc-env=IAS_HOST=api.trustedservices.intel.com");
            println!("cargo:rustc-env=IAS_SIGRL_ENDPOINT=/sgx/dev/attestation/v4/sigrl/");
            println!("cargo:rustc-env=IAS_REPORT_ENDPOINT=/sgx/dev/attestation/v4/report");
        }
    }
}
