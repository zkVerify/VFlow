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

use crate::Runtime;
use frame_support::{migrations::RemovePallet, parameter_types};

parameter_types! {
    pub const ProxyPalletName: &'static str = "Proxy";
}

pub type RemoveProxyPallet =
    RemovePallet<ProxyPalletName, <crate::Runtime as frame_system::Config>::DbWeight>;

/// Migrations to run on the next runtime upgrade.
pub type Unreleased = (
    RemoveProxyPallet,
    pallet_session::migrations::v1::MigrateV0ToV1<
        Runtime,
        pallet_session::migrations::v1::InitOffenceSeverity<Runtime>,
    >,
    cumulus_pallet_aura_ext::migration::MigrateV0ToV1<Runtime>,
);
