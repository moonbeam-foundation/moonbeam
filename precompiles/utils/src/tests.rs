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

use {
	crate::{
		prelude::*,
		solidity::{
			codec::{
				xcm::{network_id_from_bytes, network_id_to_bytes},
				Reader, Writer,
			},
			modifier::{check_function_modifier, FunctionModifier},
			revert::Backtrace,
		},
	},
	frame_support::traits::ConstU32,
	hex_literal::hex,
	pallet_evm::Context,
	sp_core::{H160, H256, U256},
	sp_std::convert::TryInto,
	xcm::latest::{Junction, Junctions, NetworkId},
};

fn u256_repeat_byte(byte: u8) -> U256 {
	let value = H256::repeat_byte(byte);

	U256::from_big_endian(value.as_bytes())
}

// When debugging it is useful to display data in chunks of 32 bytes.
#[allow(dead_code)]
fn display_bytes(bytes: &[u8]) {
	bytes
		.chunks_exact(32)
		.map(|chunk| H256::from_slice(chunk))
		.for_each(|hash| println!("{:?}", hash));
}

#[test]
fn write_bool() {
	let value = true;

	let writer_output = Writer::new().write(value).build();

	let mut expected_output = [0u8; 32];
	expected_output[31] = 1;

	assert_eq!(writer_output, expected_output);
}

#[test]
fn read_bool() {
	let value = true;

	let writer_output = Writer::new().write(value).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: bool = reader.read().expect("to correctly parse bool");

	assert_eq!(value, parsed);
}

#[test]
fn write_u64() {
	let value = 42u64;

	let writer_output = Writer::new().write(value).build();

	let mut expected_output = [0u8; 32];
	expected_output[24..].copy_from_slice(&value.to_be_bytes());

	assert_eq!(writer_output, expected_output);
}

#[test]
fn read_u64() {
	let value = 42u64;
	let writer_output = Writer::new().write(value).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: u64 = reader.read().expect("to correctly parse u64");

	assert_eq!(value, parsed);
}

#[test]
fn write_u128() {
	let value = 42u128;

	let writer_output = Writer::new().write(value).build();

	let mut expected_output = [0u8; 32];
	expected_output[16..].copy_from_slice(&value.to_be_bytes());

	assert_eq!(writer_output, expected_output);
}

#[test]
fn read_u128() {
	let value = 42u128;
	let writer_output = Writer::new().write(value).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: u128 = reader.read().expect("to correctly parse u128");

	assert_eq!(value, parsed);
}

#[test]
fn write_u256() {
	let value = U256::from(42);

	let writer_output = Writer::new().write(value).build();

	let mut expected_output = [0u8; 32];
	value.to_big_endian(&mut expected_output);

	assert_eq!(writer_output, expected_output);
}

#[test]
fn read_u256() {
	let value = U256::from(42);
	let writer_output = Writer::new().write(value).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: U256 = reader.read().expect("to correctly parse U256");

	assert_eq!(value, parsed);
}

#[test]
#[should_panic(expected = "to correctly parse U256")]
fn read_u256_too_short() {
	let value = U256::from(42);
	let writer_output = Writer::new().write(value).build();

	let mut reader = Reader::new(&writer_output[0..31]);
	let _: U256 = reader.read().expect("to correctly parse U256");
}

#[test]
fn write_h256() {
	let mut raw = [0u8; 32];
	raw[0] = 42;
	raw[12] = 43;
	raw[31] = 44;

	let value = H256::from(raw);

	let output = Writer::new().write(value).build();

	assert_eq!(&output, &raw);
}

#[test]
fn tmp() {
	let u = U256::from(1_000_000_000);
	println!("U256={:?}", u.0);
}

#[test]
fn read_h256() {
	let mut raw = [0u8; 32];
	raw[0] = 42;
	raw[12] = 43;
	raw[31] = 44;
	let value = H256::from(raw);
	let writer_output = Writer::new().write(value).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: H256 = reader.read().expect("to correctly parse H256");

	assert_eq!(value, parsed);
}

#[test]
#[should_panic(expected = "to correctly parse H256")]
fn read_h256_too_short() {
	let mut raw = [0u8; 32];
	raw[0] = 42;
	raw[12] = 43;
	raw[31] = 44;
	let value = H256::from(raw);
	let writer_output = Writer::new().write(value).build();

	let mut reader = Reader::new(&writer_output[0..31]);
	let _: H256 = reader.read().expect("to correctly parse H256");
}

