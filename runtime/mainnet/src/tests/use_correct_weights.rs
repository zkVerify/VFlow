// Copyright 2024, Horizen Labs, Inc.

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

//! Here we write the integration tests that just check pallets weighs are correctly linked.

use crate::{weights, Runtime};
use frame_support::pallet_prelude::DispatchClass;
#[test]
fn frame_system() {
    use frame_system::WeightInfo;

    assert_eq!(
        <Runtime as frame_system::Config>::SystemWeightInfo::set_heap_pages(),
        weights::frame_system::ZKVEvmWeight::<Runtime>::set_heap_pages()
    );

    assert_eq!(
        <Runtime as frame_system::Config>::DbWeight::get(),
        weights::db::constants::RocksDbWeight::get()
    );

    assert_eq!(
        <Runtime as frame_system::Config>::BlockWeights::get().base_block,
        weights::block_weights::BlockExecutionWeight::get()
    );

    assert_eq!(
        <Runtime as frame_system::Config>::BlockWeights::get()
            .per_class
            .get(DispatchClass::Normal)
            .base_extrinsic,
        weights::extrinsic_weights::ExtrinsicBaseWeight::get()
    );
}

#[test]
fn cumulus_pallet_parachain_system() {
    use cumulus_pallet_parachain_system::WeightInfo;

    assert_eq!(
        <Runtime as cumulus_pallet_parachain_system::Config>::WeightInfo::enqueue_inbound_downward_messages(0),
        weights::cumulus_pallet_parachain_system::ZKVEvmWeight::<Runtime>::enqueue_inbound_downward_messages(0)
    );
}

#[test]
fn pallet_timestamp() {
    use pallet_timestamp::WeightInfo;

    assert_eq!(
        <Runtime as pallet_timestamp::Config>::WeightInfo::set(),
        weights::pallet_timestamp::ZKVEvmWeight::<Runtime>::set()
    );
}

#[test]
fn pallet_proxy() {
    use pallet_proxy::WeightInfo;

    assert_eq!(
        <Runtime as pallet_proxy::Config>::WeightInfo::create_pure(1),
        weights::pallet_proxy::ZKVEvmWeight::<Runtime>::create_pure(1)
    );
}

#[test]
fn pallet_utility() {
    use pallet_utility::WeightInfo;

    assert_eq!(
        <Runtime as pallet_utility::Config>::WeightInfo::dispatch_as(),
        weights::pallet_utility::ZKVEvmWeight::<Runtime>::dispatch_as()
    );
}

#[test]
fn pallet_multisig() {
    use pallet_multisig::WeightInfo;

    assert_eq!(
        <Runtime as pallet_multisig::Config>::WeightInfo::as_multi_approve(3, 100),
        weights::pallet_multisig::ZKVEvmWeight::<Runtime>::as_multi_approve(3, 100)
    );
}

#[test]
fn pallet_balances() {
    use pallet_balances::WeightInfo;

    assert_eq!(
        <Runtime as pallet_balances::Config>::WeightInfo::transfer_allow_death(),
        weights::pallet_balances::ZKVEvmWeight::<Runtime>::transfer_allow_death()
    );
}

#[test]
fn pallet_sudo() {
    use pallet_sudo::WeightInfo;

    assert_eq!(
        <Runtime as pallet_sudo::Config>::WeightInfo::sudo(),
        weights::pallet_sudo::ZKVEvmWeight::<Runtime>::sudo()
    );
}

#[test]
fn pallet_collator_selection() {
    use pallet_collator_selection::WeightInfo;

    assert_eq!(
        <Runtime as pallet_collator_selection::Config>::WeightInfo::set_desired_candidates(),
        weights::pallet_collator_selection::ZKVEvmWeight::<Runtime>::set_desired_candidates()
    );
}

#[test]
fn pallet_session() {
    use pallet_session::WeightInfo;

    assert_eq!(
        <Runtime as pallet_session::Config>::WeightInfo::set_keys(),
        weights::pallet_session::ZKVEvmWeight::<Runtime>::set_keys()
    );
}

#[test]
fn cumulus_pallet_xcmp_queue() {
    use cumulus_pallet_xcmp_queue::WeightInfo;

    assert_eq!(
        <Runtime as cumulus_pallet_xcmp_queue::Config>::WeightInfo::set_config_with_u32(),
        weights::cumulus_pallet_xcmp_queue::ZKVEvmWeight::<Runtime>::set_config_with_u32()
    );
}

#[test]
fn pallet_xcm() {
    use pallet_xcm::WeightInfo;

    assert_eq!(
        <Runtime as pallet_xcm::Config>::WeightInfo::send(),
        weights::pallet_xcm::ZKVEvmWeight::<Runtime>::send()
    )
}

#[test]
fn pallet_message_queue() {
    use pallet_message_queue::WeightInfo;

    assert_eq!(
        <Runtime as pallet_message_queue::Config>::WeightInfo::ready_ring_knit(),
        weights::pallet_message_queue::ZKVEvmWeight::<Runtime>::ready_ring_knit()
    )
}

#[test]
fn pallet_evm() {
    use pallet_evm::WeightInfo;

    assert_eq!(
        <Runtime as pallet_evm::Config>::WeightInfo::withdraw(),
        weights::pallet_evm::ZKVEvmWeight::<Runtime>::withdraw()
    )
}

#[test]
fn pallet_deployment_permissions() {
    use pallet_deployment_permissions::WeightInfo;

    assert_eq!(
        <Runtime as pallet_deployment_permissions::Config>::WeightInfo::grant_deploy_permission(),
        weights::pallet_deployment_permissions::ZKVEvmWeight::<Runtime>::grant_deploy_permission()
    )
}
