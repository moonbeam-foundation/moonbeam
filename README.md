
# ![moonbeam](media/moonbeam-cover.jpg)
![Tests](https://github.com/PureStake/moonbeam/workflows/Tests/badge.svg)

Run an Ethereum compatible ~~parachain~~ (and blockchain for now, until parachains are more stable) based on Substrate.

*See [moonbeam.network](https://moonbeam.network) for the moonbeam blockchain description.*  
*See [www.substrate.io](https://www.substrate.io/) for substrate information.*

## Install (linux)

### Get the code

```bash
git clone -b moonbeam-tutorials https://github.com/PureStake/moonbeam
cd moonbeam
```

### Rust developer environment

Install Substrate pre-requisites (including Rust):  
```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

## Build Parachain

Build Wasm and native code:  
```bash
cargo build --release
```  
(Building for the first time will take a long time, to install and compile all the libraries)

If a _cargo not found_ error appears in the terminal, manually add Rust to your system path (or restart your system):
```bash
source $HOME/.cargo/env
```

## Build Standalone
```bash
cd node/standalone
cargo build --release
```  

## Run

### Single node dev

```bash
./node/standalone/target/release/moonbase-standalone --dev
```
### Docker image

You can run the moonbeam node within Docker directly.  
The Dockerfile is optimized for development speed.  
(Running the `docker run...` command will recompile the binaries but not the dependencies)

Building (takes 5-10 min):
```bash
docker build -t moonbeam-node-dev .
```

Running (takes 1 min to rebuild binaries):
```bash
docker run -t moonbeam-node-dev
```

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
