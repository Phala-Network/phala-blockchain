//! This crate defines a set of traits which describe the functionality of
//! [block ciphers][1] and [stream ciphers][2].
//!
//! [1]: https://en.wikipedia.org/wiki/Block_cipher
//! [2]: https://en.wikipedia.org/wiki/Stream_cipher

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(all(feature = "mesalock_sgx", not(target_env = "sgx")))]
#[macro_use]
extern crate sgx_tstd as std;

#[cfg(all(feature = "std", not(feature = "mesalock_sgx")))]
#[macro_use]
extern crate std;

pub mod block;
pub mod stream;

pub use crate::{
    block::{BlockCipher, BlockCipherMut, NewBlockCipher},
    stream::{NewStreamCipher, StreamCipher, SyncStreamCipher, SyncStreamCipherSeek},
};
pub use generic_array::{self, typenum::consts};
