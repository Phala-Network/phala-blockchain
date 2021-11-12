use std::fmt::Debug;
use std::marker::PhantomData;

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

use subxt::extrinsic::*;
use subxt::Config;
use subxt::sp_runtime::generic::Era;
use subxt::sp_runtime::traits::SignedExtension;
use subxt::sp_runtime::transaction_validity::TransactionValidityError;

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug, TypeInfo)]
pub struct EraInfo<Hash> {
    pub period: u64,
    pub phase: u64,
    pub birth_hash: Hash,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug, Default, TypeInfo)]
pub struct ExtraConfig<Hash> {
    pub tip: u64,
    pub era: Option<EraInfo<Hash>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug, TypeInfo)]
pub struct PhalaExtra<T: Config> {
    spec_version: u32,
    tx_version: u32,
    nonce: T::Index,
    genesis_hash: T::Hash,
    config: ExtraConfig<T::Hash>,
}

impl<T: Config + Clone + Debug + Eq + Send + Sync + TypeInfo + 'static> SignedExtra<T>
    for PhalaExtra<T>
{
    type Extra = (
        CheckSpecVersion<T>,
        CheckTxVersion<T>,
        CheckGenesis<T>,
        CheckMortality<T>,
        CheckNonce<T>,
        CheckWeight<T>,
        // NOTE: skipped the ZST CheckMqSequence<T> here.
        ChargeTransactionPayment,
    );
    type Config = ExtraConfig<T::Hash>;

    fn new(
        spec_version: u32,
        tx_version: u32,
        nonce: T::Index,
        genesis_hash: T::Hash,
        config: Self::Config,
    ) -> Self {
        PhalaExtra {
            spec_version,
            tx_version,
            nonce,
            genesis_hash,
            config,
        }
    }

    fn extra(&self) -> Self::Extra {
        let (era, birth_hash) = match self.config.era {
            None => (Era::Immortal, self.genesis_hash),
            Some(EraInfo {
                period,
                phase,
                birth_hash,
            }) => (Era::Mortal(period, phase), birth_hash),
        };

        (
            CheckSpecVersion(PhantomData, self.spec_version),
            CheckTxVersion(PhantomData, self.tx_version),
            CheckGenesis(PhantomData, self.genesis_hash),
            CheckMortality((era, PhantomData), birth_hash),
            CheckNonce(self.nonce),
            CheckWeight(PhantomData),
            // NOTE: skipped the ZST CheckMqSequence<T> here.
            ChargeTransactionPayment(self.config.tip.into()),
        )
    }
}

impl<T: Config + Clone + Debug + Eq + Send + Sync + TypeInfo + 'static> SignedExtension
    for PhalaExtra<T>
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