#[test]
fn write_address() {
	let value = H160::repeat_byte(0xAA);

	let output = Writer::new().write(Address(value)).build();

	assert_eq!(output.len(), 32);
	assert_eq!(&output[12..32], value.as_bytes());
}

#[test]
fn read_address() {
	let value = H160::repeat_byte(0xAA);
	let writer_output = Writer::new().write(Address(value)).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Address = reader.read().expect("to correctly parse Address");

	assert_eq!(value, parsed.0);
}

#[test]
fn write_h256_array() {
	let array = vec![
		H256::repeat_byte(0x11),
		H256::repeat_byte(0x22),
		H256::repeat_byte(0x33),
		H256::repeat_byte(0x44),
		H256::repeat_byte(0x55),
	];
	let writer_output = Writer::new().write(array.clone()).build();
	assert_eq!(writer_output.len(), 0xE0);

	// We can read this "manualy" using simpler functions since arrays are 32-byte aligned.
	let mut reader = Reader::new(&writer_output);

	assert_eq!(reader.read::<U256>().expect("read offset"), 32.into());
	assert_eq!(reader.read::<U256>().expect("read size"), 5.into());
	assert_eq!(reader.read::<H256>().expect("read 1st"), array[0]);
	assert_eq!(reader.read::<H256>().expect("read 2nd"), array[1]);
	assert_eq!(reader.read::<H256>().expect("read 3rd"), array[2]);
	assert_eq!(reader.read::<H256>().expect("read 4th"), array[3]);
	assert_eq!(reader.read::<H256>().expect("read 5th"), array[4]);
}

#[test]
fn read_h256_array() {
	let array = vec![
		H256::repeat_byte(0x11),
		H256::repeat_byte(0x22),
		H256::repeat_byte(0x33),
		H256::repeat_byte(0x44),
		H256::repeat_byte(0x55),
	];
	let writer_output = Writer::new().write(array.clone()).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Vec<H256> = reader.read().expect("to correctly parse Vec<H256>");

	assert_eq!(array, parsed);
}

#[test]
fn write_u256_array() {
	let array = vec![
		u256_repeat_byte(0x11),
		u256_repeat_byte(0x22),
		u256_repeat_byte(0x33),
		u256_repeat_byte(0x44),
		u256_repeat_byte(0x55),
	];
	let writer_output = Writer::new().write(array.clone()).build();
	assert_eq!(writer_output.len(), 0xE0);

	// We can read this "manualy" using simpler functions since arrays are 32-byte aligned.
	let mut reader = Reader::new(&writer_output);

	assert_eq!(reader.read::<U256>().expect("read offset"), 32.into());
	assert_eq!(reader.read::<U256>().expect("read size"), 5.into());
	assert_eq!(reader.read::<U256>().expect("read 1st"), array[0]);
	assert_eq!(reader.read::<U256>().expect("read 2nd"), array[1]);
	assert_eq!(reader.read::<U256>().expect("read 3rd"), array[2]);
	assert_eq!(reader.read::<U256>().expect("read 4th"), array[3]);
	assert_eq!(reader.read::<U256>().expect("read 5th"), array[4]);
}

#[test]
fn read_u256_array() {
	let array = vec![
		u256_repeat_byte(0x11),
		u256_repeat_byte(0x22),
		u256_repeat_byte(0x33),
		u256_repeat_byte(0x44),
		u256_repeat_byte(0x55),
	];
	let writer_output = Writer::new().write(array.clone()).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Vec<U256> = reader.read().expect("to correctly parse Vec<H256>");

	assert_eq!(array, parsed);
}

#[test]
fn write_address_array() {
	let array = vec![
		Address(H160::repeat_byte(0x11)),
		Address(H160::repeat_byte(0x22)),
		Address(H160::repeat_byte(0x33)),
		Address(H160::repeat_byte(0x44)),
		Address(H160::repeat_byte(0x55)),
	];
	let writer_output = Writer::new().write(array.clone()).build();

	// We can read this "manualy" using simpler functions since arrays are 32-byte aligned.
	let mut reader = Reader::new(&writer_output);

	assert_eq!(reader.read::<U256>().expect("read offset"), 32.into());
	assert_eq!(reader.read::<U256>().expect("read size"), 5.into());
	assert_eq!(reader.read::<Address>().expect("read 1st"), array[0]);
	assert_eq!(reader.read::<Address>().expect("read 2nd"), array[1]);
	assert_eq!(reader.read::<Address>().expect("read 3rd"), array[2]);
	assert_eq!(reader.read::<Address>().expect("read 4th"), array[3]);
	assert_eq!(reader.read::<Address>().expect("read 5th"), array[4]);
}

