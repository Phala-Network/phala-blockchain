#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate ini;
use ini::Ini;

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::os::raw::c_char;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;
use text_io::read;
use tokio::signal;
use tokio::time::sleep;

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

    #[structopt(long, default_value = "./pdata")]
    prouter_datadir: String,
}

enum PROUTER_SIGNING_KEY_TYPE {
    SIGNING_KEY_TYPE_DSA_SHA1 = 0,
    SIGNING_KEY_TYPE_ECDSA_SHA256_P256,
    SIGNING_KEY_TYPE_ECDSA_SHA384_P384,
    SIGNING_KEY_TYPE_ECDSA_SHA512_P521,
    SIGNING_KEY_TYPE_RSA_SHA256_2048,
    SIGNING_KEY_TYPE_RSA_SHA384_3072,
    SIGNING_KEY_TYPE_RSA_SHA512_4096,
    SIGNING_KEY_TYPE_EDDSA_SHA512_ED25519,
    SIGNING_KEY_TYPE_EDDSA_SHA512_ED25519ph,
    SIGNING_KEY_TYPE_GOSTR3410_CRYPTO_PRO_A_GOSTR3411_256,
    SIGNING_KEY_TYPE_GOSTR3410_TC26_A_512_GOSTR3411_512,
    SIGNING_KEY_TYPE_REDDSA_SHA512_ED25519,
}

enum PROUTER_CRYPTO_KEY_TYPE {
    CRYPTO_KEY_TYPE_ELGAMAL = 0,
    CRYPTO_KEY_TYPE_ECIES_P256_SHA256_AES256CBC = 1,
    CRYPTO_KEY_TYPE_ECIES_X25519_AEAD = 4,
    CRYPTO_KEY_TYPE_ECIES_P256_SHA256_AES256CBC_TEST = 65280,
    CRYPTO_KEY_TYPE_ECIES_GOSTR3410_CRYPTO_PRO_A_SHA256_AES256CBC = 65281,
}

#[derive(Debug)]
struct I2PD {
    argc: i32,
    argv: HashMap<String, String>,
    app_name: String,
    is_running: bool,
}

impl I2PD {
    pub fn new(app_name: String) -> I2PD {
        I2PD {
            argc: 1,
            argv: HashMap::new(),
            app_name,
            is_running: false,
        }
    }

    fn drop(&mut self) {
        if self.is_running {
            self.stop();
        }
    }

    fn argv_to_string(&self) -> String {
        let mut concat_string: String = String::from("./fake"); // Fake argv input
        for (key, data) in &self.argv {
            concat_string.push_str(format!(" --{}={}", key, data).as_str());
        }
        info!("PRouter call parameter: {:?}", &concat_string);

        concat_string
    }

    pub fn add_config(&mut self, key: String, data: String) {
        self.argv.insert(key, data);
        self.argc += 1;
    }

    pub fn load_private_keys_from_file(&self, check_existence: bool) -> Result<String> {
        let opt_datadir = self.argv.get("datadir");
        match opt_datadir {
            Some(datadir) => {
                let mut keyfile_path = PathBuf::from(&datadir);
                keyfile_path.push("pnetwork.key");
                if check_existence && !keyfile_path.as_path().exists() {
                    loop {
                        info!("There is no detected keyfile in data directory. Enter `yes` to continue and create a new keyfile and `no` to abort.");
                        let user_response: String = read!("{}\n");
                        match user_response.as_str().to_lowercase().as_str() {
                            "yes" => break,
                            "no" => {
                                return Err(anyhow!("Private key loading aborted"));
                            }
                            _ => {}
                        }
                    }
                }

                let c_str = CString::new(String::from(keyfile_path.to_string_lossy()))
                    .expect("String should be able to be converted into CString");
                let c_filename: *const c_char = c_str.as_ptr() as *const c_char;
                let ptr_identity: *const c_char = unsafe {
                    C_LoadPrivateKeysFromFile(
                        c_filename,
                        PROUTER_SIGNING_KEY_TYPE::SIGNING_KEY_TYPE_EDDSA_SHA512_ED25519 as u16,
                        PROUTER_CRYPTO_KEY_TYPE::CRYPTO_KEY_TYPE_ELGAMAL as u16,
                    )
                };
                if ptr_identity.is_null() {
                    error!("The keyfile is corrupted");
                    return Err(anyhow!("The keyfile is corrupted"));
                }

                let c_str_identity = unsafe { CStr::from_ptr(ptr_identity) };
                let identity: String = c_str_identity.to_str()?.to_owned();

                info!("Loaded PNetwork identity: {:?}", identity);
                return Ok(identity);
            }
            None => {
                info!("You must specify datadir before loading private keys");
                return Err(anyhow!("Missing datadir"));
            }
        }
    }

    pub fn init(&self) {
        let args_str = self.argv_to_string();
        let args_c_str =
            CString::new(args_str).expect("String should be able to be converted into CString");
        let c_args: *mut c_char = args_c_str.as_ptr() as *mut c_char;

        let name_c_str = CString::new(self.app_name.clone())
            .expect("String should be able to be converted into CString");
        let c_name: *const c_char = name_c_str.as_ptr() as *const c_char;
        unsafe {
            C_InitI2P(self.argc, c_args, c_name);
        }
    }

    pub fn start(&mut self) {
        self.is_running = true;
        unsafe { C_StartI2P() }
    }

    pub fn stop(&mut self) {
        self.is_running = false;
        unsafe { C_StopI2P() }
    }
}

fn preprocess_path(path_str: &String) -> Result<PathBuf> {
    // Check path exists
    let path = Path::new(&path_str);
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    // Convert to absolute path
    let absolute_path = path
        .canonicalize()
        .expect("Any path should always be able to converted into a absolute path");

    Ok(absolute_path)
}

