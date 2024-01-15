use crate::{Authorship, Balances, Runtime, Treasury};
use frame_support::traits::{fungible::Inspect, Currency, Imbalance, OnUnbalanced};
use node_primitives::{AccountId, Balance};
use pallet_balances::{NegativeImbalance, PositiveImbalance};
use pallet_evm::{EVMCurrencyAdapter, OnChargeEVMTransaction};

pub struct EvmCurrency;

trait BalanceExt {
    fn into_eth(self) -> Self;
    fn into_sub(self) -> Self;
}

impl BalanceExt for Balance {
    fn into_eth(self) -> Self {
        // The wallet UI shows balances in ETH convention, so we need to convert the balances
        // 1 ETH = 1_000_000_000_000_000_000 wei
        // 1 PHA = 1_000_000_000_000 balances
        self.checked_mul(1_000_000).expect("overflowed")
    }

    fn into_sub(self) -> Self {
        self / 1_000_000
    }
}

impl BalanceExt for PositiveImbalance<Runtime> {
    fn into_eth(self) -> Self {
        Self::new(self.peek().into_eth())
    }

    fn into_sub(self) -> Self {
        Self::new(self.peek().into_sub())
    }
}

impl BalanceExt for NegativeImbalance<Runtime> {
    fn into_eth(self) -> Self {
        Self::new(self.peek().into_eth())
    }

    fn into_sub(self) -> Self {
        Self::new(self.peek().into_sub())
    }
}

pub struct EvmDealWithFees;
impl OnUnbalanced<NegativeImbalance<Runtime>> for EvmDealWithFees {
    fn on_nonzero_unbalanced(fee: NegativeImbalance<Runtime>) {
        // tip is already transfered to the author in pallet evm
        Treasury::on_unbalanced(fee.into_sub())
    }
}

impl OnChargeEVMTransaction<Runtime> for EvmDealWithFees {
    type LiquidityInfo = Option<NegativeImbalance<Runtime>>;

    fn withdraw_fee(
        who: &sp_core::H160,
        fee: sp_core::U256,
    ) -> Result<Self::LiquidityInfo, pallet_evm::Error<Runtime>> {
        <EVMCurrencyAdapter<EvmCurrency, Self> as OnChargeEVMTransaction<Runtime>>::withdraw_fee(
            who, fee,
        )
    }

    fn correct_and_deposit_fee(
        who: &sp_core::H160,
        corrected_fee: sp_core::U256,
        base_fee: sp_core::U256,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Self::LiquidityInfo {
        <EVMCurrencyAdapter<EvmCurrency, Self> as OnChargeEVMTransaction<Runtime>>::correct_and_deposit_fee(
            who,
            corrected_fee,
            base_fee,
            already_withdrawn,
        )
    }

    fn pay_priority_fee(tip: Self::LiquidityInfo) {
        if let Some(tip) = tip {
            match Authorship::author() {
                Some(author) => {
                    let _ = EvmCurrency::deposit_creating(&author, tip.peek());
                }
                None => Self::on_unbalanced(tip),
            }
        }
    }
}

impl Currency<AccountId> for EvmCurrency {
    type Balance = Balance;

    type PositiveImbalance = PositiveImbalance<Runtime>;

    type NegativeImbalance = NegativeImbalance<Runtime>;

    fn total_balance(who: &AccountId) -> Self::Balance {
        <Balances as Currency<AccountId>>::total_balance(who).into_eth()
    }

    fn can_slash(who: &AccountId, value: Self::Balance) -> bool {
        <Balances as Currency<AccountId>>::can_slash(who, value.into_sub())
    }

    fn total_issuance() -> Self::Balance {
        <Balances as Currency<AccountId>>::total_issuance().into_eth()
    }

    fn minimum_balance() -> Self::Balance {
        <Balances as Currency<AccountId>>::minimum_balance().into_eth()
    }

    fn burn(_amount: Self::Balance) -> Self::PositiveImbalance {
        unimplemented!()
    }

    fn issue(_amount: Self::Balance) -> Self::NegativeImbalance {
        unimplemented!()
    }

    fn free_balance(who: &AccountId) -> Self::Balance {
        <Balances as Currency<AccountId>>::free_balance(who).into_eth()
    }

