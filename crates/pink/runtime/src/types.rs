use scale::{Decode, Encode};
use sp_runtime::{traits::BlakeTwo256, AccountId32};

pub use frame_support::weights::Weight;

use crate::runtime::SystemEvents;

pub type Hash = sp_core::H256;
pub type Hashing = BlakeTwo256;
pub type AccountId = AccountId32;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type Nonce = u64;

#[derive(Encode, Decode, Clone, Debug)]
pub struct EventsBlockHeader {
    pub number: u64,
    pub runtime_version: (u32, u32),
    pub parent_hash: Hash,
    pub body_hash: Hash,
}

// The body layout may change in the future. They should be decoded as type determined by
// header.runtime_version.
#[derive(Encode, Decode, Clone, Debug)]
pub struct EventsBlockBody {
    pub phala_block_number: BlockNumber,
    pub contract_call_nonce: Option<Vec<u8>>,
    pub entry_contract: Option<AccountId>,
    pub events: SystemEvents,
    pub origin: Option<AccountId>,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct EventsBlock {
    pub header: EventsBlockHeader,
    pub body: EventsBlockBody,
}

#[test]
fn test_decode_and_verify_event_chain() {
    // The log file is greped from the pruntime log in e2e with `grep 'event_chain' pruntime0.log > events.log`
    // If this test fails, it means the event chain layout changed, consider to regenerate the log file if the
    // change is expected.
    let blocks_file = include_str!("../assets/events.log");
    let mut parent_hash = Hash::default();

    fn extract_payload(log_line: &str) -> Option<&str> {
        let payload_marker = "payload=";
        log_line
            .split_whitespace()
            .find(|segment| segment.starts_with(payload_marker))
            .and_then(|segment| segment.strip_prefix(payload_marker))
    }

    for line in blocks_file.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let payload = extract_payload(line).expect("Invalid log line");
        let bytes = hex::decode(payload).unwrap();
        let event: EventsBlock = Decode::decode(&mut &bytes[..]).unwrap();
        let header_hash: Hash = sp_core::blake2_256(&event.header.encode()).into();
        let body_hash: Hash = sp_core::blake2_256(&event.body.encode()).into();
        assert!(event.header.parent_hash == parent_hash);
        assert!(event.header.body_hash == body_hash);
        parent_hash = header_hash;
    }
}
