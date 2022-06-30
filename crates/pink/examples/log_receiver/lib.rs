#![cfg_attr(not(feature = "std"), no_std)]

use pink_extension as pink;

#[pink::contract]
mod contract {
    use super::pink;

    use pink::logger::{Level, Logger};

    static LOGGER: Logger = Logger::with_max_level(Level::Info);

    pink::register_logger!(&LOGGER);

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn default() -> Self {
            let code = &include_bytes!("./sideprog.wasm")[..];
            pink::start_sidevm(code.into(), true);
            Self {}
        }
        #[pink(on_block_end)]
        pub fn on_block_end(&self) {
            let number = self.env().block_number();
            pink::info!("on block {} end", number);
        }
    }
}
