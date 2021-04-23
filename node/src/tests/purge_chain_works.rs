use assert_cmd::cargo::cargo_bin;
use std::{convert::TryInto, process::Command, thread, time::Duration};

mod common;

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
		assert!(common::wait_for(&mut cmd, 30)
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
