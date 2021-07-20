use sp_core::U256;
use sp_std::vec::Vec;

// Solidity's bool type is 256 bits as shown by these examples
// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html
// This utility function converts a Rust bool into the corresponding Solidity type
pub fn bool_to_solidity_bytes(b: bool) -> Vec<u8> {
	let mut result_bytes = [0u8; 32];

	if b {
		result_bytes[31] = 1;
	}

	result_bytes.to_vec()
}

pub fn u256_to_solidity_bytes(u: U256) -> Vec<u8> {
	let mut result_bytes = [0u8; 32];
	u.to_big_endian(&mut result_bytes);
	result_bytes.to_vec()
}
