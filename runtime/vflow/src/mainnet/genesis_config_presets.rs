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

#[cfg(feature = "runtime-benchmarks")]
use crate::get_from_seed_url;
use crate::{
    currency::VFY, from_ss58check, get_from_substrate_account, AccountEntry, AccountId, Balance,
    FundedAccount, Ids, Precompiles, Runtime, SessionKeys,
};
use alloc::{collections::BTreeMap, vec::Vec};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use parachains_common::AuraId;
use sp_core::H160;
use sp_genesis_builder::PresetId;

const EVM_CHAIN_ID: u64 = 1408;
const ENDOWMENT: Balance = 1_000_000 * VFY;
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

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

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn genesis(
    id: ParaId,
    initial_collators: Vec<Ids>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, Balance)>,
    chain_id: u64,
    allowed_deployers: Vec<H160>,
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
            "desiredCandidates": 0,
        },
        "evmChainId": {
            "chainId": chain_id
        },
        "evm": {
            "accounts": accounts
        },
        "deploymentPermissions": {
            "deployers": allowed_deployers,
        },
        "zkvXcm": {
            "safeXcmVersion": Some(SAFE_XCM_VERSION),
        },
        "sudo": { "key": Some(root_key) },
    })
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
        // Para id
        1.into(),
        // Initial PoA authorities
        initial_authorities,
        // Sudo account
        DEFAULT_ENDOWED_SEEDS[0].eth_addr.into(),
        // Pre-funded accounts
        balances
            .iter()
            .map(FundedAccount::json_data)
            .collect::<Vec<_>>(),
        // EVM chain id
        EVM_CHAIN_ID,
        // Account allowed to deploy contracts
        DEFAULT_ENDOWED_SEEDS
            .iter()
            .map(|entry| entry.eth_addr.into())
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
        1.into(),
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
        // EVM chain id
        EVM_CHAIN_ID,
        // Account allowed to deploy contracts
        DEFAULT_ENDOWED_SEEDS
            .iter()
            .map(|entry| entry.eth_addr.into())
            .collect::<Vec<_>>(),
    )
}

pub fn mainnet_config_genesis() -> serde_json::Value {
    fn aura(p: &str) -> AuraId {
        from_ss58check(p).expect("Aura is valid. qed")
    }

    let initial_authorities = vec![
        (
            hex!("a98193126fa68a9F77dE4A44B36f51a845f985c6").into(),
            aura("5FUNZpLNXTpyaWQsKsw17c2QywJCRT5EMUazbhWcBGVd2ric"),
        ),
        (
            hex!("098aE96842200399b3F89d8D2D4B77588337A148").into(),
            aura("5CyeBKChqfWHnbNn4Xcn7UM6Uw31fcbGEiA9tioy4br2WwkA"),
        ),
        (
            hex!("014a2382ce088fff1c550Ce2CD9C53B66191141C").into(),
            aura("5DHsyJzJ9EMFZihq3CUT7zmQf7VVsdLTP8s1pg5AUNpb41MD"),
        ),
    ];
    let sudo = hex!("e1b96Dd5D395E3EC55e033a1bc463b824D7Ace75").into();

    genesis(
        // parachain id
        1.into(),
        // Initial PoA authorities
        initial_authorities,
        // Sudo account
        sudo,
        // No Pre-funded accounts
        Default::default(),
        EVM_CHAIN_ID,
        // No allowed deployers in genesis: sudo will add it
        Default::default(),
    )
}

pub fn get_preset(id: &sp_genesis_builder::PresetId) -> Option<Vec<u8>> {
    let cfg = match id.as_ref() {
        "mainnet_development" => development_config_genesis(),
        "mainnet_local_testnet" => local_config_genesis(),
        "mainnet" => mainnet_config_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&cfg)
            .expect("genesis cfg must be serializable. qed.")
            .into_bytes(),
    )
}

pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from("mainnet_development"), // default for benchmarking
        PresetId::from("mainnet_local_testnet"),
        PresetId::from("mainnet"),
    ]
}
