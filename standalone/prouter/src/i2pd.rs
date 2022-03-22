use anyhow::{anyhow, Result};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

enum PRouterSigningKeyType {
    SigningKeyTypeDSASha1 = 0,
    SigningKeyTypeECDSASha256P256,
    SigningKeyTypeECDSASha384P384,
    SigningKeyTypeECDSASha512P521,
    SigningKeyTypeRSASha2562048,
    SigningKeyTypeRSASha3843072,
    SigningKeyTypeRSASha5124096,
    SigningKeyTypeEDDSASha512Ed25519,
    SigningKeyTypeEDDSASha512Ed25519ph,
    SigningKeyTypeGOSTR3410CryptoProAGostr3411256,
    SigningKeyTypeGOSTR3410Tc26A512Gostr3411512,
    SigningKeyTypeREDDSASha512Ed25519,
}

enum PRouterCryptoKeyType {
    CryptoKeyTypeElgamal = 0,
    CryptoKeyTypeECIESP256Sha256Aes256cbc = 1,
    CryptoKeyTypeECIESX25519Aead = 4,
    CryptoKeyTypeECIESP256Sha256Aes256cbcTest = 65280,
    CryptoKeyTypeECIESGOSTR3410CryptoProASha256Aes256cbc = 65281,
}

#[derive(Debug)]
pub struct TunnelInfo(pub String, pub String);

#[derive(Debug, Clone)]
pub struct I2PD {
    argc: i32,
    argv: HashMap<String, String>,
    app_name: String,
    is_running: bool,
}

pub fn generate_ident_to_file(
    abs_datadir: &PathBuf,
    filename: String,
    phala_i2p_key: Vec<u8>,
) -> Result<String> {
    let mut keyfile_path = abs_datadir.clone();
    keyfile_path.push(filename);

    // let c_str_sk = CString::new(sk).expect("sk should be able to be converted into CString");

    let c_sk: *const c_char = phala_i2p_key.as_ptr() as *const c_char;

    let c_str_filename = CString::new(String::from(keyfile_path.to_string_lossy()))
        .expect("String should be able to be converted into CString");
    let c_filename: *const c_char = c_str_filename.as_ptr() as *const c_char;
    let ptr_ident: *const c_char = unsafe {
        C_GenerateIdentToFile(
            c_filename,
            c_sk,
            PRouterSigningKeyType::SigningKeyTypeEDDSASha512Ed25519 as u16,
            PRouterCryptoKeyType::CryptoKeyTypeElgamal as u16,
        )
    };

    if ptr_ident.is_null() {
        error!("The ident returned from C is corrupted");
        return Err(anyhow!("The ident returned from C is corrupted"));
    }

    let c_str_ident = unsafe { CStr::from_ptr(ptr_ident) };
    // Ok(c_str_ident.to_bytes().to_owned())
    let ident: String = c_str_ident.to_str()?.to_owned();

    Ok(ident)
}

fn get_client_tunnel_name_by_id(index: i32) -> Result<String> {
    let ptr_name: *const c_char = unsafe { C_GetClientTunnelsName(index) };
    if ptr_name.is_null() {
        error!("The name returned from C is corrupted");
        return Err(anyhow!("The name returned from C is corrupted"));
    }

    let c_str_name = unsafe { CStr::from_ptr(ptr_name) };
    let name: String = c_str_name.to_str()?.to_owned();

    Ok(name)
}

fn get_client_tunnel_ident_by_id(index: i32) -> Result<String> {
    let ptr_ident: *const c_char = unsafe { C_GetClientTunnelsIdent(index) };
    if ptr_ident.is_null() {
        error!("The ident returned from C is corrupted");
        return Err(anyhow!("The ident returned from C is corrupted"));
    }

    let c_str_ident = unsafe { CStr::from_ptr(ptr_ident) };
    let ident: String = c_str_ident.to_str()?.to_owned();

    Ok(ident)
}

fn get_server_tunnel_name_by_id(index: i32) -> Result<String> {
    let ptr_name: *const c_char = unsafe { C_GetServerTunnelsName(index) };
    if ptr_name.is_null() {
        error!("The name returned from C is corrupted");
        return Err(anyhow!("The name returned from C is corrupted"));
    }

    let c_str_name = unsafe { CStr::from_ptr(ptr_name) };
    let name: String = c_str_name.to_str()?.to_owned();

    Ok(name)
}