fn get_relative_filepath(path: &PathBuf, filename: &str) -> PathBuf {
    let mut path_dup = path.clone();
    path_dup.push(filename);

    path_dup
}

fn pathbuf_to_string(path: PathBuf) -> Result<String> {
    let ret = match path.into_os_string().into_string() {
        Ok(d) => d,
        Err(e) => return Err(anyhow!("{:?}", e)),
    };

    Ok(ret)
}

fn get_relative_filepath_str(path: &PathBuf, filename: &str) -> Result<String> {
    let mut path_dup = path.clone();
    path_dup.push(filename);

    pathbuf_to_string(path_dup)
}

fn init_prouter_conf(datadir: &PathBuf) -> Result<()> {
    let prouterconf_path = get_relative_filepath_str(&datadir, "i2pd.conf")?;
    let tunconf_path = get_relative_filepath_str(&datadir, "tunnels.conf")?;
    let tundir = get_relative_filepath_str(&datadir, "tunnels.d")?;
    let pidfile = get_relative_filepath_str(&datadir, "prouter.pid")?;
    let certsdir = get_relative_filepath_str(&datadir, "certificates")?;
    let logfile = get_relative_filepath_str(&datadir, "prouter.log")?;

    let mut conf = Ini::new();
    conf.with_section(None::<String>)
        .set("tunconf", tunconf_path)
        .set("tunnelsdir", tundir)
        .set("pidfile", pidfile)
        .set("certsdir", certsdir)
        .set("log", "file")
        .set("logfile", logfile)
        .set("loglevel", "warn")
        .set("logclftime", "true")
        .set("daemon", "true")
        .set("nat", "true")
        .set("ipv4", "true")
        .set("ipv6", "false")
        .set("ssu", "true")
        .set("bandwidth", "L")
        .set("share", "100")
        .set("notransit", "false")
        .set("floodfill", "false");

    conf.with_section(Some("httpproxy"))
        .set("enabled", "true")
        .set("address", "127.0.0.1")
        .set("port", "4444")
        .set("addresshelper", "true")
        .set("inbound.length", "3")
        .set("inbound.quantity", "5")
        .set("outbound.length", "3")
        .set("outbound.quantity", "5");

    conf.with_section(Some("socksproxy"))
        .set("enabled", "true")
        .set("address", "127.0.0.1")
        .set("port", "4447")
        .set("inbound.length", "3")
        .set("inbound.quantity", "5")
        .set("outbound.length", "3")
        .set("outbound.quantity", "5");

    conf.with_section(Some("sam"))
        .set("enabled", "true")
        .set("address", "127.0.0.1")
        .set("port", "7656");

    conf.with_section(Some("bob"))
        .set("enabled", "false")
        .set("address", "127.0.0.1")
        .set("port", "2827");

    conf.with_section(Some("i2cp"))
        .set("enabled", "false")
        .set("address", "127.0.0.1")
        .set("port", "7654");

    conf.with_section(Some("upnp"))
        .set("enabled", "false")
        .set("name", "PRouter");

    conf.with_section(Some("precomputation"))
        .set("elgamal", "true");

    conf.with_section(Some("reseed"))
        .set("verify", "false")
        .set("threshold", "25");

    conf.with_section(Some("addressbook"));

    conf.with_section(Some("limits"))
        .set("transittunnels", "2500")
        .set("openfiles", "0")
        .set("coresize", "0")
        .set("ntcpsoft", "0")
        .set("ntcphard", "0");

    conf.with_section(Some("trust"))
        .set("enabled", "false")
        .set("hidden", "false");

    // conf.with_section(Some("websocket"))
    //     .set("enabled", "false")
    //     .set("address", "127.0.0.1")
    //     .set("port", "7666");

    conf.with_section(Some("exploratory"))
        .set("inbound.length", "2")
        .set("inbound.quantity", "3")
        .set("outbound.length", "2")
        .set("outbound.quantity", "3");

    conf.with_section(Some("persist")).set("profiles", "true");

    conf.with_section(Some("meshnets"))
        .set("yggdrasil", "false")
        .set("yggaddress", "");

    conf.write_to_file(prouterconf_path)?;

    Ok(())
}

fn init_tunnels_conf(datadir: &PathBuf) -> Result<()> {
    let tunconf_path = get_relative_filepath_str(&datadir, "tunnels.conf")?;

    let mut conf = Ini::new();
    conf.with_section(Some("PNetwork"))
        .set("type", "http")
        .set("host", "127.0.0.1")
        .set("port", "8000")
        .set("keys", "pnetwork.key");

    conf.write_to_file(tunconf_path)?;

    Ok(())
}

pub async fn prouter_main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::from_args();

    // init path
    let datadir = preprocess_path(&args.prouter_datadir)?;

    // init conf
    init_prouter_conf(&datadir)?;
    init_tunnels_conf(&datadir)?;

    // init I2PD
    let mut i2pd = I2PD::new("PRouter".parse()?);
    i2pd.add_config("datadir".parse()?, pathbuf_to_string(datadir.clone())?);
    i2pd.add_config(
        "conf".parse()?,
        get_relative_filepath_str(&datadir, "i2pd.conf")?,
    );

    let pnetwork_identity = i2pd.load_private_keys_from_file(true)?;

    i2pd.init();
    i2pd.start();
    signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c event");
    i2pd.stop();
    Ok(())
}

#[tokio::main]
async fn main() {
    match prouter_main().await {
        Ok(()) => {}
        Err(e) => panic!("Fetal error: {:?}", e),
    };
}
