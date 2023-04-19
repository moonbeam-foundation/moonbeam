// Copyright 2019-2023 PureStake Inc.
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

//! Solidity encoding following the
//! [Contract ABI Specification](https://docs.soliditylang.org/en/v0.8.19/abi-spec.html#abi)

pub mod bytes;
pub mod native;

#[cfg(any(feature = "codec-xcm", test))]
pub mod xcm;

use crate::solidity::revert::{MayRevert, RevertReason};
use core::{marker::PhantomData, ops::Range};
use sp_core::{H256, U256};
use sp_std::{convert::TryInto, vec, vec::Vec};

pub use alloc::string::String;
pub use bytes::{BoundedBytes, BoundedString, UnboundedBytes, UnboundedString};
pub use native::{Address, BoundedVec};

// derive macro
pub use precompile_utils_macro::Codec;

/// Data that can be encoded/encoded followiong the Solidity ABI Specification.
pub trait Codec: Sized {
	fn read(reader: &mut Reader) -> MayRevert<Self>;
	fn write(writer: &mut Writer, value: Self);
	fn has_static_size() -> bool;
	fn signature() -> String;
	fn is_explicit_tuple() -> bool {
		false
	}
}

/// Encode the value into its Solidity ABI format.
/// If `T` is a tuple it is encoded as a Solidity tuple with dynamic-size offset.
fn encode<T: Codec>(value: T) -> Vec<u8> {
	Writer::new().write(value).build()
}

/// Encode the value into its Solidity ABI format.
/// If `T` is a tuple every element is encoded without a prefixed offset.
/// It matches the encoding of Solidity function arguments and return value, or event data.
pub fn encode_arguments<T: Codec>(value: T) -> Vec<u8> {
	let output = encode(value);
	if T::is_explicit_tuple() && !T::has_static_size() {
		output[32..].to_vec()
	} else {
		output
	}
}

pub use self::encode_arguments as encode_return_value;
pub use self::encode_arguments as encode_event_data;

/// Encode the value as the arguments of a Solidity function with given selector.
/// If `T` is a tuple each member represents an argument of the function.
pub fn encode_with_selector<T: Codec>(selector: u32, value: T) -> Vec<u8> {
	Writer::new_with_selector(selector)
		.write_raw_bytes(&encode_arguments(value))
		.build()
}

/// Decode the value from its Solidity ABI format.
/// If `T` is a tuple it is decoded as a Solidity tuple with dynamic-size offset.
fn decode<T: Codec>(input: &[u8]) -> MayRevert<T> {
	Reader::new(input).read()
}

/// Decode the value from its Solidity ABI format.
/// If `T` is a tuple every element is decoded without a prefixed offset.
/// It matches the encoding of Solidity function arguments and return value, or event data.
pub fn decode_arguments<T: Codec>(input: &[u8]) -> MayRevert<T> {
	if T::is_explicit_tuple() && !T::has_static_size() {
		let writer = Writer::new();
		let mut writer = writer.write(U256::from(32));
		writer.write_pointer(input.to_vec());
		let input = writer.build();
		decode(&input)
	} else {
		decode(&input)
	}
}

pub use self::decode_arguments as decode_return_value;
pub use self::decode_arguments as decode_event_data;

/// Extracts the selector from the start of the input, or returns `None` if the input is too short.
pub fn selector(input: &[u8]) -> Option<u32> {
	input.get(0..4).map(|s| {
		let mut buffer = [0u8; 4];
		buffer.copy_from_slice(s);
		u32::from_be_bytes(buffer)
	})
}

/// Wrapper around an EVM input slice.
#[derive(Clone, Copy, Debug)]
pub struct Reader<'inner> {
	input: &'inner [u8],
	cursor: usize,
}

impl<'inner> Reader<'inner> {
	/// Create a Reader.
	pub fn new(input: &'inner [u8]) -> Self {
		Self { input, cursor: 0 }
	}

	/// Create a Reader while skipping an initial selector.
	pub fn new_skip_selector(input: &'inner [u8]) -> MayRevert<Self> {
		if input.len() < 4 {
			return Err(RevertReason::read_out_of_bounds("selector").into());
		}

		Ok(Self::new(&input[4..]))
	}

