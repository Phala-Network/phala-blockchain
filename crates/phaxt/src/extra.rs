use std::fmt::Debug;
use std::marker::PhantomData;

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

use subxt::extrinsic::*;
use subxt::sp_runtime::generic::Era;
use subxt::sp_runtime::traits::SignedExtension;
use subxt::sp_runtime::transaction_validity::TransactionValidityError;
use subxt::Config;

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
    additional_params: ExtraConfig<T::Hash>,
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
        ChargeTransactionPayment<T>,
    );
    type Parameters = ExtraConfig<T::Hash>;

    fn new(
        spec_version: u32,
        tx_version: u32,
        nonce: T::Index,
        genesis_hash: T::Hash,
        additional_params: Self::Parameters,
    ) -> Self {
        PhalaExtra {
            spec_version,
            tx_version,
            nonce,
            genesis_hash,
            additional_params,
        }
    }

    fn extra(&self) -> Self::Extra {
        let (era, birth_hash) = match self.additional_params.era {
            None => (Era::Immortal, self.genesis_hash),
            Some(EraInfo {
                period,
                phase,
                birth_hash,
            }) => (Era::Mortal(period, phase), birth_hash),
        };

        (
            CheckSpecVersion(Default::default(), self.spec_version),
            CheckTxVersion(Default::default(), self.tx_version),
            CheckGenesis(Default::default(), self.genesis_hash),
            CheckMortality((era, Default::default()), birth_hash),
            CheckNonce(self.nonce),
            CheckWeight(Default::default()),
            // NOTE: skipped the ZST CheckMqSequence<T> here.
            ChargeTransactionPayment(self.additional_params.tip.into(), Default::default()),
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

    fn pre_dispatch(
        self,
        _who: &Self::AccountId,
        _call: &Self::Call,
        _info: &subxt::sp_runtime::traits::DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(())
    }
}

#[derive(Encode, Decode, Clone, Debug, Default, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeTransactionPayment<T: Config>(#[codec(compact)] pub u128, pub PhantomData<T>);

impl<T> SignedExtension for ChargeTransactionPayment<T>
where
    T: Config + Clone + Debug + Eq + Send + Sync,
{
    const IDENTIFIER: &'static str = "ChargeTransactionPayment";
    type AccountId = T::AccountId;
    type Call = ();
    type AdditionalSigned = ();
    type Pre = ();
    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }
    fn pre_dispatch(
        self,
        _who: &Self::AccountId,
        _call: &Self::Call,
        _info: &subxt::sp_runtime::traits::DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(())
    }
}
