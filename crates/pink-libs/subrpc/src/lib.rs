//! Substrate json RPC module with limited functionalites
//!
//! TODO: need further polish

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use pink_extension::chain_extension::{signing, SigType};
use scale::{Compact, Encode};

use pink_json as json;

mod objects;
mod rpc;
mod ss58;
pub mod storage;
mod transaction;

use objects::*;
use rpc::call_rpc;
use ss58::{get_ss58addr_version, Ss58Codec};
use transaction::{MultiAddress, MultiSignature, Signature, UnsignedExtrinsic};

pub mod traits {
    pub mod common {
        use scale::{Decode, Encode};

        #[derive(Clone, Encode, Decode, Eq, PartialEq, Debug)]
        #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
        pub enum Error {
            BadAbi,
            BadOrigin,
            AssetAlreadyRegistered,
            AssetNotFound,
            ChainAlreadyRegistered,
            ChainNotFound,
            ExtractLocationFailed,
            InvalidAddress,
            ConstructContractFailed,
            FetchDataFailed,
            Unimplemented,
            InvalidMultilocation,
            InvalidAmount,
            SubRPCRequestFailed,
            InvalidBody,
            InvalidSignature,
            Ss58,
        }
    }
}
use traits::common::Error;

use primitive_types::H256;
pub type Result<T> = core::result::Result<T, Error>;

/// Gets the storage from the give RPC node
pub fn get_storage(rpc_node: &str, key: &[u8], at: Option<H256>) -> Result<Option<Vec<u8>>> {
    let hex_key = format!("0x{}", hex::encode(key));
    let maybe_hex_at = at.map_or("null".to_string(), |h| format!("\"0x{:x}\"", h));
    let data = format!(
        r#"{{"id":1,"jsonrpc":"2.0","method":"state_getStorage","params":["{hex_key}", {maybe_hex_at}]}}"#
    )
    .into_bytes();
    let resp_body = call_rpc(rpc_node, data)?;
    let resp: GetStorageResponse = json::from_slice(&resp_body).or(Err(Error::InvalidBody))?;
    match resp.result {
        Some(h) => hex::decode(&h[2..]).map(Some).or(Err(Error::InvalidBody)),
        None => Ok(None),
    }
}

/// Gets the next nonce of the target account
///
/// Nonce represents how many transactions the account has successfully issued
/// TODO: simplify
pub fn get_next_nonce(rpc_node: &str, ss58_addr: &str) -> core::result::Result<NextNonceOk, Error> {
    // TODO: can we contruct the json object using serde_json_core?
    let data = format!(
        r#"{{"id":1,"jsonrpc":"2.0","method":"system_accountNextIndex","params":["{}"]}}"#,
        ss58_addr
    )
    .into_bytes();
    let resp_body = call_rpc(rpc_node, data)?;

    let next_nonce: NextNonce = json::from_slice(&resp_body).or(Err(Error::InvalidBody))?;

    let next_nonce_ok = NextNonceOk {
        next_nonce: next_nonce.result,
    };

    Ok(next_nonce_ok)
}

// TODO: simplify
pub fn get_runtime_version(rpc_node: &str) -> core::result::Result<RuntimeVersionOk, Error> {
    let data = r#"{"id":1, "jsonrpc":"2.0", "method": "state_getRuntimeVersion"}"#
        .to_string()
        .into_bytes();
    let resp_body = call_rpc(rpc_node, data)?;

    let runtime_version: RuntimeVersion =
        json::from_slice(&resp_body).or(Err(Error::InvalidBody))?;

    let runtime_version_result = runtime_version.result;
    let mut api_vec: Vec<(String, u32)> = Vec::new();
    for (api_str, api_u32) in runtime_version_result.apis {
        api_vec.push((api_str.to_string().parse().unwrap(), api_u32));
    }

    let runtime_version_ok = RuntimeVersionOk {
        // TODO: replace the upwraps
        spec_name: runtime_version_result
            .spec_name
            .to_string()
            .parse()
            .unwrap(),
        impl_name: runtime_version_result
            .impl_name
            .to_string()
            .parse()
            .unwrap(),
        authoring_version: runtime_version_result.authoring_version,
        spec_version: runtime_version_result.spec_version,
        impl_version: runtime_version_result.impl_version,
        apis: api_vec,
        transaction_version: runtime_version_result.transaction_version,
        state_version: runtime_version_result.state_version,
    };
    Ok(runtime_version_ok)
}

/// Gets the block hash at block zero
pub fn get_genesis_hash(rpc_node: &str) -> core::result::Result<H256, Error> {
    get_block_hash(rpc_node, Some(0))
}

