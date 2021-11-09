#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::{anyhow, Result};
use std::os::raw::c_char;
use std::ffi::CString;
use std::collections::HashMap;
use structopt::StructOpt;

use phactory_api::blocks::{
    self, AuthoritySet, AuthoritySetChange, BlockHeaderWithChanges, HeaderToSync, StorageProof,
};
use phactory_api::prpc::{self, InitRuntimeResponse};
use phactory_api::pruntime_client;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug, StructOpt)]
#[structopt(name = "prouter")]
struct Args {
    #[structopt(
    default_value = "http://localhost:8000",
    long,
    help = "pRuntime http endpoint"
    )]
    pruntime_endpoint: String,
}

#[derive(Debug)]
struct I2PD {
    argc: i32,
    argv: HashMap<String, String>,
    app_name: String,
}

impl I2PD {
    pub fn new(argc: i32, argv: HashMap<String, String>, app_name: String) -> I2PD {
        I2PD { argc, argv, app_name }
    }

    pub fn add_config(&mut self, key: String, data: String) {
        self.argv.insert(
            key,
            data,
        );
    }

    fn argv_to_c_char(&self) -> *mut c_char {
        let mut concat_string: String = String::new();
        for (key, data) in &self.argv {
            concat_string.push_str(format!("{}={}", key, data).as_str());
        }

        let c_str = CString::new(concat_string)
            .expect("String should be able to be converted into CString");
        let c_args: *mut c_char = c_str.as_ptr() as *mut c_char;

        c_args
    }

    fn app_name_to_c_char(&self) -> *const c_char {
        let c_str = CString::new(self.app_name.clone())
            .expect("String should be able to be converted into CString");
        let c_name: *const c_char = c_str.as_ptr() as *const c_char;

        c_name
    }

    fn init(&self) {
        unsafe { C_InitI2P(self.argc, self.argv_to_c_char(), self.app_name_to_c_char()); }
    }
}

pub async fn prouter_main() -> Result<()> {
    let mut args = Args::from_args();
    let pr = pruntime_client::new_pruntime_client(args.pruntime_endpoint.clone());
    let info = pr.get_info(()).await?;
    println!("{:?}", info);

    Ok(())
}

#[tokio::main]
async fn main() {
    prouter_main().await;
}
