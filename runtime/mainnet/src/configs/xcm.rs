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

//! In this module, we provide the configurations about xcm subsystem.

use crate::{
    configs::monetary::TransactionByteFee,
    configs::system::RuntimeBlockWeights,
    constants::currency::{CENTS, MILLIS},
    weights, AccountId, AllPalletsWithSystem, Balances, MessageQueue, ParachainInfo,
    ParachainSystem, Perbill, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, XcmpQueue, ZKVXcm,
};
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::{
    pallet_prelude::Get,
    parameter_types,
    traits::tokens::imbalance::ResolveTo,
    traits::OriginTrait,
    traits::TransformOrigin,
    traits::{ConstU32, Contains, Equals, Everything, Nothing, PalletInfoAccess},
    weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use parachains_common::{
    message_queue::{NarrowOriginToSibling, ParaIdToSibling},
    xcm_config::ConcreteAssetFromSystem,
};
use sp_runtime::{
    traits::{PostDispatchInfoOf, TryConvert},
    DispatchErrorWithPostInfo,
};
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountKey20Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, DenyReserveTransferToRelayChain, DenyThenTry,
    DescribeAllTerminal, DescribeFamily, EnsureXcmOrigin, FrameTransactionalProcessor,
    FungibleAdapter, HashedDescription, IsConcrete, NativeAsset, ParentIsPreset,
    RelayChainAsNative, SendXcmFeeToAccount, SiblingParachainAsNative, SignedAccountKey20AsNative,
    SovereignSignedViaLocation, TakeWeightCredit, TrailingSetTopicAsId, UsingComponents,
    WeightInfoBounds, WithComputedOrigin, WithUniqueTopic, XcmFeeManagerFromComponents,
};
use xcm_executor::{
    traits::{CallDispatcher, ConvertLocation},
    XcmExecutor,
};

use crate::weights::pallet_xcm_benchmarks::ZKVEvmWeight as XcmZKVEvmWeight;

const ZKV_GENESIS_HASH: [u8; 32] =
    hex_literal::hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef");

parameter_types! {
    pub const RootLocation: Location = Here.into_location();
    pub const RelayLocation: Location = Location::parent();
    pub const RelayNetwork: Option<NetworkId> = Some(NetworkId::ByGenesis(ZKV_GENESIS_HASH));
    pub BalancesPalletLocation: Location = PalletInstance(<Balances as PalletInfoAccess>::index() as u8).into();
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    pub UniversalLocation: InteriorLocation = [GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(ParachainInfo::parachain_id().into())].into();
}

/// Type for specifying how a `Location` can be converted into an
/// `AccountId`. This is used when determining ownership of accounts for asset
/// transacting and when attempting to use XCM `Transact` in order to determine
/// the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the parent `AccountId`.
    ParentIsPreset<AccountId>,
    // If we receive a Location of type AccountKey20, just generate a native account
    AccountKey20Aliases<RelayNetwork, AccountId>,
    // Generate remote accounts according to polkadot standards
    HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
);

pub struct LocationAccountId32ToAccountId;
impl ConvertLocation<AccountId> for LocationAccountId32ToAccountId {
    fn convert_location(location: &Location) -> Option<AccountId> {
        use xcm::latest::Junctions::X1;
        match location.unpack() {
            (0, [AccountId32 { network, id }]) => {
                LocationToAccountId::convert_location(&Location {
                    parents: 0,
                    interior: X1(sp_std::sync::Arc::new([AccountKey20 {
                        network: *network,
                        key: id.as_slice()[id.len() - 20..] // take the last 20 bytes
                            .try_into()
                            .expect("Cannot convert AccountId32 to AccountKey20"),
                    }])),
                })
            }
            _ => LocationToAccountId::convert_location(location),
        }
    }
}

/// Means for transacting the native currency on this chain.
pub type FungibleTransactor = FungibleAdapter<
    // Use this currency:
    Balances,
    // Use this currency when it is a fungible asset matching the given location or name:
    IsConcrete<RelayLocation>,
    // Convert an XCM `Location` into a local account ID:
    LocationAccountId32ToAccountId,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We don't track any teleports of `Balances`.
    (),
>;

/// Means for transacting assets on this chain.
pub type AssetTransactors = FungibleTransactor;

/// This is the type we use to convert an (incoming) XCM origin into a local
/// `Origin` instance, ready for dispatching a transaction with Xcm's
/// `Transact`. There is an `OriginKind` which can biases the kind of local
/// `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
    // Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
    // Xcm Origins defined by a Multilocation of type AccountKey20 can be converted to a 20 byte-
    // account local origin
    SignedAccountKey20AsNative<RelayNetwork, RuntimeOrigin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
    pub const MaxInstructions: u32 = 30;
    pub const MaxAssetsIntoHolding: u32 = 64;
    pub StakingPot: AccountId = crate::CollatorSelection::account_id();
}

