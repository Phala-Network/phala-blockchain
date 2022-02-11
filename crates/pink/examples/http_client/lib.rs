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
}
