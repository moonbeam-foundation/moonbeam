// Copyright 2024 Moonbeam Foundation.
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

use std::fs::File;
use std::io::prelude::*;

// Length of encoded constructor parameters
const PARAMS_LEN: usize = 256;

fn main() {
	let hex_str = include_str!("resources/foreign_erc20_initcode.hex");
	let prefix_0x = hex_str.chars().nth(1) == Some('x');
	let bytecode = if prefix_0x {
		hex::decode(&hex_str[2..])
	} else {
		hex::decode(hex_str)
	}
	.expect("fail to decode hexadecimal string in file foreign_erc20_initcode.hex");

	// The encoded parameters at the end of the initializer bytecode should be removed,
	// (the runtime will append the constructor parameters dynamically).
	let bytecode_end = if bytecode.len() > PARAMS_LEN {
		bytecode.len() - PARAMS_LEN
	} else {
		0
	};

	let mut file = File::create("resources/foreign_erc20_initcode.bin")
		.expect("Fail to create file resources/foreign_erc20_initcode.bin");
	file.write_all(&bytecode[..bytecode_end])
		.expect("fail to write bytecode in /foreign_erc20_initcode.bin");
}
