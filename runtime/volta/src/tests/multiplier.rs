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

// Integration transaction weight-fee tests
use crate::{
    configs::{monetary::VFlowFeeUpdate, system::RuntimeBlockWeights},
    tests::run_with_system_weight,
    Runtime,
};
use frame_support::pallet_prelude::*;
use pallet_transaction_payment::Multiplier;
use sp_runtime::{traits::Convert, Perquintill};

fn min_multiplier() -> Multiplier {
    crate::configs::monetary::MinimumMultiplier::get()
}

fn target() -> Weight {
    Perquintill::from_percent(75)
        * RuntimeBlockWeights::get()
            .get(DispatchClass::Normal)
            .max_total
            .unwrap()
}

fn max() -> Weight {
    RuntimeBlockWeights::get()
        .get(DispatchClass::Normal)
        .max_total
        .unwrap()
}

fn runtime_multiplier_update(fm: Multiplier) -> Multiplier {
    <Runtime as pallet_transaction_payment::Config>::FeeMultiplierUpdate::convert(fm)
}

#[test]
fn multiplier_can_grow_from_zero() {
    // if the min is too small, then this will not change, and we are doomed forever.
    // the block ref time is 1/100th bigger than target.
    run_with_system_weight(
        target().set_ref_time(target().ref_time() * 101 / 100),
        || {
            let next = runtime_multiplier_update(min_multiplier());
            assert!(
                next > min_multiplier(),
                "{:?} !> {:?}",
                next,
                min_multiplier()
            );
        },
    );

    // the block proof size is 1/100th bigger than target.
    run_with_system_weight(
        target().set_proof_size((target().proof_size() / 100) * 101),
        || {
            let next = runtime_multiplier_update(min_multiplier());
            assert!(
                next > min_multiplier(),
                "{:?} !> {:?}",
                next,
                min_multiplier()
            );
        },
    )
}

#[test]
fn multiplier_cannot_go_below_limit() {
    // will not go any further below even if block is empty.
    run_with_system_weight(Weight::zero(), || {
        let next = runtime_multiplier_update(min_multiplier());
        assert_eq!(next, min_multiplier());
    })
}

#[test]
fn block_cost_after_k_full_blocks() {
    run_with_system_weight(max(), || {
        // We check that after k full blocks, the fee multiplier is ~26.67, so that filling a block
        // completely costs ~200 VFY, considering time only.
        let mut mul: Multiplier = 1.into();
        let k = 100;
        let final_mul = 26.67f64;
        for _i in 0..k {
            mul = VFlowFeeUpdate::<Runtime>::convert(mul);
        }

        assert!((mul.to_float() - final_mul).abs() < 1e-9f64);
    })
}

#[test]
fn block_cost_after_k_empty_blocks() {
    run_with_system_weight(Weight::zero(), || {
        let mut mul: Multiplier = 1.into();
        // We check that after k empty blocks, the fee multiplier never goes below the minimum.
        let k = 100;
        for _i in 0..k {
            mul = VFlowFeeUpdate::<Runtime>::convert(mul);
        }

        assert_eq!(mul, min_multiplier());
    })
}