/// Gets the block hash at a certain height (None for the latest block)
pub fn get_block_hash(
    rpc_node: &str,
    block_number: Option<u32>,
) -> core::result::Result<H256, Error> {
    let param = block_number.map_or("null".to_string(), |n| format!("{n}"));
    let data = format!(
        r#"{{"id":1, "jsonrpc":"2.0", "method": "chain_getBlockHash","params":[{param}]}}"#
    )
    .into_bytes();
    let resp_body = call_rpc(rpc_node, data)?;
    let genesis_hash: GenesisHash = json::from_slice(&resp_body).or(Err(Error::InvalidBody))?;
    // bypass prefix 0x
    let genesis_hash_result = &genesis_hash.result[2..];
    let decoded_hash = hex::decode(genesis_hash_result).or(Err(Error::InvalidBody))?;
    let hash: [u8; 32] = decoded_hash.try_into().or(Err(Error::InvalidBody))?;
    Ok(H256(hash))
}

/// Creates an extrinsic
///
/// An extended version of `create_transaction`, fine-grain
#[allow(clippy::too_many_arguments)]
pub fn create_transaction_ext<T: Encode>(
    signer: &[u8; 32],
    public_key: &[u8; 32],
    nonce: u64,
    spec_version: u32,
    transaction_version: u32,
    genesis_hash: &[u8; 32],
    call_data: UnsignedExtrinsic<T>,
    era: Era,
    tip: u128,
) -> core::result::Result<Vec<u8>, Error> {
    let additional_params = (
        spec_version,
        transaction_version,
        genesis_hash,
        genesis_hash,
    );
    let extra = (era, Compact(nonce), Compact(tip));

    let mut bytes = Vec::new();
    call_data.encode_to(&mut bytes);
    extra.encode_to(&mut bytes);
    additional_params.encode_to(&mut bytes);

    let signature = if bytes.len() > 256 {
        signing::sign(
            &sp_core_hashing::blake2_256(&bytes),
            signer,
            SigType::Sr25519,
        )
    } else {
        signing::sign(&bytes, signer, SigType::Sr25519)
    };

    let signature_type =
        Signature::try_from(signature.as_slice()).or(Err(Error::InvalidSignature))?;
    let multi_signature = MultiSignature::Sr25519(signature_type);

    let src_account_id: MultiAddress<[u8; 32], u32> = transaction::MultiAddress::Id(*public_key);

    // Encode Extrinsic
    let extrinsic = {
        let mut encoded_inner = Vec::new();
        // "is signed" + tx protocol v4
        (0b10000000 + 4u8).encode_to(&mut encoded_inner);
        // from address for signature
        src_account_id.encode_to(&mut encoded_inner);
        // the signature bytes
        multi_signature.encode_to(&mut encoded_inner);
        // attach custom extra params
        extra.encode_to(&mut encoded_inner);
        // and now, call data
        call_data.encode_to(&mut encoded_inner);
        // now, prefix byte length:
        let len = Compact(
            u32::try_from(encoded_inner.len()).expect("extrinsic size expected to be <4GB"),
        );
        let mut encoded = Vec::new();
        len.encode_to(&mut encoded);
        encoded.extend(encoded_inner);
        encoded
    };

    Ok(extrinsic)
}

pub fn create_transaction<T: Encode>(
    signer: &[u8; 32],
    chain: &str,
    rpc_node: &str,
    pallet_id: u8,
    call_id: u8,
    data: T,
) -> core::result::Result<Vec<u8>, Error> {
    let version = get_ss58addr_version(chain)?;
    let public_key: [u8; 32] = signing::get_public_key(signer, SigType::Sr25519)
        .try_into()
        .unwrap();
    let addr = public_key.to_ss58check_with_version(version.prefix());
    let nonce = get_next_nonce(rpc_node, &addr)?.next_nonce;
    let runtime_version = get_runtime_version(rpc_node)?;
    let genesis_hash: [u8; 32] = get_genesis_hash(rpc_node)?.0;
    let spec_version = runtime_version.spec_version;
    let transaction_version = runtime_version.transaction_version;
    let era = Era::Immortal;
    let tip: u128 = 0;
    let call_data = UnsignedExtrinsic {
        pallet_id,
        call_id,
        call: data,
    };
    create_transaction_ext(
        signer,
        &public_key,
        nonce,
        spec_version,
        transaction_version,
        &genesis_hash,
        call_data,
        era,
        tip,
    )
}

