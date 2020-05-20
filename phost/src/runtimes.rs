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
    traits::{
        BlakeTwo256,
    },
    OpaqueExtrinsic,
};

use subxt::{
    balances::{
        AccountData,
        Balances,
    },
    system::System,
};

use node_primitives::{BlockNumber, Hash, Header, Index, AccountId};

/// PhalaNode concrete type definitions compatible with those for kusama, v0.7
///
/// # Note
///
/// Main difference is `type Address = AccountId`.
/// Also the contracts module is not part of the kusama runtime.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PhalaNodeRuntime;

impl System for PhalaNodeRuntime {
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Address = Self::AccountId;
    type Header = Header;
    type Extrinsic = OpaqueExtrinsic;
    type AccountData = AccountData<<Self as Balances>::Balance>;
}

impl Balances for PhalaNodeRuntime {
    type Balance = u128;
}

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
