# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Moonbeam is an Ethereum-compatible parachain built with Polkadot-SDK. It's a Rust-based blockchain project that enables Ethereum-style smart contracts on Polkadot/Kusama networks.

## Essential Commands

### Building
```bash
# Build the Moonbeam node (optimized release build)
cargo build --release

# Build specific runtime only
cargo build --release -p moonbeam-runtime

# Build TypeScript packages
pnpm i
pnpm build
```

### Testing
```bash
# Run all Rust tests
cargo test

# Run specific pallet tests
cargo test -p pallet-parachain-staking

# Run TypeScript integration tests (from test directory)
cd test
pnpm moonwall test dev_moonbase  # Development tests
pnpm moonwall test smoke_moonbase # Smoke tests

# Run a single test file
pnpm moonwall test dev_moonbase --grep "test-description"
```

### Code Quality
```bash
# Rust linting and formatting
cargo clippy --release --workspace
cargo fmt -- --check

# TypeScript linting
pnpm check  # Runs Biome linter
```

### Running Development Node
```bash
# Using built binary
./target/release/moonbeam --dev --alice --sealing 6000 --rpc-port 9944

# Using Docker
docker run --network="host" moonbeamfoundation/moonbeam:v0.46.0 --dev --alice --sealing 6000 --rpc-port 9944
```

### Runtime Benchmarking

**Prerequisites**: Install `frame-omni-bencher` from [crates.io](https://crates.io/crates/frame-omni-bencher) or [Polkadot SDK](https://github.com/paritytech/polkadot-sdk/tree/b45f89c51fbd58e984e5e013992dd26715cb8bdc/substrate/utils/frame/omni-bencher)

```bash
# Run runtime benchmarks (may need to update frame-omni-bencher path in script)
./scripts/run-benches-for-runtime.sh moonbase release

# The script uses frame-omni-bencher with these key parameters:
# --steps=50 --repeat=20 --wasm-execution=compiled
```

## Architecture Overview

### Runtime Architecture
The runtime is the on-chain logic compiled to WASM. Moonbeam has three runtime variants:
- **moonbeam**: Production runtime for Polkadot
- **moonriver**: Production runtime for Kusama
- **moonbase**: TestNet runtime for Westend

Key architectural patterns:
1. **Pallet Structure**: Custom logic is organized into pallets (e.g., `pallet-parachain-staking`)
2. **Precompiles**: EVM precompiled contracts in `/precompiles` provide native Substrate functionality to EVM
3. **XCM Integration**: Cross-chain messaging through XCM configuration in runtime

### Client Architecture
The client (`/node`) implements:
- **RPC Layer**: Custom RPC methods for Ethereum compatibility
- **EVM Tracing**: Debug and trace EVM execution
- **Block Production**: Collator logic for parachain block production

### Testing Architecture
Tests are split into:
- **Rust Unit Tests**: In each pallet/module
- **TypeScript Integration Tests**: In `/test` using Moonwall framework
- **Smoke Tests**: Minimal tests for quick validation
- **Dev Tests**: Comprehensive feature testing

### Cross-Component Communication
1. **Runtime ↔ EVM**: Through precompiles and pallet-evm
2. **Client ↔ Runtime**: Via runtime APIs defined in `runtime/common/src/apis`
3. **Substrate ↔ Ethereum**: Through frontier pallets and custom RPC

## Network Configuration

| Network        | Chain ID | Runtime   | Purpose           |
| -------------- | -------- | --------- | ----------------- |
| Moonbeam       | 1284     | moonbeam  | Polkadot MainNet  |
| Moonriver      | 1285     | moonriver | Kusama parachain  |
| Moonbase Alpha | 1287     | moonbase  | Public TestNet    |
| Development    | 1281     | moonbase  | Local development |

## Key Development Patterns

### Adding New Functionality
1. **New Pallet**: Create in `/pallets`, add to runtime's `construct_runtime!`
2. **New Precompile**: Create in `/precompiles`, register in `precompiles.rs`
3. **Runtime API**: Define in `runtime/common/src/apis`, implement in runtime
4. **RPC Method**: Add to `/client/rpc`, expose in node service

### Testing Patterns
- Use `ExtBuilder` pattern for pallet unit tests
- Use Moonwall's `DevModeContext` for integration tests
- Test both Substrate and Ethereum interfaces when applicable

### Version Management
- Spec version in runtime when breaking changes
- Client version for node releases
- Precompile addresses are immutable once deployed

## Development Workflow

### Before Committing
1. Run `cargo fmt`
2. Run `cargo clippy --release`
3. Run `pnpm check` for TypeScript
4. Ensure tests pass for modified components

### PR Requirements
- Add appropriate label: `B7-runtimenoteworthy`, `B5-clientnoteworthy`, or `B0-silent`
- Runtime changes need migration tests if applicable
- Breaking changes require runtime version bump
