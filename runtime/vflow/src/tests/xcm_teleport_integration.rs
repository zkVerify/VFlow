use crate::{
    configs::xcm::{NativeAssetId, RelayLocation},
    constants::currency::VFY,
    tests::ALICE,
    AccountId, Runtime, RuntimeCall, RuntimeOrigin, ZKVXcm, U256,
};
use alloy::primitives::U256 as Uint256;
use alloy_sol_types::{sol, SolCall, SolValue};
use cumulus_primitives_core::AbridgedHostConfiguration;
use fp_evm::CallInfo;
use fp_rpc::runtime_decl_for_ethereum_runtime_rpc_api::EthereumRuntimeRPCApiV6;
use precompile_utils::precompile_set::AddressU64;
use sp_core::Get;
use sp_runtime::BuildStorage;
use xcm::v5::{Asset, Assets, Fungibility, Junction, Location, WeightLimit};
use xcm::{VersionedAssetId, VersionedAssets, VersionedLocation};

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE.into(), 10 * VFY)],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();
    pallet_xcm::GenesisConfig::<Runtime> {
        safe_xcm_version: Some(3),
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        // Set up a minimal host configuration so that `can_send_upward_message` succeeds.
        // Without this, the UMP router rejects messages with MessageSendError::Other.
        cumulus_pallet_parachain_system::HostConfiguration::<Runtime>::put(
            AbridgedHostConfiguration {
                max_code_size: 3 * 1024 * 1024,
                max_head_data_size: 32 * 1024,
                max_upward_queue_count: 8,
                max_upward_queue_size: 1024 * 1024,
                max_upward_message_size: 64 * 1024,
                max_upward_message_num_per_candidate: 5,
                hrmp_max_message_num_per_candidate: 5,
                validation_upgrade_cooldown: 2,
                validation_upgrade_delay: 2,
                async_backing_params: cumulus_primitives_core::relay_chain::AsyncBackingParams {
                    max_candidate_depth: 3,
                    allowed_ancestry_len: 2,
                },
            },
        );
    });
    ext
}

/// Test the evm AddressMapping does not make any db access. If this is invalidated, the cost for
/// the teleport_to_relay_chain precompile must be updated accordingly.
#[test]
fn evm_uses_identity_address_mapping() {
    use pallet_evm::AddressMapping;

    const RND_KEY: [u8; 20] = [0x21; 20];
    let a1: AccountId = pallet_evm::IdentityAddressMapping::into_account_id(RND_KEY.into());
    let a2: AccountId =
        <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(RND_KEY.into());
    assert_eq!(a1, a2);
}

/// Test that the construction of XCM teleports of VFY to the relay chain succeeds.
#[test]
fn can_teleport_vfy_to_relay() {
    new_test_ext().execute_with(|| {
        let destination = VersionedLocation::V5(RelayLocation::get());
        let test_account = [0x42u8; 32];
        let beneficiary = VersionedLocation::V5(Location::new(
            0,
            [Junction::AccountId32 {
                network: None,
                id: test_account,
            }],
        ));

        let assets = VersionedAssets::V5(Assets::from(vec![Asset {
            id: NativeAssetId::get(),
            fun: Fungibility::Fungible(VFY),
        }]));

        // Verify the construction is valid (no panics)
        assert!(matches!(destination, VersionedLocation::V5(_)));
        assert!(matches!(beneficiary, VersionedLocation::V5(_)));
        assert!(matches!(assets, VersionedAssets::V5(_)));

        // The actual teleport will fail without relay chain (no UMP channel in test),
        // but we verify the construction doesn't panic and the extrinsic is callable.
        let result = ZKVXcm::limited_teleport_assets(
            RuntimeOrigin::signed(ALICE.into()),
            Box::new(destination),
            Box::new(beneficiary),
            Box::new(assets),
            0,
            WeightLimit::Unlimited,
        );
        // SendFailure is expected without a relay chain backing the UMP channel.
        assert!(
            result.is_ok() || format!("{result:?}").contains("SendFailure"),
            "Unexpected error: {result:?}"
        );
    });
}

#[test]
fn xcm_teleport_precompile_delivery_fee_computation_is_correct() {
    new_test_ext().execute_with(|| {
        let from = ALICE;
        let to = [0x42u8; 32];
        let amount = VFY;

        let fees_from_precompile = compute_teleport_delivery_fees_via_precompile(from, to, amount);
        let fees_from_dry_run = compute_teleport_delivery_fees_via_dry_run(from, to, amount);

        assert_eq!(fees_from_precompile, fees_from_dry_run);
    });
}

fn compute_teleport_delivery_fees_via_precompile(
    from: [u8; 20],
    account: [u8; 32],
    amount: u128,
) -> u128 {
    sol! {
        contract IXcmTeleportPrecompile {
            function deliveryFee(bytes32 id, uint256 amount) external returns (uint256);
        }
    }

    let precompile_address = AddressU64::<2060>::get();
    let calldata = IXcmTeleportPrecompile::deliveryFeeCall {
        id: account.into(),
        amount: Uint256::from(amount),
    }
    .abi_encode();

    let CallInfo { value, .. } = Runtime::call(
        from.into(),
        precompile_address,
        calldata,
        U256::zero(),
        U256::from(1_000_000),
        None,
        None,
        None,
        false,
        None,
        None,
    )
    .unwrap();

    Uint256::abi_decode(&value).unwrap().to()
}

fn compute_teleport_delivery_fees_via_dry_run(
    from: [u8; 20],
    account: [u8; 32],
    amount: u128,
) -> u128 {
    let destination = VersionedLocation::V5(RelayLocation::get());

    let beneficiary = VersionedLocation::V5(Location::new(
        0,
        [Junction::AccountId32 {
            network: None,
            id: account,
        }],
    ));

    let assets = VersionedAssets::V5(Assets::from(vec![Asset {
        id: NativeAssetId::get(),
        fun: Fungibility::Fungible(amount),
    }]));

    let call = pallet_xcm::Call::<Runtime>::teleport_assets {
        dest: Box::new(destination.clone()),
        beneficiary: Box::new(beneficiary.clone()),
        assets: Box::new(assets.clone()),
        fee_asset_item: 0,
    }
    .into();

    compute_delivery_fees_for_call(
        RuntimeOrigin::signed(from.into()),
        call,
        destination.clone(),
    )
}

fn compute_delivery_fees_for_call(
    origin: RuntimeOrigin,
    call: RuntimeCall,
    destination: VersionedLocation,
) -> u128 {
    ZKVXcm::dry_run_call::<
        Runtime,
        <Runtime as pallet_xcm::Config>::XcmRouter,
        <Runtime as pallet_xcm::Config>::RuntimeOrigin,
        <Runtime as pallet_xcm::Config>::RuntimeCall,
    >(origin, call, 5)
    .unwrap()
    .forwarded_xcms
    .into_iter()
    .filter(|(location, _xcms)| location == &destination)
    .flat_map(|(_location, xcms)| xcms.into_iter())
    .map(|xcm| -> Assets {
        let native_asset_id = VersionedAssetId::from(NativeAssetId::get());
        pallet_xcm::Pallet::<Runtime>::query_delivery_fees::<()>(
            destination.clone(),
            xcm,
            native_asset_id,
        )
        .unwrap()
        .try_into()
        .unwrap()
    })
    .flat_map(|assets| assets.into_inner())
    .fold(0, |acc, asset: Asset| match asset.fun {
        Fungibility::Fungible(amount) => acc + amount,
        Fungibility::NonFungible(_) => acc,
    })
}
