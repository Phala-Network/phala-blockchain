use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::{anyhow, Result, Context};
use log::info;
use phala_crypto::ecdh::EcdhPublicKey;
use phala_mq::ContractId;
use phala_types::contract;

use crate::crypto::{CertificateBody, EncryptedData};
use crate::prpc::{
    self,
    client::{Error as ClientError, RequestClient},
    phactory_api_client::PhactoryApiClient,
    server::ProtoError as ServerError,
    Message,
};

use core::convert::TryFrom;
use phala_crypto::sr25519::KDF;
use rand::Rng;
use sp_core::{Decode, Encode, Pair};

pub type PRuntimeClient = PhactoryApiClient<RpcRequest>;

pub fn new_pruntime_client(base_url: String) -> PhactoryApiClient<RpcRequest> {
    PhactoryApiClient::new(RpcRequest::new(base_url))
}

pub struct RpcRequest {
    base_url: String,
}

impl RpcRequest {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

#[async_trait::async_trait]
impl RequestClient for RpcRequest {
    async fn request(&self, path: &str, body: Vec<u8>) -> Result<Vec<u8>, ClientError> {
        fn from_display(err: impl core::fmt::Display) -> ClientError {
            ClientError::RpcError(err.to_string())
        }

        let url = alloc::format!("{}/prpc/{}", self.base_url, path);
        let res = reqwest::Client::new()
            .post(url)
            .header("Connection", "close")
            .body(body)
            .send()
            .await
            .map_err(from_display)?;

        info!("Response: {}", res.status());
        let status = res.status();
        let body = res.bytes().await.map_err(from_display)?;
        if status.is_success() {
            Ok(body.as_ref().to_vec())
        } else {
            let err: ServerError = Message::decode(body.as_ref())?;
            Err(ClientError::ServerError(err))
        }
    }
}

pub async fn query<Request: Encode, Response: Decode>(
    url: String,
    id: ContractId,
    data: Request,
    private_key: &sp_core::sr25519::Pair,
) -> Result<Response> {
    let pr = new_pruntime_client(url);

    let info = pr.get_info(()).await?;
    let remote_pubkey = &info
        .ecdh_public_key
        .ok_or(anyhow!("Worker not initialized"))?;
    let remote_pubkey = hex::decode(remote_pubkey.strip_prefix("0x").unwrap_or(remote_pubkey))
        .context("Invalid public key")?;
    let remote_pubkey = EcdhPublicKey::try_from(&remote_pubkey[..])?;

    // 2. Make ContractQuery
    let nonce = rand::thread_rng().gen();
    let head = contract::ContractQueryHead { id, nonce };
    let query = contract::ContractQuery { head, data };

    // 3. Encrypt the ContractQuery.
    let ecdh_key = private_key
        .derive_ecdh_key()
        .or(Err(anyhow!("Derive ecdh key failed")))?;

    let iv = rand::thread_rng().gen();
    let encrypted_data = EncryptedData::encrypt(&ecdh_key, &remote_pubkey, iv, &query.encode())
        .or(Err(anyhow!("Encrypt data failed")))?;

    // 4. Sign the encrypted data.
    // 4.1 Make the root certificate.
    let root_cert_body = CertificateBody {
        pubkey: private_key.public().to_vec(),
        ttl: u32::MAX,
        config_bits: 0,
    };
    let root_cert = prpc::Certificate::new(root_cert_body, None);

    // 4.2 Generate a temporary key pair and sign it with root key.
    let (key_g, _) = sp_core::sr25519::Pair::generate();

    let data_cert_body = CertificateBody {
        pubkey: key_g.public().to_vec(),
        ttl: u32::MAX,
        config_bits: 0,
    };
    let cert_signature = prpc::Signature {
        signed_by: Some(Box::new(root_cert)),
        signature_type: prpc::SignatureType::Sr25519 as _,
        signature: private_key.sign(&data_cert_body.encode()).0.to_vec(),
    };
    let data_cert = prpc::Certificate::new(data_cert_body, Some(Box::new(cert_signature)));
    let data_signature = prpc::Signature {
        signed_by: Some(Box::new(data_cert)),
        signature_type: prpc::SignatureType::Sr25519 as _,
        signature: key_g.sign(&encrypted_data.encode()).0.to_vec(),
    };

    let request = prpc::ContractQueryRequest::new(encrypted_data, Some(data_signature));

    // 5. Do the RPC call.
    let response = pr.contract_query(request).await?;

    // 6. Decrypt the response.
    let encrypted_data = response.decode_encrypted_data()?;
    let data = encrypted_data
        .decrypt(&ecdh_key)
        .or(Err(anyhow!("Decrypt data failed")))?;

    // 7. Decode the response.
    let response: contract::ContractQueryResponse<Response> = Decode::decode(&mut &data[..])?;

    // 8. check the nonce is match the one we sent.
    if response.nonce != nonce {
        return Err(anyhow!("nonce mismatch"));
    }
    Ok(response.result)
}

pub fn gen_key() -> sp_core::sr25519::Pair {
    sp_core::sr25519::Pair::generate().0
}
