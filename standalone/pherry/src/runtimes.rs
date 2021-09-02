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

use crate::extra::PhalaExtra;

/// PhalaNode concrete type definitions compatible with those for kusama, v0.7
///
/// # Note
///
/// Main difference is `type Address = AccountId`.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PhalaNodeRuntime;

impl Runtime for PhalaNodeRuntime {
    type Signature = MultiSignature;
    type Extra = PhalaExtra<Self>;

    fn register_type_sizes(event_type_registry: &mut EventTypeRegistry<Self>) {
        event_type_registry.with_system();
        event_type_registry.with_sudo();
        event_type_registry.with_balances();
        event_type_registry.with_staking();
        event_type_registry.with_session();

        use chain_bridge::ChainBridgeEventTypeRegistry;

        event_type_registry.with_chain_bridge();

        register_default_type_sizes(event_type_registry);
        event_type_registry
            .register_type_size::<phala_types::messaging::Message>("PhalaMq::Message");
        event_type_registry
            .register_type_size::<u8>("bridge::BridgeChainId");
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

pub mod phala_registry {
    use codec::Encode;
    use phala_types::WorkerRegistrationInfo;
    use core::marker::PhantomData;
    use phala_pallets::registry::Attestation;
    use subxt::{module, system::System, Call};

    #[module]
    pub trait PhalaRegistry: System {}
    impl PhalaRegistry for super::PhalaNodeRuntime {}

    /// The call to register_worker
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct RegisterWorkerCall<T: PhalaRegistry> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The runtime info
        pub pruntime_info: WorkerRegistrationInfo<T::AccountId>,
        /// The enclave attestation
        pub attestation: Attestation,
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

pub mod chain_bridge {
    use subxt::{
        module,
        system::System,
    };
    #[module]
    pub trait ChainBridge: System {
        #![event_alias(BridgeChainId = u8)]
        #![event_alias(ResourceId = [u8; 32])]
        #![event_alias(DepositNonce = u64)]
        #![event_alias(U256 = sp_core::U256)]
    }
}

pub mod parachain_info {
    use super::PhalaNodeRuntime;
    use codec::{Encode};
    use subxt::{module, Store, system::System};
    use core::marker::PhantomData;

    pub type ParachainId = phala_enclave_api::blocks::ParaId;

    #[module]
    pub trait ParachainInfo: System {}
    impl ParachainInfo for PhalaNodeRuntime {}

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct ParachainIdStore<T: ParachainInfo> {
        #[store(returns = ParachainId)]
        /// Runtime marker.
        pub _runtime: PhantomData<T>,
    }

    impl<T: ParachainInfo> ParachainIdStore<T> {
        pub fn new() -> Self {
            Self {
                _runtime: Default::default()
            }
        }
    }
}