fn get_server_tunnel_ident_by_id(index: i32) -> Result<String> {
    let ptr_ident: *const c_char = unsafe { C_GetServerTunnelsIdent(index) };
    if ptr_ident.is_null() {
        error!("The ident returned from C is corrupted");
        return Err(anyhow!("The ident returned from C is corrupted"));
    }

    let c_str_ident = unsafe { CStr::from_ptr(ptr_ident) };
    let ident: String = c_str_ident.to_str()?.to_owned();

    Ok(ident)
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
        info!("PRouter invoked parameters: {:?}", &concat_string);

        concat_string
    }

    pub fn add_config(&mut self, key: String, data: String) {
        self.argv.insert(key, data);
        self.argc += 1;
    }

    // Basic control API for I2PD
    pub fn init(&self) {
        if self.is_running {
            return;
        }
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
        if self.is_running {
            return;
        }
        self.is_running = true;
        unsafe { C_StartI2P() };
    }

    pub fn run_peer_test(&self) {
        if !self.is_running {
            return;
        }
        unsafe { C_RunPeerTest() };
    }

    pub fn close_accepts_tunnels(&self) {
        if !self.is_running {
            return;
        }
        unsafe { C_CloseAcceptsTunnels() };
    }

    pub fn stop(&mut self) {
        if !self.is_running {
            return;
        }
        self.is_running = false;
        unsafe { C_StopI2P() };
    }

    //Fetch status
    pub fn get_network_status(&self) -> Result<String> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let ptr_status: *const c_char = unsafe { C_GetNetworkStatus() };
        if ptr_status.is_null() {
            error!("Th e status returned from C is corrupted");
            return Err(anyhow!("The status returned from C is corrupted"));
        }

        let c_str_status = unsafe { CStr::from_ptr(ptr_status) };
        let status: String = c_str_status.to_str()?.to_owned();

        Ok(status)
    }

    pub fn get_tunnel_creation_success_rate(&self) -> Result<i32> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let rate: i32 = unsafe { C_GetTunnelCreationSuccessRate() };

        Ok(rate)
    }

    pub fn get_received_byte(&self) -> Result<u64> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let byte: u64 = unsafe { C_GetReceivedByte() };

        Ok(byte)
    }

    pub fn get_in_bandwidth(&self) -> Result<u32> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let bandwidth: u32 = unsafe { C_GetInBandwidth() };

        Ok(bandwidth)
    }

    pub fn get_sent_byte(&self) -> Result<u64> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let byte: u64 = unsafe { C_GetSentByte() };

        Ok(byte)
    }

    pub fn get_out_bandwidth(&self) -> Result<u32> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let bandwidth: u32 = unsafe { C_GetOutBandwidth() };

        Ok(bandwidth)
    }

    pub fn get_transit_byte(&self) -> Result<u64> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let byte: u64 = unsafe { C_GetTransitByte() };

        Ok(byte)
    }

    pub fn get_transit_bandwidth(&self) -> Result<u32> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let bandwidth: u32 = unsafe { C_GetTransitBandwidth() };

        Ok(bandwidth)
    }

    pub fn is_httpproxy_enabled(&self) -> Result<bool> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let enabled: i32 = unsafe { C_IsHTTPProxyEnabled() };

        Ok(enabled == 1)
    }

    pub fn is_socksproxy_enabled(&self) -> Result<bool> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let enabled: i32 = unsafe { C_IsSOCKSProxyEnabled() };

        Ok(enabled == 1)
    }

    pub fn is_bob_enabled(&self) -> Result<bool> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let enabled: i32 = unsafe { C_IsBOBEnabled() };

        Ok(enabled == 1)
    }

    pub fn is_sam_enabled(&self) -> Result<bool> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let enabled: i32 = unsafe { C_IsSAMEnabled() };

        Ok(enabled == 1)
    }

    pub fn is_i2cp_enabled(&self) -> Result<bool> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }

        let enabled: i32 = unsafe { C_IsI2CPEnabled() };

        Ok(enabled == 1)
    }

    // Fetch tunnels info
    pub fn get_client_tunnels_count(&self) -> Result<i32> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let count: i32 = unsafe { C_GetClientTunnelsCount() };

        Ok(count)
    }

    pub fn get_server_tunnels_count(&self) -> Result<i32> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let count: i32 = unsafe { C_GetServerTunnelsCount() };

        Ok(count)
    }

    pub fn get_client_tunnels_info(&self) -> Result<Vec<TunnelInfo>> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let mut client_tunnels_info = Vec::<TunnelInfo>::new();
        let client_tunnels_count = self.get_client_tunnels_count()?;
        for index in 0..client_tunnels_count {
            let name = get_client_tunnel_name_by_id(index)?;
            let ident = get_client_tunnel_ident_by_id(index)?;
            client_tunnels_info.push(TunnelInfo(name, ident));
        }

        Ok(client_tunnels_info)
    }

    pub fn get_http_proxy_info(&self) -> Result<TunnelInfo> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let ptr_ident: *const c_char = unsafe { C_GetHTTPProxyIdent() };
        if ptr_ident.is_null() {
            error!("The ident returned from C is corrupted");
            return Err(anyhow!("The ident returned from C is corrupted"));
        }

        let c_str_ident = unsafe { CStr::from_ptr(ptr_ident) };
        let ident: String = c_str_ident.to_str()?.to_owned();

        Ok(TunnelInfo("HTTP Proxy".to_string(), ident))
    }

    pub fn get_socks_proxy_info(&self) -> Result<TunnelInfo> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let ptr_ident: *const c_char = unsafe { C_GetSOCKSProxyIdent() };
        if ptr_ident.is_null() {
            error!("The ident returned from C is corrupted");
            return Err(anyhow!("The ident returned from C is corrupted"));
        }

        let c_str_ident = unsafe { CStr::from_ptr(ptr_ident) };
        let ident: String = c_str_ident.to_str()?.to_owned();

        Ok(TunnelInfo("SOCKS Proxy".to_string(), ident))
    }

    pub fn get_server_tunnels_info(&self) -> Result<Vec<TunnelInfo>> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let mut server_tunnels_info = Vec::<TunnelInfo>::new();
        let server_tunnels_count = self.get_server_tunnels_count()?;
        for index in 0..server_tunnels_count {
            let name = get_server_tunnel_name_by_id(index)?;
            let ident = get_server_tunnel_ident_by_id(index)?;
            server_tunnels_info.push(TunnelInfo(name, ident));
        }

        Ok(server_tunnels_info)
    }

    pub fn get_inbound_tunnels_count(&self) -> Result<i32> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let count: i32 = unsafe { C_GetInboundTunnelsCount() };

        Ok(count)
    }

    pub fn get_outbound_tunnels_count(&self) -> Result<i32> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let count: i32 = unsafe { C_GetOutboundTunnelsCount() };

        Ok(count)
    }

    pub fn get_inbound_tunnel_formatted_info(&self, index: i32) -> Result<String> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let ptr_info: *const c_char = unsafe { C_GetInboundTunnelsFormattedInfo(index) };
        if ptr_info.is_null() {
            error!("The info returned from C is corrupted");
            return Err(anyhow!("The info returned from C is corrupted"));
        }

        let c_str_info = unsafe { CStr::from_ptr(ptr_info) };
        let info: String = c_str_info.to_str()?.to_owned();

        Ok(info)
    }

    pub fn get_outbound_tunnel_formatted_info(&self, index: i32) -> Result<String> {
        if !self.is_running {
            return Err(anyhow!("I2PD is not running"));
        }
        let ptr_info: *const c_char = unsafe { C_GetOutboundTunnelsFormattedInfo(index) };
        if ptr_info.is_null() {
            error!("The info returned from C is corrupted");
            return Err(anyhow!("The info returned from C is corrupted"));
        }

        let c_str_info = unsafe { CStr::from_ptr(ptr_info) };
        let info: String = c_str_info.to_str()?.to_owned();

        Ok(info)
    }
}
