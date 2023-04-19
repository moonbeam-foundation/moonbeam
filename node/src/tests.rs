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

//! An example of doing some integration testing against the complete node in Rust
//! This approach is similar to our current typescript tests and is inspired by
//! https://github.com/paritytech/cumulus/blob/master/rococo-parachains/tests/purge_chain_works.rs
//! However Basti seems dissatisfied with this approach, for reasons I don't fully understand.
//! https://github.com/paritytech/cumulus/pull/306#discussion_r584166203

use assert_cmd::cargo::cargo_bin;
use std::{
	convert::TryInto,
	process::{Child, Command, ExitStatus},
	thread,
	time::Duration,
};

/// Wait for the given `child` the given ammount of `secs`.
///
/// Returns the `Some(exit status)` or `None` if the process did not finish in the given time.
pub fn wait_for(child: &mut Child, secs: usize) -> Option<ExitStatus> {
	for _ in 0..secs {
		match child.try_wait().unwrap() {
			Some(status) => return Some(status),
			None => thread::sleep(Duration::from_secs(1)),
		}
	}
	eprintln!("Took too long to exit. Killing...");
	let _ = child.kill();
	child.wait().unwrap();

	None
}
