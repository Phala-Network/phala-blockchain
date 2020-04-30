// Copyright 2018-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

#![no_std]

extern crate alloc;

pub mod blake2 {
    use sp_core::Hasher;
    use hash256_std_hasher::Hash256StdHasher;
    use sp_core::hash::H256;
    use sp_trie::{TrieConfiguration, trie_types::Layout};
    use sp_runtime::traits::Hash;
    use alloc::vec::Vec;

    /// Concrete implementation of Hasher using Blake2b 256-bit hashes
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Blake2Hasher;

    impl Hasher for Blake2Hasher {
        type Out = H256;
        type StdHasher = Hash256StdHasher;
        const LENGTH: usize = 32;

        fn hash(x: &[u8]) -> Self::Out {
            sp_core::hashing::blake2_256(x).into()
        }
    }

    impl Hash for Blake2Hasher {
        type Output = H256;

        fn trie_root(input: Vec<(Vec<u8>, Vec<u8>)>) -> Self::Output {
            Layout::<Self>::trie_root(input)
        }

        fn ordered_trie_root(input: Vec<Vec<u8>>) -> Self::Output {
            Layout::<Self>::ordered_trie_root(input)
        }
    }
}
