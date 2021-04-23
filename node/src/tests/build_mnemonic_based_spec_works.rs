use assert_cmd::cargo::cargo_bin;

mod common;

#[test]
#[cfg(unix)]
fn builds_specs_based_on_mnemonic() {
	use serde_json::json;
	use std::process::Command;
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
