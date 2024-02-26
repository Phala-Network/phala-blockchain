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

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_fork::rusty_fork_test;

    #[track_caller]
    fn assert_proxy_of(domain: &str, expected: Option<&str>) {
        for fmt_builder in [
            {
                let builder = reqwest::ClientBuilder::new().env_proxy(domain);
                format!("{:?}", builder)
            },
            {
                let builder = reqwest::blocking::ClientBuilder::new().env_proxy(domain);
                format!("{:?}", builder)
            },
        ] {
            match expected {
                Some(expected) => assert!(fmt_builder.contains(expected)),
                None => assert!(!fmt_builder.contains("proxies")),
            }
        }
    }

    rusty_fork_test! {
        #[test]
        fn test_empty_proxy() {
            std::env::remove_var("all_proxy");
            assert_proxy_of("example.com", None);
        }

        #[test]
        fn test_empty_i2p_proxy() {
            std::env::remove_var("all_proxy");
            std::env::remove_var("i2p_proxy");
            assert_proxy_of("example.i2p", None);
        }

        #[test]
        fn test_all_proxy() {
            std::env::set_var("all_proxy", "socks5://127.0.0.1:1234");
            assert_proxy_of("example.com", Some("socks5://127.0.0.1:1234"));
        }

        #[test]
        fn test_i2p_proxy() {
            std::env::set_var("all_proxy", "socks5://127.0.0.1:1234");
            std::env::set_var("i2p_proxy", "socks5://127.0.0.1:4321");
            assert_proxy_of("example.i2p", Some("socks5://127.0.0.1:4321"));
        }

        #[test]
        fn test_i2p_proxy_fallback() {
            std::env::set_var("all_proxy", "socks5://127.0.0.1:1234");
            std::env::remove_var("i2p_proxy");
            assert_proxy_of("example.i2p", Some("socks5://127.0.0.1:1234"));
        }

        #[test]
        fn test_invalid_proxy() {
            std::env::set_var("all_proxy", "socks6://127.0.0.1:1234");
            assert_proxy_of("example.com", None);
        }
    }
}
