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
use core::{any::type_name, ops::Range};
use sp_core::{H160, H256, U256};
use sp_std::{convert::TryInto, vec, vec::Vec};

/// The `address` type of Solidity.
/// H160 could represent 2 types of data (bytes20 and address) that are not encoded the same way.
/// To avoid issues writing H160 is thus not supported.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Address(pub H160);

impl From<H160> for Address {
	fn from(a: H160) -> Address {
		Address(a)
	}
}

impl From<Address> for H160 {
	fn from(a: Address) -> H160 {
		a.0
	}
}

/// Wrapper around an EVM input slice, helping to parse it.
/// Provide functions to parse common types.
#[derive(Clone, Copy, Debug)]
pub struct EvmDataReader<'a> {
	input: &'a [u8],
	cursor: usize,
}

impl<'a> EvmDataReader<'a> {
	/// Create a new input parser.
	pub fn new(input: &'a [u8]) -> Self {
		Self { input, cursor: 0 }
	}

	/// Check the input has at least the correct amount of arguments before the end (32 bytes values).
	pub fn expect_arguments(&self, args: usize) -> EvmResult {
		if self.input.len() >= self.cursor + args * 32 {
			Ok(())
		} else {
			Err(error("input doesn't match expected length"))
		}
	}

	/// Read data from the input.
	pub fn read<T: EvmData>(&mut self) -> EvmResult<T> {
		T::read(self)
	}

	/// Read raw bytes from the input.
	/// Doesn't handle any alignement checks, prefer using `read` instead of possible.
	/// Returns an error if trying to parse out of bounds.
	pub fn read_raw_bytes(&mut self, len: usize) -> EvmResult<&[u8]> {
		let range = self.move_cursor(len)?;

		let data = self
			.input
			.get(range)
			.ok_or_else(|| error("tried to parse raw bytes out of bounds"))?;

		Ok(data)
	}

	/// Parse (4 bytes) selector.
	/// Returns an error if trying to parse out of bounds.
	pub fn read_selector<T>(&mut self) -> EvmResult<T>
	where
		T: num_enum::TryFromPrimitive<Primitive = u32>,
	{
		let mut buffer = [0u8; 4];
		buffer.copy_from_slice(
			self.read_raw_bytes(4)
				.map_err(|_| error("tried to parse selector out of bounds"))?,
		);
		T::try_from_primitive(u32::from_be_bytes(buffer)).map_err(|_| {
			log::trace!(
				target: "precompile-utils",
				"Failed to match function selector for {}",
				type_name::<T>()
			);
			error("unknown selector")
		})
	}

	/// Move the reading cursor with provided length, and return a range from the previous cursor
	/// location to the new one.
	/// Checks cursor overflows.
	fn move_cursor(&mut self, len: usize) -> EvmResult<Range<usize>> {
		let start = self.cursor;
		let end = self
			.cursor
			.checked_add(len)
			.ok_or_else(|| error("data reading cursor overflow"))?;

		self.cursor = end;

		Ok(start..end)
	}
}

/// Help build an EVM input/output data.
#[derive(Clone, Debug)]
pub struct EvmDataWriter {
	pub(crate) data: Vec<u8>,
	arrays: Vec<Array>,
}

#[derive(Clone, Debug)]
struct Array {
	offset_position: usize,
	data: Vec<u8>,
	inner_arrays: Vec<Array>,
}

impl EvmDataWriter {
	/// Creates a new empty output builder.
	pub fn new() -> Self {
		Self {
			data: vec![],
			arrays: vec![],
		}
	}

	/// Return the built data.
	pub fn build(mut self) -> Vec<u8> {
		Self::build_arrays(&mut self.data, self.arrays, 0);

		self.data
	}

	/// Build the array into data.
	/// `global_offset` represents the start of the frame we are modifying.
	/// While the main data will have a `global_offset` of 0, inner arrays will have a
	/// `global_offset` corresponding to the start its parent array size data.
	fn build_arrays(output: &mut Vec<u8>, arrays: Vec<Array>, global_offset: usize) {
		for mut array in arrays {
			let offset_position = array.offset_position;
			let offset_position_end = offset_position + 32;
			let free_space_offset = output.len() + global_offset;

			// Override dummy offset to the offset it will be in the final output.
			U256::from(free_space_offset)
				.to_big_endian(&mut output[offset_position..offset_position_end]);

			// Build inner arrays if any.
			Self::build_arrays(&mut array.data, array.inner_arrays, free_space_offset);

			// Append this data at the end of the current output.
			output.append(&mut array.data);
		}
	}

