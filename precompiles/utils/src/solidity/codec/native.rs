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

use super::*;
use crate::solidity::revert::InjectBacktrace;
use impl_trait_for_tuples::impl_for_tuples;
use sp_core::{ConstU32, Get, H160};

impl Codec for () {
	fn read(_reader: &mut Reader) -> MayRevert<Self> {
		Ok(())
	}

	fn write(_writer: &mut Writer, _value: Self) {}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		String::from("()")
	}
}

#[impl_for_tuples(1, 18)]
impl Codec for Tuple {
	fn has_static_size() -> bool {
		for_tuples!(#( Tuple::has_static_size() )&*)
	}

	fn read(reader: &mut Reader) -> MayRevert<Self> {
		if Self::has_static_size() {
			let mut index = 0;
			Ok(for_tuples!( ( #( {
				let elem = reader.read::<Tuple>().in_tuple(index)?;
				index +=1;
				elem
			} ),* ) ))
		} else {
			let reader = &mut reader.read_pointer()?;
			let mut index = 0;
			Ok(for_tuples!( ( #( {
				let elem = reader.read::<Tuple>().in_tuple(index)?;
				index +=1;
				elem
			} ),* ) ))
		}
	}

	fn write(writer: &mut Writer, value: Self) {
		if Self::has_static_size() {
			for_tuples!( #( Tuple::write(writer, value.Tuple); )* );
		} else {
			let mut inner_writer = Writer::new();
			for_tuples!( #( Tuple::write(&mut inner_writer, value.Tuple); )* );
			writer.write_pointer(inner_writer.build());
		}
	}

	fn signature() -> String {
		let mut subtypes = Vec::new();
		for_tuples!( #( subtypes.push(Tuple::signature()); )* );
		alloc::format!("({})", subtypes.join(","))
	}

	fn is_explicit_tuple() -> bool {
		true
	}
}

impl Codec for H256 {
	fn read(reader: &mut Reader) -> MayRevert<Self> {
		let range = reader.move_cursor(32)?;

		let data = reader
			.input
			.get(range)
			.ok_or_else(|| RevertReason::read_out_of_bounds("bytes32"))?;

		Ok(H256::from_slice(data))
	}

	fn write(writer: &mut Writer, value: Self) {
		writer.data.extend_from_slice(value.as_bytes());
	}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		String::from("bytes32")
	}
}

/// The `address` type of Solidity.
/// H160 could represent 2 types of data (bytes20 and address) that are not encoded the same way.
/// To avoid issues writing H160 is thus not supported.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

impl Address {
	pub fn as_u64(&self) -> Option<u64> {
		let _u64 = self.0.to_low_u64_be();
		if self.0 == H160::from_low_u64_be(_u64) {
			Some(_u64)
		} else {
			None
		}
	}
}

impl Codec for Address {
	fn read(reader: &mut Reader) -> MayRevert<Self> {
		let range = reader.move_cursor(32)?;

		let data = reader
			.input
			.get(range)
			.ok_or_else(|| RevertReason::read_out_of_bounds("address"))?;

		Ok(H160::from_slice(&data[12..32]).into())
	}

	fn write(writer: &mut Writer, value: Self) {
		H256::write(writer, value.0.into());
	}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		String::from("address")
	}
}

impl Codec for U256 {
	fn read(reader: &mut Reader) -> MayRevert<Self> {
		let range = reader.move_cursor(32)?;

		let data = reader
			.input
			.get(range)
			.ok_or_else(|| RevertReason::read_out_of_bounds("uint256"))?;

		Ok(U256::from_big_endian(data))
	}

	fn write(writer: &mut Writer, value: Self) {
		let mut buffer = [0u8; 32];
		value.to_big_endian(&mut buffer);
		writer.data.extend_from_slice(&buffer);
	}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		String::from("uint256")
	}
}

macro_rules! impl_evmdata_for_uints {
	($($uint:ty, )*) => {
		$(
			impl Codec for $uint {
				fn read(reader: &mut Reader) -> MayRevert<Self> {
					let value256: U256 = reader.read()
					.map_err(|_| RevertReason::read_out_of_bounds(
						Self::signature()
					))?;

					value256
						.try_into()
						.map_err(|_| RevertReason::value_is_too_large(
							Self::signature()
						).into())
				}

				fn write(writer: &mut Writer, value: Self) {
					U256::write(writer, value.into());
				}

				fn has_static_size() -> bool {
					true
				}

				fn signature() -> String {
					alloc::format!("uint{}", core::mem::size_of::<Self>() * 8)
				}
			}
		)*
	};
}

