// Copyright 2019-2020 PureStake Inc.
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

//! Provide serialization functions for various types and formats.

use ethereum_types::{H256, U256};
use serde::{
	ser::{Error, SerializeSeq},
	Serializer,
};

pub fn seq_h256_serialize<S>(data: &Option<Vec<H256>>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let d = data.clone().unwrap();
	let mut seq = serializer.serialize_seq(Some(d.len()))?;
	for h in d {
		seq.serialize_element(&format!("{:x}", h))?;
	}
	seq.end()
}

pub fn bytes_0x_serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
}

pub fn opcode_serialize<S>(opcode: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let d = std::str::from_utf8(opcode)
		.map_err(|_| S::Error::custom("Opcode serialize error."))?
		.to_uppercase();
	serializer.serialize_str(&d)
}

pub fn string_serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let d = std::str::from_utf8(value)
		.map_err(|_| S::Error::custom("String serialize error."))?
		.to_string();
	serializer.serialize_str(&d)
}

pub fn u256_serialize<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_u64(data.low_u64())
}

pub fn h256_serialize<S>(data: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("{:x}", data))
}

pub fn h256_0x_serialize<S>(data: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("0x{:x}", data))
}
