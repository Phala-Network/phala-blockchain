use anyhow::{anyhow, Result};
use codec::{Decode, Encode};
use phactory_api::{
    crypto::{CertificateBody, EncryptedData},
    prpc,
};
use phala_crypto::ecdh::EcdhPublicKey;
use phala_crypto::sr25519::KDF;
use phala_types::contract;
use phala_types::contract::ContractId;
use sp_core::Pair as _;
use std::convert::TryFrom as _;

pub async fn query<Request: Encode, Response: Decode>(
    url: String,
    id: ContractId,
    data: Request,
) -> Result<Response> {
    // 2. Make ContractQuery
    let nonce = [1; 32];
    let head = contract::ContractQueryHead { id, nonce };
    let query = contract::ContractQuery { head, data };

    let pr = phactory_api::pruntime_client::new_pruntime_client(url);

    let info = pr.get_info(()).await?;
    let remote_pubkey = info
        .system
        .ok_or(anyhow!("Worker not initialized"))?
        .ecdh_public_key;
    let remote_pubkey = super::try_decode_hex(&remote_pubkey)?;
    let remote_pubkey = EcdhPublicKey::try_from(&remote_pubkey[..])?;

    // 3. Encrypt the ContractQuery.

    let ecdh_key = sp_core::sr25519::Pair::generate()
        .0
        .derive_ecdh_key()
        .or(Err(anyhow!("Derive ecdh key failed")))?;

    let iv = [1; 12];
    let encrypted_data = EncryptedData::encrypt(&ecdh_key, &remote_pubkey, iv, &query.encode())
        .or(Err(anyhow!("Encrypt data failed")))?;

    // 4. Sign the encrypted data.
    // 4.1 Make the root certificate.
    let (root_key, _) = sp_core::sr25519::Pair::generate();
    let root_cert_body = CertificateBody {
        pubkey: root_key.public().to_vec(),
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
        signature: root_key.sign(&data_cert_body.encode()).0.to_vec(),
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
