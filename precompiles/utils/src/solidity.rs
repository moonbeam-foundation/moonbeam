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

use std::{
	fs::File,
	io::{BufRead, BufReader, Read},
};

use tiny_keccak::Hasher;

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
		let sig = self.signature();
		let mut hasher = tiny_keccak::Keccak::v256();
		hasher.update(sig.as_bytes());
		let mut output = [0u8; 32];
		hasher.finalize(&mut output);
		let mut buf = [0u8; 4];
		buf.clone_from_slice(&output[..4]);
		u32::from_be_bytes(buf)
	}

	/// Computes the selector code as a hex string for the solidity function
	pub fn compute_selector_hex(&self) -> String {
		format!("{:0>8x}", self.compute_selector())
	}
}

/// Returns a list of [SolidityFunction] defined in a solidity file
pub fn get_selectors(filename: &str) -> Vec<SolidityFunction> {
	let file = File::open(filename).expect("failed opening file");
	get_selectors_from_reader(file)
}

fn get_selectors_from_reader<R: Read>(reader: R) -> Vec<SolidityFunction> {
	#[derive(Clone, Copy)]
	enum Stage {
		Start,
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

	let mut stage = Stage::Start;
	let mut pair = Pair::First;
	let mut solidity_fn = SolidityFunction::default();
	for line in reader.lines() {
		let line = line.expect("failed unwrapping line").trim().to_string();
		// identify declared selector
		if line.starts_with("/// Selector: ") && matches!(stage, Stage::Start) {
			solidity_fn.docs_selector = format!("{}", line.replace("/// Selector: ", ""))
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
					solidity_fn.args.push(word.to_string());
					pair.next();
				}
				(Stage::Args, Pair::Second, "memory") => (),
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
		];
		assert_eq!(expected, actual);
	}
}
