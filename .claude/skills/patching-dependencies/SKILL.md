---
name: patching-dependencies
description: Patches Cargo dependencies to use local checkouts or different branches. Use when testing changes across multiple repositories, debugging dependency issues, or developing against local polkadot-sdk/frontier/moonkit forks.
---

# Patching Dependencies

## Contents
- [Overview](#overview)
- [Using Diener](#using-diener)
- [Manual Patching](#manual-patching)
- [Common Scenarios](#common-scenarios)
- [Reverting Patches](#reverting-patches)

## Overview

Moonbeam depends on multiple external repositories that must stay in sync:
- `polkadot-sdk` (moonbeam-foundation/polkadot-sdk)
- `frontier` (moonbeam-foundation/frontier)
- `moonkit` (moonbeam-foundation/moonkit)

When developing features that span multiple repositories, you need to patch dependencies to point to local checkouts or different branches.

## Using Diener

Diener is Parity's tool for managing Polkadot SDK dependencies.

### Installation

```bash
# Install from git (recommended - crates.io version is outdated)
cargo install --git https://github.com/paritytech/diener

# Or from crates.io (may not support latest Rust editions)
cargo install diener
```

### Patch to Local Checkout

Redirect all polkadot-sdk dependencies to a local path:

```bash
# Patch polkadot-sdk dependencies to local checkout
diener patch --crates-to-patch ../polkadot-sdk --target Cargo.toml
```

This adds `[patch]` sections to the workspace `Cargo.toml` for each crate found in the local checkout.

### Update Branch/Tag

Change all dependencies to point to a different branch:

```bash
# Update to a specific branch
diener update --branch moonbeam-polkadot-stable2506

# Update to a specific tag
diener update --tag polkadot-stable2407

# Update to a specific commit
diener update --rev abc123def
```

## Manual Patching

For finer control, add patch sections manually to `Cargo.toml`:

### Patch to Local Path

```toml
[patch."https://github.com/moonbeam-foundation/polkadot-sdk"]
# Patch specific crates to local paths
frame-support = { path = "../polkadot-sdk/substrate/frame/support" }
sp-runtime = { path = "../polkadot-sdk/substrate/primitives/runtime" }
```

### Patch to Different Git Source

```toml
[patch."https://github.com/moonbeam-foundation/polkadot-sdk"]
# Patch to a different branch
frame-support = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", branch = "my-feature-branch" }

# Patch to a specific commit
sp-runtime = { git = "https://github.com/moonbeam-foundation/polkadot-sdk", rev = "abc123" }
```

### Patch Frontier

```toml
[patch."https://github.com/moonbeam-foundation/frontier"]
fp-evm = { path = "../frontier/primitives/evm" }
pallet-evm = { path = "../frontier/frame/evm" }
```

### Patch Moonkit

```toml
[patch."https://github.com/moonbeam-foundation/moonkit"]
nimbus-primitives = { path = "../moonkit/primitives/nimbus-primitives" }
pallet-author-inherent = { path = "../moonkit/pallets/author-inherent" }
```

## Common Scenarios

### Testing polkadot-sdk Changes Locally

```bash
# 1. Clone polkadot-sdk if not already
git clone https://github.com/moonbeam-foundation/polkadot-sdk ../polkadot-sdk
cd ../polkadot-sdk
git checkout moonbeam-polkadot-stable2506

# 2. Make your changes
# ...

# 3. Patch moonbeam to use local checkout
cd ../moonbeam
diener patch --crates-to-patch ../polkadot-sdk --target Cargo.toml

# 4. Build and test
cargo build --release
```

### Upgrading polkadot-sdk Version

```bash
# 1. Update all dependencies to new branch
diener update --branch moonbeam-polkadot-stable2507

# 2. Check for breaking changes
cargo check 2>&1 | head -100

# 3. Fix any compilation errors
# ...

# 4. Run tests
cargo test
```

### Cross-Repository Feature Development

When a feature spans moonbeam, frontier, and polkadot-sdk:

```bash
# 1. Set up local checkouts
git clone https://github.com/moonbeam-foundation/polkadot-sdk ../polkadot-sdk
git clone https://github.com/moonbeam-foundation/frontier ../frontier

# 2. Create feature branches in each repo
cd ../polkadot-sdk && git checkout -b feature/my-feature
cd ../frontier && git checkout -b feature/my-feature
cd ../moonbeam && git checkout -b feature/my-feature

# 3. Patch moonbeam to use local checkouts
cat >> Cargo.toml << 'EOF'

[patch."https://github.com/moonbeam-foundation/polkadot-sdk"]
# Add specific crates you're modifying
frame-support = { path = "../polkadot-sdk/substrate/frame/support" }

[patch."https://github.com/moonbeam-foundation/frontier"]
pallet-evm = { path = "../frontier/frame/evm" }
EOF

# 4. Develop and test across all repos
cargo build --release
```

### Debugging Dependency Issues

When encountering version conflicts:

```bash
# 1. Check dependency tree
cargo tree -p frame-support

# 2. Find duplicate versions
cargo tree -d

# 3. Identify the source of conflicts
cargo tree -i frame-support
```

## Reverting Patches

### Remove Diener Patches

```bash
# Remove all [patch] sections from Cargo.toml
# This is manual - diener doesn't have an unpatch command

# Option 1: Git restore
git checkout Cargo.toml

# Option 2: Manual removal
# Delete all [patch."..."] sections from Cargo.toml
```

### Clean Cargo Cache

After reverting patches, clean the build:

```bash
cargo clean
cargo update
```

## Important Notes

1. **Never commit local path patches** - They won't work in CI or for other developers
2. **Keep dependencies in sync** - All polkadot-sdk crates must point to the same branch/commit
3. **Update Cargo.lock** - After patching, run `cargo update` to refresh the lock file
4. **Check CI compatibility** - Ensure patches use git URLs, not local paths, before pushing

## Dependency Tree Reference

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
```

All these must point to compatible versions to avoid conflicts.
