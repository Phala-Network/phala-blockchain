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
    },
    contracts::Contracts,
    sudo::Sudo,
    system::System,
    session::Session,
    staking::Staking,
    Runtime,
    BasicSessionKeys
};

/// PhalaNode concrete type definitions compatible with those for kusama, v0.7
///
/// # Note
///
/// Main difference is `type Address = AccountId`.
/// Also the contracts module is not part of the kusama runtime.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PhalaNodeRuntime;

impl Staking for PhalaNodeRuntime {}

impl Runtime for PhalaNodeRuntime {
    type Signature = MultiSignature;
    type Extra = DefaultExtra<Self>;
}

impl System for PhalaNodeRuntime {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
    type Address = pallet_indices::address::Address<Self::AccountId, u32>;
    type Header = Header<Self::BlockNumber, BlakeTwo256>;
    type Extrinsic = OpaqueExtrinsic;
    type AccountData = AccountData<<Self as Balances>::Balance>;
}

impl Balances for PhalaNodeRuntime {
    type Balance = u128;
}

impl Contracts for PhalaNodeRuntime {}

impl Sudo for PhalaNodeRuntime {}

impl Session for PhalaNodeRuntime {
    type ValidatorId = <Self as System>::AccountId;
    type Keys = BasicSessionKeys;
}

impl phala::PhalaModule for PhalaNodeRuntime {}

pub mod grandpa {
    use super::PhalaNodeRuntime;
    use codec::Encode;
    use subxt::{module, Store, system::{System, SystemEventsDecoder}};
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
    use codec::Encode;
    use subxt::{module, Call, Store, system::{System, SystemEventsDecoder}, balances::{Balances, BalancesEventsDecoder}};
    use core::marker::PhantomData;

    /// The subset of the `pallet_phala::Trait` that a client must implement.
    #[module]
    pub trait PhalaModule: System + Balances {}

    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct PushCommandCall<T: PhalaModule> {
        pub _runtime: PhantomData<T>,
        pub contract_id: u32,
        pub payload: Vec<u8>,
    }

    /// The call to transfer_to_tee
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct TransferToTeeCall<T: PhalaModule> {
        /// The amount will transfer to tee account
        #[codec(compact)]
        pub amount: <T as Balances>::Balance,
    }

    /// The call to transfer_to_chain
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct TransferToChainCall<T: PhalaModule> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The transfer transaction data, SCALA encoded
        pub data: Vec<u8>,
    }

    /// The call to register_worker
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct RegisterWorkerCall<T: PhalaModule> {
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

    #[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
    pub struct IngressSequenceStore<T: PhalaModule> {
        #[store(returns = u64)]
        /// Runtime marker.
        pub _runtime: PhantomData<T>,
        pub contract_id: u32,
    }
    impl<T: PhalaModule> IngressSequenceStore<T> {
        pub fn new(contract_id: u32) -> Self {
            Self {
                _runtime: Default::default(),
                contract_id,
            }
        }
    }

    /// The call to transfer_to_chain
    #[derive(Clone, Debug, PartialEq, Call, Encode)]
    pub struct HeartbeatCall<T: PhalaModule> {
        /// Runtime marker
        pub _runtime: PhantomData<T>,
        /// The heartbeat data, SCALA encoded
        pub data: Vec<u8>,
    }

}
