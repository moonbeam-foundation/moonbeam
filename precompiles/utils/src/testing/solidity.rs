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

//! Utility module to interact with solidity file.

use sp_io::hashing::keccak_256;
use std::{
	collections::HashMap,
	fs::File,
	io::{BufRead, BufReader, Read},
};

pub fn check_precompile_implements_solidity_interfaces<F>(
	files: &[&'static str],
	supports_selector: F,
) where
	F: Fn(u32) -> bool,
{
	for file in files {
		for solidity_fn in get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match in file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if !supports_selector(selector) {
				panic!(
					"precompile don't support selector {selector:x} for function '{}' listed in file\
					{file}",
					solidity_fn.signature(),
				)
			}
		}
	}
}

/// Represents a declared custom type struct within a solidity file
#[derive(Clone, Default, Debug)]
pub struct SolidityStruct {
	/// Struct name
	pub name: String,
	/// List of parameter types
	pub params: Vec<String>,
	/// Is struct an enum
	pub is_enum: bool,
}

impl SolidityStruct {
	/// Returns the representative signature for the solidity struct
	pub fn signature(&self) -> String {
		if self.is_enum {
			"uint8".to_string()
		} else {
			format!("({})", self.params.join(","))
		}
	}
}

/// Represents a declared function within a solidity file
#[derive(Clone, Default)]
pub struct SolidityFunction {
	/// Function name
	pub name: String,
	/// List of function parameter types
	pub args: Vec<String>,
	/// The declared selector in the file
	pub docs_selector: String,
}

impl SolidityFunction {
	/// Returns the representative signature for the solidity function
	pub fn signature(&self) -> String {
		format!("{}({})", self.name, self.args.join(","))
	}

	/// Computes the selector code for the solidity function
	pub fn compute_selector(&self) -> u32 {
		compute_selector(&self.signature())
	}

	/// Computes the selector code as a hex string for the solidity function
	pub fn compute_selector_hex(&self) -> String {
		format!("{:0>8x}", self.compute_selector())
	}
}

/// Computes a solidity selector from a given string
pub fn compute_selector(v: &str) -> u32 {
	let output = keccak_256(v.as_bytes());
	let mut buf = [0u8; 4];
	buf.clone_from_slice(&output[..4]);
	u32::from_be_bytes(buf)
}

/// Returns a list of [SolidityFunction] defined in a solidity file
pub fn get_selectors(filename: &str) -> Vec<SolidityFunction> {
	let file = File::open(filename)
		.unwrap_or_else(|e| panic!("failed opening file '{}': {}", filename, e));
	get_selectors_from_reader(file)
}

/// Attempts to lookup a custom struct and returns its primitive signature
fn try_lookup_custom_type(word: &str, custom_types: &HashMap<String, SolidityStruct>) -> String {
	match word.strip_suffix("[]") {
		Some(word) => {
			if let Some(t) = custom_types.get(word) {
				return format!("{}[]", t.signature());
			}
		}
		None => {
			if let Some(t) = custom_types.get(word) {
				return t.signature();
			}
		}
	};

	word.to_string()
}

