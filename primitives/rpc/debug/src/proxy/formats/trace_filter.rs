// Copyright 2019-2021 PureStake Inc.
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

use super::{
	blockscout::{BlockscoutCall, BlockscoutInner},
	Call,
};
use crate::{
	CallResult, CreateResult, CreateType,
	block::{
		TransactionTrace, TransactionTraceAction, TransactionTraceOutput, TransactionTraceResult,
	},
	proxy::v2::call_list::Listener,
};

pub use ethereum_types::{H160, H256, U256};

pub struct Response;

#[cfg(feature = "std")]
impl super::TraceResponseBuilder for Response {
	type Listener = Listener;
	type Response = Vec<TransactionTrace>;

	fn build(listener: Listener) -> Option<Vec<TransactionTrace>> {
		let mut traces = Vec::new();
		for (eth_tx_index, entry) in listener.entries.iter().enumerate() {
			let mut tx_traces: Vec<_> = entry
				.into_iter()
				.filter_map(|(_, trace)| {
					match trace {
						Call::Blockscout(BlockscoutCall {
							from,
							trace_address,
							subtraces,
							value,
							gas,
							gas_used,
							inner,
						}) => match inner.clone() {
							BlockscoutInner::Call {
								input,
								to,
								res,
								call_type,
							} => Some(TransactionTrace {
								action: TransactionTraceAction::Call {
									call_type,
									from: *from,
									gas: *gas,
									input,
									to,
									value: *value,
								},
								// Can't be known here, must be inserted upstream.
								block_hash: H256::default(),
								// Can't be known here, must be inserted upstream.
								block_number: 0,
								output: match res {
									CallResult::Output(output) => TransactionTraceOutput::Result(
										TransactionTraceResult::Call {
											gas_used: *gas_used,
											output,
										},
									),
									crate::CallResult::Error(error) => {
										TransactionTraceOutput::Error(error)
									}
								},
								subtraces: *subtraces,
								trace_address: trace_address.clone(),
								// Can't be known here, must be inserted upstream.
								transaction_hash: H256::default(),
								transaction_position: eth_tx_index as u32,
							}),
							BlockscoutInner::Create { init, res } => {
								Some(TransactionTrace {
									action: TransactionTraceAction::Create {
										creation_method: CreateType::Create,
										from: *from,
										gas: *gas,
										init,
										value: *value,
									},
									// Can't be known here, must be inserted upstream.
									block_hash: H256::default(),
									// Can't be known here, must be inserted upstream.
									block_number: 0,
									output: match res {
										CreateResult::Success {
											created_contract_address_hash,
											created_contract_code,
										} => TransactionTraceOutput::Result(
											TransactionTraceResult::Create {
												gas_used: *gas_used,
												code: created_contract_code,
												address: created_contract_address_hash,
											},
										),
										crate::CreateResult::Error { error } => {
											TransactionTraceOutput::Error(error)
										}
									},
									subtraces: *subtraces,
									trace_address: trace_address.clone(),
									// Can't be known here, must be inserted upstream.
									transaction_hash: H256::default(),
									transaction_position: eth_tx_index as u32,
								})
							}
							BlockscoutInner::SelfDestruct {
								balance,
								refund_address,
							} => Some(TransactionTrace {
								action: TransactionTraceAction::Suicide {
									address: *from,
									balance,
									refund_address,
								},
								// Can't be known here, must be inserted upstream.
								block_hash: H256::default(),
								// Can't be known here, must be inserted upstream.
								block_number: 0,
								output: TransactionTraceOutput::Result(
									TransactionTraceResult::Suicide,
								),
								subtraces: *subtraces,
								trace_address: trace_address.clone(),
								// Can't be known here, must be inserted upstream.
								transaction_hash: H256::default(),
								transaction_position: eth_tx_index as u32,
							}),
						},
						_ => None,
					}
				})
				.map(|x| x)
				.collect();

			traces.append(&mut tx_traces);
		}
		Some(traces)
	}
}
