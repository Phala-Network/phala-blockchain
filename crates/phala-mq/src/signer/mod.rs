use alloc::vec::Vec;

use crate::types::Message;

pub trait Pen {
    fn sign(&self, data: &[u8]) -> Vec<u8>;
}

pub trait MessageSigner {
    fn sign(&self, sequence: u64, message: &Message) -> Vec<u8>;
}

#[cfg(feature = "signers")]
mod signers {
    use super::Pen;
    use crate::{MessageSigner, Message};
    use alloc::prelude::v1::Vec;
    use parity_scale_codec::Encode;

    struct ConcatSignerWithPen<P: Pen>(P);

    impl<P: Pen> MessageSigner for ConcatSignerWithPen<P> {
        fn sign(&self, sequence: u64, message: &Message) -> Vec<u8> {
            let mut buf = Vec::new();
            sequence.encode_to(&mut buf);
            message.encode_to(&mut buf);
            self.0.sign(buf.as_slice())
        }
    }
}