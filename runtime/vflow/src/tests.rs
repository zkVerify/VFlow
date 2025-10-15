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

#![allow(dead_code)]

// ExtBuilder impl for all runtime integration tests
use crate::{
    configs::evm::TransactionConverter, AccountId, Balance, BuildStorage, Runtime, System,
    UncheckedExtrinsic,
};
use frame_support::weights::Weight;

mod constants_test;
mod multiplier;
mod permissioned_deploy;
mod storage;
mod use_correct_weights;
mod xcm_eth;
mod xcm_teleport_integration;

mod misc {
    use super::*;

    #[test]
    fn check_version() {
        let v_str = std::env!("CARGO_PKG_VERSION");
        let convert = |v: &str| {
            v.split('.')
                .map(|x| x.parse::<u32>().unwrap())
                .rev()
                .enumerate()
                .fold(0, |a, (step, dec)| a + dec * 1000_u32.pow(step as u32))
        };

        let v_num = convert(v_str);
        use sp_api::runtime_decl_for_core::CoreV5;
        let s_ver = Runtime::version().spec_version;
        assert_eq!(
            s_ver, v_num,
            "Version mismatch. Crate version = {v_str}, but spec_version is {s_ver} != {v_num}"
        );

        // Sanity checks
        assert_eq!(1_002_003, convert("1.2.3"));
        assert_eq!(3_002_001, convert("3.2.1"));
        assert_eq!(0, convert("0.0.0"));
        assert_eq!(5_010, convert("0.5.10"));
        assert_eq!(1_000_000, convert("1.0.0"));
    }
}

#[derive(Default)]
pub struct ExtBuilder {
    balances: Vec<(AccountId, u128)>,
    candidates: Vec<(AccountId, Balance)>,
}

pub const ALICE: [u8; 20] = [4u8; 20];
pub const BOB: [u8; 20] = [5u8; 20];

// A valid signed Alice transfer.

pub(crate) const VALID_ETH_TX: &str =
    "02f869820501808085e8d4a51000825208943cd0a705a2dc65e5b1e1205896baa2be8a07c6e00180c\
       001a061087911e877a5802142a89a40d231d50913db399eb50839bb2d04e612b22ec8a01aa313efdf2\
       793bea76da6813bda611444af16a6207a8cfef2d9c8aa8f8012f7";

pub fn run_with_system_weight<F: FnMut()>(w: Weight, mut assertions: F) {
    let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap()
        .into();
    t.execute_with(|| {
        System::set_block_consumed_resources(w, 0);
        assertions()
    });
}

impl ExtBuilder {
    pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, u128)>) -> Self {
        self.balances = balances;
        self
    }

    // Build genesis storage according to the mock runtime.
    pub(crate) fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Runtime>::default()
            .build_storage()
            .unwrap();

        // get deduplicated list of all accounts and balances
        let all_accounts = self
            .balances
            .iter()
            .copied()
            .chain(self.candidates.iter().map(|(a, b)| (*a, b * 2)))
            .collect::<Vec<_>>();
        pallet_balances::GenesisConfig::<Runtime> {
            balances: all_accounts,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_sudo::GenesisConfig::<Runtime> {
            key: Some(AccountId::from(BOB)),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        t.into()
    }
}

pub fn ethereum_transaction(raw_hex_tx: &str) -> pallet_ethereum::Transaction {
    let bytes = hex::decode(raw_hex_tx).expect("Transaction bytes.");
    let transaction = ethereum::EnvelopedDecodable::decode(&bytes[..]);
    assert!(transaction.is_ok());
    transaction.unwrap()
}

use fp_rpc::ConvertTransaction;

pub fn unchecked_eth_tx(raw_hex_tx: &str) -> UncheckedExtrinsic {
    let converter = TransactionConverter;
    converter.convert_transaction(ethereum_transaction(raw_hex_tx))
}
