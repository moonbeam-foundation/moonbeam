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

pub mod bytes;
pub mod native;
pub mod xcm;

pub use affix::paste;
pub use alloc::string::String;
pub use bytes::*;
pub use native::*;

use {
	crate::revert::{InjectBacktrace, MayRevert, RevertReason},
	alloc::borrow::ToOwned,
	core::{any::type_name, marker::PhantomData, ops::Range},
	frame_support::traits::{ConstU32, Get},
	impl_trait_for_tuples::impl_for_tuples,
	sp_core::{H160, H256, U256},
	sp_std::{convert::TryInto, vec, vec::Vec},
};

// derive macro
pub use precompile_utils_macro::EvmData;

/// Data that can be converted from and to EVM data types.
pub trait EvmData: Sized {
	fn read(reader: &mut EvmDataReader) -> MayRevert<Self>;
	fn write(writer: &mut EvmDataWriter, value: Self);
	fn has_static_size() -> bool;
	fn solidity_type() -> String;
	fn is_explicit_tuple() -> bool {
		false
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

	/// Create a new input parser from a selector-initial input.
	pub fn read_selector<T>(input: &'a [u8]) -> MayRevert<T>
	where
		T: num_enum::TryFromPrimitive<Primitive = u32>,
	{
		if input.len() < 4 {
			return Err(RevertReason::read_out_of_bounds("selector").into());
		}

		let mut buffer = [0u8; 4];
		buffer.copy_from_slice(&input[0..4]);
		let selector = T::try_from_primitive(u32::from_be_bytes(buffer)).map_err(|_| {
			log::trace!(
				target: "precompile-utils",
				"Failed to match function selector for {}",
				type_name::<T>()
			);
			RevertReason::UnknownSelector
		})?;

		Ok(selector)
	}

	/// Read selector as u32
	pub fn read_u32_selector(input: &'a [u8]) -> MayRevert<u32> {
		if input.len() < 4 {
			return Err(RevertReason::read_out_of_bounds("selector").into());
		}

		let mut buffer = [0u8; 4];
		buffer.copy_from_slice(&input[0..4]);

		Ok(u32::from_be_bytes(buffer))
	}

	/// Create a new input parser from a selector-initial input.
	pub fn new_skip_selector(input: &'a [u8]) -> MayRevert<Self> {
		if input.len() < 4 {
			return Err(RevertReason::read_out_of_bounds("selector").into());
		}

		Ok(Self::new(&input[4..]))
	}

	/// Check the input has at least the correct amount of arguments before the end (32 bytes values).
	pub fn expect_arguments(&self, args: usize) -> MayRevert<()> {
		if self.input.len() >= self.cursor + args * 32 {
			Ok(())
		} else {
			Err(RevertReason::ExpectedAtLeastNArguments(args).into())
		}
	}

	/// Read data from the input.
	pub fn read<T: EvmData>(&mut self) -> MayRevert<T> {
		T::read(self)
	}

	/// Read raw bytes from the input.
	/// Doesn't handle any alignment checks, prefer using `read` instead of possible.
	/// Returns an error if trying to parse out of bounds.
	pub fn read_raw_bytes(&mut self, len: usize) -> MayRevert<&[u8]> {
		let range = self.move_cursor(len)?;

		let data = self
			.input
			.get(range)
			.ok_or_else(|| RevertReason::read_out_of_bounds("raw bytes"))?;

		Ok(data)
	}

	/// Reads a pointer, returning a reader targetting the pointed location.
	pub fn read_pointer(&mut self) -> MayRevert<Self> {
		let offset: usize = self
			.read::<U256>()
			.map_err(|_| RevertReason::read_out_of_bounds("pointer"))?
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("pointer"))?;

		if offset >= self.input.len() {
			return Err(RevertReason::PointerToOutofBound.into());
		}

		Ok(Self {
			input: &self.input[offset..],
			cursor: 0,
		})
	}

	/// Read remaining bytes
	pub fn read_till_end(&mut self) -> MayRevert<&[u8]> {
		let range = self.move_cursor(self.input.len() - self.cursor)?;

		let data = self
			.input
			.get(range)
			.ok_or_else(|| RevertReason::read_out_of_bounds("raw bytes"))?;

		Ok(data)
	}

	/// Move the reading cursor with provided length, and return a range from the previous cursor
	/// location to the new one.
	/// Checks cursor overflows.
	fn move_cursor(&mut self, len: usize) -> MayRevert<Range<usize>> {
		let start = self.cursor;
		let end = self
			.cursor
			.checked_add(len)
			.ok_or_else(|| RevertReason::CursorOverflow)?;

		self.cursor = end;

		Ok(start..end)
	}
}

/// Help build an EVM input/output data.
///
/// Functions takes `self` to allow chaining all calls like
/// `EvmDataWriter::new().write(...).write(...).build()`.
/// While it could be more ergonomic to take &mut self, this would
/// prevent to have a `build` function that don't clone the output.
#[derive(Clone, Debug)]
pub struct EvmDataWriter {
	pub(crate) data: Vec<u8>,
	offset_data: Vec<OffsetDatum>,
	selector: Option<u32>,
}

#[derive(Clone, Debug)]
struct OffsetDatum {
	// Offset location in the container data.
	offset_position: usize,
	// Data pointed by the offset that must be inserted at the end of container data.
	data: Vec<u8>,
	// Inside of arrays, the offset is not from the start of array data (length), but from the start
	// of the item. This shift allow to correct this.
	offset_shift: usize,
}

impl EvmDataWriter {
	/// Creates a new empty output builder (without selector).
	pub fn new() -> Self {
		Self {
			data: vec![],
			offset_data: vec![],
			selector: None,
		}
	}

