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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use super::blockscout::BlockscoutCallInner as CallInner;
use crate::listeners::call_list::Listener;
use crate::types::{
	block::{
		TransactionTrace, TransactionTraceAction, TransactionTraceOutput, TransactionTraceResult,
	},
	CallResult, CreateResult, CreateType,
};
use ethereum_types::H256;

pub struct Formatter;

impl super::ResponseFormatter for Formatter {
	type Listener = Listener;
	type Response = Vec<TransactionTrace>;

	fn format(mut listener: Listener) -> Option<Vec<TransactionTrace>> {
		// Remove empty BTreeMaps pushed to `entries`.
		// I.e. InvalidNonce or other pallet_evm::runner exits
		listener.entries.retain(|x| !x.is_empty());
		let mut traces = Vec::new();
		for (eth_tx_index, entry) in listener.entries.iter().enumerate() {
			let mut tx_traces: Vec<_> = entry
				.into_iter()
				.map(|(_, trace)| match trace.inner.clone() {
					CallInner::Call {
						input,
						to,
						res,
						call_type,
					} => TransactionTrace {
						action: TransactionTraceAction::Call {
							call_type,
							from: trace.from,
							gas: trace.gas,
							input,
							to,
							value: trace.value,
						},
						// Can't be known here, must be inserted upstream.
						block_hash: H256::default(),
						// Can't be known here, must be inserted upstream.
						block_number: 0,
						output: match res {
							CallResult::Output(output) => {
								TransactionTraceOutput::Result(TransactionTraceResult::Call {
									gas_used: trace.gas_used,
									output,
								})
							}
							CallResult::Error(error) => TransactionTraceOutput::Error(error),
						},
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.clone(),
						// Can't be known here, must be inserted upstream.
						transaction_hash: H256::default(),
						transaction_position: eth_tx_index as u32,
					},
					CallInner::Create { init, res } => {
						TransactionTrace {
							action: TransactionTraceAction::Create {
								creation_method: CreateType::Create,
								from: trace.from,
								gas: trace.gas,
								init,
								value: trace.value,
							},
							// Can't be known here, must be inserted upstream.
							block_hash: H256::default(),
							// Can't be known here, must be inserted upstream.
							block_number: 0,
							output: match res {
								CreateResult::Success {
									created_contract_address_hash,
									created_contract_code,
								} => {
									TransactionTraceOutput::Result(TransactionTraceResult::Create {
										gas_used: trace.gas_used,
										code: created_contract_code,
										address: created_contract_address_hash,
									})
								}
								CreateResult::Error { error } => {
									TransactionTraceOutput::Error(error)
								}
							},
							subtraces: trace.subtraces,
							trace_address: trace.trace_address.clone(),
							// Can't be known here, must be inserted upstream.
							transaction_hash: H256::default(),
							transaction_position: eth_tx_index as u32,
						}
					}
					CallInner::SelfDestruct { balance, to } => TransactionTrace {
						action: TransactionTraceAction::Suicide {
							address: trace.from,
							balance,
							refund_address: to,
						},
						// Can't be known here, must be inserted upstream.
						block_hash: H256::default(),
						// Can't be known here, must be inserted upstream.
						block_number: 0,
						output: TransactionTraceOutput::Result(TransactionTraceResult::Suicide),
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.clone(),
						// Can't be known here, must be inserted upstream.
						transaction_hash: H256::default(),
						transaction_position: eth_tx_index as u32,
					},
				})
				.collect();

			traces.append(&mut tx_traces);
		}
		Some(traces)
	}
}
