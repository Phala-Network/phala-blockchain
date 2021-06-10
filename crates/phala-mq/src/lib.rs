#![no_std]

extern crate alloc;


pub mod types {
    use alloc::vec::Vec;

    pub enum Origin {
        /// Runtime pallets
        Runtime,
        /// A confidential contract running in some pRuntime.
        Contract(Vec<u8>),
        /// An chain user
        Account(Vec<u8>),
        /// A remote location (parachain, etc.)
        Multilocaiton(Vec<u8>)
    }

    pub struct Message {
        pub sender: Origin,
        pub sequence: u64,
        pub dst_recource: Vec<u8>,
        pub payload: Vec<u8>,
    }

    pub struct SignedMessage {
        pub message: Message,
        pub signature: Vec<u8>,
    }
}