	/// Check the input has at least the correct amount of arguments before the end (32 bytes values).
	pub fn expect_arguments(&self, args: usize) -> MayRevert {
		if self.input.len() >= self.cursor + args * 32 {
			Ok(())
		} else {
			Err(RevertReason::ExpectedAtLeastNArguments(args).into())
		}
	}

	/// Read data from the input.
	pub fn read<T: Codec>(&mut self) -> MayRevert<T> {
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
/// `Writer::new().write(...).write(...).build()`.
/// While it could be more ergonomic to take &mut self, this would
/// prevent to have a `build` function that don't clone the output.
#[derive(Clone, Debug)]
pub struct Writer {
	pub(crate) data: Vec<u8>,
	offset_data: Vec<OffsetChunk>,
	selector: Option<u32>,
}

#[derive(Clone, Debug)]
struct OffsetChunk {
	// Offset location in the container data.
	offset_position: usize,
	// Data pointed by the offset that must be inserted at the end of container data.
	data: Vec<u8>,
	// Inside of arrays, the offset is not from the start of array data (length), but from the start
	// of the item. This shift allow to correct this.
	offset_shift: usize,
}

impl Writer {
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

	// Return the built data.
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
	fn bake_offsets(output: &mut Vec<u8>, offsets: Vec<OffsetChunk>) {
		for mut offset_chunk in offsets {
			let offset_position = offset_chunk.offset_position;
			let offset_position_end = offset_position + 32;

			// The offset is the distance between the start of the data and the
			// start of the pointed data (start of a struct, length of an array).
			// Offsets in inner data are relative to the start of their respective "container".
			// However in arrays the "container" is actually the item itself instead of the whole
			// array, which is corrected by `offset_shift`.
			let free_space_offset = output.len() - offset_chunk.offset_shift;

			// Override dummy offset to the offset it will be in the final output.
			U256::from(free_space_offset)
				.to_big_endian(&mut output[offset_position..offset_position_end]);

			// Append this data at the end of the current output.
			output.append(&mut offset_chunk.data);
		}
	}

	/// Write arbitrary bytes.
	/// Doesn't handle any alignement checks, prefer using `write` instead if possible.
	fn write_raw_bytes(mut self, value: &[u8]) -> Self {
		self.data.extend_from_slice(value);
		self
	}

	/// Write data of requested type.
	pub fn write<T: Codec>(mut self, value: T) -> Self {
		T::write(&mut self, value);
		self
	}

	/// Writes a pointer to given data.
	/// The data will be appended when calling `build`.
	/// Initially write a dummy value as offset in this writer's data, which will be replaced by
	/// the correct offset once the pointed data is appended.
	///
	/// Takes `&mut self` since its goal is to be used inside `solidity::Codec` impl and not in chains.
	pub fn write_pointer(&mut self, data: Vec<u8>) {
		let offset_position = self.data.len();
		H256::write(self, H256::repeat_byte(0xff));

		self.offset_data.push(OffsetChunk {
			offset_position,
			data,
			offset_shift: 0,
		});
	}
}

/// Adapter to parse data as a first type then convert it to another one.
/// Useful for old precompiles in which Solidity arguments where set larger than
/// the needed Rust type.
#[derive(Clone, Copy, Debug)]
pub struct Convert<P, C> {
	inner: C,
	_phantom: PhantomData<P>,
}

impl<P, C> From<C> for Convert<P, C> {
	fn from(value: C) -> Self {
		Self {
			inner: value,
			_phantom: PhantomData,
		}
	}
}

impl<P, C> Convert<P, C> {
	pub fn converted(self) -> C {
		self.inner
	}
}

impl<P, C> Codec for Convert<P, C>
where
	P: Codec + TryInto<C>,
	C: Codec + Into<P>,
{
	fn read(reader: &mut Reader) -> MayRevert<Self> {
		let c = P::read(reader)?
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large(C::signature()))?;

		Ok(Self {
			inner: c,
			_phantom: PhantomData,
		})
	}

	fn write(writer: &mut Writer, value: Self) {
		P::write(writer, value.inner.into())
	}

	fn has_static_size() -> bool {
		P::has_static_size()
	}

	fn signature() -> String {
		P::signature()
	}
}