pub fn send_transaction(rpc_node: &str, signed_tx: &[u8]) -> core::result::Result<Vec<u8>, Error> {
    let tx_hex = hex::encode(signed_tx);
    let data = format!(
        r#"{{"id":1,"jsonrpc":"2.0","method":"author_submitExtrinsic","params":["{}"]}}"#,
        tx_hex
    )
    .into_bytes();
    let resp_body = call_rpc(rpc_node, data)?;
    let resp: TransactionResponse = json::from_slice(&resp_body).or(Err(Error::InvalidBody))?;
    hex::decode(&resp.result[2..]).or(Err(Error::InvalidBody))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use scale::{Compact, Encode};

    /// Test data:
    ///
    /// subkey inspect 0x9eb2ee60393aeeec31709e256d448c9e40fa64233abf12318f63726e9c417b69 --scheme sr25519 --network kusama
    /// Secret Key URI `0x9eb2ee60393aeeec31709e256d448c9e40fa64233abf12318f63726e9c417b69` is account:
    ///   Network ID:        kusama
    ///   Secret seed:       0x9eb2ee60393aeeec31709e256d448c9e40fa64233abf12318f63726e9c417b69
    ///   Public key (hex):  0x8266b3183ccc58f3d145d7a4894547bd55d7739751dd15802f36ec8a0d7be314
    ///   Account ID:        0x8266b3183ccc58f3d145d7a4894547bd55d7739751dd15802f36ec8a0d7be314
    ///   Public key (SS58): FXJFWSVDcyVi3bTy8D9ESznQM4JoNBRQLEjWFgAGnGQfpbR
    ///   SS58 Address:      FXJFWSVDcyVi3bTy8D9ESznQM4JoNBRQLEjWFgAGnGQfpbR
    #[test]
    fn can_get_next_nonce() {
        pink_extension_runtime::mock_ext::mock_all_ext();
        let version = get_ss58addr_version("kusama").unwrap();
        let public_key: [u8; 32] =
            hex_literal::hex!("8266b3183ccc58f3d145d7a4894547bd55d7739751dd15802f36ec8a0d7be314");
        let addr = public_key.to_ss58check_with_version(version.prefix());
        let _next_nonce = get_next_nonce("https://kusama-rpc.polkadot.io", &addr).unwrap();
    }

    #[test]
    fn can_get_runtime_version() {
        pink_extension_runtime::mock_ext::mock_all_ext();
        let runtime_version = get_runtime_version("https://kusama-rpc.polkadot.io").unwrap();
        assert_eq!(runtime_version.impl_name, "parity-kusama");
    }

    #[test]
    fn can_get_genesis_hash() {
        pink_extension_runtime::mock_ext::mock_all_ext();
        let genesis_hash = get_genesis_hash("https://kusama-rpc.polkadot.io").unwrap();
        assert_eq!(
            hex::encode(genesis_hash),
            "b0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe"
        );
    }

    #[test]
    fn can_correctly_encode() {
        let genesis_hash: [u8; 32] =
            hex!("ccd5874826c67d06b979c08a14c006f938a2fef6cba3eec5f8ba38d98931209d");
        let spec_version: u32 = 1;
        let transaction_version: u32 = 1;
        let era = Era::Immortal;
        let tip: u128 = 0;
        let nonce: u64 = 0;

        let extra = (era, Compact(nonce), Compact(tip));
        {
            let mut bytes = Vec::new();
            extra.encode_to(&mut bytes);
            let expected: Vec<u8> = hex!("000000").into();
            assert_eq!(bytes, expected);
        }

        let additional_params = (
            spec_version,
            transaction_version,
            genesis_hash,
            genesis_hash,
        );
        {
            let mut bytes = Vec::new();
            additional_params.encode_to(&mut bytes);
            let expected: Vec<u8> = hex!("0100000001000000ccd5874826c67d06b979c08a14c006f938a2fef6cba3eec5f8ba38d98931209dccd5874826c67d06b979c08a14c006f938a2fef6cba3eec5f8ba38d98931209d").into();
            assert_eq!(bytes, expected);
        }

        pink_extension_runtime::mock_ext::mock_all_ext();
        let signer =
            hex!("9eb2ee60393aeeec31709e256d448c9e40fa64233abf12318f63726e9c417b69").to_vec();
        let public_key: [u8; 32] = signing::get_public_key(&signer, SigType::Sr25519)
            .try_into()
            .unwrap();
        let account_id: MultiAddress<[u8; 32], u32> = transaction::MultiAddress::Id(public_key);
        {
            let mut bytes = Vec::new();
            account_id.encode_to(&mut bytes);
            let expected =
                hex!("008266b3183ccc58f3d145d7a4894547bd55d7739751dd15802f36ec8a0d7be314").to_vec();
            assert_eq!(bytes, expected);
        }
    }

    /// Sends a remark extrinsic to khala
    #[test]
    #[ignore = "only for demostration purposes"]
    fn can_send_remark() {
        pink_extension_runtime::mock_ext::mock_all_ext();
        let rpc_node = "https://khala.api.onfinality.io:443/public-ws";
        let signer: [u8; 32] =
            hex!("9eb2ee60393aeeec31709e256d448c9e40fa64233abf12318f63726e9c417b69");
        let remark = "Greetings from unit tests!".to_string();
        let signed_tx = create_transaction(&signer, "khala", rpc_node, 0u8, 1u8, remark);
        if signed_tx.is_err() {
            println!("failed to signed tx");
            return;
        };
        let signed_tx = signed_tx.unwrap();
        let tx_id = send_transaction(rpc_node, &signed_tx);
        if tx_id.is_err() {
            println!("failed to send tx");
            return;
        }
        let tx_id = tx_id.unwrap();
        // https://khala.subscan.io/extrinsic/2676952-2
        dbg!(hex::encode(tx_id));
    }

    /// Calls the xtransfer function
    #[test]
    #[ignore = "this is very expensive so we don't test it often"]
    fn can_call_xtransfer() {
        pink_extension_runtime::mock_ext::mock_all_ext();
        use xcm::v1::MultiAsset;
        use xcm::v1::{AssetId, Fungibility, Junction, Junctions, MultiLocation};

        let rpc_node = "https://rhala-api.phala.network/api";
        let signer: [u8; 32] =
            hex!("9eb2ee60393aeeec31709e256d448c9e40fa64233abf12318f63726e9c417b69");
        let recipient: Vec<u8> = hex!("8266b3183Ccc58f3D145D7a4894547bd55D77397").to_vec();
        let amount: u128 = 301_000_000_000_000;

        let multi_asset = MultiAsset {
            id: AssetId::Concrete(Junctions::Here.into()),
            fun: Fungibility::Fungible(amount),
        };

        let dest = MultiLocation::new(
            0,
            Junctions::X3(
                Junction::GeneralKey(b"cb".to_vec().try_into().unwrap()),
                Junction::GeneralIndex(0u128),
                Junction::GeneralKey(recipient.try_into().unwrap()),
            ),
        );

        let dest_weight: std::option::Option<u64> = None;

        let call_data = transaction::UnsignedExtrinsic {
            pallet_id: 0x52u8,
            call_id: 0x0u8,
            call: (multi_asset.clone(), dest.clone(), dest_weight),
        };

        let mut bytes = Vec::new();
        call_data.encode_to(&mut bytes);
        let expected: Vec<u8> = hex!("5200000000000f00d01306c21101000306086362050006508266b3183ccc58f3d145d7a4894547bd55d7739700").into();
        assert_eq!(bytes, expected);

        let signed_tx = create_transaction(
            &signer,
            "khala",
            rpc_node,
            0x52u8,
            0x0u8,
            (multi_asset, dest, dest_weight),
        );
        if signed_tx.is_err() {
            println!("failed to signed tx");
            return;
        };
        let signed_tx = signed_tx.unwrap();
        let tx_id = send_transaction(rpc_node, &signed_tx);
        if tx_id.is_err() {
            println!("failed to send tx");
            return;
        }
        let tx_id = tx_id.unwrap();
        // example output:
        //  tx id: 95d107457ab905d8187b70fac146b68a9ce87c5a3c2e10f93cf0732ffe400d20
        //  block: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frhala-api.phala.network%2Fws#/explorer/query/0x0586620d60fd5ec5d92a75ca5a095ac8a0cb66bcb4d2ff147d93e532d4d67e95
        //     or: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frhala-api.phala.network%2Fws#/explorer/query/0xa4188ef17ad0a170e5c0054191013e202cc2437f0462523e9a13989ef7829517
        dbg!(hex::encode(tx_id));
    }

    #[test]
    #[ignore = "this is very expensive so we don't test it often"]
    fn test_read_storage() {
        use std::str::FromStr;
        pink_extension_runtime::mock_ext::mock_all_ext();

        let some_block =
            H256::from_str("0xbaa0b58522c8af4acaa147604839230a57aad53b9c9f67652feeeea8a0c04679")
                .unwrap();
        let r = get_storage(
            "https://rhala-api.phala.network/api",
            &hex_literal::hex!("f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb"),
            Some(some_block),
        )
        .map(|b| b.map(|data| hex::encode(&data)));
        dbg!(r);

        let r = get_storage(
            "https://rhala-api.phala.network/api",
            &hex_literal::hex!("f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbc"),
            None,
        )
        .map(|b| b.map(|data| hex::encode(&data)));
        dbg!(r);
    }
}
