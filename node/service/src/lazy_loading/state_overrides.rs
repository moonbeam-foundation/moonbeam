// Copyright 2024 Moonbeam foundation
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

use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct StateEntryConcrete {
	pub(crate) pallet: String,
	pub(crate) storage: String,
	#[serde(
		skip_serializing_if = "Option::is_none",
		deserialize_with = "serde_hex::deserialize_as_option",
		default
	)]
	pub(crate) key: Option<Vec<u8>>,
	#[serde(deserialize_with = "serde_hex::deserialize")]
	pub(crate) value: Vec<u8>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StateEntryRaw {
	#[serde(deserialize_with = "serde_hex::deserialize")]
	pub(crate) key: Vec<u8>,
	#[serde(deserialize_with = "serde_hex::deserialize")]
	pub(crate) value: Vec<u8>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StateEntry {
	Concrete(StateEntryConcrete),
	Raw(StateEntryRaw),
}

/// Mandatory state overrides that most exist when starting a node in lazy loading mode.
fn base_state_overrides(runtime_code: Option<PathBuf>) -> Vec<StateEntry> {
	let runtime_code = if let Some(path) = runtime_code {
		let mut reader = std::fs::File::open(path).expect("Can open file");
		let mut data = vec![];
		reader
			.read_to_end(&mut data)
			.expect("Runtime code override invalid.");

		data.to_vec()
	} else {
		moonbeam_runtime::WASM_BINARY
			.expect(
				"Wasm binary is not available. This means the client is built with \
							 `SKIP_WASM_BUILD` flag. Please rebuild with the flag disabled.",
			)
			.to_vec()
	};
	vec![
		// Set runtime code
		StateEntry::Raw(
			StateEntryRaw {
				key: sp_core::storage::well_known_keys::CODE.to_vec(),
				value: runtime_code
			}
		),
		// Setup Alith account
		StateEntry::Concrete(
			StateEntryConcrete {
				pallet: "System".to_string(),
				storage: "Account".to_string(),
				key: Some(hex_literal::hex!("9dfefc73f89d24437a9c2dce5572808af24ff3a9cf04c71dbc94d0b566f7a27b94566cac").to_vec()), // editorconfig-checker-disable-line
				value: hex_literal::hex!("360000000100000001000000010000002c1c7fd5d87ce640b80700000000000000006c139ba62d3b2402000000000000000040b2bac9e0191e02000000000000000040b2bac9e0191e02000000000080").to_vec() // editorconfig-checker-disable-line
			}
		),
		StateEntry::Concrete(
			StateEntryConcrete {
				pallet: "AuthorMapping".to_string(),
				storage: "NimbusLookup".to_string(),
				key: Some(hex_literal::hex!("9dfefc73f89d24437a9c2dce5572808af24ff3a9cf04c71dbc94d0b566f7a27b94566cac").to_vec()), // editorconfig-checker-disable-line
				value: hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").to_vec() // editorconfig-checker-disable-line
			}
		),
		StateEntry::Concrete(
			StateEntryConcrete {
				pallet: "AuthorMapping".to_string(),
				storage: "MappingWithDeposit".to_string(),
				key: Some(hex_literal::hex!("de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").to_vec()), // editorconfig-checker-disable-line
				value: hex_literal::hex!("f24ff3a9cf04c71dbc94d0b566f7a27b94566cac000010632d5ec76b0500000000000000d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").to_vec() // editorconfig-checker-disable-line
			}
		),
		// Set candidate pool
		StateEntry::Concrete(
			StateEntryConcrete {
				pallet: "ParachainStaking".to_string(),
				storage: "CandidatePool".to_string(),
				key: None,
				value: hex_literal::hex!("04f24ff3a9cf04c71dbc94d0b566f7a27b94566cac0000a0dec5adc9353600000000000000").to_vec() // editorconfig-checker-disable-line
			}
		),
		// Set Alith as selected candidate
		StateEntry::Concrete(
			StateEntryConcrete {
				pallet: "ParachainStaking".to_string(),
				storage: "SelectedCandidates".to_string(),
				key: None,
				value: hex_literal::hex!("04f24ff3a9cf04c71dbc94d0b566f7a27b94566cac").to_vec()
			}
		),
		// AtStake
		StateEntry::Concrete(
			StateEntryConcrete {
				pallet: "ParachainStaking".to_string(),
				storage: "AtStake".to_string(),
				key: Some(hex_literal::hex!("5153cb1f00942ff4010000004a6bb7c01d316509f24ff3a9cf04c71dbc94d0b566f7a27b94566cac").to_vec()),
				value: hex_literal::hex!("0000a0dec5adc9353600000000000000000000a0dec5adc9353600000000000000").to_vec()
			}
		),
	]
}

pub fn read(path: PathBuf, runtime_code_path: Option<PathBuf>) -> Result<Vec<StateEntry>, String> {
	let reader = std::fs::File::open(path).expect("Can open file");
	let state = serde_json::from_reader(reader).expect("Can parse state overrides JSON");

	Ok([base_state_overrides(runtime_code_path), state].concat())
}

mod serde_hex {
	use hex::FromHex;
	use serde::{Deserialize, Deserializer};

	fn sanitize(data: &str) -> &str {
		if data.starts_with("0x") {
			&data[2..]
		} else {
			data
		}
	}

	pub fn deserialize_as_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
	where
		D: Deserializer<'de>,
		T: FromHex,
		<T as FromHex>::Error: std::fmt::Display + std::fmt::Debug,
	{
		Option::<String>::deserialize(deserializer).map(|value| {
			value.map(|data| FromHex::from_hex(sanitize(data.as_str())).expect("Invalid option"))
		})
	}

	pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
	where
		D: Deserializer<'de>,
		T: FromHex,
		<T as FromHex>::Error: std::fmt::Display + std::fmt::Debug,
	{
		String::deserialize(deserializer).map(|data| {
			FromHex::from_hex(sanitize(data.as_str())).expect("Invalid hex encoded string")
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;

	#[test]
	pub fn can_read_state_overrides() -> Result<(), String> {
		let file = "/Users/romarq/Projects/Moonsong/Moonbeam/moonbeam/state_overrides.json";
		let path = std::path::PathBuf::from_str(file).expect("File exists");

		read(path).map(|_| ())
	}
}