#[test]
fn read_address_array() {
	let array = vec![
		Address(H160::repeat_byte(0x11)),
		Address(H160::repeat_byte(0x22)),
		Address(H160::repeat_byte(0x33)),
		Address(H160::repeat_byte(0x44)),
		Address(H160::repeat_byte(0x55)),
	];
	let writer_output = Writer::new().write(array.clone()).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Vec<Address> = reader.read().expect("to correctly parse Vec<H256>");

	assert_eq!(array, parsed);
}

#[test]
fn read_address_array_size_too_big() {
	let array = vec![
		Address(H160::repeat_byte(0x11)),
		Address(H160::repeat_byte(0x22)),
		Address(H160::repeat_byte(0x33)),
		Address(H160::repeat_byte(0x44)),
		Address(H160::repeat_byte(0x55)),
	];
	let mut writer_output = Writer::new().write(array).build();

	U256::from(6u32).to_big_endian(&mut writer_output[0x20..0x40]);

	let mut reader = Reader::new(&writer_output);

	match reader.read::<Vec<Address>>().in_field("field") {
		Ok(_) => panic!("should not parse correctly"),
		Err(err) => {
			assert_eq!(
				err.to_string(),
				"field[5]: Tried to read address out of bounds"
			)
		}
	}
}

#[test]
fn write_address_nested_array() {
	let array = vec![
		vec![
			Address(H160::repeat_byte(0x11)),
			Address(H160::repeat_byte(0x22)),
			Address(H160::repeat_byte(0x33)),
		],
		vec![
			Address(H160::repeat_byte(0x44)),
			Address(H160::repeat_byte(0x55)),
		],
	];
	let writer_output = Writer::new().write(array.clone()).build();
	assert_eq!(writer_output.len(), 0x160);

	// We can read this "manualy" using simpler functions since arrays are 32-byte aligned.
	let mut reader = Reader::new(&writer_output);

	assert_eq!(reader.read::<U256>().expect("read offset"), 0x20.into()); // 0x00
	assert_eq!(reader.read::<U256>().expect("read size"), 2.into()); // 0x20
	assert_eq!(reader.read::<U256>().expect("read 1st offset"), 0x40.into()); // 0x40
	assert_eq!(reader.read::<U256>().expect("read 2st offset"), 0xc0.into()); // 0x60
	assert_eq!(reader.read::<U256>().expect("read 1st size"), 3.into()); // 0x80
	assert_eq!(reader.read::<Address>().expect("read 1-1"), array[0][0]); // 0xA0
	assert_eq!(reader.read::<Address>().expect("read 1-2"), array[0][1]); // 0xC0
	assert_eq!(reader.read::<Address>().expect("read 1-3"), array[0][2]); // 0xE0
	assert_eq!(reader.read::<U256>().expect("read 2nd size"), 2.into()); // 0x100
	assert_eq!(reader.read::<Address>().expect("read 2-1"), array[1][0]); // 0x120
	assert_eq!(reader.read::<Address>().expect("read 2-2"), array[1][1]); // 0x140
}

#[test]
fn read_address_nested_array() {
	let array = vec![
		vec![
			Address(H160::repeat_byte(0x11)),
			Address(H160::repeat_byte(0x22)),
			Address(H160::repeat_byte(0x33)),
		],
		vec![
			Address(H160::repeat_byte(0x44)),
			Address(H160::repeat_byte(0x55)),
		],
	];
	let writer_output = Writer::new().write(array.clone()).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Vec<Vec<Address>> = reader.read().expect("to correctly parse Vec<Vec<Address>>");

	assert_eq!(array, parsed);
}

#[test]

