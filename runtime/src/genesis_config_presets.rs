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

use crate::constants::currency::tVFY;
use crate::{AccountId, Balance, Precompiles, Runtime, SessionKeys};
use alloc::{collections::BTreeMap, format, vec::Vec};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use parachains_common::AuraId;
use sp_core::crypto::SecretStringError;
use sp_core::{Pair, Public, H160};
use sp_genesis_builder::PresetId;

const ENDOWMENT: Balance = 1_000_000 * tVFY;
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

struct AccountEntry<'a> {
    /// The seed use to generate the key pair with the url "DEFAULT_SUBSTRATE_SEED_PHRASE//seed".
    pub seed: &'a str,
    /// Eth address from "DEFAULT_SUBSTRATE_SEED_PHRASE//seed".
    /// They can also be generated with a wallet created using
    /// the below SUBSTRATE_DEFAULT_SEED_PHRASE with Metamask
    /// or Ganache
    pub eth_addr: [u8; 20],
}

impl<'a> AccountEntry<'a> {
    const fn new(seed: &'a str, eth_addr: [u8; 20]) -> Self {
        Self { seed, eth_addr }
    }
}

const DEFAULT_ENDOWED_SEEDS: &[AccountEntry<'static>] = &[
    AccountEntry::new("Alith", hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
    AccountEntry::new(
        "Baltathar",
        hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"),
    ),
    AccountEntry::new("Charleth", hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")),
    AccountEntry::new("Doroty", hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")),
    AccountEntry::new("Ethan", hex!("Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB")),
    AccountEntry::new("Faith", hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")),
];

/// Generate a crypto pair from seed.
pub fn try_get_from_seed_url<TPublic: Public>(
    seed: &str,
) -> Result<<TPublic::Pair as Pair>::Public, SecretStringError> {
    TPublic::Pair::from_string(seed, None).map(|pair| pair.public())
}

/// Generate a crypto pair from seed.
pub fn get_from_seed_url<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    try_get_from_seed_url::<TPublic>(seed).expect("static values are valid; qed")
}

/// Generate a crypto pair from seed.
pub fn get_from_substrate_account<TPublic: Public>(
    account: &str,
) -> <TPublic::Pair as Pair>::Public {
    get_from_seed_url::<TPublic>(&format!("//{}", account))
}

type Ids = (AccountId, AuraId);

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn genesis(
    id: ParaId,
    initial_collators: Vec<Ids>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, Balance)>,
) -> serde_json::Value {
    #[cfg(feature = "runtime-benchmarks")]
    let endowed_accounts = endowed_accounts
        .into_iter()
        .chain(Some((
            get_from_seed_url::<sp_core::ecdsa::Public>("//Bob").into(),
            ENDOWMENT,
        )))
        .collect::<Vec<_>>();

    let precompiles = Precompiles::<Runtime>::used_addresses().map(|addr| {
        (
            addr.into(),
            fp_evm::GenesisAccount {
                nonce: Default::default(),
                balance: Default::default(),
                storage: Default::default(),
                // bytecode to revert without returning data
                // (PUSH1 0x00 PUSH1 0x00 REVERT)
                code: vec![0x60, 0x00, 0x60, 0x00, 0xFD],
            },
        )
    });
    let accounts: BTreeMap<H160, fp_evm::GenesisAccount> = precompiles.collect();

    serde_json::json!({
        "balances": {
            // Configure endowed accounts with initial balance.
            "balances": endowed_accounts,
        },
        "parachainInfo": {
            "parachainId": id,
        },
        "session": {
            "keys": initial_collators.iter()
                .cloned()
                .map(|(account, aura)| { (account, account, SessionKeys { aura }) })
                .collect::<Vec<_>>(),
        },
        "collatorSelection": {
            "invulnerables": initial_collators.into_iter().map(|(acc, _)| acc).collect::<Vec<_>>(),
            "candidacyBond": 100,
        },
        "evmChainId": {
            "chainId": 9999
        },
        "evm": {
            "accounts": accounts
        },
        "zkvXcm": {
            "safeXcmVersion": Some(SAFE_XCM_VERSION),
        },
        "sudo": { "key": Some(root_key) },
    })
}

#[derive(Clone)]
struct FundedAccount {
    /// The account-id
    account_id: AccountId,
    /// Initial balance
    balance: Balance,
}

impl FundedAccount {
    pub const fn new(account_id: AccountId, balance: Balance) -> Self {
        Self {
            account_id,
            balance,
        }
    }

    fn from_account_entry(entry: &AccountEntry, balance: Balance) -> Self {
        Self::from_addr(entry.eth_addr, balance)
    }

    fn from_addr(eth_address: [u8; 20], balance: Balance) -> Self {
        Self::new(eth_address.into(), balance)
    }

    fn json_data(&self) -> (AccountId, Balance) {
        (self.account_id, self.balance)
    }
}

pub fn development_config_genesis() -> serde_json::Value {
    let balances = DEFAULT_ENDOWED_SEEDS
        .iter()
        .map(|entry| FundedAccount::from_account_entry(entry, ENDOWMENT))
        .collect::<Vec<_>>();

    let authorities_num = 2;
    let initial_authorities = DEFAULT_ENDOWED_SEEDS
        .iter()
        .take(authorities_num)
        .map(|entry| {
            (
                entry.eth_addr.into(),
                get_from_substrate_account::<AuraId>(entry.seed),
            )
        })
        .collect::<Vec<_>>();

    genesis(
        2000.into(),
        // Initial PoA authorities
        initial_authorities,
        // Sudo account
        DEFAULT_ENDOWED_SEEDS[0].eth_addr.into(),
        // Pre-funded accounts
        balances
            .iter()
            .map(FundedAccount::json_data)
            .collect::<Vec<_>>(),
    )
}

pub fn local_config_genesis() -> serde_json::Value {
    let balances = DEFAULT_ENDOWED_SEEDS
        .iter()
        .map(|entry| FundedAccount::from_account_entry(entry, ENDOWMENT))
        .collect::<Vec<_>>();

    let authorities_num = 2;
    let initial_authorities = DEFAULT_ENDOWED_SEEDS
        .iter()
        .take(authorities_num)
        .map(|entry| {
            (
                entry.eth_addr.into(),
                get_from_substrate_account::<AuraId>(entry.seed),
            )
        })
        .collect::<Vec<_>>();

    genesis(
        2000.into(),
        // Initial PoA authorities
        initial_authorities,
        // Sudo account
        DEFAULT_ENDOWED_SEEDS[0].eth_addr.into(),
        // Pre-funded accounts
        balances
            .iter()
            .take(authorities_num)
            .map(FundedAccount::json_data)
            .collect::<Vec<_>>(),
    )
}

pub fn get_preset(id: &sp_genesis_builder::PresetId) -> Option<Vec<u8>> {
    let cfg = match id.as_ref() {
        "development" => development_config_genesis(),
        "local" => local_config_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&cfg)
            .expect("genesis cfg must be serializable. qed.")
            .into_bytes(),
    )
}

pub fn preset_names() -> Vec<PresetId> {
    vec![PresetId::from("development"), PresetId::from("local")]
}
