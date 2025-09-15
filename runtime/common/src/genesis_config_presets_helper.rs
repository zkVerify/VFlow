// Copyright 2025, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.

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

use crate::{AccountId, Balance};
use alloc::format;
use parachains_common::AuraId;
use sp_core::crypto::SecretStringError;
use sp_core::{Pair, Public};

pub struct AccountEntry<'a> {
    /// The seed use to generate the key pair with the url "DEFAULT_SUBSTRATE_SEED_PHRASE//seed".
    pub seed: &'a str,
    /// Eth address from "DEFAULT_SUBSTRATE_SEED_PHRASE//seed".
    /// They can also be generated with a wallet created using
    /// the below SUBSTRATE_DEFAULT_SEED_PHRASE with Metamask
    /// or Ganache
    pub eth_addr: [u8; 20],
}

impl<'a> AccountEntry<'a> {
    pub const fn new(seed: &'a str, eth_addr: [u8; 20]) -> Self {
        Self { seed, eth_addr }
    }
}

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
    get_from_seed_url::<TPublic>(&format!("//{account}"))
}

pub fn from_ss58check<T: sp_core::crypto::Ss58Codec>(
    key: &str,
) -> Result<T, sp_core::crypto::PublicError> {
    <T as sp_core::crypto::Ss58Codec>::from_ss58check(key)
}

pub type Ids = (AccountId, AuraId);

#[derive(Clone)]
pub struct FundedAccount {
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

    pub fn from_account_entry(entry: &AccountEntry, balance: Balance) -> Self {
        Self::from_addr(entry.eth_addr, balance)
    }

    pub fn from_addr(eth_address: [u8; 20], balance: Balance) -> Self {
        Self::new(eth_address.into(), balance)
    }

    pub fn json_data(&self) -> (AccountId, Balance) {
        (self.account_id, self.balance)
    }
}