impl_evmdata_for_uints!(u8, u16, u32, u64, u128,);

impl Codec for bool {
	fn read(reader: &mut Reader) -> MayRevert<Self> {
		let h256 = H256::read(reader).map_err(|_| RevertReason::read_out_of_bounds("bool"))?;

		Ok(!h256.is_zero())
	}

	fn write(writer: &mut Writer, value: Self) {
		let mut buffer = [0u8; 32];
		if value {
			buffer[31] = 1;
		}

		writer.data.extend_from_slice(&buffer);
	}

	fn has_static_size() -> bool {
		true
	}

	fn signature() -> String {
		String::from("bool")
	}
}

type ConstU32Max = ConstU32<{ u32::MAX }>;

impl<T: Codec> Codec for Vec<T> {
	fn read(reader: &mut Reader) -> MayRevert<Self> {
		BoundedVec::<T, ConstU32Max>::read(reader).map(|x| x.into())
	}

	fn write(writer: &mut Writer, value: Self) {
		BoundedVec::<T, ConstU32Max>::write(
			writer,
			BoundedVec {
				inner: value,
				_phantom: PhantomData,
			},
		)
	}

	fn has_static_size() -> bool {
		false
	}

	fn signature() -> String {
		alloc::format!("{}[]", T::signature())
	}
}

/// Wrapper around a Vec that provides a max length bound on read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedVec<T, S> {
	inner: Vec<T>,
	_phantom: PhantomData<S>,
}

impl<T: Codec, S: Get<u32>> Codec for BoundedVec<T, S> {
	fn read(reader: &mut Reader) -> MayRevert<Self> {
		let mut inner_reader = reader.read_pointer()?;

		let array_size: usize = inner_reader
			.read::<U256>()
			.map_err(|_| RevertReason::read_out_of_bounds("length"))?
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("length"))?;

		if array_size > S::get() as usize {
			return Err(RevertReason::value_is_too_large("length").into());
		}

		let mut array = vec![];

		let mut item_reader = Reader {
			input: inner_reader
				.input
				.get(32..)
				.ok_or_else(|| RevertReason::read_out_of_bounds("array content"))?,
			cursor: 0,
		};

		for i in 0..array_size {
			array.push(item_reader.read().in_array(i)?);
		}

		Ok(BoundedVec {
			inner: array,
			_phantom: PhantomData,
		})
	}

	fn write(writer: &mut Writer, value: Self) {
		let value: Vec<_> = value.into();
		let mut inner_writer = Writer::new().write(U256::from(value.len()));

		for inner in value {
			// Any offset in items are relative to the start of the item instead of the
			// start of the array. However if there is offseted data it must but appended after
			// all items (offsets) are written. We thus need to rely on `compute_offsets` to do
			// that, and must store a "shift" to correct the offsets.
			let shift = inner_writer.data.len();
			let item_writer = Writer::new().write(inner);

			inner_writer = inner_writer.write_raw_bytes(&item_writer.data);
			for mut offset_datum in item_writer.offset_data {
				offset_datum.offset_shift += 32;
				offset_datum.offset_position += shift;
				inner_writer.offset_data.push(offset_datum);
			}
		}

		writer.write_pointer(inner_writer.build());
	}

	fn has_static_size() -> bool {
		false
	}

	fn signature() -> String {
		alloc::format!("{}[]", T::signature())
	}
}

impl<T, S> From<Vec<T>> for BoundedVec<T, S> {
	fn from(value: Vec<T>) -> Self {
		BoundedVec {
			inner: value,
			_phantom: PhantomData,
		}
	}
}

impl<T: Clone, S> From<&[T]> for BoundedVec<T, S> {
	fn from(value: &[T]) -> Self {
		BoundedVec {
			inner: value.to_vec(),
			_phantom: PhantomData,
		}
	}
}

impl<T: Clone, S, const N: usize> From<[T; N]> for BoundedVec<T, S> {
	fn from(value: [T; N]) -> Self {
		BoundedVec {
			inner: value.to_vec(),
			_phantom: PhantomData,
		}
	}
}

impl<T, S> From<BoundedVec<T, S>> for Vec<T> {
	fn from(value: BoundedVec<T, S>) -> Self {
		value.inner
	}
}
