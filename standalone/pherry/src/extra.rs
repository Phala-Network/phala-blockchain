use std::fmt::Debug;
use std::marker::PhantomData;

use codec::{Decode, Encode};

use subxt::balances::Balances;
use subxt::extrinsic::*;
use subxt::system::System;

use sp_runtime::generic::Era;
use sp_runtime::traits::SignedExtension;
use sp_runtime::transaction_validity::TransactionValidityError;

use core::sync::atomic::{AtomicU64, Ordering};

static TIP: AtomicU64 = AtomicU64::new(0);

pub fn set_tip(tip: u64) {
    TIP.store(tip, Ordering::Relaxed);
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
pub struct PhalaExtra<T: System> {
    spec_version: u32,
    tx_version: u32,
    nonce: T::Index,
    genesis_hash: T::Hash,
}

impl<T: System + Balances + Clone + Debug + Eq + Send + Sync> SignedExtra<T> for PhalaExtra<T>
where
    <T as Balances>::Balance: From<u64>,
{
    type Extra = (
        CheckSpecVersion<T>,
        CheckTxVersion<T>,
        CheckGenesis<T>,
        CheckEra<T>,
        CheckNonce<T>,
        CheckWeight<T>,
        // NOTE: skipped the ZST CheckMqSequence<T> here.
        ChargeTransactionPayment<T>,
    );

    fn new(spec_version: u32, tx_version: u32, nonce: T::Index, genesis_hash: T::Hash) -> Self {
        PhalaExtra {
            spec_version,
            tx_version,
            nonce,
            genesis_hash,
        }
    }

    fn extra(&self) -> Self::Extra {
        let tip = TIP.load(Ordering::Relaxed);

        (
            CheckSpecVersion(PhantomData, self.spec_version),
            CheckTxVersion(PhantomData, self.tx_version),
            CheckGenesis(PhantomData, self.genesis_hash),
            CheckEra((Era::Immortal, PhantomData), self.genesis_hash),
            CheckNonce(self.nonce),
            CheckWeight(PhantomData),
            // NOTE: skipped the ZST CheckMqSequence<T> here.
            ChargeTransactionPayment(tip.into()),
        )
    }
}

impl<T: System + Balances + Clone + Debug + Eq + Send + Sync> SignedExtension for PhalaExtra<T>
where
    <T as Balances>::Balance: From<u64>,
{
    const IDENTIFIER: &'static str = "PhalaExtra";
    type AccountId = T::AccountId;
    type Call = ();
    type AdditionalSigned = <<Self as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned;
    type Pre = ();

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        self.extra().additional_signed()
    }
}
