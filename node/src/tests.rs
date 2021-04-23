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
			.arg("local")
			.arg("--")
			.spawn()
			.unwrap();

		// Let it produce some blocks.
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

		// Make sure both databases were created
		assert!(base_path.path().join("chains/local_testnet/db").exists());
		assert!(base_path
			.path()
			.join("polkadot/chains/rococo_local_testnet/db")
			.exists());

		// Run the purge chain command without further args which should delete both databases
		let status = Command::new(cargo_bin("moonbeam"))
			.args(&["purge-chain", "-d"])
			.arg(base_path.path())
			.arg("--chain")
			.arg("local")
			.arg("-y")
			.status()
			.unwrap();
		assert!(status.success());

		// Make sure the parachain data directory exists
		assert!(base_path.path().join("chains/local_testnet").exists());
		// Make sure its database is deleted
		assert!(!base_path.path().join("chains/local_testnet/db").exists());

		// Make sure the relay data directory exists
		assert!(base_path
			.path()
			.join("polkadot/chains/rococo_local_testnet")
			.exists());
		// Make sure its chain is purged
		assert!(!base_path
			.path()
			.join("polkadot/chains/rococo_local_testnet/db")
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
		.unwrap();
	let chain_spec: serde_json::Value = serde_json::from_slice(output.stdout.as_slice()).unwrap();
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
		chain_spec["genesis"]["runtime"]["palletBalances"]["balances"]
			.as_array()
			.unwrap(),
		expected.as_array().unwrap()
	);
}

#[test]
#[cfg(unix)]
fn export_genesis_state() {
	let output = Command::new(cargo_bin("moonbeam"))
		.arg("export-genesis-state")
		.arg("--chain")
		.arg("local")
		.output()
		.unwrap();

	let expected = "307830303030303030303030303030303030303030303030303030303030303030303030\
	3030303030303030303030303030303030303030303030303030303030303030303865323533313462316534313533\
	3539353039623063666566363664343063316437663030666237393637623461386330663231336135626634633130\
	3033303331373061326537353937623762376533643834633035333931643133396136326231353765373837383664\
	386330383266323964636634633131313331343030";

	assert_eq!(expected, hex::encode(output.stdout.as_slice()))
}

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
			.arg("--chain")
			.arg("local")
			.arg("--dev-service")
			.arg("--sealing")
			.arg("1000")
			.arg("--collator")
			.arg("--author-id")
			.arg("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b")
			.arg("--")
			.spawn()
			.unwrap();

		// Let it produce some blocks.
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

		let output = Command::new(cargo_bin("moonbeam"))
			.args(&["export-blocks", "-d"])
			.arg(base_path.path())
			.arg("--chain")
			.arg("local")
			.arg("--from")
			.arg("1")
			.arg("--to")
			.arg("1")
			.output()
			.unwrap();

		let block_1: serde_json::Value = serde_json::from_slice(output.stdout.as_slice()).unwrap();
		assert_eq!(
			block_1["block"]["header"]["number"].as_str().unwrap(),
			"0x1",
		);
	}
}
