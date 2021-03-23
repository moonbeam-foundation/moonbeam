# ![Moonbeam](media/moonbeam-cover.jpg)

![Tests](https://github.com/PureStake/moonbeam/workflows/Release/badge.svg)

An Ethereum compatible ~~[Parachain](https://polkadot.network/technology/)~~ built with [Substrate](https://substrate.dev)


_Discover the Moonbeam project at [moonbeam.network](https://moonbeam.network)._
_Learn to [use the Moonbeam network](https://docs.moonbeam.network/) with our technical docs._
_Reference our [crate-level docs (rustdocs)](https://purestake.github.io) to contribute._

## Run a node with Docker

Docker images are published for every tagged release. Learn more with `moonbeam --help`.

```bash
# Join the public testnet
docker run --network="host" purestake/moonbeam:v0.6.1 --chain alphanet
```

### Local development node

Developers who are building dApps to run on moonbeam, may want a lightweight node to work with
locally. You can quickly spin up a single node with no relay chain backing it using the development
service.

```bash
# Run a dev service node.
docker run --network="host" purestake/moonbeam:v0.6.1 --dev
```


## Build the Moonbeam Node

To build Moonbeam, you will need a proper Substrate development environment. If you've never worked
with a Substrate-based blockchain before, you should probably try the [Setting Up a Moonbeam Node]
(https://docs.moonbeam.network/getting-started/local-node/setting-up-a-node/) docs first. If you
need a refresher setting up your Substrate environment, see [Substrate's Getting Started Guide]
(https://substrate.dev/docs/en/knowledgebase/getting-started/).

```bash
# Fetch the code
git clone https://github.com/PureStake/moonbeam
cd moonbeam

# Optional: Ensure you have the exact nightly toolchain used by Moonbeam's CI
./scripts/init.sh

# Build the node (The first build will be long (~30min))
cargo build --release
```

## Run tests

Moonbeam has Rust unit tests as well as typescript integration tests.

```bash
# Run the Rust unit tests
cargo test
```

```bash
# Install dependencies for integration tests
cd moonbeam-types-bundle
npm i

cd ../tests
npm i

# Run integration tests
npm test
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

- _Utility_: Allows users to use derivative accounts, and batch calls
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

Moonbeam is open source under the terms of the GPL3. We welcome contributions. You can explore our
crate-level documentation at https://purestake.github.io/moonbeam

### Code style

Moonbeam is following the
[Substrate code style](https://github.com/paritytech/substrate/blob/master/docs/STYLE_GUIDE.md)  
We provide a [.editorconfig](.editorconfig) (_compatible with VSCode using RLS_)
