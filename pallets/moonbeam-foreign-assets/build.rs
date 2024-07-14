use std::fs::File;
use std::io::prelude::*;

fn main() {
	let bytecode = hex::decode(include_str!("resources/foreign_erc20_initcode.hex"))
		.expect("fail to decode hexadecimal string in file foreign_erc20_initcode.hex");

	let mut file = File::create("resources/foreign_erc20_initcode.bin")
		.expect("Fail to create file resources/foreign_erc20_initcode.bin");
	file.write_all(&bytecode)
		.expect("fail to write bytecode in /foreign_erc20_initcode.bin");
}
