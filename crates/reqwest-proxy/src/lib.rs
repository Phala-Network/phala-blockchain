use reqwest::Proxy;

pub trait EnvProxyBuilder {
    fn env_proxy(self) -> Self;
}

fn proxies_from_env() -> Option<(Proxy, Proxy)> {
    let uri = std::env::var("all_proxy").ok()?;
    let http_proxy = Proxy::http(&uri).ok()?;
    let https_proxy = Proxy::https(&uri).ok()?;
    Some((http_proxy, https_proxy))
}

impl EnvProxyBuilder for reqwest::ClientBuilder {
    fn env_proxy(self) -> Self {
        match proxies_from_env() {
            Some((http_proxy, https_proxy)) => self.proxy(http_proxy).proxy(https_proxy),
            None => self,
        }
    }
}

#[cfg(feature = "blocking")]
impl EnvProxyBuilder for reqwest::blocking::ClientBuilder {
    fn env_proxy(self) -> Self {
        match proxies_from_env() {
            Some((http_proxy, https_proxy)) => self.proxy(http_proxy).proxy(https_proxy),
            None => self,
        }
    }
}
