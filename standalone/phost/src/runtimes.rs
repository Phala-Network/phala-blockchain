// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-subxt.  If not, see <http://www.gnu.org/licenses/>.

use sp_runtime::{
    generic::Header,
    traits::{
        BlakeTwo256,
        IdentifyAccount,
        Verify,
    },
    MultiSignature,
    OpaqueExtrinsic,
};

use subxt::{
    extrinsic::DefaultExtra,
    balances::{
        AccountData,
        Balances,
        BalancesEventTypeRegistry,
    },
    session::{
        Session,
        SessionEventTypeRegistry,
    },
    staking::{
        Staking,
        StakingEventTypeRegistry,
    },
    sudo::{
        Sudo,
        SudoEventTypeRegistry,
    },
    system::{
        System,
        SystemEventTypeRegistry,
    },
    EventTypeRegistry,
    Runtime,
    BasicSessionKeys,
    register_default_type_sizes
};

/// PhalaNode concrete type definitions compatible with those for kusama, v0.7
///
/// # Note
///
/// Main difference is `type Address = AccountId`.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PhalaNodeRuntime;

impl Runtime for PhalaNodeRuntime {
    type Signature = MultiSignature;
    type Extra = DefaultExtra<Self>;

    fn register_type_sizes(event_type_registry: &mut EventTypeRegistry<Self>) {
        event_type_registry.with_system();
        event_type_registry.with_sudo();
        event_type_registry.with_balances();
        event_type_registry.with_staking();
        event_type_registry.with_session();

        use phala::PhalaEventTypeRegistry;
        use chain_bridge::ChainBridgeEventTypeRegistry;

        event_type_registry.with_phala();
        event_type_registry.with_chain_bridge();

        register_default_type_sizes(event_type_registry);
    }
}

impl System for PhalaNodeRuntime {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
    type Address = sp_runtime::MultiAddress<Self::AccountId, u32>;
    type Header = Header<Self::BlockNumber, BlakeTwo256>;
    type Extrinsic = OpaqueExtrinsic;
    type AccountData = AccountData<<Self as Balances>::Balance>;
}

impl Balances for PhalaNodeRuntime {
    type Balance = u128;
}

impl Staking for PhalaNodeRuntime {}

impl Sudo for PhalaNodeRuntime {}

impl Session for PhalaNodeRuntime {
    type ValidatorId = <Self as System>::AccountId;
    type Keys = BasicSessionKeys;
}

impl phala::Phala for PhalaNodeRuntime {}
impl phala_mq::PhalaMq for PhalaNodeRuntime {}

impl mining_staking::MiningStaking for PhalaNodeRuntime {}

impl chain_bridge::ChainBridge for PhalaNodeRuntime {}

pub mod grandpa {
    use super::PhalaNodeRuntime;
    use codec::Encode;
    use subxt::{module, Store, system::System};
    use core::marker::PhantomData;
    use pallet_grandpa::fg_primitives::SetId;

    #[module]
    pub trait Grandpa: System {}
    impl Grandpa for PhalaNodeRuntime {}

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct CurrentSetIdStore<T: Grandpa> {
        #[store(returns = SetId)]
        /// Runtime marker.
        pub _runtime: PhantomData<T>,
    }

    impl<T: Grandpa> CurrentSetIdStore<T> {
        pub fn new() -> Self {
            Self {
                _runtime: Default::default()
            }
        }
    }
}

pub mod phala {
    use codec::{Encode, Decode};
    use subxt::{
        module, Call, Store,
        system::System,
        balances::Balances
    };
    use core::marker::PhantomData;

    use phala_types::{BlockRewardInfo, PayoutReason};

    #[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
    pub struct EthereumTxHash([u8; 32]);
    #[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
    pub struct EthereumAddress([u8; 20]);

    #[module]
    pub trait Phala: System + Balances {
        #![event_type(BlockRewardInfo)]
        #![event_type(PayoutReason)]
    }

    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct PushCommandCall<T: Phala> {
        pub _runtime: PhantomData<T>,
        pub contract_id: u32,
        pub payload: Vec<u8>,
    }

    /// The call to transfer_to_tee
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct TransferToTeeCall<T: Phala> {
        /// The amount will transfer to tee account
        #[codec(compact)]
        pub amount: <T as Balances>::Balance,
    }