fn get_selectors_from_reader<R: Read>(reader: R) -> Vec<SolidityFunction> {
	#[derive(Clone, Copy)]
	enum Stage {
		Start,
		Enum,
		Struct,
		StructParams,
		FnName,
		Args,
	}
	#[derive(Clone, Copy)]
	enum Pair {
		First,
		Second,
	}
	impl Pair {
		fn next(&mut self) {
			*self = match self {
				Pair::First => Pair::Second,
				Pair::Second => Pair::First,
			}
		}
	}

	let reader = BufReader::new(reader);
	let mut functions = vec![];
	let mut custom_types = HashMap::new();
	let mut solidity_struct = SolidityStruct::default();

	let mut stage = Stage::Start;
	let mut pair = Pair::First;
	let mut solidity_fn = SolidityFunction::default();
	for line in reader.lines() {
		let line = line.expect("failed unwrapping line").trim().to_string();
		// identify declared selector
		if line.starts_with("/// @custom:selector ") && matches!(stage, Stage::Start) {
			solidity_fn.docs_selector = line.replace("/// @custom:selector ", "").to_string();
		}

		// skip comments
		if line.starts_with("//") {
			continue;
		}

		for word in line.split(&[';', ',', '(', ')', ' ']) {
			// skip whitespace
			if word.trim().is_empty() {
				continue;
			}
			match (stage, pair, word) {
				// parse custom type enums
				(Stage::Start, Pair::First, "enum") => {
					stage = Stage::Enum;
					pair.next();
				}
				(Stage::Enum, Pair::Second, _) => {
					custom_types.insert(
						word.to_string(),
						SolidityStruct {
							name: word.to_string(),
							is_enum: true,
							params: vec![],
						},
					);
					stage = Stage::Start;
					pair = Pair::First;
				}

				// parse custom type structs
				(Stage::Start, Pair::First, "struct") => {
					stage = Stage::Struct;
					pair.next();
				}
				(Stage::Struct, Pair::Second, _) => {
					solidity_struct.name = word.to_string();
					stage = Stage::StructParams;
					pair.next();
				}
				(Stage::StructParams, Pair::First, "{") => (),
				(Stage::StructParams, Pair::First, "}") => {
					custom_types.insert(solidity_struct.name.clone(), solidity_struct);
					stage = Stage::Start;
					solidity_struct = SolidityStruct::default();
				}
				(Stage::StructParams, Pair::First, _) => {
					let param = try_lookup_custom_type(word, &custom_types);
					solidity_struct.params.push(param);
					pair.next();
				}
				(Stage::StructParams, Pair::Second, _) => {
					pair.next();
				}

				// parse function
				(Stage::Start, Pair::First, "function") => {
					stage = Stage::FnName;
					pair.next();
				}
				(Stage::FnName, Pair::Second, _) => {
					solidity_fn.name = word.to_string();
					stage = Stage::Args;
					pair.next();
				}
				(Stage::Args, Pair::First, "external") => {
					functions.push(solidity_fn);
					stage = Stage::Start;
					pair = Pair::First;
					solidity_fn = SolidityFunction::default()
				}
				(Stage::Args, Pair::First, _) => {
					let mut arg = word.to_string();
					arg = try_lookup_custom_type(&arg, &custom_types);

					solidity_fn.args.push(arg);
					pair.next();
				}
				(Stage::Args, Pair::Second, "memory" | "calldata" | "storage") => (),
				(Stage::Args, Pair::Second, _) => pair.next(),
				_ => {
					stage = Stage::Start;
					pair = Pair::First;
					solidity_fn = SolidityFunction::default()
				}
			}
		}
	}

	functions
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_selectors_are_parsed() {
		let actual = get_selectors("tests/solidity_test.sol")
			.into_iter()
			.map(|sol_fn| {
				(
					sol_fn.compute_selector_hex(),
					sol_fn.docs_selector.clone(),
					sol_fn.signature(),
				)
			})
			.collect::<Vec<_>>();
		let expected = vec![
			(
				String::from("f7af8d91"),
				String::from(""),
				String::from("fnNoArgs()"),
			),
			(
				String::from("d43a9a43"),
				String::from("c4921133"),
				String::from("fnOneArg(address)"),
			),
			(
				String::from("40d6a43d"),
				String::from("67ea837e"),
				String::from("fnTwoArgs(address,uint256)"),
			),
			(
				String::from("cee150c8"),
				String::from("d6b423d9"),
				String::from("fnSameArgs(uint64,uint64)"),
			),
			(
				String::from("c6024207"),
				String::from("b9904a86"),
				String::from("fnOneArgSameLine(uint64)"),
			),
			(
				String::from("fcbc04c3"),
				String::from("28f0c44e"),
				String::from("fnTwoArgsSameLine(uint64,bytes32)"),
			),
			(
				String::from("c590304c"),
				String::from("06f0c1ce"),
				String::from("fnTwoArgsSameLineExternalSplit(uint64,bytes32)"),
			),
			(
				String::from("a19a07e1"),
				String::from("18001a4e"),
				String::from("fnMemoryArrayArgs(address[],uint256[],bytes[])"),
			),
			(
				String::from("ec26cf1c"),
				String::from("1ea61a4e"),
				String::from("fnCalldataArgs(string,bytes[])"),
			),
			(
				String::from("f29f96de"),
				String::from("d8af1a4e"),
				String::from("fnCustomArgs((uint8,bytes[]),bytes[],uint64)"),
			),
			(
				String::from("d751d651"),
				String::from("e8af1642"),
				String::from("fnEnumArgs(uint8,uint64)"),
			),
			(
				String::from("b2c9f1a3"),
				String::from("550c1a4e"),
				String::from(
					"fnCustomArgsMultiple((uint8,bytes[]),(address[],uint256[],bytes[]),bytes[],\
					uint64)",
				),
			),
			(
				String::from("d5363eee"),
				String::from("77af1a40"),
				String::from("fnCustomArrayArgs((uint8,bytes[])[],bytes[])"),
			),
			(
				String::from("b82da727"),
				String::from("80af0a40"),
				String::from(
					"fnCustomComposedArg(((uint8,bytes[]),\
				(address[],uint256[],bytes[])[]),uint64)",
				),
			),
			(
				String::from("586a2193"),
				String::from("97baa040"),
				String::from(
					"fnCustomComposedArrayArg(((uint8,bytes[]),\
				(address[],uint256[],bytes[])[])[],uint64)",
				),
			),
		];

		assert_eq!(expected, actual);
	}
}
