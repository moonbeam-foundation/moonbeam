
# ![moonbeam](media/moonbeam-cover.jpg)
![Tests](https://github.com/PureStake/moonbeam/workflows/Tests/badge.svg)

Run an Ethereum compatible ~~parachain~~ (and blockchain for now, until parachains are more stable) based on Substrate.

*See [moonbeam.network](https://moonbeam.network) for the moonbeam blockchain description.*  
*See [www.substrate.io](https://www.substrate.io/) for substrate information.*

## Install (linux)

### Get the code
Get the tutorial specific tag of the PureStake/Moonbeam repo:
```bash
git clone -b tutorial-v2 https://github.com/PureStake/moonbeam
cd moonbeam
```

### Setting up enviroment

Install Substrate pre-requisites (including Rust):  
```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

Run the initialization script, which checks the correct rust nightly version and adds the WASM to that specific version:
```bash
./scripts/init.sh
```

## Build Standalone
Build the corresponding binary file:

```bash
cd node/standalone
cargo build --release
```  

## Build Parachain
Build the corresponding binary file:
```bash
cargo build --release
```  
The first build takes a long time, as it compiles all the necessary libraries.

### Troubleshooting
If a _cargo not found_ error appears in the terminal, manually add Rust to your system path (or restart your system):
```bash
source $HOME/.cargo/env
```

## Run

### Standalone Node in dev mode

```bash
./node/standalone/target/release/moonbase-standalone --dev
```

## Docker image

### Standlone node

You can run a standalone Moonbeam node with Docker directly:
```bash
docker run purestake/moonbase:tutorial-v2.2 /moonbase/moonbase-standalone
```

## Chain IDs

The ethereum specification described a numeric Chain Id. The Moonbeam mainnet Chain Id will be 1284
because it takes 1284 milliseconds for a moonbeam to reach Earth.

Moonbeam nodes support multiple public chains and testnets, with the following Chain Ids.

| Network Description | Chain ID |
| --- | --- |
| Local parachain testnet | 1280 |
| Local standalone testnet | 1281 |
| Reserved for other testnets | 1282 - 1283 |
| Moonbeam (Polkadot) | 1284 |
| Moonriver (Kusama) | 1285|
| Moonrock (Rococo) | 1286 |
| Public parachain testnet (alphanet) | 1287 |
| Reserved for other public networks | 1288 - 1289 |

## Pallets
* *aura*: Time-based Authority Consensus (for simplicity until more development is done)
* *balances*: Account & Balance management
* *grandpa*: GRANDPA Authority consensus (This will be removed once it becomes a parachain)
* *sudo*: Allow specific account to call any dispatchable ("Alice": `0x57d213d0927ccc7596044c6ba013dd05522aacba`, will get removed at some point)
* *timestamp*: On-Chain time management
* *transaction*-payment: Transaction payement (fee) management
* *evm*: EVM Execution. (Temporary until we work on pallet-ethereum)

* ***mb-core***: Currently serves as a way to experiments with pallets and substrate (will get removed)
* ***mb-session***: Logic for selecting validators based on a endorsement system

## Tests

Tests are run with the following command:
```bash
cargo test --verbose
```

This github repository is also linked to Gitlab CI

## Contribute

### Code style

Moonbeam is following the [Substrate code style](https://github.com/paritytech/substrate/blob/master/docs/STYLE_GUIDE.md)  
We provide a [.editorconfig](.editorconfig) (*compatible with VSCode using RLS*)