	/// Creates a new empty output builder with provided selector.
	/// Selector will only be appended before the data when calling
	/// `build` to not mess with the offsets.
	pub fn new_with_selector(selector: impl Into<u32>) -> Self {
		Self {
			data: vec![],
			offset_data: vec![],
			selector: Some(selector.into()),
		}
	}

	/// Return the built data.
	pub fn build(mut self) -> Vec<u8> {
		Self::bake_offsets(&mut self.data, self.offset_data);

		if let Some(selector) = self.selector {
			let mut output = selector.to_be_bytes().to_vec();
			output.append(&mut self.data);
			output
		} else {
			self.data
		}
	}

	/// Add offseted data at the end of this writer's data, updating the offsets.
	fn bake_offsets(output: &mut Vec<u8>, offsets: Vec<OffsetDatum>) {
		for mut offset_datum in offsets {
			let offset_position = offset_datum.offset_position;
			let offset_position_end = offset_position + 32;

			// The offset is the distance between the start of the data and the
			// start of the pointed data (start of a struct, length of an array).
			// Offsets in inner data are relative to the start of their respective "container".
			// However in arrays the "container" is actually the item itself instead of the whole
			// array, which is corrected by `offset_shift`.
			let free_space_offset = output.len() - offset_datum.offset_shift;

			// Override dummy offset to the offset it will be in the final output.
			U256::from(free_space_offset)
				.to_big_endian(&mut output[offset_position..offset_position_end]);

			// Append this data at the end of the current output.
			output.append(&mut offset_datum.data);
		}
	}

	/// Write arbitrary bytes.
	/// Doesn't handle any alignement checks, prefer using `write` instead if possible.
	fn write_raw_bytes(mut self, value: &[u8]) -> Self {
		self.data.extend_from_slice(value);
		self
	}

	/// Write data of requested type.
	pub fn write<T: EvmData>(mut self, value: T) -> Self {
		T::write(&mut self, value);
		self
	}

	/// Writes a pointer to given data.
	/// The data will be appended when calling `build`.
	/// Initially write a dummy value as offset in this writer's data, which will be replaced by
	/// the correct offset once the pointed data is appended.
	///
	/// Takes `&mut self` since its goal is to be used inside `EvmData` impl and not in chains.
	pub fn write_pointer(&mut self, data: Vec<u8>) {
		let offset_position = self.data.len();
		H256::write(self, H256::repeat_byte(0xff));

		self.offset_data.push(OffsetDatum {
			offset_position,
			data,
			offset_shift: 0,
		});
	}
}

impl Default for EvmDataWriter {
	fn default() -> Self {
		Self::new()
	}
}

/// Adapter to parse data as a first type then convert it to another one.
/// Useful for old precompiles in which Solidity arguments where set larger than
/// the needed Rust type.
#[derive(Clone, Copy, Debug)]
pub struct SolidityConvert<P, C> {
	inner: C,
	_phantom: PhantomData<P>,
}

impl<P, C> From<C> for SolidityConvert<P, C> {
	fn from(value: C) -> Self {
		Self {
			inner: value,
			_phantom: PhantomData,
		}
	}
}

impl<P, C> SolidityConvert<P, C> {
	pub fn converted(self) -> C {
		self.inner
	}
}

impl<P, C> EvmData for SolidityConvert<P, C>
where
	P: EvmData + TryInto<C>,
	C: EvmData + Into<P>,
{
	fn read(reader: &mut EvmDataReader) -> MayRevert<Self> {
		let c = P::read(reader)?
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large(C::solidity_type()))?;

		Ok(Self {
			inner: c,
			_phantom: PhantomData,
		})
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		P::write(writer, value.inner.into())
	}

	fn has_static_size() -> bool {
		P::has_static_size()
	}

	fn solidity_type() -> String {
		P::solidity_type()
	}
}

/// Wrapper around values being returned by functions.
/// Handle special case with tuple encoding.
pub fn encode_as_function_return_value<T: EvmData>(value: T) -> Vec<u8> {
	let output = EvmDataWriter::new().write(value).build();
	if T::is_explicit_tuple() && !T::has_static_size() {
		output[32..].to_vec()
	} else {
		output
	}
}
