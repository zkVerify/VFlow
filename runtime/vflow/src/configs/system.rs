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

//! In this module, we provide the configurations about system's pallets: the set
//! of pallets used for base functionalities and utilities.

use crate::{
    configs::VERSION,
    constants::{
        currency::deposit, AVERAGE_ON_INITIALIZE_RATIO, MAXIMUM_BLOCK_WEIGHT, MAX_BLOCK_LENGTH,
        NORMAL_DISPATCH_RATIO, RELAY_PARENT_OFFSET, SLOT_DURATION,
    },
    types::ConsensusHook,
    weights,
    weights::ExtrinsicBaseWeight,
    AccountId, Aura, Balance, Balances, Block, Hash, MessageQueue, Nonce, OriginCaller, PalletInfo,
    Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, RuntimeTask, XcmpQueue,
};
use cumulus_pallet_parachain_system::{
    DefaultCoreSelector, ParachainSetCode, RelayNumberMonotonicallyIncreases,
};
use cumulus_primitives_core::AggregateMessageOrigin;
use frame_support::{
    derive_impl,
    dispatch::DispatchClass,
    pallet_prelude::ConstU32,
    parameter_types,
    traits::{ConstU64, Everything},
};
use frame_system::limits::{BlockLength, BlockWeights};
use polkadot_runtime_common::BlockHashCount;
use sp_runtime::traits::IdentityLookup;
use sp_version::RuntimeVersion;
use sp_weights::Weight;

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;

    // This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
    //  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
    // `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
    // the lazy contract deletion.
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(MAX_BLOCK_LENGTH, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(weights::block_weights::BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();

    // System parachain prefix. We use the same approach that polkadot do: all system parachain use
    // 0 ss58prefix
    pub const SS58Prefix: u16 = 0;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`ParaChainDefaultConfig`](`struct@frame_system::config_preludes::ParaChainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = Everything;
    /// The block type.
    type Block = Block;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = weights::db::constants::RocksDbWeight;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The lookup mechanism to get account ID from whatever is passed in
    /// dispatchers.
    type Lookup = IdentityLookup<AccountId>;
    /// The maximum number of consumers allowed on a single account.
    type MaxConsumers = ConstU32<16>;
    /// The index type for storing how many extrinsics an account has signed.
    type Nonce = Nonce;
    /// The action to take on a Runtime Upgrade
    type OnSetCode = ParachainSetCode<Self>;
    /// Converts a module to an index of this module in the runtime.
    type PalletInfo = PalletInfo;
    /// The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    /// The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// Runtime version.
    type Version = Version;
    /// Weights
    type SystemWeightInfo = weights::frame_system::ZKVEvmWeight<Runtime>;
    /// Extensions Weights
    type ExtensionsWeightInfo = weights::frame_system_extensions::ZKVEvmWeight<Runtime>;
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnSystemEvent = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type OutboundXcmpMessageSource = XcmpQueue;
    type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
    type ReservedDmpWeight = ReservedDmpWeight;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
    type WeightInfo = weights::cumulus_pallet_parachain_system::ZKVEvmWeight<Runtime>;
    type ConsensusHook = ConsensusHook;
    type RelayParentOffset = frame_support::traits::ConstU32<RELAY_PARENT_OFFSET>;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_weight_reclaim::Config for Runtime {
    type WeightInfo = ();
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type BlockNumberProvider = frame_system::Pallet<Runtime>;
    type WeightInfo = weights::pallet_multisig::ZKVEvmWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = weights::pallet_utility::ZKVEvmWeight<Runtime>;
}
impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = weights::pallet_timestamp::ZKVEvmWeight<Runtime>;
}