fn write_multiple_arrays() {
	let array1 = vec![
		Address(H160::repeat_byte(0x11)),
		Address(H160::repeat_byte(0x22)),
		Address(H160::repeat_byte(0x33)),
	];

	let array2 = vec![H256::repeat_byte(0x44), H256::repeat_byte(0x55)];

	let writer_output = Writer::new()
		.write(array1.clone())
		.write(array2.clone())
		.build();

	assert_eq!(writer_output.len(), 0x120);

	// We can read this "manualy" using simpler functions since arrays are 32-byte aligned.
	let mut reader = Reader::new(&writer_output);

	assert_eq!(reader.read::<U256>().expect("read 1st offset"), 0x40.into()); // 0x00
	assert_eq!(reader.read::<U256>().expect("read 2nd offset"), 0xc0.into()); // 0x20
	assert_eq!(reader.read::<U256>().expect("read 1st size"), 3.into()); // 0x40
	assert_eq!(reader.read::<Address>().expect("read 1-1"), array1[0]); // 0x60
	assert_eq!(reader.read::<Address>().expect("read 1-2"), array1[1]); // 0x80
	assert_eq!(reader.read::<Address>().expect("read 1-3"), array1[2]); // 0xA0
	assert_eq!(reader.read::<U256>().expect("read 2nd size"), 2.into()); // 0xC0
	assert_eq!(reader.read::<H256>().expect("read 2-1"), array2[0]); // 0xE0
	assert_eq!(reader.read::<H256>().expect("read 2-2"), array2[1]); // 0x100
}

#[test]
fn read_multiple_arrays() {
	let array1 = vec![
		Address(H160::repeat_byte(0x11)),
		Address(H160::repeat_byte(0x22)),
		Address(H160::repeat_byte(0x33)),
	];

	let array2 = vec![H256::repeat_byte(0x44), H256::repeat_byte(0x55)];

	let writer_output = Writer::new()
		.write(array1.clone())
		.write(array2.clone())
		.build();

	// offset 0x20
	// offset 0x40
	// size 0x60
	// 3 addresses 0xC0
	// size 0xE0
	// 2 H256 0x120
	assert_eq!(writer_output.len(), 0x120);

	let mut reader = Reader::new(&writer_output);

	let parsed: Vec<Address> = reader.read().expect("to correctly parse Vec<Address>");
	assert_eq!(array1, parsed);

	let parsed: Vec<H256> = reader.read().expect("to correctly parse Vec<H256>");
	assert_eq!(array2, parsed);
}

#[test]
fn read_bytes() {
	let data = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod\
	tempor incididunt ut labore et dolore magna aliqua.";
	let writer_output = Writer::new().write(UnboundedBytes::from(&data[..])).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: UnboundedBytes = reader.read().expect("to correctly parse Bytes");

	assert_eq!(data, parsed.as_bytes());
}

#[test]
fn write_bytes() {
	let data = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod\
	tempor incididunt ut labore et dolore magna aliqua.";

	let writer_output = Writer::new().write(UnboundedBytes::from(&data[..])).build();

	// We can read this "manualy" using simpler functions.
	let mut reader = Reader::new(&writer_output);

	// We pad data to a multiple of 32 bytes.
	let mut padded = data.to_vec();
	assert!(data.len() < 0x80);
	padded.resize(0x80, 0);

	assert_eq!(reader.read::<U256>().expect("read offset"), 32.into());
	assert_eq!(reader.read::<U256>().expect("read size"), data.len().into());
	let mut read = |e| reader.read::<H256>().expect(e); // shorthand
	assert_eq!(read("read part 1"), H256::from_slice(&padded[0x00..0x20]));
	assert_eq!(read("read part 2"), H256::from_slice(&padded[0x20..0x40]));
	assert_eq!(read("read part 3"), H256::from_slice(&padded[0x40..0x60]));
	assert_eq!(read("read part 4"), H256::from_slice(&padded[0x60..0x80]));
}

#[test]
fn read_string() {
	let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod\
	tempor incididunt ut labore et dolore magna aliqua.";
	let writer_output = Writer::new().write(UnboundedBytes::from(data)).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: UnboundedBytes = reader.read().expect("to correctly parse Bytes");

	assert_eq!(data, parsed.as_str().expect("valid utf8"));
}

