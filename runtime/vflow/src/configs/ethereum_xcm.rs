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

//! In this module, we provide the configurations for the ethereum-xcm pallet.

use crate::{configs::system::ReservedXcmpWeight, AccountId, Runtime, RuntimeEvent};
use frame_system::EnsureRoot;

pub struct EthereumXcmEnsureProxy;
impl xcm_primitives::EnsureProxy<AccountId> for EthereumXcmEnsureProxy {
    fn ensure_ok(_delegator: AccountId, _delegatee: AccountId) -> Result<(), &'static str> {
        Err("proxy pallet removed")
    }
}

impl pallet_ethereum_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type InvalidEvmTransactionError = pallet_ethereum::InvalidTransactionWrapper;
    type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
    type XcmEthereumOrigin = pallet_ethereum_xcm::EnsureXcmEthereumTransaction;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type EnsureProxy = EthereumXcmEnsureProxy;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ForceOrigin = EnsureRoot<AccountId>;
}
