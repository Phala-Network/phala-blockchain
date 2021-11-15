extern crate ini;
use ini::Ini;

use anyhow::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::path::Path;
use std::path::PathBuf;

use crate::utils::get_relative_filepath_str;

pub fn init_prouter_conf(datadir: &PathBuf) -> Result<()> {
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

pub fn init_tunnels_conf(datadir: &PathBuf, existed_tunconf: &Option<String>, pruntime_host: &String, pruntime_port: &String, ignore_pnetwork: &bool,) -> Result<()> {
    let tunconf_path = get_relative_filepath_str(&datadir, "tunnels.conf")?;

    let mut conf = match existed_tunconf {
        Some(path) => match Ini::load_from_file(&path) {
            Ok(ini) => ini,
            Err(e) => {
                warn!("Failed to load the existed tunnel file: {}", e);
                Ini::new()
            }
        },
        _ => Ini::new(),
    };

    if !ignore_pnetwork {
        conf.with_section(Some("PNetwork"))
            .set("type", "http")
            .set("host", pruntime_host)
            .set("port", pruntime_port)
            .set("keys", "pnetwork.key")
            .set("inbound.length", "3")
            .set("inbound.quantity", "5")
            .set("outbound.length", "3")
            .set("outbound.quantity", "5");
    }

    conf.write_to_file(tunconf_path)?;

    Ok(())
}
