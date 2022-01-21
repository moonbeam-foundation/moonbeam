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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::Get, BoundedVec};
use precompile_utils::{Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer};
use sp_core::U256;
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
};

/// Allow to read/write non Solidity compliant encoding from/to EVM data.
/// Separate trait from `EvmData` to prevent reading/writing from Solidity compliant source/targets.
///
/// Use `Wrapped` to wrap it into a `bytes` to be Solidity compliant. It can also be used
/// in other non-Solidity compliant structures.
pub trait RawEvmData: Sized {
	fn read(reader: &mut EvmDataReader, gasometer: &mut Gasometer) -> EvmResult<Self>;
	fn write(writer: &mut EvmDataWriter, value: Self);
}

/// Allow to wrap non Solidity compliant encodings into a `bytes`.
pub struct Wrapped<T>(pub T);

impl<T: RawEvmData> EvmData for Wrapped<T> {
	fn read(reader: &mut EvmDataReader, gasometer: &mut Gasometer) -> EvmResult<Self> {
		let inner: Bytes = reader.read(gasometer)?;

		let mut inner_reader = EvmDataReader::new(&inner.as_bytes());
		let data = T::read(&mut inner_reader, gasometer)?;

		Ok(Wrapped(data))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let mut inner_writer = EvmDataWriter::new();
		T::write(&mut inner_writer, value.0);

		let raw_data = Bytes(inner_writer.build());

		Bytes::write(writer, raw_data);
	}
}

/// Allow to encode/decode raw `pallet_identity::Data` Solidity encoding.
/// It doesn't follow the general Solidity encoding, and is thus a building block
/// of more complex data structures: `IdentityInfo`, which contains many `IdentityData`
/// in a packed format, or `WrappedIdentityData` which wraps this data in a Solidity `bytes`
/// to make it usable from Solidity.
pub struct IdentityData(pub pallet_identity::Data);

impl RawEvmData for IdentityData {
	fn read(reader: &mut EvmDataReader, gasometer: &mut Gasometer) -> EvmResult<Self> {
		use pallet_identity::Data;

		let discriminant = reader.read_raw_bytes(gasometer, 1).map_err(|_| {
			gasometer.revert("tried to read IdentityData discriminant out of bounds")
		})?[0];

		match discriminant {
			0xff => Ok(Self(Data::None)),
			x @ 0..=32 => {
				let data = reader
					.read_raw_bytes(gasometer, x as usize)
					.map_err(|_| {
						gasometer.revert("tried to read IdentityData::Raw data out of bounds")
					})?
					.to_vec();

				let vec = BoundedVec::try_from(data).expect("size can't exceed 32");

				Ok(Self(Data::Raw(vec)))
			}
			x @ 0xfb..=0xfe => {
				let data = reader
					.read_raw_bytes(gasometer, 32)
					.map_err(|_| {
						gasometer.revert("tried to read IdentityData hash data out of bounds")
					})?
					.to_vec();

				let data: [u8; 32] = data.try_into().expect("size is exactly 32 bytes");

				match x {
					0xfe => Ok(Self(Data::BlakeTwo256(data))),
					0xfd => Ok(Self(Data::Sha256(data))),
					0xfc => Ok(Self(Data::Keccak256(data))),
					0xfb => Ok(Self(Data::ShaThree256(data))),
					_ => unreachable!("x can only be in range 0xfb..=0xfe"),
				}
			}
			x => Err(gasometer.revert(format!("unknown IdentityData discriminant {}", x))),
		}
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		use pallet_identity::Data;

		match value.0 {
			Data::None => {
				writer.write_raw_bytes(&[0xff]);
			}
			Data::Raw(vec) => {
				writer.write_raw_bytes(&[vec.len() as u8]);
				writer.write_raw_bytes(&vec);
			}
			Data::BlakeTwo256(data) => {
				writer.write_raw_bytes(&[0xfe]);
				writer.write_raw_bytes(&data);
			}
			Data::Sha256(data) => {
				writer.write_raw_bytes(&[0xfd]);
				writer.write_raw_bytes(&data);
			}
			Data::Keccak256(data) => {
				writer.write_raw_bytes(&[0xfc]);
				writer.write_raw_bytes(&data);
			}
			Data::ShaThree256(data) => {
				writer.write_raw_bytes(&[0xfb]);
				writer.write_raw_bytes(&data);
			}
		}
	}
}

pub struct IdentityInfo<FieldLimit: Get<u32>>(pub pallet_identity::IdentityInfo<FieldLimit>);