#[test]
fn write_string() {
	let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod\
	tempor incididunt ut labore et dolore magna aliqua.";

	let writer_output = Writer::new().write(UnboundedBytes::from(data)).build();

	// We can read this "manualy" using simpler functions.
	let mut reader = Reader::new(&writer_output);

	// We pad data to next multiple of 32 bytes.
	let mut padded = data.as_bytes().to_vec();
	assert!(data.len() < 0x80);
	padded.resize(0x80, 0);

	assert_eq!(reader.read::<U256>().expect("read offset"), 32.into());
	assert_eq!(reader.read::<U256>().expect("read size"), data.len().into());
	let mut read = |e| reader.read::<H256>().expect(e); // shorthand
	assert_eq!(read("read part 1"), H256::from_slice(&padded[0x00..0x20]));
	assert_eq!(read("read part 2"), H256::from_slice(&padded[0x20..0x40]));
	assert_eq!(read("read part 3"), H256::from_slice(&padded[0x40..0x60]));
	assert_eq!(read("read part 4"), H256::from_slice(&padded[0x60..0x80]));
}

#[test]
fn write_vec_bytes() {
	let data = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod\
	tempor incididunt ut labore et dolore magna aliqua.";

	let writer_output = Writer::new()
		.write(vec![
			UnboundedBytes::from(&data[..]),
			UnboundedBytes::from(&data[..]),
		])
		.build();

	writer_output
		.chunks_exact(32)
		.map(|chunk| H256::from_slice(chunk))
		.for_each(|hash| println!("{:?}", hash));

	// We pad data to a multiple of 32 bytes.
	let mut padded = data.to_vec();
	assert!(data.len() < 0x80);
	padded.resize(0x80, 0);

	let mut reader = Reader::new(&writer_output);

	// Offset of vec
	assert_eq!(reader.read::<U256>().expect("read offset"), 32.into());

	// Length of vec
	assert_eq!(reader.read::<U256>().expect("read offset"), 2.into());

	// Relative offset of first bytgmes object
	assert_eq!(reader.read::<U256>().expect("read offset"), 0x40.into());
	// Relative offset of second bytes object
	assert_eq!(reader.read::<U256>().expect("read offset"), 0xe0.into());

	// Length of first bytes object
	assert_eq!(reader.read::<U256>().expect("read size"), data.len().into());

	// First byte objects data
	let mut read = |e| reader.read::<H256>().expect(e); // shorthand
	assert_eq!(read("read part 1"), H256::from_slice(&padded[0x00..0x20]));
	assert_eq!(read("read part 2"), H256::from_slice(&padded[0x20..0x40]));
	assert_eq!(read("read part 3"), H256::from_slice(&padded[0x40..0x60]));
	assert_eq!(read("read part 4"), H256::from_slice(&padded[0x60..0x80]));

	// Length of second bytes object
	assert_eq!(reader.read::<U256>().expect("read size"), data.len().into());

	// Second byte objects data
	let mut read = |e| reader.read::<H256>().expect(e); // shorthand
	assert_eq!(read("read part 1"), H256::from_slice(&padded[0x00..0x20]));
	assert_eq!(read("read part 2"), H256::from_slice(&padded[0x20..0x40]));
	assert_eq!(read("read part 3"), H256::from_slice(&padded[0x40..0x60]));
	assert_eq!(read("read part 4"), H256::from_slice(&padded[0x60..0x80]));
}

#[test]
fn read_vec_of_bytes() {
	let data = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod\
	tempor incididunt ut labore et dolore magna aliqua.";

	let writer_output = Writer::new()
		.write(vec![
			UnboundedBytes::from(&data[..]),
			UnboundedBytes::from(&data[..]),
		])
		.build();

	writer_output
		.chunks_exact(32)
		.map(|chunk| H256::from_slice(chunk))
		.for_each(|hash| println!("{:?}", hash));

	let mut reader = Reader::new(&writer_output);
	let parsed: Vec<UnboundedBytes> = reader.read().expect("to correctly parse Vec<u8>");

	assert_eq!(
		vec![
			UnboundedBytes::from(&data[..]),
			UnboundedBytes::from(&data[..])
		],
		parsed
	);
}

// The following test parses input data generated by web3 from a Solidity contract.
// This is important to test on external data since all the above tests can only test consistency
// between `Reader` and `Writer`.
//
// It also provides an example on how to impl `solidity::Codec` for Solidity structs.
//
// struct MultiLocation {
// 	   uint8 parents;
// 	   bytes [] interior;
// }
//
// function transfer(
//     address currency_address,
//     uint256 amount,
//     MultiLocation memory destination,
//     uint64 weight
// ) external;

