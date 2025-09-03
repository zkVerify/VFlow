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

// Storage indices integration checks
use crate::*;
use frame_support::traits::PalletInfo;

fn assert_pallet_prefix<P: 'static>(name: &str) {
    assert_eq!(
        <Runtime as frame_system::Config>::PalletInfo::name::<P>(),
        Some(name)
    );
}

#[test]
fn verify_pallet_prefixes() {
    assert_pallet_prefix::<System>("System");
    assert_pallet_prefix::<ParachainSystem>("ParachainSystem");
    assert_pallet_prefix::<Timestamp>("Timestamp");
    assert_pallet_prefix::<ParachainInfo>("ParachainInfo");
    assert_pallet_prefix::<Proxy>("Proxy");
    assert_pallet_prefix::<Balances>("Balances");
    assert_pallet_prefix::<TransactionPayment>("TransactionPayment");
    assert_pallet_prefix::<Sudo>("Sudo");
    assert_pallet_prefix::<Multisig>("Multisig");
    assert_pallet_prefix::<Authorship>("Authorship");
    assert_pallet_prefix::<Session>("Session");
    assert_pallet_prefix::<Aura>("Aura");
    assert_pallet_prefix::<AuraExt>("AuraExt");
    assert_pallet_prefix::<XcmpQueue>("XcmpQueue");
    assert_pallet_prefix::<ZKVXcm>("ZKVXcm");
    assert_pallet_prefix::<CumulusXcm>("CumulusXcm");
    assert_pallet_prefix::<MessageQueue>("MessageQueue");
}
