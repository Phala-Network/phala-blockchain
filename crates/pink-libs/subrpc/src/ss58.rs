use crate::traits::common::Error;
use alloc::{string::String, vec, vec::Vec};
use base58::ToBase58;
use ss58_registry::Ss58AddressFormat;

pub trait Ss58Codec: Sized + AsMut<[u8]> + AsRef<[u8]> {
    fn to_ss58check_with_version(&self, version: u16) -> String {
        // We mask out the upper two bits of the ident - SS58 Prefix currently only supports 14-bits
        let ident: u16 = version & 0b0011_1111_1111_1111;
        let mut v = match ident {
            0..=63 => vec![ident as u8],
            64..=16_383 => {
                // upper six bits of the lower byte(!)
                let first = ((ident & 0b0000_0000_1111_1100) as u8) >> 2;
                // lower two bits of the lower byte in the high pos,
                // lower bits of the upper byte in the low pos
                let second = ((ident >> 8) as u8) | ((ident & 0b0000_0000_0000_0011) as u8) << 6;
                vec![first | 0b01000000, second]
            }
            _ => unreachable!("masked out the upper two bits; qed"),
        };
        v.extend(self.as_ref());
        let r = ss58hash(&v);
        v.extend(&r[0..2]);
        v.to_base58()
    }
}

const PREFIX: &[u8] = b"SS58PRE";

fn ss58hash(data: &[u8]) -> [u8; 64] {
    let mut preimage: Vec<u8> = Vec::with_capacity(PREFIX.len() + data.len());
    preimage.extend_from_slice(PREFIX);
    preimage.extend_from_slice(data);
    sp_core_hashing::blake2_512(&preimage)
}

impl Ss58Codec for [u8; 32] {}

pub fn get_ss58addr_version(chain: &str) -> core::result::Result<Ss58AddressFormat, Error> {
    let chain = if chain.to_lowercase() == "khala" {
        "phala"
    } else {
        chain
    };
    Ss58AddressFormat::try_from(chain).or(Err(Error::Ss58))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ss58_works() {
        let version = get_ss58addr_version("khala").unwrap();
        assert_eq!(30, version.prefix());

        // Alice in Phala
        let alice =
            hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
                .to_ss58check_with_version(30);
        assert_eq!(alice, "45R2pfjQUW2s9PQRHU48HQKLKHVMaDja7N3wpBtmF28UYDs2");
    }
}
