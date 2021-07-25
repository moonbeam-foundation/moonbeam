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

#![cfg_attr(not(feature = "std"), no_std)]

use cumulus_primitives_core::relay_chain;
use evm::ExitError;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Get,
};
use pallet_evm::{GasWeightMapping, Log};
use sp_core::{H160, H256, U256};
use sp_std::{marker::PhantomData, vec, vec::Vec};

/// Alias for Result returning an EVM precompile error.
pub type EvmResult<T = ()> = Result<T, ExitError>;

/// Return an error with provided (static) text.
pub fn error(text: &'static str) -> ExitError {
	ExitError::Other(text.into())
}

/// Wrapper around an EVM input slice, helping to parse it.
/// Provide functions to parse common types.
#[derive(Clone, Copy)]
pub struct InputReader<'a> {
	input: &'a [u8],
	cursor: usize,
}

impl<'a> InputReader<'a> {
	/// Create a new input parser.
	pub fn new(input: &'a [u8]) -> EvmResult<Self> {
		if input.len() >= 4 {
			Ok(Self { input, cursor: 4 })
		} else {
			Err(error("input must at least contain a selector"))
		}
	}

	/// Extract selector from input.
	pub fn selector(&self) -> &[u8] {
		&self.input[0..4]
	}

	/// Check the input has the correct amount of arguments (32 bytes values).
	pub fn expect_arguments(&self, args: usize) -> EvmResult {
		if self.input.len() == 4 + args * 32 {
			Ok(())
		} else {
			Err(error("input doesn't match expected length"))
		}
	}

	/// Check the input has the correct amount of arguments (32 bytes values).
	pub fn expect_minimum_arguments(&self, args: usize) -> EvmResult {
		if self.input.len() >= 4 + args * 32 {
			Ok(())
		} else {
			Err(error("input doesn't match expected length"))
		}
	}

	/// Parse a U256 value.
	/// Returns an error if trying to parse out of bound.
	pub fn read_u256(&mut self) -> EvmResult<U256> {
		let range_end = self.cursor + 32;
		let data = self
			.input
			.get(self.cursor..range_end)
			.ok_or_else(|| error("tried to parse out of bound"))?;

		self.cursor += 32;

		Ok(U256::from_big_endian(data))
	}

	/// Parse a u32 value.
	/// Returns an error if trying to parse out of bound.
	pub fn read_u32(&mut self) -> EvmResult<u32> {
		let range_end = self.cursor + 32;

		let data = self
			.input
			.get(self.cursor..range_end)
			.ok_or_else(|| error("tried to parse out of bound"))?;

		self.cursor += 32;
		let mut output: [u8; 4] = [0u8; 4];
		output.copy_from_slice(&data[28..32]);
		Ok(u32::from_be_bytes(output))
	}

	/// Parse a u32 value.
	/// Returns an error if trying to parse out of bound.
	pub fn read_bool(&mut self) -> EvmResult<bool> {
		let range_end = self.cursor + 32;

		let data = self
			.input
			.get(self.cursor..range_end)
			.ok_or_else(|| error("tried to parse out of bound"))?;

		self.cursor += 32;
		let value = if data[31] == 0 { false } else { true };
		Ok(value)
	}

	/// Parse an address value.
	/// Returns an error if trying to parse out of bound.
	/// Ignores the 12 higher bytes.
	pub fn read_address(&mut self) -> EvmResult<H160> {
		let range_end = self.cursor + 32;

		let data = self
			.input
			.get(self.cursor..range_end)
			.ok_or_else(|| error("tried to parse out of bound"))?;

		self.cursor += 32;

		Ok(H160::from_slice(&data[12..32]))
	}

	/// Parse a relay address value.
	/// Returns an error if trying to parse out of bound.
	pub fn read_relay_address(&mut self) -> EvmResult<relay_chain::AccountId> {
		let range_end = self.cursor + 32;

		let data = self
			.input
			.get(self.cursor..range_end)
			.ok_or_else(|| error("tried to parse out of bound"))?;

		self.cursor += 32;

		let mut output: [u8; 32] = [0u8; 32];
		output.copy_from_slice(&data[0..32]);

		Ok(relay_chain::AccountId::from(output))
	}
}

/// Help build an EVM output data.
pub struct OutputBuilder {
	data: Vec<u8>,
}

impl OutputBuilder {
	/// Creates a new empty output builder.
	pub fn new() -> Self {
		Self { data: vec![] }
	}

	/// Return the built data.
	pub fn build(self) -> Vec<u8> {
		self.data
	}

	/// Push a U256 to the output.
	pub fn write_u256<T: Into<U256>>(mut self, value: T) -> Self {
		let mut buffer = [0u8; 32];
		value.into().to_big_endian(&mut buffer);
		self.data.extend_from_slice(&buffer);
		self
	}

