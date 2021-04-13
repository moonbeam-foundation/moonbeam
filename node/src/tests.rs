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
	eprintln!("Took to long to exit. Killing...");
	let _ = child.kill();
	child.wait().unwrap();

	None
}

#[test]
#[cfg(unix)]
fn purge_chain_works() {
	fn run_node_and_stop() -> tempfile::TempDir {
		use nix::{
			sys::signal::{kill, Signal::SIGINT},
			unistd::Pid,
		};

		let base_path = tempfile::tempdir().unwrap();

		let mut cmd = Command::new(cargo_bin("rococo-collator"))
			.args(&["-d"])
			.arg(base_path.path())
			.args(&["--"])
			.spawn()
			.unwrap();

		// Let it produce some blocks.
		thread::sleep(Duration::from_secs(30));
		assert!(
			cmd.try_wait().unwrap().is_none(),
			"the process should still be running"
		);

		// Stop the process
		kill(Pid::from_raw(cmd.id().try_into().unwrap()), SIGINT).unwrap();
		assert!(common::wait_for(&mut cmd, 30)
			.map(|x| x.success())
			.unwrap_or_default());

		base_path
	}

	// Check that both databases are deleted
	{
		let base_path = run_node_and_stop();

		assert!(base_path.path().join("chains/local_testnet/db").exists());
		assert!(base_path
			.path()
			.join("polkadot/chains/westend_dev/db")
			.exists());

		let status = Command::new(cargo_bin("rococo-collator"))
			.args(&["purge-chain", "-d"])
			.arg(base_path.path())
			.arg("-y")
			.status()
			.unwrap();
		assert!(status.success());

		// Make sure that the `parachain_local_testnet` chain folder exists, but the `db` is deleted.
		assert!(base_path.path().join("chains/local_testnet").exists());
		assert!(!base_path.path().join("chains/local_testnet/db").exists());
		assert!(base_path
			.path()
			.join("polkadot/chains/westend_dev")
			.exists());
		assert!(!base_path
			.path()
			.join("polkadot/chains/westend_dev/db")
			.exists());
	}
}
