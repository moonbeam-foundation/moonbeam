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

use precompile_utils::solidity::codec::{Address, Codec, Reader, Writer};
use sp_core::H160;

#[derive(Debug, Clone, PartialEq, Eq, Codec)]
struct StaticSize {
	id: u32,
	address: Address,
}

#[derive(Debug, Clone, PartialEq, Eq, Codec)]
struct DynamicSize<T> {
	id: u32,
	array: Vec<T>,
}

fn main() {
	// static
	let static_size = StaticSize {
		id: 5,
		address: H160::repeat_byte(0x42).into(),
	};

	assert!(StaticSize::has_static_size());
	assert_eq!(&StaticSize::signature(), "(uint32,address)");

	let bytes = Writer::new().write(static_size.clone()).build();
	assert_eq!(
		bytes,
		Writer::new()
			.write(5u32)
			.write(Address::from(H160::repeat_byte(0x42)))
			.build()
	);

	let mut reader = Reader::new(&bytes);
	let static_size_2: StaticSize = reader.read().expect("to decode properly");
	assert_eq!(static_size_2, static_size);

	// dynamic
	let dynamic_size = DynamicSize {
		id: 6,
		array: vec![10u32, 15u32],
	};
	assert!(!DynamicSize::<u32>::has_static_size());
	assert_eq!(DynamicSize::<u32>::signature(), "(uint32,uint32[])");

	let bytes = Writer::new().write(dynamic_size.clone()).build();
	assert_eq!(
		bytes,
		Writer::new()
			.write(0x20u32) // offset of struct
			.write(6u32) // id
			.write(0x40u32) // array offset
			.write(2u32) // array size
			.write(10u32) // array[0]
			.write(15u32) // array[1]
			.build()
	);

	let mut reader = Reader::new(&bytes);
	let dynamic_size_2: DynamicSize<u32> = reader.read().expect("to decode properly");
	assert_eq!(dynamic_size_2, dynamic_size);
}