#[derive(Clone, Debug, Eq, PartialEq, solidity::Codec)]
struct MultiLocation {
	parents: u8,
	interior: Vec<UnboundedBytes>,
}

#[test]
fn read_complex_solidity_function() {
	// Function call data generated by web3.
	// transfer_multiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)
	let data = hex!(
		"b38c60fa
		0000000000000000000000000000000000000000000000000000000000000080
		0000000000000000000000000000000000000000000000000000000000000064
		00000000000000000000000000000000000000000000000000000000000001a0
		0000000000000000000000000000000000000000000000000000000000000064
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000080
		0000000000000000000000000000000000000000000000000000000000000005
		00000003e8000000000000000000000000000000000000000000000000000000
		0000000000000000000000000000000000000000000000000000000000000002
		0403000000000000000000000000000000000000000000000000000000000000
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000022
		0101010101010101010101010101010101010101010101010101010101010101
		0100000000000000000000000000000000000000000000000000000000000000"
	);

	let selector = solidity::codec::selector(&data);
	let mut reader = Reader::new_skip_selector(&data).expect("to read selector");

	assert_eq!(selector, Some(0xb38c60fa));
	// asset
	assert_eq!(
		reader.read::<MultiLocation>().unwrap(),
		MultiLocation {
			parents: 1,
			interior: vec![
				UnboundedBytes::from(&hex!("00000003e8")[..]),
				UnboundedBytes::from(&hex!("0403")[..]),
			],
		}
	);

	// amount
	assert_eq!(reader.read::<U256>().unwrap(), 100u32.into());

	// destination
	assert_eq!(
		reader.read::<MultiLocation>().unwrap(),
		MultiLocation {
			parents: 1,
			interior: vec![UnboundedBytes::from(
				&hex!("01010101010101010101010101010101010101010101010101010101010101010100")[..]
			)],
		}
	);

	// weight
	assert_eq!(reader.read::<U256>().unwrap(), 100u32.into());
}

#[test]
fn junctions_decoder_works() {
	let writer_output = Writer::new()
		.write(Junctions::X1(Junction::OnlyChild))
		.build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Junctions = reader
		.read::<Junctions>()
		.expect("to correctly parse Junctions");

	assert_eq!(parsed, Junctions::X1(Junction::OnlyChild));

	let writer_output = Writer::new()
		.write(Junctions::X2(Junction::OnlyChild, Junction::OnlyChild))
		.build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Junctions = reader
		.read::<Junctions>()
		.expect("to correctly parse Junctions");

	assert_eq!(
		parsed,
		Junctions::X2(Junction::OnlyChild, Junction::OnlyChild)
	);

	let writer_output = Writer::new()
		.write(Junctions::X3(
			Junction::OnlyChild,
			Junction::OnlyChild,
			Junction::OnlyChild,
		))
		.build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Junctions = reader
		.read::<Junctions>()
		.expect("to correctly parse Junctions");

	assert_eq!(
		parsed,
		Junctions::X3(
			Junction::OnlyChild,
			Junction::OnlyChild,
			Junction::OnlyChild
		),
	);
}

#[test]
fn junction_decoder_works() {
	let writer_output = Writer::new().write(Junction::Parachain(0)).build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Junction = reader
		.read::<Junction>()
		.expect("to correctly parse Junctions");

	assert_eq!(parsed, Junction::Parachain(0));

	let writer_output = Writer::new()
		.write(Junction::AccountId32 {
			network: None,
			id: [1u8; 32],
		})
		.build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Junction = reader
		.read::<Junction>()
		.expect("to correctly parse Junctions");

	assert_eq!(
		parsed,
		Junction::AccountId32 {
			network: None,
			id: [1u8; 32],
		}
	);

	let writer_output = Writer::new()
		.write(Junction::AccountIndex64 {
			network: None,
			index: u64::from_be_bytes([1u8; 8]),
		})
		.build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Junction = reader
		.read::<Junction>()
		.expect("to correctly parse Junctions");

	assert_eq!(
		parsed,
		Junction::AccountIndex64 {
			network: None,
			index: u64::from_be_bytes([1u8; 8]),
		}
	);

	let writer_output = Writer::new()
		.write(Junction::AccountKey20 {
			network: None,
			key: H160::repeat_byte(0xAA).as_bytes().try_into().unwrap(),
		})
		.build();

	let mut reader = Reader::new(&writer_output);
	let parsed: Junction = reader
		.read::<Junction>()
		.expect("to correctly parse Junctions");

	assert_eq!(
		parsed,
		Junction::AccountKey20 {
			network: None,
			key: H160::repeat_byte(0xAA).as_bytes().try_into().unwrap(),
		}
	);
}

