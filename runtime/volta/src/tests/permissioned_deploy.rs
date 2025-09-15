use super::*;
use crate::{
    constants::currency::{CENTS, MICROCENTS},
    tests::{ExtBuilder, ALICE, BOB},
    RuntimeOrigin,
};
use frame_support::{assert_err_ignore_postinfo, assert_ok};
use sp_core::H256;
use sp_runtime::DispatchError;

#[test]
fn create_with_whitelisted_account_succeeds() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), 11 * CENTS)])
        .build()
        .execute_with(|| {
            pallet_deployment_permissions::Pallet::<Runtime>::grant_deploy_permission(
                RuntimeOrigin::root(),
                ALICE.into(),
            )
            .unwrap();

            assert_ok!(pallet_evm::Pallet::<Runtime>::create(
                RuntimeOrigin::root(),
                ALICE.into(),
                contract_bytecode(),
                0.into(),
                100_000,
                (100 * MICROCENTS).into(),
                None,
                None,
                Vec::new(),
            ));
        });
}

#[test]
fn create2_with_whitelisted_account_succeeds() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), 11 * CENTS)])
        .build()
        .execute_with(|| {
            pallet_deployment_permissions::Pallet::<Runtime>::grant_deploy_permission(
                RuntimeOrigin::root(),
                ALICE.into(),
            )
            .unwrap();

            assert_ok!(pallet_evm::Pallet::<Runtime>::create2(
                RuntimeOrigin::root(),
                ALICE.into(),
                contract_bytecode(),
                H256::default(),
                0.into(),
                100_000,
                (100 * MICROCENTS).into(),
                None,
                None,
                Vec::new(),
            ));
        });
}

#[test]
fn create_with_non_whitelisted_account_fails() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), 1_000 * CENTS)])
        .build()
        .execute_with(|| {
            pallet_deployment_permissions::Pallet::<Runtime>::grant_deploy_permission(
                RuntimeOrigin::root(),
                BOB.into(),
            )
            .unwrap();

            assert_err_ignore_postinfo!(
                pallet_evm::Pallet::<Runtime>::create(
                    RuntimeOrigin::root(),
                    ALICE.into(),
                    contract_bytecode(),
                    0.into(),
                    1_000_000,
                    MICROCENTS.into(),
                    None,
                    None,
                    Vec::new(),
                ),
                DispatchError::Other("Not allowed to deploy")
            );
        });
}

#[test]
fn create2_with_non_whitelisted_account_fails() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), 1_000 * CENTS)])
        .build()
        .execute_with(|| {
            pallet_deployment_permissions::Pallet::<Runtime>::grant_deploy_permission(
                RuntimeOrigin::root(),
                BOB.into(),
            )
            .unwrap();

            assert_err_ignore_postinfo!(
                pallet_evm::Pallet::<Runtime>::create2(
                    RuntimeOrigin::root(),
                    ALICE.into(),
                    contract_bytecode(),
                    H256::default(),
                    0.into(),
                    1_000_000,
                    MICROCENTS.into(),
                    None,
                    None,
                    Vec::new(),
                ),
                DispatchError::Other("Not allowed to deploy")
            );
        });
}

fn contract_bytecode() -> Vec<u8> {
    // pragma solidity >=0.8.2 <0.9.0;
    //
    // contract Storage {
    //     uint256 number;
    //
    //     function store(uint256 num) public {
    //         number = num;
    //     }
    //
    //     function retrieve() public view returns (uint256){
    //         return number;
    //     }
    // }
    hex::decode(concat!(
        "608060405234801561000f575f80fd5b5060043610610034575f3560e01c8063",
        "2e64cec1146100385780636057361d14610056575b5f80fd5b61004061007256",
        "5b60405161004d919061009b565b60405180910390f35b610070600480360381",
        "019061006b91906100e2565b61007a565b005b5f8054905090565b805f819055",
        "5050565b5f819050919050565b61009581610083565b82525050565b5f602082",
        "0190506100ae5f83018461008c565b92915050565b5f80fd5b6100c181610083",
        "565b81146100cb575f80fd5b50565b5f813590506100dc816100b8565b929150",
        "50565b5f602082840312156100f7576100f66100b4565b5b5f61010484828501",
        "6100ce565b9150509291505056fea26469706673582212209a0dd35336aff1eb",
        "3eeb11db76aa60a1427a12c1b92f945ea8c8d1dfa337cf2264736f6c63430008",
        "1a0033",
    ))
    .unwrap()
}
