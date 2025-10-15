// Copyright 2025, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Contains code to setup the command invocations in [`crate::command`] which would
//! otherwise bloat that module.

use std::sync::Arc;

use cumulus_client_parachain_inherent::MockValidationDataInherentDataProvider;
use cumulus_primitives_core::ParaId;
// Frontier
use parity_scale_codec::Encode;
// Substrate
use sc_client_api::BlockBackend;
use sc_client_api::UsageProvider;
use sp_core::{ecdsa, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_runtime::{generic::Era, OpaqueExtrinsic, SaturatedConversion};
use vflow_runtime::AccountId;

use crate::service::ParachainClient;

/// Generates `System::Remark` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder {
    client: Arc<ParachainClient>,
}

impl RemarkBuilder {
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<ParachainClient>) -> Self {
        Self { client }
    }
}

impl frame_benchmarking_cli::ExtrinsicBuilder for RemarkBuilder {
    fn pallet(&self) -> &str {
        "system"
    }

    fn extrinsic(&self) -> &str {
        "remark"
    }

    fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        // We apply the extrinsic directly, so let's take some random period.
        let period = 128;
        let genesis = self
            .client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis exists; qed");
        let signer = ecdsa::Pair::from_string("//Bob", None).expect("static values are valid; qed");
        let current_block = 0;

        let call = vflow_runtime::RuntimeCall::System(
                    vflow_runtime::SystemCall::remark { remark: vec![] }
        );
        Ok(sign_call(
            call,
            nonce,
            current_block,
            period,
            genesis,
            signer,
        ))
    }
}

fn sign_call(
    call: vflow_runtime::RuntimeCall,
    nonce: u32,
    current_block: u64,
    period: u64,
    genesis: sp_core::H256,
    acc: ecdsa::Pair,
) -> OpaqueExtrinsic {
    use vflow_runtime as runtime;
    let extra: runtime::types::SignedExtra = (
        frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckMortality::<runtime::Runtime>::from(Era::mortal(
            period,
            current_block.saturated_into(),
        )),
        frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
        frame_system::CheckWeight::<runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
        frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
        cumulus_primitives_storage_weight_reclaim::StorageWeightReclaim::<runtime::Runtime>::new(),
    );

    let raw_payload = runtime::types::SignedPayload::from_raw(
        call.clone(),
        extra.clone(),
        (
            (),
            runtime::configs::VERSION.spec_version,
            runtime::configs::VERSION.transaction_version,
            genesis,
            genesis,
            (),
            (),
            (),
            None,
            (),
        ),
    );

    let signature =
        raw_payload.using_encoded(|e| acc.sign_prehashed(&sp_io::hashing::keccak_256(e)));

    runtime::types::UncheckedExtrinsic::new_signed(
        call,
        AccountId::from(acc.public()),
        runtime::Signature::new(signature),
        extra,
    )
    .into()
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// This function constructs the inherent data required for block execution,
/// including the relay chain state and validation data. Not all of these
/// inherents are required for every chain. The runtime will pick the ones
/// it requires based on their identifier.
///
/// Note: Should only be used for benchmarking.
pub fn create_inherent_data(client: Arc<ParachainClient>, para_id: ParaId) -> InherentData {
    let genesis = client.usage_info().chain.best_hash;
    let header = client.header(genesis).unwrap().unwrap();

    let mut inherent_data = InherentData::new();

    // Para inherent can only makes sense when we are handling a parachain.
    let parachain_validation_data_provider = MockValidationDataInherentDataProvider::<()> {
        para_id,
        current_para_block_head: Some(header.encode().into()),
        relay_offset: 1,
        ..Default::default()
    };
    let _ = futures::executor::block_on(
        parachain_validation_data_provider.provide_inherent_data(&mut inherent_data),
    );

    // Parachain inherent that is used on relay chains to perform parachain validation.
    let para_inherent = polkadot_primitives::InherentData {
        bitfields: Vec::new(),
        backed_candidates: Vec::new(),
        disputes: Vec::new(),
        parent_header: header,
    };

    // Timestamp inherent that is very common in substrate chains.
    let timestamp = sp_timestamp::InherentDataProvider::new(std::time::Duration::default().into());

    let _ = futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data));
    let _ = inherent_data.put_data(
        polkadot_primitives::PARACHAINS_INHERENT_IDENTIFIER,
        &para_inherent,
    );

    inherent_data
}
