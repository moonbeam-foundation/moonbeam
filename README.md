# ![moonbeam](media/moonbeam-cover.jpg)

![Tests](https://github.com/PureStake/moonbeam/workflows/Release/badge.svg)

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

Run the initialization script, which checks the correct rust nightly version and adds the
`wasm32-unknown-unknown` target to that specific version:

```bash
./scripts/init.sh
```

## Build the Moonbeam Node

Build the corresponding binary file:

```bash
cargo build --release
```

The first build takes a long time, as it compiles all the necessary libraries.

> If a _cargo not found_ error appears in the terminal, manually add Rust to your system path (or
> restart your system):
>
> ```bash
> source $HOME/.cargo/env
> ```

## Run a Development Node

Moonbeam is designed to be a parachain on the Polkadot network. For testing your
contracts locally, spinning up a full relay-para network is a lot of overhead.

A simpler solution is to run the `--dev` node, a simple node that is not backed
by any relay chain, but still runs the Moonbeam runtime logic.

```bash
./target/release/moonbase-standalone --dev
```

### Docker image

An alternative to building locally is to use docker to run a pre-build binary.
The only requirement is to have Docker installed.

```bash
# Pull the docker image
docker pull purestake/moonbase-parachain-testnet:latest

# Start a dev node
docker run --rm --network host purestake/moonbase /moonbase/moonbase-standalone --dev
```

## Chain IDs

The Ethereum specification described a numeric Chain Id. The Moonbeam mainnet Chain Id will be 1284
because it takes 1284 milliseconds for a moonbeam to reach Earth.

Moonbeam nodes support multiple public chains and testnets, with the following Chain Ids.

| Network Description                | Chain ID    |
| ---------------------------------- | ----------- |
| Local Parachain TestNet            | 1280        |
| Local Development TestNet          | 1281        |
| Reserved for other TestNets        | 1282 - 1283 |
| Moonbeam (Polkadot)                | 1284        |
| Moonriver (Kusama)                 | 1285        |
| Moonrock (Rococo)                  | 1286        |
| Moonbase Alpha TestNet             | 1287        |
| Reserved for other public networks | 1288 - 1289 |

## Runtime Architecture

The Moonbeam Runtime is built using FRAME and consists of pallets from substrate, frontier, cumulus, and `pallets/`.

From substrate:

- _Balances_: Tracks GLMR token balances
- _Sudo_: Allows a privileged account to make arbitrary runtime changes - will be removed before
  launch
- _Timestamp_: On-Chain notion of time
- _Transaction Payment_: Transaction payment (fee) management
- _Randomness Collective Flip_: A (mock) onchain randomness beacon. Will be replaced by parachain
  randomness by mainnet.

From frontier:

- _EVM_: Encapsulates execution logic for an Ethereum Virtual Machine
- _Ethereum_: Ethereum-style data encoding and access for the EVM.

From cumulus:

- _ParachainUpgrade_: A helper to perform runtime upgrades on parachains
- _ParachainInfo_: A place to store parachain-relevant constants like parachain id

The following pallets are stored in `pallets/`. They are designed for Moonbeam's specific requirements:

- _Ethereum Chain Id_: A place to store the chain id for each Moonbeam network
- _Author Inherent_: Allows block authors to include their identity in a block via an inherent
- _Stake_: Minimal staking pallet that implements ordered validator selection by total amount at stake

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