#[test]
fn network_id_decoder_works() {
	assert_eq!(network_id_from_bytes(network_id_to_bytes(None)), Ok(None));

	let mut name = [0u8; 32];
	name[0..6].copy_from_slice(b"myname");
	assert_eq!(
		network_id_from_bytes(network_id_to_bytes(Some(NetworkId::ByGenesis(name)))),
		Ok(Some(NetworkId::ByGenesis(name)))
	);

	assert_eq!(
		network_id_from_bytes(network_id_to_bytes(Some(NetworkId::Kusama))),
		Ok(Some(NetworkId::Kusama))
	);

	assert_eq!(
		network_id_from_bytes(network_id_to_bytes(Some(NetworkId::Polkadot))),
		Ok(Some(NetworkId::Polkadot))
	);
}

#[test]
fn test_check_function_modifier() {
	let context = |value: u32| Context {
		address: H160::zero(),
		caller: H160::zero(),
		apparent_value: U256::from(value),
	};

	let payable_error = || Revert::new(RevertReason::custom("Function is not payable"));
	let static_error = || {
		Revert::new(RevertReason::custom(
			"Can't call non-static function in static context",
		))
	};

	// Can't call non-static functions in static context.
	assert_eq!(
		check_function_modifier(&context(0), true, FunctionModifier::Payable),
		Err(static_error())
	);
	assert_eq!(
		check_function_modifier(&context(0), true, FunctionModifier::NonPayable),
		Err(static_error())
	);
	assert_eq!(
		check_function_modifier(&context(0), true, FunctionModifier::View),
		Ok(())
	);

	// Static check is performed before non-payable check.
	assert_eq!(
		check_function_modifier(&context(1), true, FunctionModifier::Payable),
		Err(static_error())
	);
	assert_eq!(
		check_function_modifier(&context(1), true, FunctionModifier::NonPayable),
		Err(static_error())
	);
	// FunctionModifier::View pass static check but fail for payable.
	assert_eq!(
		check_function_modifier(&context(1), true, FunctionModifier::View),
		Err(payable_error())
	);

	// Can't send funds to non payable function
	assert_eq!(
		check_function_modifier(&context(1), false, FunctionModifier::Payable),
		Ok(())
	);
	assert_eq!(
		check_function_modifier(&context(1), false, FunctionModifier::NonPayable),
		Err(payable_error())
	);
	assert_eq!(
		check_function_modifier(&context(1), false, FunctionModifier::View),
		Err(payable_error())
	);

	// Any function can be called without funds.
	assert_eq!(
		check_function_modifier(&context(0), false, FunctionModifier::Payable),
		Ok(())
	);
	assert_eq!(
		check_function_modifier(&context(0), false, FunctionModifier::NonPayable),
		Ok(())
	);
	assert_eq!(
		check_function_modifier(&context(0), false, FunctionModifier::View),
		Ok(())
	);
}

#[test]
fn read_static_size_tuple() {
	// (address, uint256) encoded by web3
	let data = hex!(
		"0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000000000000000000000000000000000000000000001"
	);

	let mut reader = Reader::new(&data);

	assert_eq!(
		reader.read::<(Address, U256)>().unwrap(),
		(Address(H160::repeat_byte(0x11)), U256::from(1u8))
	);
}

#[test]
fn read_dynamic_size_tuple() {
	// (uint8, bytes[]) encoded by web3
	let data = hex!(
		"0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000001
		0100000000000000000000000000000000000000000000000000000000000000"
	);

	let mut reader = Reader::new(&data);

	assert_eq!(
		reader.read::<(u8, Vec<UnboundedBytes>)>().unwrap(),
		(1, vec![UnboundedBytes::from(vec![0x01])])
	);
}

