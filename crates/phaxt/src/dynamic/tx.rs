use parity_scale_codec::Encode;
use phala_types::messaging::SignedMessage;
use subxt::{tx::TxPayload, utils::Encoded};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedPayload {
    pallet_name: &'static str,
    call_name: &'static str,
    call_data: Encoded,
}

impl EncodedPayload {
    pub fn new(pallet_name: &'static str, call_name: &'static str, call_data: Vec<u8>) -> Self {
        Self {
            pallet_name,
            call_name,
            call_data: Encoded(call_data),
        }
    }
}

impl TxPayload for EncodedPayload {
    fn encode_call_data_to(
        &self,
        metadata: &subxt::Metadata,
        out: &mut Vec<u8>,
    ) -> Result<(), subxt::Error> {
        let pallet = metadata.pallet_by_name_err(self.pallet_name)?;
        let call = pallet.call_variant_by_name(self.call_name).ok_or_else(|| {
            subxt::error::MetadataError::CallNameNotFound((*self.call_name).to_owned())
        })?;

        let pallet_index = pallet.index();
        let call_index = call.index;

        pallet_index.encode_to(out);
        call_index.encode_to(out);
        self.call_data.encode_to(out);
        Ok(())
    }
}

pub fn register_worker(pruntime_info: Vec<u8>, attestation: Vec<u8>, v2: bool) -> EncodedPayload {
    let call_name = if v2 {
        "register_worker_v2"
    } else {
        "register_worker"
    };
    EncodedPayload::new(
        "PhalaRegistry",
        call_name,
        (Encoded(pruntime_info), Encoded(attestation)).encode(),
    )
}

pub fn update_worker_endpoint(signed_endpoint: Vec<u8>, signature: Vec<u8>) -> EncodedPayload {
    let args = (Encoded(signed_endpoint), signature).encode();
    EncodedPayload::new("PhalaRegistry", "update_worker_endpoint", args)
}

pub fn sync_offchain_message(message: SignedMessage) -> EncodedPayload {
    let args = message.encode();
    EncodedPayload::new("PhalaMq", "sync_offchain_message", args)
}
