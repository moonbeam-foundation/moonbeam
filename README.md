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

### Dev Addresses

Launching the node in devmode will prefund a list of dev addresses that are derived from
the canonical mnemonic: "bottom drive obey lake curtain smoke basket hold race lonely fit walk"

#### Alith:

- Address:0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac
- PrivKey:0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133

#### Baltathar:

- Address:0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0
- PrivKey:0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b

#### Charleth:

- Address:0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc
- PrivKey:0x0b6e18cafb6ed99687ec547bd28139cafdd2bffe70e6b688025de6b445aa5c5b

#### Dorothy:

- Address:0x773539d4Ac0e786233D90A233654ccEE26a613D9
- PrivKey:0x39539ab1876910bbf3a223d84a29e28f1cb4e2e456503e7e91ed39b2e7223d68

#### Ethan:

- Address:0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB
- PrivKey:0x7dce9bc8babb68fec1409be38c8e1a52650206a7ed90ff956ae8a6d15eeaaef4

#### Faith:

- Address:0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d
- PrivKey:0xb9d2ea9a615f3165812e8d44de0d24da9bbd164b65c4f0573e1ce2c8dbd9c8df

#### Gerald:

- Address:0x7BF369283338E12C90514468aa3868A551AB2929
- PrivKey:0x96b8a38e12e1a31dee1eab2fffdf9d9990045f5b37e44d8cc27766ef294acf18

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