	/// Push a U256 to the output.
	pub fn write_bool<T: Into<bool>>(mut self, value: T) -> Self {
		let mut buffer = [0u8; 32];
		if value.into() {
			buffer[31] = 1;
		}
		self.data.extend_from_slice(&buffer);
		self
	}

	/// Write bytes
	/// https://rinkeby.etherscan.io/tx/
	///	0xd7426d39a1ee7a59e94d0546023cbeec5cfd362ffca66b7e524ca06bc52814d0
	// Bytes
	pub fn write_bytes<T: Into<Vec<u8>>>(mut self, value: T) -> Self {
		let data = value.into();
		let length = data.len();
		let mut cursor = 0;
		while cursor < length {
			let mut buffer = [0u8; 32];
			if cursor + 32 > length {
				buffer[0..length - cursor].copy_from_slice(&data[cursor..]);
			} else {
				buffer.copy_from_slice(&data[cursor..cursor + 32]);
			}
			self.data.extend_from_slice(&buffer);
			cursor += 32;
		}
		self
	}
}

impl Default for OutputBuilder {
	fn default() -> Self {
		Self::new()
	}
}

/// Builder for PrecompileOutput.
pub struct LogsBuilder {
	address: H160,
	logs: Vec<Log>,
}

impl LogsBuilder {
	/// Create a new builder with no logs.
	/// Takes the address of the precompile (usualy `context.address`).
	pub fn new(address: H160) -> Self {
		Self {
			logs: vec![],
			address,
		}
	}

	/// Returns the logs array.
	pub fn build(self) -> Vec<Log> {
		self.logs
	}

	/// Add a 0-topic log.
	pub fn log0<D>(mut self, data: D) -> Self
	where
		D: Into<Vec<u8>>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![],
		});
		self
	}

	/// Add a 1-topic log.
	pub fn log1<D, T0>(mut self, topic0: T0, data: D) -> Self
	where
		D: Into<Vec<u8>>,
		T0: Into<H256>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![topic0.into()],
		});
		self
	}

	/// Add a 2-topics log.
	pub fn log2<D, T0, T1>(mut self, topic0: T0, topic1: T1, data: D) -> Self
	where
		D: Into<Vec<u8>>,
		T0: Into<H256>,
		T1: Into<H256>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![topic0.into(), topic1.into()],
		});
		self
	}

	/// Add a 3-topics log.
	pub fn log3<D, T0, T1, T2>(mut self, topic0: T0, topic1: T1, topic2: T2, data: D) -> Self
	where
		D: Into<Vec<u8>>,
		T0: Into<H256>,
		T1: Into<H256>,
		T2: Into<H256>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![topic0.into(), topic1.into(), topic2.into()],
		});
		self
	}

	/// Add a 4-topics log.
	pub fn log4<D, T0, T1, T2, T3>(
		mut self,
		topic0: T0,
		topic1: T1,
		topic2: T2,
		topic3: T3,
		data: D,
	) -> Self
	where
		D: Into<Vec<u8>>,
		T0: Into<H256>,
		T1: Into<H256>,
		T2: Into<H256>,
		T3: Into<H256>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![topic0.into(), topic1.into(), topic2.into(), topic3.into()],
		});
		self
	}
}

/// Helper functions requiring a Runtime.
/// This runtime must of course implement `pallet_evm::Config`.
pub struct RuntimeHelper<Runtime>(PhantomData<Runtime>);

impl<Runtime> RuntimeHelper<Runtime>
where
	Runtime: pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
{
	/// Try to dispatch a Substrate call.
	/// Return an error if there are not enough gas, or if the call fails.
	/// If succesful returns the used gas using the Runtime GasWeightMapping.
	pub fn try_dispatch<Call>(
		origin: <Runtime::Call as Dispatchable>::Origin,
		call: Call,
		target_gas: Option<u64>,
	) -> EvmResult<u64>
	where
		Runtime::Call: From<Call>,
	{
		let call = Runtime::Call::from(call);
		let dispatch_info = call.get_dispatch_info();

		// Make sure there is enough gas.
		if let Some(gas_limit) = target_gas {
			let required_gas = Runtime::GasWeightMapping::weight_to_gas(dispatch_info.weight);
			if required_gas > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		// Dispatch call.
		let used_weight = call
			.dispatch(origin)
			.map_err(|_| error("dispatched call failed"))?
			.actual_weight;

		// Return used weight by converting weight to gas.
		Ok(Runtime::GasWeightMapping::weight_to_gas(
			used_weight.unwrap_or(dispatch_info.weight),
		))
	}

	/// Cost of a Substrate DB write in gas.
	pub fn db_write_gas_cost() -> u64 {
		<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().write,
		)
	}

	/// Cost of a Substrate DB read in gas.
	pub fn db_read_gas_cost() -> u64 {
		<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		)
	}
}
