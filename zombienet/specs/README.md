# Instructions

## Requirements:

1. Clone [polkadot-fellows/runtimes](https://github.com/polkadot-fellows/runtimes.git) repository
2. Build `cargo build --release --features fast-runtime`

Generate the chain spec files with the following commands:

```sh 
./target/release/chain-spec-generator polkadot-local > polkadot-local.json
./target/release/chain-spec-generator polkadot-local > kusama-local.json
```