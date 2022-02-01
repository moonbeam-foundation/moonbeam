// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam. If not, see <http://www.gnu.org/licenses/>.

use crate::cli::MoonbeamCli;
use moonbase_runtime::DAYS;
use parachain_inherent::ParachainInherentData;
use sc_service::TFullBackend;
use sp_runtime::generic::Era;
use std::{error::Error, sync::Arc};
use substrate_simnode::{FullClientFor, SignatureVerificationOverride};

/// A unit struct which implements `NativeExecutionDispatch` feeding in the
/// hard-coded runtime.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
	type ExtendHostFunctions = (
		frame_benchmarking::benchmarking::HostFunctions,
		moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
		SignatureVerificationOverride,
	);

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		moonbase_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		moonbase_runtime::native_version()
	}
}

/// ChainInfo implementation.
pub struct ChainInfo;

impl substrate_simnode::ChainInfo for ChainInfo {
	type Block = moonbase_runtime::opaque::Block;
	type ExecutorDispatch = ExecutorDispatch;
	type Runtime = moonbase_runtime::Runtime;
	type RuntimeApi = moonbase_runtime::RuntimeApi;
	type SelectChain = sc_consensus::LongestChain<TFullBackend<Self::Block>, Self::Block>;
	type BlockImport = Arc<FullClientFor<Self>>;
	type SignedExtras = moonbase_runtime::SignedExtra;
	type InherentDataProviders = (sp_timestamp::InherentDataProvider, ParachainInherentData);
	type Cli = MoonbeamCli;

	fn signed_extras(from: <Self::Runtime as system::Config>::AccountId) -> Self::SignedExtras {
		(
			system::CheckSpecVersion::<Self::Runtime>::new(),
			system::CheckTxVersion::<Self::Runtime>::new(),
			system::CheckGenesis::<Self::Runtime>::new(),
			system::CheckMortality::<Self::Runtime>::from(Era::Immortal),
			system::CheckNonce::<Self::Runtime>::from(
				system::Pallet::<Self::Runtime>::account_nonce(from),
			),
			system::CheckWeight::<Self::Runtime>::new(),
			transaction_payment::ChargeTransactionPayment::<Self::Runtime>::from(0),
		)
	}
}

/// run all integration tests
pub fn run() -> Result<(), Box<dyn Error>> {
	crate::client::moonbeam_node::<ChainInfo, _, _>(|node| async move {
		// test authoring some blocks
		node.seal_blocks(10).await;

		// test runtime upgrades
		let code = moonbase_runtime::WASM_BINARY
			.ok_or("Runtime wasm not available")?
			.to_vec();
		crate::runtime_upgrade::parachain_runtime_upgrades(&node, code).await?;

		// Exemple of things that we can test
		//_parachain_info_storage_override_test(&node).await?;

		// try to create blocks for one day, if it doesn't panic, all good.
		node.seal_blocks((1 * DAYS) as usize).await;

		Ok(())
	})
}

/*async fn _parachain_info_storage_override_test(
	node: &substrate_simnode::Node<ChainInfo>,
) -> Result<(), Box<dyn Error>> {
	// sudo account on-chain
	let sudo = node.with_state(None, sudo::Pallet::<moonbase_runtime::Runtime>::key);

	// gotten from
	// hex::encode(&parachain_info::ParachainId::<Runtime>::storage_value_final_key().to_vec());
	let key = hex::decode("0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f")?;

	let raw_key_value: Option<u32> = node.with_state(None, || support::storage::unhashed::get(&key[..]));

	assert_eq!(raw_key_value, Some(2104));
	let new_para_id: u32 = 2087;

	// gotten from hex::encode(new_para_id.encode())
	let value = hex::decode("27080000")?;

	let call = sudo::Call::sudo_unchecked_weight {
		call: Box::new(system::Call::set_storage { items: vec![(key.clone(), value)] }.into()),
		weight: 0,
	};
	node.submit_extrinsic(call, Some(sudo.clone())).await?;
	node.seal_blocks(1).await;
	let raw_key_value: Option<u32> = node.with_state(None, || support::storage::unhashed::get(&key[..]));

	assert_eq!(raw_key_value, Some(new_para_id));

	Ok(())
}*/
