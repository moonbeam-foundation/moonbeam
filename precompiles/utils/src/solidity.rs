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
	collections::BTreeMap,
	fs::File,
	io::{BufRead, BufReader, Read},
};

use tiny_keccak::Hasher;

pub fn get_selectors(filename: &str) -> BTreeMap<u32, String> {
	let file = File::open(filename).expect("failed opening file");
	get_selectors_from_reader(file)
}

fn get_selectors_from_reader<R: Read>(reader: R) -> BTreeMap<u32, String> {
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
	let mut fn_name = "".to_string();
	let mut args: Vec<String> = vec![];
	for line in reader.lines() {
		let line = line.expect("failed unwrapping line");
		// skip comments
		if line.trim_start().starts_with("//") {
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
					fn_name = word.to_string();
					stage = Stage::Args;
					pair.next();
				}
				(Stage::Args, Pair::First, "external") => {
					functions.push(format!("{}({})", fn_name, args.join(",")));
					stage = Stage::Start;
					pair = Pair::First;
					fn_name = "".to_string();
					args = vec![];
				}
				(Stage::Args, Pair::First, _) => {
					args.push(word.to_string());
					pair.next();
				}
				(Stage::Args, Pair::Second, _) => pair.next(),
				_ => {
					stage = Stage::Start;
					pair = Pair::First;
					fn_name = "".to_string();
					args = vec![];
				}
			}
		}
	}

	functions
		.into_iter()
		.map(|func| {
			let mut hasher = tiny_keccak::Keccak::v256();
			hasher.update(func.as_bytes());
			let mut output = [0u8; 32];
			hasher.finalize(&mut output);
			let mut buf = [0u8; 4];
			buf.clone_from_slice(&output[..4]);
			let selector = u32::from_be_bytes(buf);

			(selector, func)
		})
		.collect::<BTreeMap<_, _>>()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_selectors_are_parsed() {
		let actual = get_selectors("solidity_test.sol")
			.into_iter()
			.collect::<Vec<_>>();
		let expected = vec![
			(0x40d6a43d, "fnTwoArgs(address,uint256)".to_string()),
			(
				0xc590304c,
				"fnTwoArgsSameLineExternalSplit(uint64,bytes32)".to_string(),
			),
			(0xc6024207, "fnOneArgSameLine(uint64)".to_string()),
			(0xcee150c8, "fnSameArgs(uint64,uint64)".to_string()),
			(0xd43a9a43, "fnOneArg(address)".to_string()),
			(0xf7af8d91, "fnNoArgs()".to_string()),
			(0xfcbc04c3, "fnTwoArgsSameLine(uint64,bytes32)".to_string()),
		];
		assert_eq!(expected, actual);
	}
}
