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

#[path = "../../common/configs/configs.rs"]
mod common_configs;
pub use common_configs::*;

use crate::*;

// Version of the runtime.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("vflow-runtime"),
    impl_name: Cow::Borrowed("vflow-node"),
    authoring_version: 1,
    spec_version: 1_000_000,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

const ZKV_GENESIS_HASH: [u8; 32] =
    hex_literal::hex!("060e3dd3fa2904d031206bb913c954687a2bcc350e5a83d33d9e273ad21460f1");
