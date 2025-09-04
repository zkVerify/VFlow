use crate::{
    configs::xcm::{NativeAssetId, RelayLocation},
    constants::currency::VFY,
    tests::ALICE,
    AccountId, Runtime, RuntimeOrigin, ZKVXcm,
};
use frame_support::assert_ok;
use sp_runtime::BuildStorage;
use xcm::v5::{Asset, Assets, Fungibility, Junction, Location, WeightLimit};
use xcm::{VersionedAssets, VersionedLocation};

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE.into(), 10 * VFY)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    pallet_xcm::GenesisConfig::<Runtime> {
        safe_xcm_version: Some(3),
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    sp_io::TestExternalities::new(t)
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

        // The actual teleport will fail without relay chain, but construction works
        assert_ok!(ZKVXcm::limited_teleport_assets(
            RuntimeOrigin::signed(ALICE.into()),
            Box::new(destination),
            Box::new(beneficiary),
            Box::new(assets),
            0,
            WeightLimit::Unlimited
        ));
    });
}
