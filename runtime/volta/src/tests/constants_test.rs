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

use crate::*;

mod constant_tests {
    use super::*;
    use constants::currency::*;

    #[test]
    fn test_constants() {
        assert_eq!(MICROCENTS, 10_000_000_000);

        assert_eq!(MILLICENTS, 10_000_000_000_000);

        assert_eq!(CENTS, 1_000 * MILLICENTS);

        assert_eq!(VFY, 100 * CENTS);

        #[cfg(not(feature = "runtime-benchmarks"))]
        let expected_existential_deposit = 0;
        #[cfg(feature = "runtime-benchmarks")]
        let expected_existential_deposit = 100;

        assert_eq!(EXISTENTIAL_DEPOSIT, expected_existential_deposit);

        // Ensure deposit function behavior remains constant
        assert_eq!(deposit(2, 3), 2 * 200 * CENTS + 3 * 100 * MILLICENTS);
    }
}

mod runtime_tests {
    use super::*;
    use crate::configs::system::SS58Prefix;
    use constants::{currency::*, *};
    use frame_support::{pallet_prelude::Weight, PalletId};

    #[test]
    fn frame_system_constants() {
        assert_eq!(
            MAXIMUM_BLOCK_WEIGHT,
            Weight::from_parts(
                frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
                cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64
            )
        );

        assert_eq!(AVERAGE_ON_INITIALIZE_RATIO, Perbill::from_percent(5));

        assert_eq!(NORMAL_DISPATCH_RATIO, Perbill::from_percent(75));

        assert_eq!(UNINCLUDED_SEGMENT_CAPACITY, 3);

        assert_eq!(BLOCK_PROCESSING_VELOCITY, 1);

        assert_eq!(RELAY_CHAIN_SLOT_DURATION_MILLIS, 6000);

        assert_eq!(MILLISECS_PER_BLOCK, 6000);

        assert_eq!(SLOT_DURATION, MILLISECS_PER_BLOCK);

        assert_eq!(MINUTES, 60_000 / (MILLISECS_PER_BLOCK as BlockNumber));

        assert_eq!(HOURS, MINUTES * 60);

        assert_eq!(DAYS, HOURS * 24);

        assert_eq!(MAX_BLOCK_LENGTH, 5 * 1024 * 1024);

        assert_eq!(SS58Prefix::get(), 0);

        assert_eq!(
            <<Runtime as frame_system::Config>::MaxConsumers as sp_core::Get<u32>>::get(),
            16
        );
    }

    #[test]
    fn proxy_constants() {
        use configs::system::*;
        assert_eq!(MaxProxies::get(), 32);

        assert_eq!(MaxPending::get(), 32);

        assert_eq!(ProxyDepositBase::get(), deposit(1, 40));

        assert_eq!(AnnouncementDepositBase::get(), deposit(1, 48));

        assert_eq!(ProxyDepositFactor::get(), deposit(0, 33));

        assert_eq!(AnnouncementDepositFactor::get(), deposit(0, 66));
    }

    #[test]
    fn balances_constants() {
        use configs::monetary::*;
        assert_eq!(MaxFreezes::get(), 0);

        assert_eq!(MaxLocks::get(), 50);

        assert_eq!(MaxReserves::get(), 50);
    }

    #[test]
    fn transaction_payment_constants() {
        use configs::monetary::*;

        assert_eq!(TransactionPicosecondFee::get(), 5000000);

        assert_eq!(TransactionByteFee::get(), 5000000);

        assert_eq!(OperationalFeeMultiplier::get(), 5);
    }

    #[test]
    fn cumulus_pallet_parachain_system_constants() {
        use configs::system::*;

        assert_eq!(
            ReservedXcmpWeight::get(),
            MAXIMUM_BLOCK_WEIGHT.saturating_div(4)
        );

        assert_eq!(
            ReservedDmpWeight::get(),
            MAXIMUM_BLOCK_WEIGHT.saturating_div(4)
        );
    }

    #[test]
    fn message_queue_constants() {
        use configs::xcm::*;

        assert_eq!(HeapSize::get(), 103 * 1024);
        assert_eq!(MaxStale::get(), 8);
    }

    #[test]
    fn cumulus_pallet_xcmp_queue_constants() {
        use configs::xcm::*;
        assert_eq!(MaxInboundSuspended::get(), 1000);
    }

    #[test]
    fn multisig_constants() {
        use configs::system::*;

        assert_eq!(DepositBase::get(), deposit(1, 88));

        assert_eq!(DepositFactor::get(), deposit(0, 32));

        assert_eq!(MaxSignatories::get(), 100);
    }

    #[test]
    fn session_constants() {
        use configs::consensus::*;

        assert_eq!(Period::get(), 6 * HOURS);

        assert_eq!(Offset::get(), 0);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn aura_constants() {
        use configs::consensus::*;

        assert!(AllowMultipleBlocksPerSlot::get());

        assert_eq!(MaxAuthorities::get(), 100_000);
    }

    #[test]
    fn collator_selection_constants() {
        use configs::consensus::*;

        let pallet_id_to_string = |id: PalletId| -> String {
            core::str::from_utf8(&id.0).unwrap_or_default().to_string()
        };

        assert_eq!(
            pallet_id_to_string(PotId::get()),
            pallet_id_to_string(PalletId(*b"PotStake"))
        );

        assert_eq!(SessionLength::get(), 6 * HOURS);

        assert_eq!(MaxCandidates::get(), 30);

        assert_eq!(MaxInvulnerables::get(), 10);

        assert_eq!(MinEligibleCollators::get(), 1);
    }
}

mod xcm_tests {
    use super::*;
    use configs::xcm::*;

    #[test]
    fn xcm_executor_constants() {
        assert_eq!(MaxInstructions::get(), 30);
        assert_eq!(MaxAssetsIntoHolding::get(), 64);
    }

    #[test]
    fn pallet_xcm_constants() {
        assert_eq!(MaxLockers::get(), 8);
        assert_eq!(MaxRemoteLockConsumers::get(), 0);
        assert_eq!(
            <Runtime as pallet_xcm::Config>::VERSION_DISCOVERY_QUEUE_SIZE,
            100
        );
    }
}
