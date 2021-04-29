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

use crate::executor::util::opcodes;

use super::*;

use ethereum_types::{H160, H256};
use evm::{gasometer, ExitSucceed, Handler, Opcode};
use moonbeam_rpc_primitives_debug::single::RawStepLog;
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

pub struct State {
	disable_storage: bool,
	disable_memory: bool,
	disable_stack: bool,

	depth: usize,
	step_logs: Vec<RawStepLog>,
	context_stack: Vec<Context>,

	/// Gas at the end of the EVM execution.
	final_gas: u64,
	/// Return value of the EVM execution.
	return_value: Vec<u8>,
}

struct Context {
	storage_cache: BTreeMap<H256, H256>,
	address: H160,
	before_step: Option<BeforeStep>,
}

struct BeforeStep {
	/// Current opcode.
	opcode: Opcode,
	/// Depth of the context.
	depth: usize,
	/// Key of a storage entry in case of storage interaction.
	storage_key: Option<H256>,
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

impl State {
	pub fn new(disable_storage: bool, disable_memory: bool, disable_stack: bool) -> Self {
		Self {
			disable_storage,
			disable_memory,
			disable_stack,

			depth: 0,
			step_logs: vec![],
			context_stack: vec![],

			final_gas: 0,
			return_value: vec![],
		}
	}

	/// Called before the execution of a context.
	pub fn before_loop<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		_executor: &StackExecutor<'config, S, H>,
		runtime: &Runtime,
	) {
		self.depth += 1;
		self.context_stack.push(Context {
			storage_cache: BTreeMap::new(),
			address: runtime.context().address,
			before_step: None,
		});
	}

	/// Called before each step.
	pub fn before_step<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		executor: &StackExecutor<'config, S, H>,
		runtime: &Runtime,
	) {
		if let Some((opcode, stack)) = runtime.machine().inspect() {
			// Get all data.
			let depth = executor.state().metadata().depth().unwrap_or_default() + 1;
			let gas = executor.gas();
			let gas_cost = match gasometer::static_opcode_cost(opcode) {
				Some(cost) => cost,
				_ => {
					match gasometer::dynamic_opcode_cost(
						runtime.context().address,
						opcode,
						stack,
						executor.state().metadata().is_static(),
						executor.config(),
						executor,
					) {
						Ok((opcode_cost, _)) => executor
							.state()
							.metadata()
							.gasometer()
							.gas_cost(opcode_cost, gas)
							.unwrap_or(0),
						Err(_) => 0,
					}
				}
			};

			let position = *runtime.machine().position().as_ref().unwrap_or(&0);

			let memory_copy = if self.disable_memory {
				None
			} else {
				Some(runtime.machine().memory().data().clone())
			};

			let stack_copy = if self.disable_stack {
				None
			} else {
				Some(runtime.machine().stack().data().clone())
			};

			let mut before_step = BeforeStep {
				depth,
				opcode,
				storage_key: None,
				gas,
				gas_cost,
				position,
				memory: memory_copy,
				stack: stack_copy,
			};

			// If an opcode is directly reading/writing storage we need to
			// keep track of it.
			if single_storage_opcode(opcode) {
				if let Ok(key) = stack.peek(0) {
					before_step.storage_key = Some(key)
				}
			}

			// Keep this data in the context so it can be retreived
			// after this opcode in this context is executed.
			self.context_stack
				.last_mut()
				.expect("before_step is called after before_loop")
				.before_step = Some(before_step);
		}
	}

	/// Called after each step. Will not be called if runtime exited
	/// from the loop.
	pub fn after_step<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		executor: &StackExecutor<'config, S, H>,
		_runtime: &Runtime,
	) {
		let context = self
			.context_stack
			.last_mut()
			.expect("after_step is called after before_loop");

		let before_step = context
			.before_step
			.take()
			.expect("after_step is called after before_step");

		self.end_step(executor, before_step);
	}

	/// Called after the execution of a context.
	pub fn after_loop<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		executor: &StackExecutor<'config, S, H>,
		runtime: &Runtime,
		reason: &ExitReason,
	) {
		self.depth -= 1;
		let context = self
			.context_stack
			.last_mut()
			.expect("after_step is called after before_loop");

		// If we're exiting the root scope, we store the final gas
		// and result.
		if self.depth == 0 {
			self.final_gas = executor.gas();

			if &ExitReason::Succeed(ExitSucceed::Returned) == reason {
				self.return_value = runtime.machine().return_value();
			}
		}

		// If the last opcode of the scope reverted then the
		// step data is still here, and we need to process it
		// now.
		if let Some(before_step) = context.before_step.take() {
			self.end_step(executor, before_step);
		}

		// We pop the last context as we're exiting it.
		let _ = self.context_stack.pop();
	}

	fn end_step<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		executor: &StackExecutor<'config, S, H>,
		before_step: BeforeStep,
	) {
		let context = self
			.context_stack
			.last_mut()
			.expect("after_step is called after before_loop");

		let BeforeStep {
			depth,
			opcode,
			storage_key,
			gas,
			gas_cost,
			position,
			memory,
			stack,
		} = before_step;

		// Update the storage cache if necessary.
		if let Some(key) = storage_key {
			let _ = context
				.storage_cache
				.insert(key, executor.storage(context.address, key));
		}
		// Call opcodes can indirectly change the storage values
		// in subcalls.
		else if rescan_storage_opcode(opcode) {
			for (key, value) in context.storage_cache.iter_mut() {
				*value = executor.storage(context.address, *key);
			}
		}

		// Copy cached storage if not disabled.
		let storage = if self.disable_storage {
			None
		} else {
			Some(context.storage_cache.clone())
		};

		// Convert memory format.
		let memory = memory.map(convert_memory);

		// Add log to result array.
		self.step_logs.push(RawStepLog {
			depth: depth.into(),
			gas: gas.into(),
			gas_cost: gas_cost.into(),
			memory,
			op: opcodes(opcode),
			pc: position.into(),
			stack,
			storage,
		});
	}

	pub fn finish(self) -> TransactionTrace {
		TransactionTrace::Raw {
			gas: self.final_gas.into(),
			return_value: self.return_value,
			step_logs: self.step_logs,
		}
	}
}

fn single_storage_opcode(opcode: Opcode) -> bool {
	matches!(
		opcode,
		Opcode(0x54) | // sload
        Opcode(0x55) // sstore
	)
}

fn rescan_storage_opcode(opcode: Opcode) -> bool {
	matches!(
		opcode,
		Opcode(240) | // create
        Opcode(241) | // call
        Opcode(242) | // call code
        Opcode(244) | // delegate call
        Opcode(245) | // create 2
        Opcode(250) // static call
	)
}

fn convert_memory(memory: Vec<u8>) -> Vec<H256> {
	let size = 32;
	memory
		.chunks(size)
		.map(|c| {
			let mut msg = [0u8; 32];
			let chunk = c.len();
			if chunk < size {
				let left = size - chunk;
				let remainder = vec![0; left];
				msg[0..left].copy_from_slice(&remainder[..]);
				msg[left..size].copy_from_slice(c);
			} else {
				msg[0..size].copy_from_slice(c)
			}
			H256::from_slice(&msg[..])
		})
		.collect()
}
