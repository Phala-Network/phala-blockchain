use anyhow::{anyhow, Result};
use phactory_api::{
    contracts::{Query, QueryError, Response},
    crypto::{CertificateBody, EncryptedData},
    prpc,
};
use phala_crypto::ecdh::EcdhPublicKey;
use phala_crypto::sr25519::KDF;
use phala_types::contract;
use phala_types::contract::ContractId;
use scale::{Decode, Encode};
use sp_core::Pair as _;
use std::convert::TryFrom as _;
use tracing::debug;

#[derive(Debug, Encode, Decode)]
pub enum Command {
    InkMessage { nonce: Vec<u8>, message: Vec<u8> },
}

#[derive(Debug, Encode, Decode)]
pub enum LangError {
    CouldNotReadInput,
}

impl std::fmt::Display for LangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LangError::CouldNotReadInput => write!(f, "Could not read input"),
        }
    }
}
impl std::error::Error for LangError {}

pub async fn pink_query<I: Encode, O: Decode>(
    worker_pubkey: &[u8],
    url: &str,
    id: ContractId,
    selector: u32,
    args: I,
    key: &sp_core::sr25519::Pair,
) -> Result<O> {
    let call_data = (selector.to_be_bytes(), args).encode();
    let payload = pink_query_raw(worker_pubkey, url, id, call_data, key).await??;
    let output = crate::primitives::ContractExecResult::<u128>::decode(&mut &payload[..])?
        .result
        .map_err(|err| anyhow::anyhow!("DispatchError({err:?})"))?;
    if output.did_revert() {
        debug!("Contract execution reverted, output={:?}", output.data);
    }
    let r = Result::<O, LangError>::decode(&mut &output.data[..])??;
    Ok(r)
}

pub async fn pink_query_raw(
    worker_pubkey: &[u8],
    url: &str,
    id: ContractId,
    call_data: Vec<u8>,
    key: &sp_core::sr25519::Pair,
) -> Result<Result<Vec<u8>, QueryError>> {
    let query = Query::InkMessage {
        payload: call_data,
        deposit: 1_000_000_000_000_000_u128,
        transfer: 0,
        estimating: false,
    };
    let result: Result<Response, QueryError> =
        contract_query(worker_pubkey, url, id, query, key).await?;
    Ok(result.map(|r| {
        let Response::Payload(payload) = r;
        payload
    }))
}

pub async fn contract_query<Request: Encode, Response: Decode>(
    worker_pubkey: &[u8],
    url: &str,
    id: ContractId,
    data: Request,
    key: &sp_core::sr25519::Pair,
) -> Result<Response> {
    // 2. Make ContractQuery
    let nonce = rand::random::<[u8; 32]>();
    let head = contract::ContractQueryHead { id, nonce };
    let query = contract::ContractQuery { head, data };

    let pr = phactory_api::pruntime_client::new_pruntime_client_no_log(url.into());

    let remote_pubkey = EcdhPublicKey::try_from(worker_pubkey)?;

    // 3. Encrypt the ContractQuery.

    let ecdh_key = sp_core::sr25519::Pair::generate()
        .0
        .derive_ecdh_key()
        .map_err(|_| anyhow!("Derive ecdh key failed"))?;

    let iv = rand::random();
    let encrypted_data = EncryptedData::encrypt(&ecdh_key, &remote_pubkey, iv, &query.encode())
        .map_err(|_| anyhow!("Encrypt data failed"))?;

    let data_cert_body = CertificateBody {
        pubkey: key.public().to_vec(),
        ttl: u32::MAX,
        config_bits: 0,
    };
    let data_cert = prpc::Certificate::new(data_cert_body, None);
    let data_signature = prpc::Signature {
        signed_by: Some(Box::new(data_cert)),
        signature_type: prpc::SignatureType::Sr25519 as _,
        signature: key.sign(&encrypted_data.encode()).0.to_vec(),
    };

    let request = prpc::ContractQueryRequest::new(encrypted_data, Some(data_signature));

    // 5. Do the RPC call.
    let response = pr.contract_query(request).await?;

    // 6. Decrypt the response.
    let encrypted_data = response.decode_encrypted_data()?;
    let data = encrypted_data
        .decrypt(&ecdh_key)
        .map_err(|_| anyhow!("Decrypt data failed"))?;

    // 7. Decode the response.
    let response: contract::ContractQueryResponse<Response> = Decode::decode(&mut &data[..])?;

    // 8. check the nonce is match the one we sent.
    if response.nonce != nonce {
        return Err(anyhow!("nonce mismatch"));
    }
    Ok(response.result)
}
