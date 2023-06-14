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

use sha3::{Digest, Keccak256};

#[test]
fn test_keccak256() {
	assert_eq!(
		&precompile_utils_macro::keccak256!(""),
		Keccak256::digest(b"").as_slice(),
	);
	assert_eq!(
		&precompile_utils_macro::keccak256!("toto()"),
		Keccak256::digest(b"toto()").as_slice(),
	);
	assert_ne!(
		&precompile_utils_macro::keccak256!("toto()"),
		Keccak256::digest(b"tata()").as_slice(),
	);
}

#[test]
#[ignore]
fn ui() {
	let t = trybuild::TestCases::new();
	t.compile_fail("tests/compile-fail/**/*.rs");
	t.pass("tests/pass/**/*.rs");
}

// Cargo expand is not supported on stable rust
#[test]
#[ignore]
fn expand() {
	// Use `expand` to update the expansions
	// Replace it with `expand_without_refresh` afterward so that
	// CI checks the expension don't change

	// macrotest::expand("tests/expand/**/*.rs");
	macrotest::expand_without_refresh("tests/expand/**/*.rs");
}
