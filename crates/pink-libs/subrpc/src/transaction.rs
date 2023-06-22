use alloc::{string::String, vec::Vec};
use scale::{Compact, Decode, Encode};

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct UnsignedExtrinsic<Call> {
    pub pallet_id: u8,
    pub call_id: u8,
    pub call: Call,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Remark {
    pub remark: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
struct Transfer {
    dest: MultiAddress<[u8; 32], u32>,
    currency_id: CurrencyId,
    amount: Compact<u128>,
}

/// A multi-format address wrapper for on-chain accounts.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(Hash, scale_info::TypeInfo))]
pub enum MultiAddress<AccountId, AccountIndex> {
    /// It's an account ID (pubkey).
    Id(AccountId),
    /// It's an account index.
    Index(#[codec(compact)] AccountIndex),
    /// It's some arbitrary raw bytes.
    Raw(Vec<u8>),
    /// It's a 32 byte representation.
    Address32([u8; 32]),
    /// Its a 20 byte representation.
    Address20([u8; 20]),
}

impl<AccountId, AccountIndex> From<AccountId> for MultiAddress<AccountId, AccountIndex> {
    fn from(a: AccountId) -> Self {
        Self::Id(a)
    }
}

/// A signature (a 512-bit value).
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Hash, scale_info::TypeInfo))]
pub struct Signature(pub [u8; 64]);

impl TryFrom<&[u8]> for Signature {
    type Error = ();

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() == 64 {
            let mut inner = [0u8; 64];
            inner.copy_from_slice(data);
            Ok(Signature(inner))
        } else {
            Err(())
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(Hash, scale_info::TypeInfo))]
pub enum MultiSignature {
    /// An Ed25519 signature.
    #[codec(index = 0)]
    Ed25519(Signature),
    /// An Sr25519 signature.
    #[codec(index = 1)]
    Sr25519(Signature),
    /// An ECDSA/SECP256k1 signature.
    #[codec(index = 2)]
    Ecdsa(Signature),
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CurrencyId {
    Fren,
    Gm,
    Gn,
}

/// Struct used to send an encoded transaction to the contract
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ContractCall {
    /// Contract address
    pub dest: MultiAddress<[u8; 32], u32>, //TODO AccountId
    /// Only for payable messages, call will fail otherwise
    #[codec(compact)]
    pub value: u128, //TODO Balance
    /// Maximum gas to be consumed. If it is too small the extrinsic will fail
    pub gas_limit: WeightV2,
    /// A limit to how much Balance to be used to pay for the storage created by the contract call.
    /// if None is passed, unlimited balance can be used
    pub storage_deposit_limit: Option<Compact<u128>>, //TODO Balance
    /// data: method name + args
    pub data: Vec<u8>,
}

/// Gas to be consumed: gaz = ref_time * proof_size
#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct WeightV2 {
    #[codec(compact)]
    pub ref_time: u64,
    #[codec(compact)]
    pub proof_size: u64,
}


/// Struct used to query a wasm contract
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ContractQuery<AccountId, Balance> {
    pub origin: AccountId,
    pub dest: AccountId,
    pub value: Balance,
    pub gas_limit: Option<WeightV2>,
    pub storage_deposit_limit: Option<Balance>,
    pub data: Vec<u8>,
}


/// Result when we query a wasm contract
#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ContractQueryResult<Error, Balance>{
    pub gas_consumed: WeightV2,
    pub gas_required: WeightV2,
    pub storage_deposit: StorageDeposit<Balance>,
    pub debug_message: Vec<u8>,
    pub result: ExecReturnValue<Error>,
}

/*
#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    Error1,
    Error2,
}
 */

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ExecReturnValue<Error> {
    pub flags: u32,
    pub data: core::result::Result<Vec<u8>, Error>,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum StorageDeposit<Balance> {
    Refund(Balance),
    Charge(Balance),
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Encoded(pub Vec<u8>);
