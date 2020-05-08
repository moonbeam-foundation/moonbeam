
# ![moonbeam](media/moonbeam-cover.jpg)

Run an Ethereum compatible blockchain based on Substrate.

*See [moonbeam.network](https://moonbeam.network) for the moonbeam blockchain description.*
*See [www.substrate.io](https://www.substrate.io/) for substrate information.*

## Install (linux)

### Moonbeam

```bash
git clone https://github.com/PureStake/moonbeam
cd moonbeam
```

### Dependencies

Install Rust:
```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment (*required for compiling Rust to Wasm*):
```bash
./scripts/init.sh
```

## Build

Build Wasm and native code:
```bash
cargo build --release
```
(Building for the first time will take a long time, to install and compile all the libraries)

## Run

### Single node dev

```bash
target/release/node-moonbeam --dev
```

## Contribute

### Code style

Moonbeam is following the [substrate code style](https://openethereum.github.io/wiki/Substrate-Style-Guide)
We provide a [.editorconfig](.editorconfig) (*compatible with VSCode using RLS*)