impl<FieldLimit: Get<u32>> RawEvmData for IdentityInfo<FieldLimit> {
	fn read(reader: &mut EvmDataReader, gasometer: &mut Gasometer) -> EvmResult<Self> {
		let array_size = U256::read(reader, gasometer)?;
		let array_size =
			u32::try_from(array_size).map_err(|_| gasometer.revert("array size is too big"))?;

		if array_size > FieldLimit::get() {
			return Err(gasometer.revert(format!(
				"array size is bigger than allowed ({})",
				FieldLimit::get()
			)));
		}

		let mut additional = BoundedVec::default();
		for _ in 0..array_size {
			let item0 = IdentityData::read(reader, gasometer)?.0;
			let item1 = IdentityData::read(reader, gasometer)?.0;
			additional
				.try_push((item0, item1))
				.expect("size is below FieldLimit");
		}

		let display = IdentityData::read(reader, gasometer)?.0;
		let legal = IdentityData::read(reader, gasometer)?.0;
		let web = IdentityData::read(reader, gasometer)?.0;
		let riot = IdentityData::read(reader, gasometer)?.0;
		let email = IdentityData::read(reader, gasometer)?.0;

		let some_pgp: u8 = reader.read(gasometer)?;

		let pgp_fingerprint = match some_pgp {
			0x00 => None,
			0x01 => {
				let data = reader.read_raw_bytes(gasometer, 20)?;
				let data: [u8; 20] = data.try_into().expect("size is exactly 20 bytes");
				Some(data)
			}
			_ => return Err(gasometer.revert("unknown 'pgp_fingerprint' discriminant")),
		};

		let image = IdentityData::read(reader, gasometer)?.0;
		let twitter = IdentityData::read(reader, gasometer)?.0;

		Ok(IdentityInfo(pallet_identity::IdentityInfo::<FieldLimit> {
			additional,
			display,
			legal,
			web,
			riot,
			email,
			pgp_fingerprint,
			image,
			twitter,
		}))
	}
	fn write(writer: &mut EvmDataWriter, value: Self) {
		U256::write(writer, value.0.additional.len().into());

		for item in value.0.additional {
			IdentityData::write(writer, IdentityData(item.0));
			IdentityData::write(writer, IdentityData(item.1));
		}

		IdentityData::write(writer, IdentityData(value.0.display));
		IdentityData::write(writer, IdentityData(value.0.legal));
		IdentityData::write(writer, IdentityData(value.0.web));
		IdentityData::write(writer, IdentityData(value.0.riot));
		IdentityData::write(writer, IdentityData(value.0.email));

		match value.0.pgp_fingerprint {
			None => writer.write_raw_bytes(&[0x00]),
			Some(bytes) => {
				writer.write_raw_bytes(&[0x01]);
				writer.write_raw_bytes(&bytes);
			}
		}

		IdentityData::write(writer, IdentityData(value.0.image));
		IdentityData::write(writer, IdentityData(value.0.twitter));
	}
}

pub struct Judgement<B>(pallet_identity::Judgement<B>)
where
	B: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq;

impl<B> EvmData for Judgement<B>
where
	B: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq,
	B: Into<U256> + TryFrom<U256>,
{
	fn read(reader: &mut EvmDataReader, gasometer: &mut Gasometer) -> EvmResult<Self> {
		use pallet_identity::Judgement as Inner;

		// We expect the balances amount to not be as large as starting with 0xff when encoded to
		// U256. Moonbeam balance type is u128 so it is not an issue, and anyway would likely
		// represent amounts too large to even exist.
		let data = reader.read_raw_bytes(gasometer, 32)?;

		Ok(Judgement(match data {
			[0xff, 0, ..] => Inner::Unknown,
			[0xff, 1, ..] => Inner::Reasonable,
			[0xff, 2, ..] => Inner::KnownGood,
			[0xff, 3, ..] => Inner::OutOfDate,
			[0xff, 4, ..] => Inner::LowQuality,
			[0xff, 5, ..] => Inner::Erroneous,
			[0xff, ..] => return Err(gasometer.revert("unknown Judgement variant")),
			[..] => {
				let amount = U256::from_little_endian(data);
				let amount: B = amount
					.try_into()
					.map_err(|_| gasometer.revert("amount too large"))?;
				Inner::FeePaid(amount)
			}
		}))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		use pallet_identity::Judgement;

		let encode = |byte| {
			let mut bytes = [0u8; 32];
			bytes[0] = 0xff;
			bytes[1] = byte;
			U256::from_big_endian(&bytes)
		};

		let value = match value.0 {
			Judgement::FeePaid(balance) => balance.into(),
			Judgement::Unknown => encode(0),
			Judgement::Reasonable => encode(1),
			Judgement::KnownGood => encode(2),
			Judgement::OutOfDate => encode(3),
			Judgement::LowQuality => encode(4),
			Judgement::Erroneous => encode(5),
		};

		U256::write(writer, value);
	}
}