    fn ensure_can_withdraw(
        who: &AccountId,
        amount: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        new_balance: Self::Balance,
    ) -> frame_support::pallet_prelude::DispatchResult {
        <Balances as Currency<AccountId>>::ensure_can_withdraw(
            who,
            amount.into_sub(),
            reasons,
            new_balance.into_sub(),
        )
    }

    fn transfer(
        source: &AccountId,
        dest: &AccountId,
        value: Self::Balance,
        existence_requirement: frame_support::traits::ExistenceRequirement,
    ) -> frame_support::pallet_prelude::DispatchResult {
        <Balances as Currency<AccountId>>::transfer(
            source,
            dest,
            value.into_sub(),
            existence_requirement,
        )
    }

    fn slash(who: &AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        let (imbalance, remaining) =
            <Balances as Currency<AccountId>>::slash(who, value.into_sub());
        (imbalance.into_eth(), remaining.into_eth())
    }

    fn deposit_into_existing(
        who: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, sp_runtime::DispatchError> {
        <Balances as Currency<AccountId>>::deposit_into_existing(who, value.into_sub())
            .map(|im| im.into_eth())
    }

    fn deposit_creating(who: &AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        <Balances as Currency<AccountId>>::deposit_creating(who, value.into_sub()).into_eth()
    }

    fn withdraw(
        who: &AccountId,
        value: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        liveness: frame_support::traits::ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, sp_runtime::DispatchError> {
        <Balances as Currency<AccountId>>::withdraw(who, value.into_sub(), reasons, liveness)
            .map(|im| im.into_eth())
    }

    fn make_free_balance_be(
        _who: &AccountId,
        _balance: Self::Balance,
    ) -> frame_support::traits::SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        unimplemented!()
    }

    fn active_issuance() -> Self::Balance {
        <Balances as Currency<AccountId>>::active_issuance().into_eth()
    }

    fn deactivate(amount: Self::Balance) {
        <Balances as Currency<AccountId>>::deactivate(amount.into_sub())
    }

    fn reactivate(amount: Self::Balance) {
        <Balances as Currency<AccountId>>::reactivate(amount.into_sub())
    }

    fn pair(_amount: Self::Balance) -> (Self::PositiveImbalance, Self::NegativeImbalance) {
        unimplemented!()
    }

    fn resolve_into_existing(
        _who: &AccountId,
        _value: Self::NegativeImbalance,
    ) -> Result<(), Self::NegativeImbalance> {
        unimplemented!()
    }

    fn resolve_creating(_who: &AccountId, _value: Self::NegativeImbalance) {
        unimplemented!()
    }

    fn settle(
        _who: &AccountId,
        _value: Self::PositiveImbalance,
        _reasons: frame_support::traits::WithdrawReasons,
        _liveness: frame_support::traits::ExistenceRequirement,
    ) -> Result<(), Self::PositiveImbalance> {
        unimplemented!()
    }
}

impl Inspect<AccountId> for EvmCurrency {
    type Balance = Balance;

    fn total_issuance() -> Self::Balance {
        <Balances as Inspect<AccountId>>::total_issuance().into_eth()
    }

    fn minimum_balance() -> Self::Balance {
        <Balances as Inspect<AccountId>>::minimum_balance().into_eth()
    }

    fn total_balance(who: &AccountId) -> Self::Balance {
        <Balances as Inspect<AccountId>>::total_balance(who).into_eth()
    }

    fn balance(who: &AccountId) -> Self::Balance {
        <Balances as Inspect<AccountId>>::balance(who).into_eth()
    }

    fn reducible_balance(
        who: &AccountId,
        preservation: frame_support::traits::tokens::Preservation,
        force: frame_support::traits::tokens::Fortitude,
    ) -> Self::Balance {
        <Balances as Inspect<AccountId>>::reducible_balance(who, preservation, force).into_eth()
    }

    fn can_deposit(
        who: &AccountId,
        amount: Self::Balance,
        provenance: frame_support::traits::tokens::Provenance,
    ) -> frame_support::traits::tokens::DepositConsequence {
        <Balances as Inspect<AccountId>>::can_deposit(who, amount.into_sub(), provenance)
    }

    fn can_withdraw(
        who: &AccountId,
        amount: Self::Balance,
    ) -> frame_support::traits::tokens::WithdrawConsequence<Self::Balance> {
        <Balances as Inspect<AccountId>>::can_withdraw(who, amount.into_sub())
    }

    fn active_issuance() -> Self::Balance {
        <Balances as Inspect<AccountId>>::active_issuance().into_eth()
    }
}
