use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{bounded::BoundedVec, ConstU32};

pub use phala_mq::{ContractClusterId, ContractId};

pub type ContractId32 = u32;
pub const SYSTEM: ContractId32 = 0;
pub const DATA_PLAZA: ContractId32 = 1;
pub const BALANCES: ContractId32 = 2;
pub const ASSETS: ContractId32 = 3;
pub const WEB3_ANALYTICS: ContractId32 = 4;
pub const _DIEM: ContractId32 = 5;
pub const SUBSTRATE_KITTIES: ContractId32 = 6;
pub const BTC_LOTTERY: ContractId32 = 7;
pub const GEOLOCATION: ContractId32 = 8;
pub const GUESS_NUMBER: ContractId32 = 100;
pub const BTC_PRICE_BOT: ContractId32 = 101;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum CodeIndex<CodeHash> {
    WasmCode(CodeHash),
}

#[derive(Decode, Encode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum InkCommand {
    InkMessage {
        nonce: BoundedVec<u8, ConstU32<32>>,
        message: Vec<u8>,
        // Amount of tokens transfer to the target contract
        transfer: u128,
        // Max value gas allowed to be consumed
        gas_limit: u64,
        // Max value token allowed to be deposited to the storage usage
        storage_deposit_limit: Option<u128>,
    },
}

impl<CodeHash: AsRef<[u8]>> CodeIndex<CodeHash> {
    pub fn code_hash(&self) -> Vec<u8> {
        match self {
            CodeIndex::WasmCode(code_hash) => code_hash.as_ref().to_vec(),
        }
    }
}

pub mod messaging {
    use alloc::{collections::BTreeMap, vec::Vec};
    use codec::{Decode, Encode};
    use core::fmt::Debug;
    use scale_info::TypeInfo;

    use super::{ContractClusterId, ContractInfo};
    use crate::messaging::EncryptedKey;
    use crate::{ClusterPublicKey, WorkerIdentity, WorkerPublicKey};
    use phala_mq::bind_topic;
    use sp_core::crypto::AccountId32;

