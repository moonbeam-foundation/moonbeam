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

#[test]
#[cfg(unix)]
fn purge_chain_purges_relay_and_para() {
	fn run_node_and_stop() -> tempfile::TempDir {
		use nix::{
			sys::signal::{kill, Signal::SIGINT},
			unistd::Pid,
		};

		let base_path = tempfile::tempdir().unwrap();

		let mut cmd = Command::new(cargo_bin("moonbeam"))
			.arg("-d")
			.arg(base_path.path())
			.arg("--chain")
			.arg("moonbase-dev")
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
		assert!(wait_for(&mut cmd, 30)
			.map(|x| x.success())
			.unwrap_or_default());

		base_path
	}

	{
		let base_path = run_node_and_stop();

		// Make sure dev database was created
		assert!(base_path.path().join("chains/moonbase_dev/db").exists());

		// Run the purge chain command without further args which should delete both databases
		let status = Command::new(cargo_bin("moonbeam"))
			.args(&["purge-chain", "-d"])
			.arg(base_path.path())
			.arg("--chain")
			.arg("moonbase-dev")
			.arg("-y")
			.status()
			.unwrap();
		assert!(status.success());

		// Make sure the parachain data directory exists
		assert!(base_path.path().join("chains/moonbase_dev").exists());
		// Make sure its database is deleted
		assert!(!base_path
			.path()
			.join("chains/moonbase_dev/db/full")
			.exists());
	}
}

#[test]
#[cfg(unix)]
fn builds_specs_based_on_mnemonic() {
	use serde_json::json;

	let output = Command::new(cargo_bin("moonbeam"))
		.arg("build-spec")
		.arg("--dev")
		.arg("--mnemonic")
		.arg("myself dutch allow coast planet high glow parrot parent choice identify match")
		.arg("--accounts")
		.arg("3")
		.output()
		.expect("Failed to start moonbeam");

	// Gather output as json
	let chain_spec: serde_json::Value =
		serde_json::from_slice(output.stdout.as_slice()).expect("Failed to parse spec json file");
	let expected = json!([
		[
			json!("0x3d5bd6a54d5f5292b9fb914db40cd5f7c5540f80"),
			json!(1208925819614629200000000.0)
		],
		[
			json!("0x8055e8c75a862c0da765ed0848366ef4dc492b33"),
			json!(1208925819614629200000000.0)
		],
		[
			json!("0x764d008debe9493d851d0476ccba9ec23817e2c9"),
			json!(1208925819614629200000000.0)
		],
		// This is Geralds, which is also added
		[
			json!("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b"),
			json!(1208925819614629200000000.0)
		]
	]);

	assert_eq!(
		chain_spec["genesis"]["runtime"]["balances"]["balances"]
			.as_array()
			.expect("Failed to extract balances"),
		expected.as_array().unwrap()
	);
}

#[test]
#[cfg(unix)]
fn export_genesis_state() {
	let output = Command::new(cargo_bin("moonbeam"))
		.arg("export-genesis-state")
		.arg("--chain")
		.arg("moonriver")
		.output()
		.unwrap();

	let expected = "3078303030303030303030303030303030303030303030303030303030303030303030303\
	0303030303030303030303030303030303030303030303030303030303030306136333435616666313632643465366\
	3363633336439316164663962323161336638346461356539306535383736363866313635333964346365343936643\
	0383033313730613265373539376237623765336438346330353339316431333961363262313537653738373836643\
	86330383266323964636634633131313331343030";

	assert_eq!(expected, hex::encode(output.stdout.as_slice()))
}

#[ignore]
#[test]
#[cfg(unix)]
fn export_current_state() {
	fn run_node_and_stop() -> tempfile::TempDir {
		use nix::{
			sys::signal::{kill, Signal::SIGINT},
			unistd::Pid,
		};

		let base_path = tempfile::tempdir().unwrap();

		let mut cmd = Command::new(cargo_bin("moonbeam"))
			.arg("-d")
			.arg(base_path.path())
			.arg("--dev")
			.arg("--sealing")
			.arg("1000")
			.spawn()
			.unwrap();

		// Let it produce some blocks.
		// This fails if is not a minimum of 20
		thread::sleep(Duration::from_secs(20));
		assert!(
			cmd.try_wait().unwrap().is_none(),
			"the process should still be running"
		);

		// Stop the process
		kill(Pid::from_raw(cmd.id().try_into().unwrap()), SIGINT).unwrap();
		assert!(wait_for(&mut cmd, 30)
			.map(|x| x.success())
			.unwrap_or_default());

		base_path
	}
	{
		let base_path = run_node_and_stop();

		// Test whether we can export one of the generated blocks.
		let output = Command::new(cargo_bin("moonbeam"))
			.args(&["export-blocks", "-d"])
			.arg(base_path.path())
			.arg("--dev")
			.arg("--from")
			.arg("1")
			.arg("--to")
			.arg("1")
			.arg("--pruning")
			.arg("archive")
			.output()
			.unwrap();

		let block_1: serde_json::Value = serde_json::from_slice(output.stdout.as_slice()).unwrap();
		assert_eq!(
			block_1["block"]["header"]["number"].as_str().unwrap(),
			"0x1",
		);
	}
}
