# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

Moonbeam is an Ethereum-compatible parachain built with Polkadot-SDK. It enables Ethereum-style smart contracts on Polkadot/Kusama networks.

## Quick Commands

```bash
# Build
cargo build --release

# Test
cargo test -p pallet-parachain-staking
cd test && pnpm moonwall test dev_moonbase

# Lint
cargo clippy --release --workspace
cargo fmt -- --check

# Run dev node
./target/release/moonbeam --dev --alice --sealing 6000 --rpc-port 9944
```

## Project Structure

```
pallets/        # Custom FRAME pallets
precompiles/    # EVM precompiled contracts
runtime/        # Runtime implementations (moonbase, moonbeam, moonriver)
node/           # Node binary
client/         # Client RPC and services
test/           # TypeScript integration tests
```

## Dependency Forks

Moonbeam maintains forks of key dependencies:

```
moonbeam
├── polkadot-sdk (moonbeam-foundation/polkadot-sdk)
├── moonkit
│   ├── polkadot-sdk (moonbeam-foundation/polkadot-sdk)
│   ├── evm
│   └── frontier
├── evm
└── frontier
    ├── evm
    └── polkadot-sdk (moonbeam-foundation/polkadot-sdk)

parity-bridges-common
└── frontier (paritytech/polkadot-sdk)
```

| Dependency   | Fork                             | Purpose                                         |
| ------------ | -------------------------------- | ----------------------------------------------- |
| polkadot-sdk | moonbeam-foundation/polkadot-sdk | Substrate/Cumulus with Moonbeam patches         |
| frontier     | moonbeam-foundation/frontier     | Ethereum compatibility layer                    |
| evm          | moonbeam-foundation/evm          | SputnikVM fork for EVM execution                |
| moonkit      | moonbeam-foundation/moonkit      | Shared Moonbeam components (nimbus, randomness) |

## Networks

| Network        | Chain ID | Runtime   |
| -------------- | -------- | --------- |
| Moonbeam       | 1284     | moonbeam  |
| Moonriver      | 1285     | moonriver |
| Moonbase Alpha | 1287     | moonbase  |
| Development    | 1281     | moonbase  |

## Before Committing

1. `cargo fmt`
2. `cargo clippy --release`
3. `pnpm check` (TypeScript)
4. Ensure tests pass

## PR Requirements

- Label: `B7-runtimenoteworthy`, `B5-clientnoteworthy`, or `B0-silent`
- Runtime changes need migration tests
- Breaking changes require spec version bump
