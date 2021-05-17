// Copyright 2019-2020 PureStake Inc.
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

use crate::util::*;

use ethereum_types::{H160, H256};
use evm::{Capture, ExitReason};
use moonbeam_rpc_primitives_debug::single::{RawStepLog, TransactionTrace};
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

#[derive(Debug)]
pub struct RawTracer {
	disable_storage: bool,
	disable_memory: bool,
	disable_stack: bool,

	step_logs: Vec<RawStepLog>,
	return_value: Vec<u8>,
	final_gas: u64,

	new_context: Option<u64>, // gas limit of the new context
	context_stack: Vec<Context>,
}

#[derive(Debug)]
struct Context {
	storage_cache: BTreeMap<H256, H256>,
	address: H160,
	current_step: Option<Step>,
	gas: u64,
	global_storage_changes: BTreeMap<H160, BTreeMap<H256, H256>>,
}

#[derive(Debug)]
struct Step {
	/// Current opcode.
	opcode: Opcode,
	/// Depth of the context.
	depth: usize,
	/// Remaining gas.
	gas: u64,
	/// Gas cost of the following opcode.
	gas_cost: u64,
	/// Program counter position.
	position: usize,
	/// EVM memory copy (if not disabled).
	memory: Option<Vec<u8>>,
	/// EVM stack copy (if not disabled).
	stack: Option<Vec<H256>>,
}

impl RawTracer {
	pub fn new(disable_storage: bool, disable_memory: bool, disable_stack: bool) -> Self {
		Self {
			disable_storage,
			disable_memory,
			disable_stack,

			step_logs: vec![],
			return_value: vec![],
			final_gas: 0,

			new_context: None,
			context_stack: vec![],
		}
	}

	pub fn trace<R, F: FnOnce() -> R>(self, f: F) -> (Self, R) {
		let wrapped = Rc::new(RefCell::new(self));

		let result = {
			let mut gasometer = ListenerProxy(Rc::clone(&wrapped));
			let mut runtime = ListenerProxy(Rc::clone(&wrapped));

			let f = || runtime_using(&mut runtime, f);
			let f = || gasometer_using(&mut gasometer, f);
			f()
		};

		(Rc::try_unwrap(wrapped).unwrap().into_inner(), result)
	}

	pub fn into_tx_trace(self) -> TransactionTrace {
		TransactionTrace::Raw {
			step_logs: self.step_logs,
			gas: self.final_gas.into(),
			return_value: self.return_value,
		}
	}
}

impl GasometerListener for RawTracer {
	fn event(&mut self, event: GasometerEvent) {
		match event {
			GasometerEvent::RecordTransaction(_) => {
				// First event of a transaction.
				// Next step will be the first context.
				self.new_context = Some(0); // we don't know the gas limit yet.
			}
			GasometerEvent::RecordCost(gas_cost) => {
				// When a new context is created the gas limit is recorded afterward.
				if let Some(gas_limit) = &mut self.new_context {
					*gas_limit = gas_cost;
				}
				// If we're not creating a new context.
				else if let Some(context) = self.context_stack.last_mut() {
					context.gas -= gas_cost;

					// Register opcode cost.
					if let Some(step) = &mut context.current_step {
						step.gas_cost += gas_cost;
					}
				}
			}
			GasometerEvent::RecordDynamicCost {
				gas_cost,
				memory_gas,
				gas_refund: _,
			} => {
				if let Some(context) = self.context_stack.last_mut() {
					context.gas -= gas_cost;
					context.gas -= memory_gas;

					// Register opcode cost.
					if let Some(step) = &mut context.current_step {
						step.gas_cost += gas_cost;
					}
				}
			}
			GasometerEvent::RecordStipend(stipend) => {
				if let Some(context) = self.context_stack.last_mut() {
					context.gas += stipend;
				}
			}
			_ => (),
		}
	}
}

impl RuntimeListener for RawTracer {
	fn event(&mut self, event: RuntimeEvent) {
		match event {
			RuntimeEvent::Step {
				context,
				opcode,
				position,
				stack,
				memory,
			} => {
				// Create a context if needed.
				if let Some(gas_limit) = self.new_context {
					self.new_context = None;

					self.context_stack.push(Context {
						storage_cache: BTreeMap::new(),
						address: context.address,
						current_step: None,
						gas: gas_limit,
						global_storage_changes: BTreeMap::new(),
					});
				}

				let depth = self.context_stack.len();

				// Ignore steps outside of any context (shouldn't even be possible).
				if let Some(context) = self.context_stack.last_mut() {
					context.current_step = Some(Step {
						opcode,
						depth,
						gas: context.gas,
						gas_cost: 0, // 0 for now, will add with gas events
						position: *position.as_ref().unwrap_or(&0),
						memory: if self.disable_memory {
							None
						} else {
							Some(memory.data().clone())
						},
						stack: if self.disable_stack {
							None
						} else {
							Some(stack.data().clone())
						},
					});
				}
			}
			RuntimeEvent::StepResult {
				result,
				return_value,
			} => {
				// StepResult is expected to be emited after a step (in a context).
				if let Some(context) = self.context_stack.last_mut() {
					if let Some(current_step) = context.current_step.take() {
						let Step {
							opcode,
							depth,
							gas,
							gas_cost,
							position,
							memory,
							stack,
						} = current_step;

						let memory = memory.map(convert_memory);

						let storage = if self.disable_storage {
							None
						} else {
							Some(context.storage_cache.clone())
						};

						self.step_logs.push(RawStepLog {
							depth: depth.into(),
							gas: gas.into(),
							gas_cost: gas_cost.into(),
							memory,
							op: opcodes_string(opcode),
							pc: position.into(),
							stack,
							storage,
						});
					}
				}

				// We match on the capture to handle traps/exits.
				match result {
					Err(Capture::Exit(reason)) => {
						// Exit = we exit the context (should always be some)
						if let Some(mut context) = self.context_stack.pop() {
							// If final context is exited, we store gas and return value.
							if self.context_stack.is_empty() {
								self.final_gas = context.gas;
								self.return_value = return_value.to_vec();
							}

							// If the context exited without revert we must keep track of the
							// updated storage keys.
							if !self.disable_storage && matches!(reason, &ExitReason::Succeed(_)) {
								if let Some(parent_context) = self.context_stack.last_mut() {
									// Add cache to storage changes.
									context
										.global_storage_changes
										.insert(context.address, context.storage_cache);

									// Apply storage changes to parent, either updating its cache or map of changes.
									for (address, mut storage) in
										context.global_storage_changes.into_iter()
									{
										// Same address => We update its cache (only tracked keys)
										if parent_context.address == address {
											for (cached_key, cached_value) in
												parent_context.storage_cache.iter_mut()
											{
												if let Some(value) = storage.remove(cached_key) {
													*cached_value = value;
												}
											}
										}
										// Otherwise, update the storage changes.
										else {
											parent_context
												.global_storage_changes
												.entry(address)
												.or_insert_with(BTreeMap::new)
												.append(&mut storage);
										}
									}
								}
							}
						}
					}
					Err(Capture::Trap(opcode)) if is_subcall(*opcode) => {
						self.new_context = Some(0);
					}
					_ => (),
				}
			}
			RuntimeEvent::SLoad {
				address: _,
				index,
				value,
			}
			| RuntimeEvent::SStore {
				address: _,
				index,
				value,
			} => {
				if let Some(context) = self.context_stack.last_mut() {
					if !self.disable_storage {
						context.storage_cache.insert(index, value);
					}
				}
			}
		}
	}
}