#[test]
fn write_static_size_tuple() {
	let output = Writer::new()
		.write((Address(H160::repeat_byte(0x11)), U256::from(1u8)))
		.build();

	// (address, uint256) encoded by web3
	let data = hex!(
		"0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000000000000000000000000000000000000000000001"
	);

	assert_eq!(output, data);
}

#[test]
fn write_dynamic_size_tuple() {
	let output = Writer::new()
		.write((1u8, vec![UnboundedBytes::from(vec![0x01])]))
		.build();

	// (uint8, bytes[]) encoded by web3
	let data = hex!(
		"0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000001
		0100000000000000000000000000000000000000000000000000000000000000"
	);

	assert_eq!(output, data);
}

#[test]
fn write_static_size_tuple_in_return_position() {
	let output = solidity::encode_return_value((Address(H160::repeat_byte(0x11)), U256::from(1u8)));

	// (address, uint256) encoded by web3
	let data = hex!(
		"0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000000000000000000000000000000000000000000001"
	);

	assert_eq!(output, data);
}

#[test]
fn write_dynamic_size_tuple_in_return_position() {
	let output = solidity::encode_return_value((1u8, vec![UnboundedBytes::from(vec![0x01])]));

	// (uint8, bytes[]) encoded by web3
	let data = hex!(
		"0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000001
		0100000000000000000000000000000000000000000000000000000000000000"
	);

	assert_eq!(output, data);
}

#[test]
fn error_location_formatting() {
	assert_eq!(
		Backtrace::new()
			.in_field("foo")
			.in_array(2)
			.in_array(3)
			.in_field("bar")
			.in_field("fuz")
			.to_string(),
		"fuz.bar[3][2].foo"
	);
}

#[test]
fn error_formatting() {
	assert_eq!(
		Revert::new(RevertReason::custom("Test"))
			.in_field("foo")
			.in_array(2)
			.in_array(3)
			.in_field("bar")
			.in_field("fuz")
			.to_string(),
		"fuz.bar[3][2].foo: Test"
	);
}

#[test]
fn evm_data_solidity_types() {
	use crate::solidity::Codec;
	// Simple types
	assert_eq!(bool::signature(), "bool");
	assert_eq!(u8::signature(), "uint8");
	assert_eq!(u16::signature(), "uint16");
	assert_eq!(u32::signature(), "uint32");
	assert_eq!(u64::signature(), "uint64");
	assert_eq!(u128::signature(), "uint128");
	assert_eq!(U256::signature(), "uint256");
	assert_eq!(H256::signature(), "bytes32");
	assert_eq!(Address::signature(), "address");
	assert_eq!(UnboundedBytes::signature(), "bytes");
	assert_eq!(BoundedBytes::<ConstU32<5>>::signature(), "bytes");

	// Arrays
	assert_eq!(Vec::<bool>::signature(), "bool[]");
	assert_eq!(Vec::<u8>::signature(), "uint8[]");
	assert_eq!(Vec::<u16>::signature(), "uint16[]");
	assert_eq!(Vec::<u32>::signature(), "uint32[]");
	assert_eq!(Vec::<u64>::signature(), "uint64[]");
	assert_eq!(Vec::<u128>::signature(), "uint128[]");
	assert_eq!(Vec::<U256>::signature(), "uint256[]");
	assert_eq!(Vec::<H256>::signature(), "bytes32[]");
	assert_eq!(Vec::<Address>::signature(), "address[]");
	assert_eq!(Vec::<UnboundedBytes>::signature(), "bytes[]");
	assert_eq!(Vec::<BoundedBytes<ConstU32<5>>>::signature(), "bytes[]");

	// Few tuples mixed with arrays
	assert_eq!(<(bool, Address)>::signature(), "(bool,address)");
	assert_eq!(<(Vec<bool>, Address)>::signature(), "(bool[],address)");
	assert_eq!(<(bool, Vec<Address>)>::signature(), "(bool,address[])");
	assert_eq!(Vec::<(bool, Address)>::signature(), "(bool,address)[]");
	assert_eq!(
		Vec::<(bool, Vec<Address>)>::signature(),
		"(bool,address[])[]"
	);

	// Struct encode like tuples
	assert_eq!(MultiLocation::signature(), "(uint8,bytes[])");
}
