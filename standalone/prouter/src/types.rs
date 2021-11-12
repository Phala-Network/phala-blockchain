use anyhow::{anyhow, Result};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::path::PathBuf;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use text_io::read;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[allow(dead_code)]
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
pub struct TunnelInfo(pub String, pub String);

#[derive(Debug)]
pub struct I2PD {
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
        let mut concat_string: String = String::from("./fake-argv0"); // Fake argv input
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
                    error!("The keyfile returned from C is corrupted");
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

    fn get_client_tunnel_name_by_id(&self, index: i32) -> Result<String> {
        let ptr_name: *const c_char = unsafe { C_GetClientTunnelsName(index) };
        if ptr_name.is_null() {
            error!("The name returned from C is corrupted");
            return Err(anyhow!("The name returned from C is corrupted"));
        }

        let c_str_name = unsafe { CStr::from_ptr(ptr_name) };
        let name: String = c_str_name.to_str()?.to_owned();

        Ok(name)
    }

    fn get_client_tunnel_ident_by_id(&self, index: i32) -> Result<String> {
        let ptr_ident: *const c_char = unsafe { C_GetClientTunnelsIdent(index) };
        if ptr_ident.is_null() {
            error!("The ident returned from C is corrupted");
            return Err(anyhow!("The ident returned from C is corrupted"));
        }

        let c_str_ident = unsafe { CStr::from_ptr(ptr_ident) };
        let ident: String = c_str_ident.to_str()?.to_owned();

        Ok(ident)
    }

    fn get_server_tunnel_name_by_id(&self, index: i32) -> Result<String> {
        let ptr_name: *const c_char = unsafe { C_GetServerTunnelsName(index) };
        if ptr_name.is_null() {
            error!("The name returned from C is corrupted");
            return Err(anyhow!("The name returned from C is corrupted"));
        }

        let c_str_name = unsafe { CStr::from_ptr(ptr_name) };
        let name: String = c_str_name.to_str()?.to_owned();

        Ok(name)
    }

    fn get_server_tunnel_ident_by_id(&self, index: i32) -> Result<String> {
        let ptr_ident: *const c_char = unsafe { C_GetServerTunnelsIdent(index) };
        if ptr_ident.is_null() {
            error!("The ident returned from C is corrupted");
            return Err(anyhow!("The ident returned from C is corrupted"));
        }

        let c_str_ident = unsafe { CStr::from_ptr(ptr_ident) };
        let ident: String = c_str_ident.to_str()?.to_owned();

        Ok(ident)
    }

    ///
    /// Basic control API for I2PD
    ///

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
        unsafe { C_StartI2P() };
    }

    pub fn close_accepts_tunnels(&self) {
        unsafe { C_CloseAcceptsTunnels() };
    }

    pub fn stop(&mut self) {
        self.is_running = false;
        unsafe { C_StopI2P() };
    }

    ///
    /// Fetch tunnels info
    ///

    pub fn get_client_tunnels_count(&self) -> i32 {
        return unsafe { C_GetClientTunnelsCount() };
    }

    pub fn get_server_tunnels_count(&self) -> i32 {
        return unsafe { C_GetServerTunnelsCount() };
    }

    pub fn get_client_tunnels_info(&self) -> Result<Vec<TunnelInfo>> {
        let mut client_tunnels_info = Vec::<TunnelInfo>::new();
        let client_tunnels_count = self.get_client_tunnels_count();
        for index in 0..client_tunnels_count {
            info!("client {}", index);
            let name = self.get_client_tunnel_name_by_id(index)?;
            let ident = self.get_client_tunnel_ident_by_id(index)?;
            client_tunnels_info.push(TunnelInfo(name, ident));
        }

        Ok(client_tunnels_info)
    }

    pub fn get_server_tunnels_info(&self) -> Result<Vec<TunnelInfo>> {
        let mut server_tunnels_info = Vec::<TunnelInfo>::new();
        let server_tunnels_count = self.get_server_tunnels_count();
        for index in 0..server_tunnels_count {
            let name = self.get_server_tunnel_name_by_id(index)?;
            let ident = self.get_server_tunnel_ident_by_id(index)?;
            server_tunnels_info.push(TunnelInfo(name, ident));
        }

        Ok(server_tunnels_info)
    }
}