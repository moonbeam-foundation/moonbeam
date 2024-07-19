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
