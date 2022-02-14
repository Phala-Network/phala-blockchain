#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;


#[pink::contract(env=PinkEnvironment)]
mod http_client {
    use super::pink;
    use pink::{PinkEnvironment, http_post, http_get};
    use alloc::vec::Vec;

    #[ink(storage)]
    pub struct HttpClient {}

    impl HttpClient {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn get_ip(&self) -> (u16, Vec<u8>) {
            let resposne = http_get!("https://ip.kvin.wang");
            (resposne.status_code, resposne.body)
        }

        #[ink(message)]
        pub fn post_data(&self) -> (u16, Vec<u8>) {
            let resposne = http_post!("https://example.com", b"payload".to_vec());
            (resposne.status_code, resposne.body)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;
        #[ink::test]
        fn get_ip_works() {
            use pink_extension::chain_extension::{HttpResponse, test::MockHttpRequest};

            ink_env::test::register_chain_extension(MockHttpRequest::new(|request| {
                if request.url == "https://ip.kvin.wang" {
                    HttpResponse::ok(b"1.1.1.1".to_vec())
                } else {
                    HttpResponse::not_found()
                }
            }));

            let contract = HttpClient::default();
            assert_eq!(contract.get_ip().1, b"1.1.1.1");
        }
    }
}
