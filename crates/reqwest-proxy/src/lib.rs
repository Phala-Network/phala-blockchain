use reqwest::Proxy;

pub trait EnvProxyBuilder {
    fn env_proxy(self, domain: &str) -> Self;
}

fn proxies_from_env(domain: &str) -> Option<(Proxy, Proxy)> {
    let uri = if domain.ends_with(".i2p") {
        std::env::var("i2p_proxy").ok()
    } else {
        None
    };

    let uri = uri.or_else(|| std::env::var("all_proxy").ok())?;

    let http_proxy = Proxy::http(&uri).ok()?;
    let https_proxy = Proxy::https(&uri).ok()?;
    Some((http_proxy, https_proxy))
}

impl EnvProxyBuilder for reqwest::ClientBuilder {
    fn env_proxy(self, domain: &str) -> Self {
        match proxies_from_env(domain) {
            Some((http_proxy, https_proxy)) => self.proxy(http_proxy).proxy(https_proxy),
            None => self,
        }
    }
}

#[cfg(feature = "blocking")]
impl EnvProxyBuilder for reqwest::blocking::ClientBuilder {
    fn env_proxy(self, domain: &str) -> Self {
        match proxies_from_env(domain) {
            Some((http_proxy, https_proxy)) => self.proxy(http_proxy).proxy(https_proxy),
            None => self,
        }
    }
}
