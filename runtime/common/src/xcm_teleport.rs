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

use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H256, U256};
use sp_runtime::traits::{Dispatchable, Get};
use sp_std::{boxed::Box, marker::PhantomData, vec};
use xcm::v5::{Asset, AssetId, Assets, Fungibility, Junction, Location};
use xcm::{VersionedAssets, VersionedLocation};

pub struct XcmTeleportPrecompile<R, O, C, L, A>(PhantomData<(R, O, C, L, A)>);

#[precompile_utils::precompile]
impl<R, O, C, L, A> XcmTeleportPrecompile<R, O, C, L, A>
where
    R: pallet_xcm::Config + pallet_evm::Config<RuntimeOrigin = O>,
    C: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + core::convert::From<pallet_xcm::Call<R>>,
    <R as frame_system::Config>::RuntimeCall:
        Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + From<C>,
    O: core::convert::From<
        frame_system::RawOrigin<
            <<R as pallet_evm::Config>::AccountProvider as fp_evm::AccountProvider>::AccountId,
        >,
    >,
    L: Get<Location>,
    A: Get<AssetId>,
{
    #[precompile::public("teleportToRelayChain(bytes32,uint256)")]
    fn teleport_to_relay_chain(
        handle: &mut impl PrecompileHandle,
        destination_account: H256,
        amount: U256,
    ) -> EvmResult {
        // No benchmarks availabe yet for precompiles, so charge some arbitrary gas as a spam
        // prevention mechanism.
        handle.record_cost(1000)?;

        // We use IdentityAddressMapping, so no db access
        let account_id =
            <R as pallet_evm::Config>::AddressMapping::into_account_id(handle.context().caller);
        let origin: O = frame_system::RawOrigin::Signed(account_id).into();

        let destination = VersionedLocation::V5(L::get());

        let beneficiary = VersionedLocation::V5(Location::new(
            0,
            [Junction::AccountId32 {
                network: None,
                id: destination_account.into(),
            }],
        ));

        let amount_u128: u128 = amount.try_into().map_err(|_| revert("Amount too large"))?;

        let assets = VersionedAssets::V5(Assets::from(vec![Asset {
            id: A::get(),
            fun: Fungibility::Fungible(amount_u128),
        }]));

        let fee_asset_item = 0;

        let call: C = pallet_xcm::Call::<R>::teleport_assets {
            dest: Box::new(destination),
            beneficiary: Box::new(beneficiary),
            assets: Box::new(assets),
            fee_asset_item,
        }
        .into();

        RuntimeHelper::<R>::try_dispatch::<C>(handle, origin, call, 0)?;

        Ok(())
    }
}
