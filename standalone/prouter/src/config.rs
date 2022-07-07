use ini::Ini;

use crate::Args;
use anyhow::Result;
use std::path::PathBuf;

use phala_types::EndpointType;

use crate::utils::push_str;

pub fn init_prouter_conf(
    abs_datadir: &PathBuf,
    tunconf_path: String,
    args: &Args,
) -> Result<(String, String)> {
    let conf_path = push_str(&abs_datadir, "i2pd.conf")?;
    let tundir = push_str(&abs_datadir, "tunnels.d")?;
    let pidfile = push_str(&abs_datadir, "prouter.pid")?;
    let certsdir = push_str(&abs_datadir, "certificates")?;
    let logfile = push_str(&abs_datadir, "prouter.log")?;
    let _su3file = push_str(&abs_datadir, "prouter.su3")?;

    // Construct default config
    let mut conf = Ini::new();
    conf.with_section(None::<String>)
        .set("tunconf", tunconf_path)
        .set("tunnelsdir", tundir)
        .set("pidfile", pidfile)
        .set("certsdir", certsdir)
        .set("log", "file")
        .set("logfile", logfile)
        .set("loglevel", "debug")
        .set("logclftime", "true")
        .set("daemon", "true")
        .set("nat", "true")
        .set("ipv4", "true")
        .set("ipv6", "false")
        .set("ssu", "true")
        .set("bandwidth", "X")
        .set("share", "100")
        .set("netid", "2")
        .set("notransit", "false")
        .set("floodfill", "true");

    conf.with_section(Some("httpproxy"))
        .set("enabled", "true")
        .set("address", "127.0.0.1")
        .set("port", "4444")
        .set("addresshelper", "true")
        .set("inbound.length", "1")
        .set("inbound.quantity", "16")
        .set("outbound.length", "1")
        .set("outbound.quantity", "16");

    conf.with_section(Some("socksproxy"))
        .set("enabled", "true")
        .set("address", "127.0.0.1")
        .set("port", "4447")
        .set("inbound.length", "1")
        .set("inbound.quantity", "16")
        .set("outbound.length", "1")
        .set("outbound.quantity", "16");

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
        // .set("urls", "")
        // .set("file", su3file)
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

    conf.with_section(Some("exploratory"))
        .set("inbound.length", "2")
        .set("inbound.quantity", "16")
        .set("outbound.length", "2")
        .set("outbound.quantity", "16");

    conf.with_section(Some("persist")).set("profiles", "true");

    conf.with_section(Some("meshnets"))
        .set("yggdrasil", "false")
        .set("yggaddress", "");

    // overriding
    if let Some(override_conf) = &args.override_i2pd {
        if let Ok(o_ini) = Ini::load_from_file(&override_conf) {
            for (sec, prop) in o_ini.iter() {
                for (k, v) in prop.iter() {
                    conf.with_section(sec).set(k, v);
                }
            }
        }
    }

    let address = conf
        .section(Some("socksproxy"))
        .expect("socksproxy should exist")
        .get("address")
        .expect("socksproxy.address should exist");
    let port = conf
        .section(Some("socksproxy"))
        .expect("socksproxy should exist")
        .get("port")
        .expect("socksproxy.port should exist");
    conf.write_to_file(conf_path.clone())?;

    Ok((conf_path, format!("{}:{}", address, port)))
}

pub fn init_tunnels_conf(abs_datadir: &PathBuf, args: &Args) -> Result<String> {
    let tunconf_path = push_str(&abs_datadir, "tunnels.conf")?;

    let mut conf = match &args.override_tun {
        Some(path) => match Ini::load_from_file(&path) {
            Ok(ini) => ini,
            Err(_e) => Ini::new(),
        },
        _ => Ini::new(),
    };

    if matches!(args.endpoint_type, EndpointType::I2P) {
        // need to use i2p to proxy local API
        conf.with_section(Some("Phala Network"))
            .set("type", "http")
            .set("host", args.exposed_address.clone())
            .set("port", args.exposed_port.clone().to_string())
            .set("keys", "phala.key")
            .set("inbound.length", "3")
            .set("inbound.quantity", "5")
            .set("outbound.length", "3")
            .set("outbound.quantity", "5");
    }

    conf.write_to_file(tunconf_path.clone())?;

    Ok(tunconf_path)
}
