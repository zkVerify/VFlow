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

use crate::alloc::borrow::ToOwned;
use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H256, U256};
use sp_runtime::traits::{Dispatchable, Get};
use sp_std::{boxed::Box, marker::PhantomData, vec};
use xcm::{
    v5::{
        Asset,
        AssetFilter::Wild,
        AssetId, Assets, Fungibility,
        Instruction::{BuyExecution, ClearOrigin, DepositAsset, ReceiveTeleportedAsset},
        Junction, Location, Reanchorable, WeightLimit,
        WildAsset::AllCounted,
        Xcm,
    },
    VersionedAssets, VersionedLocation, VersionedXcm,
};

pub struct XcmTeleportPrecompile<R, O, C, L, A>(PhantomData<(R, O, C, L, A)>);

struct TeleportCallParams {
    destination: Location,
    beneficiary: Location,
    assets: Assets,
    fee_asset_item: u32,
}

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

        let TeleportCallParams {
            destination,
            beneficiary,
            assets,
            fee_asset_item,
        } = Self::build_teleport_params(destination_account, amount)?;

        let call: C = pallet_xcm::Call::<R>::teleport_assets {
            dest: Box::new(VersionedLocation::V5(destination)),
            beneficiary: Box::new(VersionedLocation::V5(beneficiary)),
            assets: Box::new(VersionedAssets::V5(assets)),
            fee_asset_item,
        }
        .into();

        RuntimeHelper::<R>::try_dispatch::<C>(handle, origin, call, 0)?;

        Ok(())
    }

    #[precompile::public("deliveryFee(bytes32,uint256)")]
    fn delivery_fee(
        handle: &mut impl PrecompileHandle,
        destination_account: H256,
        amount: U256,
    ) -> EvmResult<U256> {
        // No benchmarks availabe yet for precompiles, so charge some arbitrary gas as a spam
        // prevention mechanism.
        handle.record_cost(1000)?;

        let TeleportCallParams {
            destination,
            beneficiary,
            assets,
            fee_asset_item,
        } = Self::build_teleport_params(destination_account, amount)?;

        let program = Self::teleport_assets_program(
            destination.clone().into(),
            beneficiary.into(),
            assets.into(),
            fee_asset_item,
        )?;

        let versioned_fees = pallet_xcm::Pallet::<R>::query_delivery_fees(
            VersionedLocation::V5(destination),
            VersionedXcm::V5(program),
        )
        .map_err(|_| revert("cannot query delivery fees"))?;

        let fees: Assets = versioned_fees.try_into().unwrap();
        match { &fees.get(fee_asset_item as usize).unwrap().fun } {
            Fungibility::Fungible(amount) => Ok(U256::from(*amount)),
            _ => unreachable!(),
        }
    }

    fn teleport_assets_program(
        dest: Location,
        beneficiary: Location,
        assets: Assets,
        fee_asset_item: u32,
    ) -> EvmResult<Xcm<()>> {
        let fees = assets
            .get(fee_asset_item as usize)
            .ok_or_else(|| RevertReason::read_out_of_bounds("fees"))?
            .to_owned();
        let max_assets = assets.len() as u32;
        let context = R::UniversalLocation::get();
        let reanchored_assets = assets
            .reanchored(&dest, &context)
            .map_err(|_| revert("cannot reanchor assets"))?;
        let reanchored_fees = fees
            .reanchored(&dest, &context)
            .map_err(|_| revert("cannot reanchor fees"))?;

        Ok(Xcm(vec![
            ReceiveTeleportedAsset(reanchored_assets),
            ClearOrigin,
            BuyExecution {
                fees: reanchored_fees,
                weight_limit: WeightLimit::Unlimited,
            },
            DepositAsset {
                assets: Wild(AllCounted(max_assets)),
                beneficiary,
            },
        ]))
    }

    fn build_teleport_params(
        destination_account: H256,
        amount: U256,
    ) -> EvmResult<TeleportCallParams> {
        let destination = L::get();

        let beneficiary = Location::new(
            0,
            [Junction::AccountId32 {
                network: None,
                id: destination_account.into(),
            }],
        );

        let amount_u128 = amount
            .try_into()
            .map_err(|_| RevertReason::value_is_too_large("amount"))?;

        let assets = Assets::from(vec![Asset {
            id: A::get(),
            fun: Fungibility::Fungible(amount_u128),
        }]);

        let fee_asset_item = 0;

        Ok(TeleportCallParams {
            destination,
            beneficiary,
            assets,
            fee_asset_item,
        })
    }
}
