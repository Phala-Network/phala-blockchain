#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;


#[pink::contract(env=PinkEnvironment)]
mod signing {
    use super::pink;
    use pink::PinkEnvironment;
    use pink::logger::{Logger, Level};

    #[ink(storage)]
    pub struct Signing {}

    static LOGGER: Logger = Logger::with_max_level(Level::Info);

    pink::register_logger!(&LOGGER);

    impl Signing {
        #[ink(constructor)]
        pub fn default() -> Self {
            pink::error!("instantiated");
            Self {}
        }

        #[ink(message)]
        pub fn test(&self) {
            pink::error!("a test message received");
            pink::warn!("test end");
        }
    }
}
