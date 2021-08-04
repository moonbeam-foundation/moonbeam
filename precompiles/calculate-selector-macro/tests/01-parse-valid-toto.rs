use calculate_selector_macro::calculate_fn_selector_for;
use sha3::{Digest, Keccak256};

fn main() {
	assert_eq!(&calculate_fn_selector_for!("toto()"), &Keccak256::digest(b"toto()")[0..4]);
}
