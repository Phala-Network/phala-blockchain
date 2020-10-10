use std::env;

fn main () {
    if env::var("SKIP_IAS").is_ok() || env::var("SGX_MODE") == Ok("SW".to_string()) {
        println!("cargo:rustc-env=IAS_SPID=''");
        println!("cargo:rustc-env=IAS_API_KEY=''");
    }

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
