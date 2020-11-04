use alloc::vec::Vec;
use codec::{Encode, Decode};

#[derive(Encode, Decode)]
pub struct Transfer<AccountId, Balance> {
	pub dest: AccountId,
	pub amount: Balance,
	pub sequence: u64,
}

#[derive(Encode, Decode)]
pub struct TransferData<AccountId, Balance> {
	pub data: Transfer<AccountId, Balance>,
	pub signature: Vec<u8>,
}

#[derive(Encode, Decode)]
pub struct Heartbeat {
	pub block_num: u32,
}

#[derive(Encode, Decode)]
pub struct HeartbeatData {
	pub data: Heartbeat,
	pub signature: Vec<u8>,
}

pub trait SignedDataType<T> {
	fn raw_data(&self) -> Vec<u8>;
	fn signature(&self) -> T;
}

impl<AccountId: Encode, Balance: Encode> SignedDataType<Vec<u8>> for TransferData<AccountId, Balance> {
	fn raw_data(&self) -> Vec<u8> {
		Encode::encode(&self.data)
	}

	fn signature(&self) -> Vec<u8> {
		self.signature.clone()
	}
}

impl SignedDataType<Vec<u8>> for HeartbeatData {
	fn raw_data(&self) -> Vec<u8> {
		Encode::encode(&self.data)
	}

	fn signature(&self) -> Vec<u8> {
		self.signature.clone()
	}
}

// Types used in storage

#[derive(Encode, Decode, Default)]
pub struct WorkerInfo {
	// identity
	pub machine_id: Vec<u8>,
	pub pubkey: Vec<u8>,
	pub last_updated: u64,
	// contract
	// ...
	// mining
	pub status: i32,
	// preformance
	pub score: Option<Score>
}

#[derive(Encode, Decode, Default)]
pub struct StashInfo<AccountId: Default> {
	pub controller: AccountId,
	pub payout_prefs: PayoutPrefs::<AccountId>,
}

#[derive(Encode, Decode, Default)]
pub struct PayoutPrefs<AccountId: Default> {
	pub commission: u32,
	pub target: AccountId,
}

#[derive(Encode, Decode, Default)]
pub struct Score {
	pub overall_score: u32,
	pub features: Vec<u32>
}

type MachineId = [u8; 16];
type WorkerPublicKey = [u8; 33];
#[derive(Encode, Decode)]
pub struct PRuntimeInfo {
	pub version: u8,
	pub machine_id: MachineId,
	pub pubkey: WorkerPublicKey,
	pub features: Vec<u32>
}

#[derive(Encode, Decode, Debug, Default)]
pub struct MiningInfo<BlockNumber> {
	pub is_mining: bool,
	pub start_block: Option<BlockNumber>,
}
