use parity_scale_codec::Encode;
use phala_types::messaging::SignedMessage;
use subxt::{tx::StaticTxPayload, utils::Encoded};

pub fn register_worker(
    pruntime_info: Vec<u8>,
    attestation: Vec<u8>,
) -> StaticTxPayload<Encoded> {
    let args = (Encoded(pruntime_info), Encoded(attestation)).encode();
    StaticTxPayload::new(
        "PhalaRegistry",
        "register_worker",
        Encoded(args),
        Default::default(),
    )
    .unvalidated()
}

pub fn update_worker_endpoint(
    signed_endpoint: Vec<u8>,
    signature: Vec<u8>,
) -> StaticTxPayload<Encoded> {
    let args = (Encoded(signed_endpoint), signature).encode();
    StaticTxPayload::new(
        "PhalaRegistry",
        "update_worker_endpoint",
        Encoded(args),
        Default::default(),
    )
    .unvalidated()
}

pub fn sync_offchain_message(message: SignedMessage) -> StaticTxPayload<SignedMessage> {
    StaticTxPayload::new(
        "PhalaMq",
        "sync_offchain_message",
        message,
        Default::default(),
    )
    .unvalidated()
}
