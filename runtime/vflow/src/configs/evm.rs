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

//! In this module, we provide the configurations about evm.

use crate::{
    constants::{MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO, WEIGHT_PER_GAS},
    opaque, weights, AccountId, Aura, Balances, CollatorSelection, DeploymentPermissions,
    EVMChainId, Precompiles, Runtime, RuntimeEvent, Timestamp, TransactionPayment,
    UncheckedExtrinsic,
};
use fp_evm::FeeCalculator;
use frame_support::{
    pallet_prelude::ConstU32,
    parameter_types,
    traits::{tokens::imbalance::ResolveTo, FindAuthor},
};
use pallet_ethereum::PostLogContent;
use pallet_evm::{
    EVMFungibleAdapter, EnsureAccountId20, EnsureAddressRoot, IdentityAddressMapping,
};
use parity_scale_codec::{Decode, Encode};
use sp_core::{H160, U256};
use sp_runtime::{ConsensusEngineId, FixedPointNumber};
use sp_std::marker::PhantomData;
use sp_weights::Weight;

parameter_types! {
    pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self::Version>;
    type PostLogContent = PostBlockAndTxnHashes;
    type ExtraDataLength = ConstU32<30>;
}

parameter_types! {
    pub BlockGasLimit: U256 = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT.ref_time() / WEIGHT_PER_GAS);
    pub GasLimitPovSizeRatio: u64 = BlockGasLimit::get().as_u64().saturating_div(cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64);
    pub GasLimitStorageGrowthRatio: u64 = 0; // Disabled
    pub PrecompilesValue: Precompiles<Runtime> = Precompiles::<_>::new();
    pub WeightPerGas: Weight = Weight::from_parts(WEIGHT_PER_GAS, 0);
    pub StakingPot: AccountId = CollatorSelection::account_id();
}

impl pallet_deployment_permissions::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = weights::pallet_deployment_permissions::ZKVEvmWeight<Self>;
}

type BaseRunner<T> = pallet_evm::runner::stack::Runner<T>;
type PermissionedRunner<T> = pallet_deployment_permissions::runner::PermissionedDeploy<
    T,
    BaseRunner<T>,
    DeploymentPermissions,
>;

pub struct TransactionPaymentAsGasPrice;
impl FeeCalculator for TransactionPaymentAsGasPrice {
    fn min_gas_price() -> (U256, Weight) {
        // note: transaction-payment differs from EIP-1559 in that its tip and length fees are not
        //       scaled by the multiplier, which means its multiplier will be overstated when
        //       applied to an ethereum transaction
        // note: transaction-payment uses both a congestion modifier (next_fee_multiplier, which is
        //       updated once per block in on_finalize) and a 'WeightToFee' implementation. Our
        //       runtime implements this as a 'ConstantModifier', so we can get away with a simple
        //       multiplication here.
        // It is imperative that `saturating_mul_int` be performed as late as possible in the
        // expression since it involves fixed point multiplication with a division by a fixed
        // divisor. This leads to truncation and subsequent precision loss if performed too early.
        // This can lead to min_gas_price being same across blocks even if the multiplier changes.
        // There's still some precision loss when the final `gas_price` (used_gas * min_gas_price)
        // is computed in frontier, but that's currently unavoidable.
        let min_gas_price = TransactionPayment::next_fee_multiplier().saturating_mul_int(
            (crate::configs::monetary::TransactionPicosecondFee::get())
                .saturating_mul(WEIGHT_PER_GAS as u128),
        );
        (
            min_gas_price.into(),
            <Runtime as frame_system::Config>::DbWeight::get().reads(1),
        )
    }
}

impl pallet_evm::Config for Runtime {
    type AccountProvider = pallet_evm::FrameSystemAccountProvider<Self>;
    type FeeCalculator = TransactionPaymentAsGasPrice;
    type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
    type WeightPerGas = WeightPerGas;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressRoot<AccountId>;
    type WithdrawOrigin = EnsureAccountId20;
    type AddressMapping = IdentityAddressMapping;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type PrecompilesType = Precompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = EVMChainId;
    type BlockGasLimit = BlockGasLimit;
    type Runner = PermissionedRunner<Self>;
    type OnChargeTransaction = EVMFungibleAdapter<Balances, ResolveTo<StakingPot, Balances>>;
    type OnCreate = ();
    type FindAuthor = FindAuthorSession<pallet_session::FindAccountFromAuthorIndex<Self, Aura>>;
    type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
    type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
    type Timestamp = Timestamp;
    type WeightInfo = weights::pallet_evm::ZKVEvmWeight<Self>;
}

impl pallet_evm_chain_id::Config for Runtime {}

pub struct FindAuthorSession<Inner>(PhantomData<Inner>);
impl<Inner> FindAuthor<H160> for FindAuthorSession<Inner>
where
    Inner: FindAuthor<AccountId>,
{
    fn find_author<'a, I>(digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        let tmp = Inner::find_author(digests);
        tmp.map(Into::into)
    }
}

#[derive(Clone)]
pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
        UncheckedExtrinsic::new_bare(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        )
    }
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(
        &self,
        transaction: pallet_ethereum::Transaction,
    ) -> opaque::UncheckedExtrinsic {
        let extrinsic = UncheckedExtrinsic::new_bare(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        );
        let encoded = extrinsic.encode();
        opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
            .expect("Encoded extrinsic is always valid")
    }
}