	/// Write arbitrary bytes.
	/// Doesn't handle any alignement checks, prefer using `write` instead of possible.
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
		let range = reader.move_cursor(32)?;

		let data = reader
			.input
			.get(range)
			.ok_or_else(|| error("tried to parse H256 out of bounds"))?;

		Ok(H256::from_slice(data))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		writer.data.extend_from_slice(&value.as_bytes());
	}
}

impl EvmData for Address {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let range = reader.move_cursor(32)?;

		let data = reader
			.input
			.get(range)
			.ok_or_else(|| error("tried to parse H160 out of bounds"))?;

		Ok(H160::from_slice(&data[12..32]).into())
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		H256::write(writer, value.0.into());
	}
}

impl EvmData for U256 {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let range = reader.move_cursor(32)?;

		let data = reader
			.input
			.get(range)
			.ok_or_else(|| error("tried to parse U256 out of bounds"))?;

		Ok(U256::from_big_endian(data))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let mut buffer = [0u8; 32];
		value.to_big_endian(&mut buffer);
		writer.data.extend_from_slice(&buffer);
	}
}

macro_rules! impl_evmdata_for_uints {
	($($uint:ty, )*) => {
		$(
			impl EvmData for $uint {
				fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
					let range = reader.move_cursor(32)?;

					let data = reader
						.input
						.get(range)
						.ok_or_else(|| error(alloc::format!(
							"tried to parse {} out of bounds", core::any::type_name::<Self>()
						)))?;

					let mut buffer = [0u8; core::mem::size_of::<Self>()];
					buffer.copy_from_slice(&data[32 - core::mem::size_of::<Self>()..]);
					Ok(Self::from_be_bytes(buffer))
				}

				fn write(writer: &mut EvmDataWriter, value: Self) {
					let mut buffer = [0u8; 32];
					buffer[32 - core::mem::size_of::<Self>()..].copy_from_slice(&value.to_be_bytes());
					writer.data.extend_from_slice(&buffer);
				}
			}
		)*
	};
}
impl_evmdata_for_uints!(u16, u32, u64, u128,);

// The implementation for u8 is specific, for performance reasons.
impl EvmData for u8 {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let range = reader.move_cursor(32)?;

		let data = reader
			.input
			.get(range)
			.ok_or_else(|| error("tried to parse u64 out of bounds"))?;

		Ok(data[31])
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let mut buffer = [0u8; 32];
		buffer[31] = value;

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

impl<T: EvmData> EvmData for Vec<T> {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let array_start: usize = reader
			.read::<U256>()
			.map_err(|_| error("tried to parse array offset out of bounds"))?
			.try_into()
			.map_err(|_| error("array offset is too large"))?;

		// We temporarily move the cursor to the offset, we'll set it back afterward.
		let original_cursor = reader.cursor;
		reader.cursor = array_start;

		let array_size: usize = reader
			.read::<U256>()
			.map_err(|_| error("tried to parse array length out of bounds"))?
			.try_into()
			.map_err(|_| error("array length is too large"))?;

		let mut array = vec![];

		for _ in 0..array_size {
			array.push(reader.read()?);
		}

		// We set back the cursor to its original location.
		reader.cursor = original_cursor;

		Ok(array)
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let offset_position = writer.data.len();
		H256::write(writer, H256::repeat_byte(0xff));
		// 0xff = When debugging it makes spoting offset values easier.

		let mut inner_writer = EvmDataWriter::new();

		// Write length.
		inner_writer = inner_writer.write(U256::from(value.len()));

		// Write elements of array.
		for inner in value {
			inner_writer = inner_writer.write(inner);
		}

		let array = Array {
			offset_position,
			data: inner_writer.data,
			inner_arrays: inner_writer.arrays,
		};

		writer.arrays.push(array);
	}
}