    /// The call to transfer_to_chain
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct TransferToChainCall<T: Phala> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The transfer transaction data, SCALE encoded
        pub data: Vec<u8>,
    }

    /// The call to register_worker
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct RegisterWorkerCall<T: Phala> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The encoded runtime info
        pub encoded_runtime_info: Vec<u8>,
        /// The report
        pub report: Vec<u8>,
        /// The signature
        pub signature: Vec<u8>,
        /// The signing cert
        pub raw_signing_cert: Vec<u8>,
    }

    /// The call to reset_worker
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct ResetWorkerCall<T: Phala> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct IngressSequenceStore<T: Phala> {
        #[store(returns = u64)]
        /// Runtime marker.
        pub _runtime: PhantomData<T>,
        pub contract_id: u32,
    }
    impl<T: Phala> IngressSequenceStore<T> {
        pub fn new(contract_id: u32) -> Self {
            Self {
                _runtime: Default::default(),
                contract_id,
            }
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct MachineOwnerStore<T: Phala> {
        #[store(returns = [u8; 32])]
        /// Runtime marker.
        pub _runtime: PhantomData<T>,
        pub machine_id: Vec<u8>,
    }
    impl<T: Phala> MachineOwnerStore<T> {
        pub fn new(machine_id: Vec<u8>) -> Self {
            Self {
                _runtime: Default::default(),
                machine_id,
            }
        }
    }
    /// The call to transfer_to_chain
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct HeartbeatCall<T: Phala> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The heartbeat data, SCALE encoded
        pub data: Vec<u8>,
    }

    /// Storage: Stash
    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct StashStore<T: Phala> {
        #[store(returns = T::AccountId)]
        pub _runtime: PhantomData<T>,
        pub account_id: T::AccountId,
    }
    impl<T: Phala> StashStore<T> {
        pub fn new(account_id: T::AccountId) -> Self {
            Self {
                _runtime: Default::default(),
                account_id,
            }
        }
    }

    /// Storage: WorkerIngress
    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct WorkerIngressStore<T: Phala> {
        #[store(returns = u64)]
        pub _runtime: PhantomData<T>,
        pub account_id: T::AccountId,
    }
    impl<T: Phala> WorkerIngressStore<T> {
        pub fn new(account_id: T::AccountId) -> Self {
            Self {
                _runtime: Default::default(),
                account_id,
            }
        }
    }

    /// Storage: OnlineWorkers
    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode, Default)]
    pub struct OnlineWorkers<T: Phala> {
        #[store(returns = u32)]
        pub _runtime: PhantomData<T>,
    }
    /// Storage: ComputeWorkers
    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode, Default)]
    pub struct ComputeWorkers<T: Phala> {
        #[store(returns = u32)]
        pub _runtime: PhantomData<T>,
    }

    /// Storage: WorkerState
    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct WorkerStateStore<T: Phala> {
        #[store(returns = phala_types::WorkerInfo<T::BlockNumber>)]
        pub _runtime: PhantomData<T>,
        pub account_id: T::AccountId,
    }

    /// The call to sync_worker_message
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct SyncWorkerMessageCall<T: Phala> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The raw message, SCALE encoded
        pub msg: Vec<u8>,
    }

    /// The call to sync_lottery_message
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct SyncLotteryMessageCall<T: Phala> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The raw message, SCALE encoded
        pub msg: Vec<u8>,
    }
}

pub mod phala_mq {
    use codec::Encode;
    use core::marker::PhantomData;
    use subxt::{balances::Balances, module, system::System, Call, Store};

    use phala_types::messaging::{MessageOrigin, SignedMessage};

    #[module]
    pub trait PhalaMq: System + Balances {}

    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct SyncOffchainMessageCall<T: PhalaMq> {
        pub _runtime: PhantomData<T>,
        pub message: SignedMessage,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct OffchainIngressStore<T: PhalaMq> {
        #[store(returns = u64)]
        /// Runtime marker.
        pub _runtime: PhantomData<T>,
        pub sender: MessageOrigin,
    }

    impl<T: PhalaMq> OffchainIngressStore<T> {
        pub fn new(sender: MessageOrigin) -> Self {
            Self {
                _runtime: Default::default(),
                sender,
            }
        }
    }
}

pub mod mining_staking {
    use codec::Encode;
    use subxt::{
        module, Store,
        system::System,
        balances::Balances
    };
    use core::marker::PhantomData;

    #[module]
    pub trait MiningStaking: System + Balances {}

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct StakedStore<T: MiningStaking> {
        #[store(returns = <T as Balances>::Balance)]
        pub _runtime: PhantomData<T>,
        pub from: T::AccountId,
        pub to: T::AccountId,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct StakeReceivedStore<T: MiningStaking> {
        #[store(returns = <T as Balances>::Balance)]
        pub _runtime: PhantomData<T>,
        pub to: T::AccountId,
    }
}

pub mod kitties {
    use super::PhalaNodeRuntime;
    use codec::Encode;
    use subxt::{module, Call, Store, system::System, balances::Balances};
    use core::marker::PhantomData;

    /// The subset of the `pallet_phala::Trait` that a client must implement.
    #[module]
    pub trait KittyStorage: System + Balances {
    }

    impl KittyStorage for PhalaNodeRuntime {}
    /// The call to transfer_to_chain
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct TransferToChainCall<T: KittyStorage> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The transfer transaction data, SCALA encoded
        pub data: Vec<u8>,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct IngressSequenceStore<T: KittyStorage> {
        #[store(returns = u64)]
        /// Runtime marker.
        pub _runtime: PhantomData<T>,
        pub contract_id: u32,
    }
    impl<T: KittyStorage> IngressSequenceStore<T> {
        pub fn new(contract_id: u32) -> Self {
            Self {
                _runtime: Default::default(),
                contract_id,
            }
        }
    }
}

pub mod chain_bridge {
    use subxt::{
        module,
        system::System,
    };
    #[module]
    pub trait ChainBridge: System {
        #![event_alias(ChainId = u8)]
        #![event_alias(ResourceId = [u8; 32])]
        #![event_alias(DepositNonce = u64)]
    }
}