    bind_topic!(ClusterEvent, b"phala/cluster/event");
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, ::scale_info::TypeInfo)]
    pub enum ClusterEvent {
        // TODO.shelven: enable add and remove workers
        DeployCluster {
            owner: AccountId32,
            cluster: ContractClusterId,
            workers: Vec<WorkerIdentity>,
            deposit: u128, // Amount of balance transfering from chain into the cluster for the owner
            gas_price: u128,
            deposit_per_item: u128,
            deposit_per_byte: u128,
            treasury_account: AccountId32,
        },
    }

    bind_topic!(ContractOperation<CodeHash, AccountId>, b"phala/contract/op");
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, ::scale_info::TypeInfo)]
    pub enum ContractOperation<CodeHash, AccountId> {
        InstantiateCode {
            contract_info: ContractInfo<CodeHash, AccountId>,
            transfer: u128,
            gas_limit: u64,
            storage_deposit_limit: Option<u128>,
        },
    }

    impl<CodeHash, AccountId> ContractOperation<CodeHash, AccountId> {
        pub fn instantiate_code(
            contract_info: ContractInfo<CodeHash, AccountId>,
            transfer: u128,
            gas_limit: u64,
            storage_deposit_limit: Option<u128>,
        ) -> Self {
            ContractOperation::InstantiateCode {
                contract_info,
                transfer,
                gas_limit,
                storage_deposit_limit,
            }
        }
    }

    // Pink messages
    #[derive(Encode, Decode, Debug, PartialEq, Eq, TypeInfo, Clone, Copy)]
    pub enum ResourceType {
        InkCode,
        SidevmCode,
        IndeterministicInkCode,
    }

    bind_topic!(WorkerClusterReport, b"phala/cluster/worker/report");
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub enum WorkerClusterReport {
        ClusterDeployed {
            id: ContractClusterId,
            pubkey: ClusterPublicKey,
        },
        ClusterDeploymentFailed {
            id: ContractClusterId,
        },
    }

    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct BatchDispatchClusterKeyEvent {
        pub secret_keys: BTreeMap<WorkerPublicKey, EncryptedKey>,
        pub cluster: ContractClusterId,
        /// The owner of the cluster
        pub owner: AccountId32,
        pub deposit: u128,
        pub gas_price: u128,
        pub deposit_per_item: u128,
        pub deposit_per_byte: u128,
        pub treasury_account: AccountId32,
    }

    bind_topic!(ClusterOperation<AccountId>, b"phala/cluster/key");
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub enum ClusterOperation<AccountId> {
        // TODO.shelven: a better way for real large batch key distribution
        /// MessageOrigin::Gatekeeper -> ALL
        DispatchKeys(BatchDispatchClusterKeyEvent),
        /// Force destroying a cluster.
        ///
        /// This leaves a door to clean up the beta clusters in phat v1.
        /// We might need to redesign a more graceful one in the future.
        DestroyCluster(ContractClusterId),
        /// Upload ink code to the cluster.
        UploadResource {
            origin: AccountId,
            cluster_id: ContractClusterId,
            resource_type: ResourceType,
            resource_data: Vec<u8>,
        },
        Deposit {
            cluster_id: ContractClusterId,
            account: AccountId,
            amount: u128,
        },
        RemoveWorker {
            cluster_id: ContractClusterId,
            worker: WorkerPublicKey,
        },
    }

    impl<AccountId> ClusterOperation<AccountId> {
        #[allow(clippy::too_many_arguments)]
        pub fn batch_distribution(
            secret_keys: BTreeMap<WorkerPublicKey, EncryptedKey>,
            cluster: ContractClusterId,
            owner: AccountId32,
            deposit: u128,
            gas_price: u128,
            deposit_per_item: u128,
            deposit_per_byte: u128,
            treasury_account: AccountId32,
        ) -> Self {
            ClusterOperation::DispatchKeys(BatchDispatchClusterKeyEvent {
                secret_keys,
                cluster,
                owner,
                deposit,
                gas_price,
                deposit_per_item,
                deposit_per_byte,
                treasury_account,
            })
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum ClusterPermission<AccountId> {
    Public,
    OnlyOwner(AccountId),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct ClusterInfo<AccountId> {
    pub owner: AccountId,
    pub permission: ClusterPermission<AccountId>,
    pub system_contract: ContractId,
    pub gas_price: u128,
    pub deposit_per_item: u128,
    pub deposit_per_byte: u128,
}

/// On-chain contract registration info
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct ContractInfo<CodeHash, AccountId> {
    pub deployer: AccountId,
    pub code_index: CodeIndex<CodeHash>,
    pub salt: Vec<u8>,
    pub cluster_id: ContractClusterId,
    pub instantiate_data: Vec<u8>,
}

/// Use blake2_256 on the preimage for the final contract id
pub fn contract_id_preimage(
    deployer: &[u8],
    code_hash: &[u8],
    cluster_id: &[u8],
    salt: &[u8],
) -> Vec<u8> {
    let buf: Vec<_> = deployer
        .iter()
        .chain(code_hash)
        .chain(cluster_id)
        .chain(salt)
        .cloned()
        .collect();
    buf
}

impl<CodeHash: AsRef<[u8]>, AccountId: AsRef<[u8]>> ContractInfo<CodeHash, AccountId> {
    pub fn contract_id(&self, blake2_256: impl Fn(&[u8]) -> [u8; 32]) -> ContractId {
        let buf = contract_id_preimage(
            self.deployer.as_ref(),
            self.code_index.code_hash().as_ref(),
            self.cluster_id.as_ref(),
            self.salt.as_ref(),
        );
        ContractId::from(blake2_256(buf.as_ref()))
    }
}

/// Contract query request parameters, to be encrypted.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct ContractQuery<Data> {
    pub head: ContractQueryHead,
    /// The request data.
    pub data: Data,
}

/// Contract query head
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct ContractQueryHead {
    /// The contract id.
    pub id: ContractId,
    /// A random byte array generated by the client.
    pub nonce: [u8; 32],
}

/// Contract query response, to be encrypted.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct ContractQueryResponse<Data> {
    /// The nonce from the client.
    pub nonce: [u8; 32],
    /// The query result.
    pub result: Data,
}

pub struct Data(pub Vec<u8>);

impl Encode for Data {
    fn size_hint(&self) -> usize {
        self.0.len()
    }
    fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&self.0)
    }
}

/// Contract query error define
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum ContractQueryError {
    /// Signature is invalid.
    InvalidSignature,
    /// No such contract.
    ContractNotFound,
    /// Unable to decode the request data.
    DecodeError,
    /// Other errors reported during the contract query execution.
    OtherError(String),
}

impl From<ContractQueryError> for prpc::server::Error {
    fn from(err: ContractQueryError) -> Self {
        Self::ContractQueryError(alloc::format!("{err:?}"))
    }
}

pub fn command_topic(id: ContractId) -> Vec<u8> {
    format!("phala/contract/{}/command", hex::encode(id))
        .as_bytes()
        .to_vec()
}