pub struct ParentRelayChain;
impl Contains<Location> for ParentRelayChain {
    fn contains(location: &Location) -> bool {
        // match the relay chain and any account on it
        matches!(location.unpack(), (1, [..]))
    }
}

pub type Barrier = TrailingSetTopicAsId<
    DenyThenTry<
        DenyReserveTransferToRelayChain,
        (
            TakeWeightCredit,
            AllowKnownQueryResponses<ZKVXcm>,
            WithComputedOrigin<
                (AllowTopLevelPaidExecutionFrom<ParentRelayChain>,),
                UniversalLocation,
                ConstU32<8>,
            >,
            AllowSubscriptionsFrom<ParentRelayChain>,
        ),
    >,
>;

pub type TrustedTeleporters = ConcreteAssetFromSystem<RelayLocation>;

pub type WaivedLocations = (Equals<RelayLocation>, Equals<RootLocation>);

pub struct RemoteEVMCall;
impl CallDispatcher<RuntimeCall> for RemoteEVMCall {
    fn dispatch(
        call: RuntimeCall,
        origin: RuntimeOrigin,
    ) -> Result<
        PostDispatchInfoOf<RuntimeCall>,
        DispatchErrorWithPostInfo<PostDispatchInfoOf<RuntimeCall>>,
    > {
        if let Ok(raw_origin) =
            TryInto::<frame_system::RawOrigin<AccountId>>::try_into(origin.clone().caller)
        {
            if let (
                RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::transact { .. })
                | RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::transact_through_proxy {
                    ..
                }),
                frame_system::RawOrigin::Signed(account_id),
            ) = (call.clone(), raw_origin)
            {
                return RuntimeCall::dispatch(
                    call,
                    pallet_ethereum_xcm::Origin::XcmEthereumTransaction(account_id.into()).into(),
                );
            }
        }
        RuntimeCall::dispatch(call, origin)
    }
}

pub struct SafeCallFilter;
impl frame_support::traits::Contains<RuntimeCall> for SafeCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        matches!(
            call,
            RuntimeCall::EthereumXcm(_)
                // Used for baseline benchmarks
                | RuntimeCall::System(frame_system::Call::remark_with_event { remark: _ })
        )
    }
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = XcmRouter;
    // How to withdraw and deposit an asset.
    type AssetTransactor = AssetTransactors;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type IsReserve = NativeAsset;
    type IsTeleporter = TrustedTeleporters;
    type Aliasers = Nothing;
    type UniversalLocation = UniversalLocation;
    type Barrier = Barrier;
    type Weigher = WeightInfoBounds<XcmZKVEvmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
    // Can only buy weight with the native token
    type Trader = UsingComponents<
        <Runtime as pallet_transaction_payment::Config>::WeightToFee,
        RelayLocation,
        AccountId,
        Balances,
        ResolveTo<StakingPot, Balances>,
    >;
    type ResponseHandler = ZKVXcm;
    type AssetTrap = ZKVXcm;
    type AssetLocker = ();
    type AssetExchanger = ();
    type AssetClaims = ZKVXcm;
    type SubscriptionService = ZKVXcm;
    type PalletInstancesInfo = AllPalletsWithSystem;
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type FeeManager = XcmFeeManagerFromComponents<
        WaivedLocations,
        SendXcmFeeToAccount<AssetTransactors, StakingPot>,
    >;
    type MessageExporter = ();
    type UniversalAliases = Nothing;
    type CallDispatcher = RemoteEVMCall;
    type SafeCallFilter = SafeCallFilter;
    type TransactionalProcessor = FrameTransactionalProcessor;
    type HrmpNewChannelOpenRequestHandler = ();
    type HrmpChannelAcceptedHandler = ();
    type HrmpChannelClosingHandler = ();
    type XcmRecorder = ZKVXcm;
}

// Convert a local Origin (i.e., a signed 20 byte account Origin)  to a Multilocation
pub struct SignedToAccountId20<Origin, AccountId, Network>(
    sp_std::marker::PhantomData<(Origin, AccountId, Network)>,
);
impl<Origin: OriginTrait + Clone, AccountId: Into<[u8; 20]>, Network: Get<Option<NetworkId>>>
    TryConvert<Origin, Location> for SignedToAccountId20<Origin, AccountId, Network>
