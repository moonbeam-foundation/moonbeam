// Copyright 2019-2021 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

use calculate_selector_macro::calculate_fn_selector_for;
use sha3::{Digest, Keccak256};

fn main() {
	assert_eq!(
		&calculate_fn_selector_for!("toto()"),
		&Keccak256::digest(b"toto()")[0..4]
	);
}
