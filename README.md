# ![moonbeam](media/moonbeam-cover.jpg)

![Tests](https://github.com/PureStake/moonbeam/workflows/Tests/badge.svg)

Run an Ethereum compatible ~~parachain~~ (and blockchain for now, until parachains are more stable)
based on Substrate.

_See [moonbeam.network](https://moonbeam.network) for the moonbeam blockchain description._  
_See [www.substrate.io](https://www.substrate.io/) for substrate information._

## Install (linux)

### Get the code

Get the tutorial specific tag of the PureStake/Moonbeam repo:

```bash
git clone -b tutorial-v3 https://github.com/PureStake/moonbeam
cd moonbeam
```

### Setting up enviroment

Install Substrate pre-requisites (including Rust):

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

Run the initialization script, which checks the correct rust nightly version and adds the WASM to
that specific version:

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

If a _cargo not found_ error appears in the terminal, manually add Rust to your system path (or
restart your system):

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
An alternative to the steps higlighted before is to use docker to run a pre-build binary. Doing so, you prevent having to install Substrate and all the dependencies, and you can skip the building the node process as well. The only requirement is to have Docker installed, and then you can execute the following command to download the corresponding image:

```bash
docker pull purestake/moonbase:tutorial-v3
```

Once the Docker image is downloaded, you can run it with the following line:

```bash
docker run --rm --name moonbeam_standalone --network host purestake/moonbase:tutorial-v3 /moonbase/moonbase-standalone --dev
```

## Chain IDs

The ethereum specification described a numeric Chain Id. The Moonbeam mainnet Chain Id will be 1284
because it takes 1284 milliseconds for a moonbeam to reach Earth.

Moonbeam nodes support multiple public chains and testnets, with the following Chain Ids.

| Network Description                 | Chain ID    |
| ----------------------------------- | ----------- |
| Local Parachain TestNet             | 1280        |
| Local Standalone Node               | 1281        |
| Reserved for other TestNets         | 1282 - 1283 |
| Moonbeam (Polkadot)                 | 1284        |
| Moonriver (Kusama)                  | 1285        |
| Moonrock (Rococo)                   | 1286        |
| Moonbase Alpha TestNet              | 1287        |
| Reserved for other public networks  | 1288 - 1289 |

## Runtime Architecture

The Moonbeam Runtime is built using FRAME and consists of several core pallets, as well as a few
pallets that are only present conditionally. The core pallets are:

- _Balances_: Tracks GLMR token balances
- _Sudo_: Allows a privledged acocunt to make arbitrary runtime changes - will be removed before
  launch
- _Timestamp_: On-Chain notion of time
- _EVM_: Encapsulates execution logic for an Ethereum Virtual Machine
- _Ethereum_: Ethereum-style data encoding and access for the EVM.
- _Ethereum Chain Id_: A place to store the chain id for each Moonbeam network
- _Transaction Payment_: Transaction payment (fee) management
- _Randomness Collective Flip_: A (mock) onchain randomness beacon. Will be replaced by parachain
  randomness by mainnet.

### Parachain

In addition to the core pallets above, the parachain node also features

- _ParachainUpgrade_: A helper to perform runtime upgrades on parachains
- _MessageBroker_: A helper to receive incoming XCPM messages
- _ParachainInfo_: A place to store parachain-relevant constants like parachain id
- _TokenDealer_: A helper for accepting incoming cross-chain asset transfers

### Standalone

In addition to the core pallets above, the standalone node also features

- _Aura_: Slot-based Authority Consensus
- _Grandpa_: GRANDPA Authority consensus (This will be removed once it becomes a parachain)

## Tests

Tests are run with the following command:

```bash
cargo test --verbose
```

This github repository is also linked to Gitlab CI

## Contribute

### Code style

Moonbeam is following the
[Substrate code style](https://github.com/paritytech/substrate/blob/master/docs/STYLE_GUIDE.md)  
We provide a [.editorconfig](.editorconfig) (_compatible with VSCode using RLS_)
