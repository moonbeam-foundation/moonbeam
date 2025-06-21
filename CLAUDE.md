# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Build Commands
```bash
# Build debug version
cargo build

# Build release version (required for running nodes)
cargo build --release

# Build with specific features
cargo build --release --features metadata-hash

# Alternative using Makefile
make build-release
```

### Run Commands
```bash
# Run development node with automatic block sealing
make start-dev

# Run with manual block sealing
./target/release/moonbeam --chain moonbase-dev --sealing manual

# Run specific networks
./target/release/moonbeam --chain moonriver-dev
./target/release/moonbeam --chain moonbeam-dev
```

### Test Commands
```bash
# Rust unit tests
cargo test

# TypeScript integration tests (requires setup first)
cd test && pnpm install
pnpm moonwall test dev_moonbase

# Run specific test suites
pnpm moonwall test dev_moonbeam
pnpm moonwall test dev_moonriver
pnpm moonwall test zombie_moonbase

# Run a single test file
pnpm moonwall test dev_moonbase --grep "test-name"
```

### Code Quality Commands
```bash
# Format Rust code
cargo fmt

# Check Rust code formatting
cargo fmt -- --check

# Run clippy linter
cargo clippy --release --workspace

# Format/lint TypeScript code
cd test && pnpm check
cd test && pnpm check:fix  # Auto-fix issues
```

## High-Level Architecture

Moonbeam is an Ethereum-compatible blockchain built as a Polkadot/Kusama parachain using Substrate. Key architectural components:

### Multi-Runtime Architecture
- **Moonbeam**: Production network on Polkadot
- **Moonriver**: Canary network on Kusama  
- **Moonbase Alpha**: TestNet for development
- Each network has its own runtime in `/runtime/[network]/`

### Core Components

1. **Runtime (`/runtime/`)**: On-chain logic compiled to WASM
   - Contains pallets (Substrate modules) configuration
   - Defines transaction types, storage, and state transitions
   - Common code shared in `/runtime/common/`

2. **Pallets (`/pallets/`)**: Custom Substrate modules for Moonbeam features
   - `pallet-moonbeam-orbiters`: Collator management
   - `pallet-asset-manager`: Cross-chain asset handling
   - `pallet-xcm-transactor`: Cross-chain messaging

3. **Precompiles (`/precompiles/`)**: Ethereum-style precompiled contracts
   - Bridge Substrate functionality to Ethereum API
   - Located at specific addresses (e.g., 0x800+ range)
   - Enable calling Substrate pallets via Ethereum transactions

4. **Node (`/node/`)**: Client implementation
   - RPC endpoints for Ethereum and Substrate APIs
   - Block production and validation logic
   - Network and consensus handling

5. **Tests (`/test/`)**: Comprehensive integration tests using Moonwall
   - `suites/dev/`: Development environment tests
   - `suites/tracing/`: EVM tracing tests
   - `suites/zombie/`: Multi-node network tests

### Key Development Concepts

- **Ethereum Compatibility**: Full EVM implementation via Frontier pallets
- **Dual Transaction Types**: Supports both Substrate extrinsics and Ethereum transactions
- **XCM Integration**: Cross-chain messaging for asset transfers between parachains
- **Nimbus Consensus**: Block authoring system for parachain collators

### Development Requirements

- **Node.js**: Version 22 is required. Use `nvm use 22` before running tests.
- **Rust**: See rust-toolchain file for required version

### Development Accounts

Pre-funded accounts for testing (mnemonic: `bottom drive obey lake curtain smoke basket hold race lonely fit walk`):
- **Alith**: 0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac
- **Baltathar**: 0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0
- See README.md for complete list

### Git Workflow

- Development branches: `<name>-<feature>` (e.g., `gav-my-feature`)
- Hotfix branches: `perm-runtime-XXXX` for runtime fixes
- All PRs target `master` branch