#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use async_trait::async_trait;
use derive_more::Display;
use prost::DecodeError;

pub use prost::Message;

pub mod server {
    use super::*;

    #[derive(Display, Debug)]
    pub enum Error {
        NotFound,
        DecodeError(DecodeError),
        AppError(String),
    }

    impl From<DecodeError> for Error {
        fn from(e: DecodeError) -> Self {
            Self::DecodeError(e)
        }
    }

    /// Error in protobuf format
    #[derive(Display, Message)]
    pub struct ProtoError {
        #[prost(string, tag = "1")]
        pub message: ::prost::alloc::string::String,
    }
}

pub mod client {
    use super::*;

    #[derive(Display, Debug)]
    pub enum Error {
        DecodeError(DecodeError),
        RpcError(String),
    }

    impl From<DecodeError> for Error {
        fn from(e: DecodeError) -> Self {
            Self::DecodeError(e)
        }
    }

    #[async_trait]
    pub trait RequestClient {
        async fn request(&self, path: &str, body: impl AsRef<[u8]>) -> Result<Vec<u8>, Error>;
    }
}

pub mod codec {
    use super::*;

    pub fn encode_message_to_vec(msg: &impl Message) -> Vec<u8> {
        let mut buf = Vec::with_capacity(msg.encoded_len());

        msg.encode_raw(&mut buf);
        buf
    }
}
