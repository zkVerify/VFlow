// Copyright 2025, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! In this module we provide the configurations about monetary: the set
//! of pallets used for currency management.

use crate::{
    currency::EXISTENTIAL_DEPOSIT, weights, AccountId, Balance, Balances, CollatorSelection,
    Runtime, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, System,
};
use frame_support::{parameter_types, traits::tokens::imbalance::ResolveTo};
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{traits::One, FixedPointNumber, Perquintill};
use sp_weights::ConstantMultiplier;

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const MaxFreezes: u32 = 0;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type WeightInfo = weights::pallet_balances::ZKVEvmWeight<Runtime>;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type MaxFreezes = MaxFreezes;
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const TransactionPicosecondFee: Balance = 5000000;
    pub const TransactionByteFee: Balance = 5000000;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(75);
    // AdjustmentVariable computed to result in a desired cost for filling n blocks in a row.
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1313646132342424i64, 10000000000000000i64);
    pub MinimumMultiplier: Multiplier = Multiplier::one();
    pub MaximumMultiplier: Multiplier = Multiplier::from(100_000u128);
    pub const OperationalFeeMultiplier: u8 = 5;
    pub StakingPot: AccountId = CollatorSelection::account_id();
}

pub type VFlowFeeUpdate<R> = TargetedFeeAdjustment<
    R,
    TargetBlockFullness,
    AdjustmentVariable,
    MinimumMultiplier,
    MaximumMultiplier,
>;

#[cfg(feature = "runtime-benchmarks")]
mod runtime_benchmarks {
    use crate::constants::currency::CENTS;
    use crate::Runtime;
    use core::marker::PhantomData;
    use sp_runtime::traits::{DispatchInfoOf, PostDispatchInfoOf};
    use sp_runtime::transaction_validity::TransactionValidityError;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type RuntimeCallOf<T> = <T as frame_system::Config>::RuntimeCall;

    /// This decorator will just forward all calls to the original `OnChargeTransaction` and
    /// override the `minimum_balance` method to return a fixed value of 1 cent instead the
    /// original implementation that return the `EXISTENTIAL_DEPOSIT`: not enough to perform a real
    /// transaction with a valid `WeightToFee` configuration.
    pub struct OnChargeTransactionRuntimeBenchmarks<T>(PhantomData<T>);

    impl<T: pallet_transaction_payment::OnChargeTransaction<Runtime>>
        pallet_transaction_payment::OnChargeTransaction<Runtime>
        for OnChargeTransactionRuntimeBenchmarks<T>
    where
        T::Balance: From<u128>,
    {
        type Balance = T::Balance;
        type LiquidityInfo = T::LiquidityInfo;

        fn withdraw_fee(
            who: &AccountIdOf<Runtime>,
            call: &RuntimeCallOf<Runtime>,
            dispatch_info: &DispatchInfoOf<RuntimeCallOf<Runtime>>,
            fee: Self::Balance,
            tip: Self::Balance,
        ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
            T::withdraw_fee(who, call, dispatch_info, fee, tip)
        }

        fn can_withdraw_fee(
            who: &AccountIdOf<Runtime>,
            call: &RuntimeCallOf<Runtime>,
            dispatch_info: &DispatchInfoOf<RuntimeCallOf<Runtime>>,
            fee: Self::Balance,
            tip: Self::Balance,
        ) -> Result<(), TransactionValidityError> {
            T::can_withdraw_fee(who, call, dispatch_info, fee, tip)
        }

        fn correct_and_deposit_fee(
            who: &AccountIdOf<Runtime>,
            dispatch_info: &DispatchInfoOf<RuntimeCallOf<Runtime>>,
            post_info: &PostDispatchInfoOf<RuntimeCallOf<Runtime>>,
            corrected_fee: Self::Balance,
            tip: Self::Balance,
            already_withdrawn: Self::LiquidityInfo,
        ) -> Result<(), TransactionValidityError> {
            T::correct_and_deposit_fee(
                who,
                dispatch_info,
                post_info,
                corrected_fee,
                tip,
                already_withdrawn,
            )
        }

        fn endow_account(who: &AccountIdOf<Runtime>, amount: Self::Balance) {
            T::endow_account(who, amount)
        }

        fn minimum_balance() -> Self::Balance
        where
            <T as pallet_transaction_payment::OnChargeTransaction<Runtime>>::Balance: From<u128>,
        {
            CENTS.into()
        }
    }
}

type OnChargeTransaction =
    pallet_transaction_payment::FungibleAdapter<Balances, ResolveTo<StakingPot, Balances>>;

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    // All the fees go to the collators, passing through the Pot of the CollatorSelection pallet

    #[cfg(not(feature = "runtime-benchmarks"))]
    type OnChargeTransaction = OnChargeTransaction;
    #[cfg(feature = "runtime-benchmarks")]
    // We are wrapping the original `OnChargeTransaction` with the decorator provided in
    // `runtime_benchmarks`. This decorator will provide the corrected `runtime-benchmarks`
    // behavior to test the benchmarked cases.
    type OnChargeTransaction =
        runtime_benchmarks::OnChargeTransactionRuntimeBenchmarks<OnChargeTransaction>;

    type WeightToFee = ConstantMultiplier<Balance, TransactionPicosecondFee>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = VFlowFeeUpdate<Self>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightInfo = weights::pallet_transaction_payment::ZKVEvmWeight<Runtime>;
}