where
    Origin::PalletsOrigin: From<frame_system::RawOrigin<AccountId>>
        + TryInto<frame_system::RawOrigin<AccountId>, Error = Origin::PalletsOrigin>,
{
    fn try_convert(o: Origin) -> Result<Location, Origin> {
        o.try_with_caller(|caller| match caller.try_into() {
            Ok(frame_system::RawOrigin::Signed(who)) => Ok(AccountKey20 {
                key: who.into(),
                network: Network::get(),
            }
            .into()),
            Ok(other) => Err(other.into()),
            Err(other) => Err(other),
        })
    }
}

// Converts a Signed Local Origin into a Location
pub type LocalOriginToLocation = SignedToAccountId20<RuntimeOrigin, AccountId, RelayNetwork>;

parameter_types! {
    /// The asset ID for the asset that we use to pay for message delivery fees.
    pub NativeAssetId: AssetId = AssetId(RelayLocation::get()); // the relay chain native asset
    pub FeeAssetId: AssetId = NativeAssetId::get();
    /// The base fee for the message delivery fees.
    pub const ToParentBaseDeliveryFee: u128 = MILLIS.saturating_mul(3);
}

pub type PriceForParentDelivery = polkadot_runtime_common::xcm_sender::ExponentialPrice<
    FeeAssetId,
    ToParentBaseDeliveryFee,
    TransactionByteFee,
    ParachainSystem,
>;

/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = WithUniqueTopic<(
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ZKVXcm, PriceForParentDelivery>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
)>;

parameter_types! {
    pub const MaxLockers: u32 = 8;
    pub const MaxRemoteLockConsumers: u32 = 0;
}

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Everything;
    type XcmReserveTransferFilter = Nothing;
    type Weigher = WeightInfoBounds<XcmZKVEvmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
    type UniversalLocation = UniversalLocation;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    // ^ Override for AdvertisedXcmVersion default
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type AdminOrigin = EnsureRoot<AccountId>;
    type TrustedLockers = ();
    type SovereignAccountOf = LocationToAccountId;
    type MaxLockers = MaxLockers;
    type MaxRemoteLockConsumers = MaxLockers;
    type RemoteLockConsumerIdentifier = ();

    type WeightInfo = weights::pallet_xcm::ZKVEvmWeight<Runtime>;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

parameter_types! {
    pub MessageQueueServiceWeight: Weight = Perbill::from_percent(25) * RuntimeBlockWeights::get().max_block;
    pub const HeapSize: u32 = 103 * 1024;
    pub const MaxStale: u32 = 8;
}

impl pallet_message_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = weights::pallet_message_queue::ZKVEvmWeight<Runtime>;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type MessageProcessor = pallet_ethereum_xcm::MessageProcessorWrapper<
        xcm_builder::ProcessXcmMessage<
            AggregateMessageOrigin,
            xcm_executor::XcmExecutor<crate::configs::xcm::XcmConfig>,
            RuntimeCall,
        >,
    >;
    #[cfg(feature = "runtime-benchmarks")]
    type MessageProcessor =
        pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;

    type Size = u32;
    // The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
    type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
    type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
    type HeapSize = HeapSize;
    type MaxStale = MaxStale;
    type ServiceWeight = MessageQueueServiceWeight;
    type IdleMaxServiceWeight = MessageQueueServiceWeight;
}

parameter_types! {
    pub const MaxInboundSuspended: u32 = 1000;
    /// The base fee for the message delivery fees. zkVerify is based for the reference.
    pub const ToSiblingBaseDeliveryFee: u128 = CENTS.saturating_mul(3);
}

/// Price For Sibling Parachain Delivery
type PriceForSiblingParachainDelivery = polkadot_runtime_common::xcm_sender::ExponentialPrice<
    FeeAssetId,
    ToSiblingBaseDeliveryFee,
    TransactionByteFee,
    XcmpQueue,
>;

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = ();
    // Enqueue XCMP messages from siblings for later processing.
    type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
    type MaxInboundSuspended = MaxInboundSuspended;
    type MaxActiveOutboundChannels = ConstU32<128>;
    // from polkadot-sdk:
    // Most on-chain HRMP channels are configured to use 102400 bytes of max message size, so we
    // need to set the page size larger than that until we reduce the channel size on-chain.
    type MaxPageSize = ConstU32<{ 103 * 1024 }>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type PriceForSiblingDelivery = PriceForSiblingParachainDelivery;
    type WeightInfo = weights::cumulus_pallet_xcmp_queue::ZKVEvmWeight<Runtime>;
}
