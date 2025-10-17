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

pub(crate) mod consensus;
pub mod ethereum_xcm;
pub mod evm;
mod governance;
pub mod monetary;
pub mod system;
pub mod xcm;

#[macro_export]
macro_rules! runtime_version {
    ( $spec_name:tt ) => {
        #[sp_version::runtime_version]
        pub const VERSION: RuntimeVersion = RuntimeVersion {
            spec_name: Cow::Borrowed($spec_name),
            impl_name: Cow::Borrowed("vflow_node"),
            authoring_version: 1,
            spec_version: 1_000_000,
            impl_version: 0,
            apis: RUNTIME_API_VERSIONS,
            transaction_version: 1,
            system_version: 1,
        };
    };
}
