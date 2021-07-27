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

use super::{error, EvmResult};
use sp_core::{H160, H256, U256};
use sp_std::{vec, vec::Vec};

/// Wrapper around an EVM input slice, helping to parse it.
/// Provide functions to parse common types.
#[derive(Clone, Copy, Debug)]
pub struct EvmDataReader<'a> {
	input: &'a [u8],
	cursor: usize,
	max_read_position: usize,
}

impl<'a> EvmDataReader<'a> {
	/// Create a new input parser.
	pub fn new(input: &'a [u8]) -> Self {
		Self {
			input,
			cursor: 0,
			max_read_position: 0,
		}
	}

	/// Check the input has the correct amount of arguments before end (32 bytes values).
	/// This cannot be used if the arguments contains arrays as array parsing is context dependent.
	/// If at least one argument is an array, parse it first, then call `validate` after parsing
	/// the entire input.
	pub fn expect_arguments(&self, args: usize) -> EvmResult {
		if self.input.len() == self.cursor + args * 32 {
			Ok(())
		} else {
			Err(error("input doesn't match expected length"))
		}
	}

	/// Check the input has been completely read.
	pub fn check_complete(&self) -> EvmResult {
		if self.max_read_position == self.input.len() {
			Ok(())
		} else {
			Err(error("input doesn't match expected length"))
		}
	}

	/// Read data from the input.
	pub fn read<T: EvmData>(&mut self) -> EvmResult<T> {
		T::read(self)
	}

	/// Parse (4 bytes) selector.
	/// Returns an error if trying to parse out of bounds.
	pub fn read_selector(&mut self) -> EvmResult<&[u8]> {
		let range_end = self.cursor + 4;

		let data = self
			.input
			.get(self.cursor..range_end)
			.ok_or_else(|| error("tried to parse selector out of bounds"))?;

		self.cursor += 4;
		self.update_max_read_position(self.cursor);

		Ok(data)
	}

	fn update_max_read_position(&mut self, local_max: usize) {
		if self.max_read_position < local_max {
			self.max_read_position = local_max;
		}
	}
}

/// Help build an EVM input/output data.
#[derive(Clone, Debug)]
pub struct EvmDataWriter {
	data: Vec<u8>,
}

impl EvmDataWriter {
	/// Creates a new empty output builder.
	pub fn new() -> Self {
		Self { data: vec![] }
	}

	/// Return the built data.
	pub fn build(self) -> Vec<u8> {
		self.data
	}

	/// Write arbitrary bytes.
	pub fn write_raw_bytes(mut self, value: &[u8]) -> Self {
		self.data.extend_from_slice(value);
		self
	}

	/// Write data of requested type.
	pub fn write<T: EvmData>(mut self, value: T) -> Self {
		T::write(&mut self, value);
		self
	}
}

impl Default for EvmDataWriter {
	fn default() -> Self {
		Self::new()
	}
}

/// Data that can be converted from and to EVM data types.
pub trait EvmData: Sized {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self>;
	fn write(writer: &mut EvmDataWriter, value: Self);
}

impl EvmData for H256 {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let range_end = reader.cursor + 32;

		let data = reader
			.input
			.get(reader.cursor..range_end)
			.ok_or_else(|| error("tried to parse H256 out of bounds"))?;

		reader.cursor += 32;
		reader.update_max_read_position(reader.cursor);

		Ok(H256::from_slice(data))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		writer.data.extend_from_slice(&value.as_bytes());
	}
}

impl EvmData for H160 {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let range_end = reader.cursor + 32;

		let data = reader
			.input
			.get(reader.cursor..range_end)
			.ok_or_else(|| error("tried to parse H160 out of bounds"))?;

		reader.cursor += 32;
		reader.update_max_read_position(reader.cursor);

		Ok(H160::from_slice(&data[12..32]))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		H256::write(writer, value.into());
	}
}

impl EvmData for U256 {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let range_end = reader.cursor + 32;

		let data = reader
			.input
			.get(reader.cursor..range_end)
			.ok_or_else(|| error("tried to parse U256 out of bounds"))?;

		reader.cursor += 32;
		reader.update_max_read_position(reader.cursor);

		Ok(U256::from_big_endian(data))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let mut buffer = [0u8; 32];
		value.to_big_endian(&mut buffer);
		writer.data.extend_from_slice(&buffer);
	}
}

impl EvmData for bool {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let h256 = H256::read(reader).map_err(|_| error("tried to parse bool out of bounds"))?;

		Ok(!h256.is_zero())
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let mut buffer = [0u8; 32];
		if value {
			buffer[31] = 1;
		}

		writer.data.extend_from_slice(&buffer);
	}
}
