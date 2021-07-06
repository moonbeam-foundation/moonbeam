environmental::environmental!(listener: dyn Listener + 'static);

use crate::block::{
	TransactionTrace as BlockTrace, TransactionTraceAction, TransactionTraceOutput,
	TransactionTraceResult,
};
use crate::single::{Call, CallInner, RawStepLog, TransactionTrace as SingleTrace};
use crate::{CallResult, CreateResult, CreateType};
use codec::{Decode, Encode};
use ethereum_types::{H256, U256};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

pub trait Listener {
	fn event(&mut self, event: Event);
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub enum Event {
	RawStep(RawStepLog),
	RawGas(U256),
	RawReturnValue(Vec<u8>),
	CallListEntry((u32, Call)),
	CallListNew(),
}

impl Event {
	pub fn emit(self) {
		listener::with(|listener| listener.event(self));
	}
}

// Raw
#[derive(Debug)]
pub struct RawProxy {
	gas: U256,
	return_value: Vec<u8>,
	step_logs: Vec<RawStepLog>,
}

impl RawProxy {
	pub fn new() -> Self {
		Self {
			gas: U256::zero(),
			return_value: Vec::new(),
			step_logs: Vec::new(),
		}
	}

	pub fn using<R, F: FnOnce() -> R>(&mut self, f: F) {
		listener::using(self, f);
	}

	pub fn into_tx_trace(self) -> SingleTrace {
		SingleTrace::Raw {
			step_logs: self.step_logs,
			gas: self.gas,
			return_value: self.return_value,
		}
	}
}

impl Listener for RawProxy {
	fn event(&mut self, event: Event) {
		match event {
			Event::RawStep(step) => self.step_logs.push(step),
			Event::RawGas(gas) => self.gas = gas,
			Event::RawReturnValue(value) => self.return_value = value,
			_ => {}
		};
	}
}

// List
#[derive(Debug)]
pub struct CallListProxy {
	entries: Vec<BTreeMap<u32, Call>>,
}

impl CallListProxy {
	pub fn new() -> Self {
		Self {
			entries: Vec::new(),
		}
	}
	pub fn using<R, F: FnOnce() -> R>(&mut self, f: F) {
		listener::using(self, f);
	}

	pub fn into_tx_trace(self) -> SingleTrace {
		SingleTrace::CallList(
			self.entries
				.last()
				.unwrap()
				.into_iter()
				.map(|(_, value)| value.clone())
				.collect(),
		)
	}

	pub fn into_tx_traces(self) -> Vec<BlockTrace> {
		let mut traces = Vec::new();
		for (eth_tx_index, entry) in self.entries.iter().enumerate() {
			let mut tx_traces: Vec<_> = entry
				.into_iter()
				.map(|(_, trace)| match trace.inner.clone() {
					CallInner::Call {
						input,
						to,
						res,
						call_type,
					} => BlockTrace {
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
							crate::CallResult::Error(error) => TransactionTraceOutput::Error(error),
						},
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.clone(),
						// Can't be known here, must be inserted upstream.
						transaction_hash: H256::default(),
						transaction_position: eth_tx_index as u32,
					},
					CallInner::Create { init, res } => {
						BlockTrace {
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
								crate::CreateResult::Error { error } => {
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
					CallInner::SelfDestruct {
						balance,
						refund_address,
					} => BlockTrace {
						action: TransactionTraceAction::Suicide {
							address: trace.from,
							balance,
							refund_address,
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
		traces
	}
}

impl Listener for CallListProxy {
	fn event(&mut self, event: Event) {
		match event {
			Event::CallListNew() => {
				self.entries.push(BTreeMap::new());
			}
			Event::CallListEntry((index, value)) => {
				if self.entries.is_empty() {
					self.entries.push(BTreeMap::new());
				}
				self.entries.last_mut().unwrap().insert(index, value);
			}
			_ => {}
		};
	}
}
