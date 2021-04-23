use assert_cmd::cargo::cargo_bin;
use std::{convert::TryInto, process::Command, thread, time::Duration};

mod common;


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