pub trait ConvertTo<To> {
    fn convert_to(&self) -> To;
}

impl<F, T> ConvertTo<T> for F
where
    F: AsRef<[u8; 32]>,
    T: From<[u8; 32]>,
{
    fn convert_to(&self) -> T {
        (*self.as_ref()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::{messaging::*, *};
    use codec::{Decode, Encode};
    use scale_info::TypeInfo;
    use sp_core::crypto::AccountId32;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
    struct TestData {
        code_index: CodeIndex<Vec<u8>>,
        ink_command: InkCommand,
        cluster_event: ClusterEvent,
        cluster_operation: ClusterOperation<AccountId32>,
        res_type: ResourceType,
        cluster_report: WorkerClusterReport,
        cluster_info: ClusterInfo<AccountId32>,
        contract_operation: ContractOperation<Vec<u8>, AccountId32>,
        query: ContractQuery<Vec<u8>>,
        query_response: ContractQueryResponse<Vec<u8>>,
        query_error: ContractQueryError,
    }

    #[test]
    fn test_codecs() {
        let contract_info = ContractInfo {
            deployer: AccountId32::new([1; 32]),
            code_index: CodeIndex::WasmCode(vec![1, 2, 3]),
            salt: vec![1, 2, 3],
            cluster_id: ContractClusterId([1; 32]),
            instantiate_data: vec![1, 2, 3],
        };
        assert_eq!(
            contract_info.contract_id(sp_core::blake2_256),
            ContractId(hex_literal::hex!(
                "29a773373d12f21a9407333f6e1c9db09aed5c03d21e91060c2f34ac2b86ecb4"
            ))
        );
        let data = TestData {
            code_index: CodeIndex::WasmCode(vec![1, 2, 3]),
            ink_command: InkCommand::InkMessage {
                nonce: Default::default(),
                message: vec![1, 2, 3],
                transfer: 0,
                gas_limit: 0,
                storage_deposit_limit: None,
            },
            cluster_event: ClusterEvent::DeployCluster {
                owner: AccountId32::new([1; 32]).convert_to(),
                cluster: ContractClusterId([1; 32]),
                workers: vec![],
                deposit: 0,
                gas_price: 0,
                deposit_per_item: 0,
                deposit_per_byte: 0,
                treasury_account: AccountId32::new([1; 32]),
            },
            cluster_operation: ClusterOperation::batch_distribution(
                Default::default(),
                Default::default(),
                AccountId32::new([1; 32]),
                0,
                0,
                0,
                0,
                AccountId32::new([1; 32]),
            ),
            res_type: ResourceType::InkCode,
            cluster_report: WorkerClusterReport::ClusterDeployed {
                id: ContractClusterId([1; 32]),
                pubkey: crate::ClusterPublicKey([1; 32]),
            },
            cluster_info: ClusterInfo {
                owner: AccountId32::new([1; 32]),
                permission: ClusterPermission::Public,
                system_contract: [1; 32].into(),
                gas_price: 0,
                deposit_per_item: 0,
                deposit_per_byte: 0,
            },
            contract_operation: ContractOperation::instantiate_code(contract_info, 0, 0, None),
            query: ContractQuery {
                head: ContractQueryHead {
                    id: ContractId([1; 32]),
                    nonce: [1; 32],
                },
                data: vec![1, 2, 3],
            },
            query_response: ContractQueryResponse {
                nonce: [1; 32],
                result: vec![1, 2, 3],
            },
            query_error: ContractQueryError::InvalidSignature,
        };

        let cloned = data.clone();
        let encoded = data.encode();
        let decoded = TestData::decode(&mut &encoded[..]).unwrap();

        insta::assert_debug_snapshot!(decoded);

        assert_eq!(cloned, decoded);

        assert_eq!(decoded.code_index.code_hash(), vec![1, 2, 3]);
    }

    #[test]
    fn dump_type_info() {
        use type_info_stringify::type_info_stringify;
        insta::assert_display_snapshot!(type_info_stringify::<TestData>());
    }

    #[test]
    fn helper_fn_works() {
        let img = contract_id_preimage(&[1, 2, 3], &[4, 5, 6], &[7, 8, 9], &[10, 11, 12]);
        assert_eq!(img, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let cmd_topic = command_topic(ContractId([1; 32]));
        assert_eq!(cmd_topic, b"phala/contract/0101010101010101010101010101010101010101010101010101010101010101/command".to_vec());
        let _: prpc::server::Error = ContractQueryError::ContractNotFound.into();
        let data = Data(vec![1, 2, 3]).encode();
        assert_eq!(data, vec![1, 2, 3]);
    }
}
