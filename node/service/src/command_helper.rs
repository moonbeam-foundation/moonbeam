// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

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

//! Contains code to setup the command invocations in [`super::command`] which would
//! otherwise bloat that module.

use std::{sync::Arc, time::Duration};

use account::EthereumSigner;
use parity_scale_codec::Encode;
use frame_system::Call as SystemCall;
use moonbase_runtime as runtime;
use sc_cli::Result;
use sc_client_api::BlockBackend;
use sp_core::{ecdsa, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_runtime::{generic::{Era, SignedPayload}, OpaqueExtrinsic};
use sha3::{Digest, Keccak256};

use crate::FullClient;

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct BenchmarkExtrinsicBuilder {
	client: Arc<FullClient<moonbase_runtime::RuntimeApi, crate::MoonbaseExecutor>>,
}

impl BenchmarkExtrinsicBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<FullClient<moonbase_runtime::RuntimeApi, crate::MoonbaseExecutor>>) -> Self {
		Self { client }
	}
}

impl frame_benchmarking_cli::ExtrinsicBuilder for BenchmarkExtrinsicBuilder {
	fn remark(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		// TODO: use well-known testing keypairs
		const ALICE_PRIV_KEY: &str = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
		let keypair = sp_core::ecdsa::Pair::
			from_string(ALICE_PRIV_KEY, None)
			.expect("Alices' private key works");

		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			self.client.as_ref(),
			keypair,
			SystemCall::remark { remark: vec![] }.into(),
			nonce,
		)
		.into();

		Ok(extrinsic)
	}
}

/// Create a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
pub fn create_benchmark_extrinsic(
	client: &FullClient<moonbase_runtime::RuntimeApi, crate::MoonbaseExecutor>,
	sender: ecdsa::Pair,
	call: runtime::Call,
	nonce: u32,
) -> runtime::UncheckedExtrinsic {
	use sp_runtime::traits::IdentifyAccount;

	let genesis_hash = client
		.block_hash(0)
		.ok()
		.flatten()
		.expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	// let best_block = client.chain_info().best_number;

	let extra: runtime::SignedExtra = (
		frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
		frame_system::CheckTxVersion::<runtime::Runtime>::new(),
		frame_system::CheckGenesis::<runtime::Runtime>::new(),
		frame_system::CheckEra::<runtime::Runtime>::from(Era::Immortal),
		frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<runtime::Runtime>::new(),
		pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
	);

	let raw_payload = SignedPayload::from_raw(
		call.clone(),
		extra.clone(),
		(
			runtime::VERSION.spec_version,
			runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		),
	);

	let signature = raw_payload.using_encoded(|e| {
		let mut m = [0u8; 32];
		let digest = &Keccak256::digest(e);
		m.copy_from_slice(digest);
		sender.sign_prehashed(&m)
	});
	let signer: EthereumSigner = sender.public().into();

	runtime::UncheckedExtrinsic::new_signed(
		call.clone(),
		signer.into_account(),
		signature.into(),
		extra.clone(),
	)
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub fn inherent_benchmark_data() -> Result<InherentData> {

	use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
	use cumulus_primitives_core::PersistedValidationData;
	use cumulus_primitives_parachain_inherent::ParachainInherentData;

	let sproof_builder = RelayStateSproofBuilder::default();
    let (relay_parent_storage_root, relay_chain_state) =
        sproof_builder.into_state_root_and_proof();

	log::warn!("building inherent data...");

    let vfp = PersistedValidationData {
        relay_parent_number: 1u32,
        relay_parent_storage_root,
        ..Default::default()
    };

	let mut inherent_data = {
		let mut inherent_data = InherentData::default();
		let system_inherent_data = ParachainInherentData {
			validation_data: vfp.clone(),
			relay_chain_state: relay_chain_state,
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
		};
		inherent_data
			.put_data(
				cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
				&system_inherent_data,
			)
			.expect("failed to put VFP inherent");
		inherent_data
	};

	let d = Duration::from_millis(0);
	let timestamp = sp_timestamp::InherentDataProvider::new(d.into());
	timestamp
		.provide_inherent_data(&mut inherent_data)
		.map_err(|e| format!("creating inherent data: {:?}", e))?;

	Ok(inherent_data)
}
