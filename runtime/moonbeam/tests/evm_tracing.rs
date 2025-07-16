// Copyright 2019-2025 PureStake Inc.
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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Moonbeam EVM tracing Integration Tests

mod common;

#[cfg(test)]
#[cfg(feature = "evm-tracing")]
mod tests {
	use super::common::*;

	use pallet_evm::AddressMapping;
	use sp_core::{H160, U256};

	use moonbeam_core_primitives::Header;
	use moonbeam_rpc_primitives_debug::runtime_decl_for_debug_runtime_api::DebugRuntimeApi;
	use std::str::FromStr;

	#[test]
	fn debug_runtime_api_trace_transaction() {
		let alith = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
			H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
				.expect("internal H160 is valid; qed"),
		);
		ExtBuilder::default()
			.with_balances(vec![
				(alith, 2_000 * GLMR),
				(AccountId::from(ALICE), 2_000 * GLMR),
				(AccountId::from(BOB), 1_000 * GLMR),
			])
			.build()
			.execute_with(|| {
				let non_eth_uxt = UncheckedExtrinsic::new_bare(
					pallet_balances::Call::<Runtime>::transfer_allow_death {
						dest: AccountId::from(BOB),
						value: 1 * GLMR,
					}
					.into(),
				);
				let transaction = ethereum_transaction(VALID_ETH_TX);
				let eth_uxt = unchecked_eth_tx(VALID_ETH_TX);
				let block = Header {
					digest: Default::default(),
					extrinsics_root: Default::default(),
					number: 1,
					parent_hash: Default::default(),
					state_root: Default::default(),
				};
				assert!(Runtime::trace_transaction(
					vec![non_eth_uxt.clone(), eth_uxt, non_eth_uxt.clone()],
					&transaction,
					&block
				)
				.is_ok());
			});
	}

	#[test]
	fn debug_runtime_api_trace_block() {
		let alith = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
			H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
				.expect("internal H160 is valid; qed"),
		);
		ExtBuilder::default()
			.with_balances(vec![
				(alith, 2_000 * GLMR),
				(AccountId::from(ALICE), 2_000 * GLMR),
				(AccountId::from(BOB), 1_000 * GLMR),
			])
			.build()
			.execute_with(|| {
				let non_eth_uxt = UncheckedExtrinsic::new_bare(
					pallet_balances::Call::<Runtime>::transfer_allow_death {
						dest: AccountId::from(BOB),
						value: 1 * GLMR,
					}
					.into(),
				);
				let eth_uxt = unchecked_eth_tx(VALID_ETH_TX);
				let eth_tx = ethereum_transaction(VALID_ETH_TX);
				let eth_extrinsic_hash = eth_tx.hash();
				let block = Header {
					digest: Default::default(),
					extrinsics_root: Default::default(),
					number: 1,
					parent_hash: Default::default(),
					state_root: Default::default(),
				};
				assert!(Runtime::trace_block(
					vec![non_eth_uxt.clone(), eth_uxt.clone(), non_eth_uxt, eth_uxt],
					vec![eth_extrinsic_hash, eth_extrinsic_hash],
					&block
				)
				.is_ok());
			});
	}

	#[test]
	fn debug_runtime_api_trace_call() {
		let block = Header {
			digest: Default::default(),
			extrinsics_root: Default::default(),
			number: 1,
			parent_hash: Default::default(),
			state_root: Default::default(),
		};
		let alith = H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
			.expect("internal H160 is valid; qed");
		let alith_account_id =
			<Runtime as pallet_evm::Config>::AddressMapping::into_account_id(alith);
		ExtBuilder::default()
			.with_balances(vec![(alith_account_id, 100 * GLMR)])
			.build()
			.execute_with(|| {
				assert!(Runtime::trace_call(
					&block,
					alith,
					H160::random(),
					Vec::new(),
					U256::from(99),
					U256::max_value(),
					Some(U256::one()),
					Some(U256::one()),
					None,
					None,
					None,
				)
				.is_ok());
			});
	}
}
