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

use fp_account::EthereumSignature;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
};

pub use crate::{
    constants::{
        BLOCK_PROCESSING_VELOCITY, RELAY_CHAIN_SLOT_DURATION_MILLIS, UNINCLUDED_SEGMENT_CAPACITY,
    },
    //AllPalletsWithSystem, Runtime, RuntimeCall, XcmpQueue,
};

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic<Runtime, RuntimeCall> =
    fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra<Runtime>>;

/// Ethereum Signature
pub type Signature = EthereumSignature;

/// AccountId20 because 20 bytes long like H160 Ethereum Addresses
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Identifier of an asset
pub type AssetId = u32;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = AccountId;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

///// Block type as expected by this runtime.
pub type Block<Runtime, RuntimeCall> =
    generic::Block<Header, UncheckedExtrinsic<Runtime, RuntimeCall>>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra<Runtime> = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
    cumulus_primitives_storage_weight_reclaim::StorageWeightReclaim<Runtime>,
);

/// Executive: handles dispatch to the various modules.
pub type Executive<Runtime, RuntimeCall, AllPalletsWithSystem> = frame_executive::Executive<
    Runtime,
    Block<Runtime, RuntimeCall>,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

/// Configures the number of blocks that can be created without submission of validity proof to the relay chain
pub type ConsensusHook<Runtime> = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
    Runtime,
    RELAY_CHAIN_SLOT_DURATION_MILLIS,
    BLOCK_PROCESSING_VELOCITY,
    UNINCLUDED_SEGMENT_CAPACITY,
>;
