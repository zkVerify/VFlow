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

pub use vflow_runtime_common::constants::{
    BLOCK_PROCESSING_VELOCITY, RELAY_CHAIN_SLOT_DURATION_MILLIS, UNINCLUDED_SEGMENT_CAPACITY,
};

pub use crate::{AllPalletsWithSystem, Runtime, RuntimeCall};
use vflow_runtime_common::types as ct;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = ct::UncheckedExtrinsic<Runtime, RuntimeCall>;

/// Block type as expected by this runtime.
pub type Block = ct::Block<Runtime, RuntimeCall>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = ct::SignedExtra<Runtime>;

/// Executive: handles dispatch to the various modules.
pub type Executive = ct::Executive<Runtime, RuntimeCall, AllPalletsWithSystem>;

/// Configures the number of blocks that can be created without submission of validity proof to the relay chain
pub type ConsensusHook = ct::ConsensusHook<Runtime>;
