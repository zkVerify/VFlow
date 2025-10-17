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
runtime_version!("vflow-runtime", "vflow_node");

const ZKV_GENESIS_HASH: [u8; 32] =
    hex_literal::hex!("060e3dd3fa2904d031206bb913c954687a2bcc350e5a83d33d9e273ad21460f1");

pub(crate) const ERC20_NAME: sp_runtime::Cow<'_, str> = Cow::Borrowed("VFY token");
pub(crate) const ERC20_SYMBOL: sp_runtime::Cow<'_, str> = Cow::Borrowed("VFY");
