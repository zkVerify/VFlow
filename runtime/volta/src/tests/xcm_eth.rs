use super::*;
use crate::{
    configs::xcm::RelayLocation,
    constants::currency::tVFY,
    tests::{ExtBuilder, ALICE, BOB},
    Balances, RuntimeCall, RuntimeOrigin,
};
use frame_support::{
    assert_err_ignore_postinfo, assert_noop, assert_ok,
    dispatch::{Pays, PostDispatchInfo},
    traits::ConstU32,
    BoundedVec,
};
use pallet_ethereum_xcm::RawOrigin;
use parity_scale_codec::Encode;
use sp_core::{H160, U256};
use sp_runtime::{DispatchError, DispatchErrorWithPostInfo, ModuleError};
use xcm::{
    latest::{Fungibility::Fungible, OriginKind, Xcm},
    prelude::*,
    VersionedXcm,
};
use xcm_primitives::{EthereumXcmTransaction, EthereumXcmTransactionV2};

const CONTRACT_BYTECODE: &[u8] = &hex_literal::hex!("6080604052348015600e575f5ffd5b506101298061001c5f395ff3fe6080604052348015600e575f5ffd5b50600436106030575f3560e01c80632e64cec11460345780636057361d14604e575b5f5ffd5b603a6066565b60405160459190608d565b60405180910390f35b606460048036038101906060919060cd565b606e565b005b5f5f54905090565b805f8190555050565b5f819050919050565b6087816077565b82525050565b5f602082019050609e5f8301846080565b92915050565b5f5ffd5b60af816077565b811460b8575f5ffd5b50565b5f8135905060c78160a8565b92915050565b5f6020828403121560df5760de60a4565b5b5f60ea8482850160bb565b9150509291505056fea264697066735822122063f96a57b86a37af1ac0fbf522233470beb0ae3e330dcafa317cb897259fa87364736f6c634300081e0033");

fn xcm_evm_transfer_eip_1559_transaction(destination: H160, value: U256) -> EthereumXcmTransaction {
    EthereumXcmTransaction::V2(EthereumXcmTransactionV2 {
        gas_limit: U256::from(21000),
        action: ethereum::TransactionAction::Call(destination),
        value,
        input:
            BoundedVec::<u8, ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>>::try_from(
                vec![],
            )
            .unwrap(),
        access_list: None,
    })
}

fn xcm_evm_create_eip_1559_transaction(bytecode: &[u8]) -> EthereumXcmTransaction {
    EthereumXcmTransaction::V2(EthereumXcmTransactionV2 {
        gas_limit: U256::from(300000),
        action: ethereum::TransactionAction::Create,
        value: U256::zero(),
        input:
            BoundedVec::<u8, ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>>::try_from(
                bytecode.to_vec(),
            )
            .unwrap(),
        access_list: None,
    })
}

#[test]
fn xcm_can_do_direct_eth_transfer() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), tVFY), (BOB.into(), tVFY)])
        .build()
        .execute_with(|| {
            let balances_before = Balances::free_balance(AccountId::from(BOB));
            let tx_value = tVFY / 2;
            assert_ok!(pallet_ethereum_xcm::Pallet::<Runtime>::transact(
                RawOrigin::XcmEthereumTransaction(ALICE.into()).into(),
                xcm_evm_transfer_eip_1559_transaction(BOB.into(), U256::from(tx_value)),
            ));

            assert_eq!(
                Balances::free_balance(AccountId::from(BOB)),
                balances_before + tx_value
            );
        });
}

#[test]
fn alice_cannot_do_direct_eth_transfer() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), tVFY), (BOB.into(), tVFY)])
        .build()
        .execute_with(|| {
            let tx_value = tVFY / 2;
            assert_noop!(
                pallet_ethereum_xcm::Pallet::<Runtime>::transact(
                    RuntimeOrigin::signed(ALICE.into()),
                    xcm_evm_transfer_eip_1559_transaction(BOB.into(), U256::from(tx_value)),
                ),
                DispatchError::BadOrigin
            );
        });
}

#[test]
fn can_call_eth_from_xcm() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), tVFY), (BOB.into(), tVFY)])
        .build()
        .execute_with(|| {
            let balances_before = Balances::free_balance(AccountId::from(BOB));
            let tx_value = tVFY / 10;
            let xcm_cost = tVFY / 2;
            let eth_call_bytes =
                RuntimeCall::from(pallet_ethereum_xcm::Call::<Runtime>::transact {
                    xcm_transaction: xcm_evm_transfer_eip_1559_transaction(
                        BOB.into(),
                        U256::from(tx_value),
                    ),
                })
                .encode()
                .into();

            let base_xcm = Box::new(VersionedXcm::from(Xcm(vec![
                WithdrawAsset((RelayLocation::get(), Fungible(xcm_cost)).into()),
                BuyExecution {
                    fees: (RelayLocation::get(), Fungible(xcm_cost)).into(),
                    weight_limit: Unlimited,
                },
                Transact {
                    origin_kind: OriginKind::Native,
                    call: eth_call_bytes,
                    fallback_max_weight: None,
                },
            ])));

            assert_ok!(pallet_xcm::Pallet::<Runtime>::execute(
                RuntimeOrigin::signed(ALICE.into()),
                base_xcm,
                Weight::from_parts(10000000000, 10000),
            ));
            assert_eq!(
                Balances::free_balance(AccountId::from(BOB)),
                balances_before + tx_value
            );
        })
}

// This is currently not supported by pallet_ethereum_xcm
#[test]
fn cannot_create_eth_from_xcm() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), tVFY)])
        .build()
        .execute_with(|| {
            assert_noop!(
                pallet_ethereum_xcm::Pallet::<Runtime>::transact(
                    RawOrigin::XcmEthereumTransaction(ALICE.into()).into(),
                    xcm_evm_create_eip_1559_transaction(CONTRACT_BYTECODE),
                ),
                DispatchErrorWithPostInfo {
                    post_info: PostDispatchInfo {
                        // as per pallet_ethereum_xcm implementation
                        actual_weight: Some(
                            <Runtime as frame_system::Config>::DbWeight::get().reads(1)
                        ),
                        pays_fee: Pays::Yes,
                    },
                    error: DispatchError::Other("Cannot convert xcm payload to known type"),
                }
            );
        })
}

#[test]
fn cannot_call_transact_remark_from_xcm() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE.into(), tVFY), (BOB.into(), tVFY)])
        .build()
        .execute_with(|| {
            let xcm_cost = tVFY / 2;
            let eth_call_bytes = RuntimeCall::from(frame_system::Call::<Runtime>::remark {
                remark: hex_literal::hex!("beeb").to_vec(),
            })
            .encode()
            .into();

            let base_xcm = Box::new(VersionedXcm::from(Xcm(vec![
                WithdrawAsset((RelayLocation::get(), Fungible(xcm_cost)).into()),
                BuyExecution {
                    fees: (RelayLocation::get(), Fungible(xcm_cost)).into(),
                    weight_limit: Unlimited,
                },
                Transact {
                    origin_kind: OriginKind::Native,
                    call: eth_call_bytes,
                    fallback_max_weight: None,
                },
            ])));

            assert_err_ignore_postinfo!(
                pallet_xcm::Pallet::<Runtime>::execute(
                    RuntimeOrigin::signed(ALICE.into()),
                    base_xcm,
                    Weight::from_parts(10000000000, 10000),
                ),
                DispatchError::Module(ModuleError {
                    index: 31,
                    error: [24, 0, 0, 0],
                    message: Some("LocalExecutionIncomplete")
                }),
            );
        })
}
