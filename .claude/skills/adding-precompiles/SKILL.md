---
name: adding-precompiles
description: Creates and integrates EVM precompiled contracts that expose Substrate functionality to the EVM. Use when exposing pallet functionality to Solidity contracts, creating Ethereum-compatible interfaces, implementing standard Ethereum precompiles, or bridging EVM and Substrate state.
---

# Adding a New Precompile

## Contents
- [Precompile Creation Workflow](#precompile-creation-workflow)
- [Checklist](#checklist)
- [Common Patterns](#common-patterns)
- [Precompile Address Conventions](#precompile-address-conventions)

## Precompile Creation Workflow

### 1. Create Precompile Structure

```bash
mkdir -p precompiles/my-precompile/src
```

### 2. Create Cargo.toml

```toml
# precompiles/my-precompile/Cargo.toml
[package]
name = "pallet-evm-precompile-my-precompile"
version = "0.1.0"
edition = "2021"
description = "Precompile for MyPallet functionality"

[dependencies]
# Moonbeam
precompile-utils = { workspace = true }

# Substrate
frame-support = { workspace = true }
frame-system = { workspace = true }
pallet-evm = { workspace = true }
sp-core = { workspace = true }
sp-runtime = { workspace = true }
sp-std = { workspace = true }

# Frontier
fp-evm = { workspace = true }

# The pallet being exposed
pallet-my-pallet = { workspace = true }

[dev-dependencies]
derive_more = { workspace = true }
hex-literal = { workspace = true }
precompile-utils = { workspace = true, features = ["testing"] }
scale-info = { workspace = true }
serde = { workspace = true }
sha3 = { workspace = true }

[features]
default = ["std"]
std = [
    "precompile-utils/std",
    "frame-support/std",
    "frame-system/std",
    "pallet-evm/std",
    "sp-core/std",
    "sp-runtime/std",
    "sp-std/std",
    "fp-evm/std",
    "pallet-my-pallet/std",
]
```

### 3. Create Precompile Implementation

```rust
// precompiles/my-precompile/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_runtime::traits::Dispatchable;
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Precompile address (must be unique, typically 0x800+)
pub const PRECOMPILE_ADDRESS: u64 = 0x0000_0000_0000_0000_0000_0000_0000_0000_0000_0820;

/// Solidity selector for doSomething(uint256)
/// keccak256("doSomething(uint256)")[0:4]
pub const SELECTOR_DO_SOMETHING: u32 = 0x12345678;

pub struct MyPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> MyPrecompile<Runtime>
where
    Runtime: pallet_my_pallet::Config + pallet_evm::Config,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    Runtime::RuntimeCall: From<pallet_my_pallet::Call<Runtime>>,
{
    /// Solidity signature: doSomething(uint256 value)
    #[precompile::public("doSomething(uint256)")]
    fn do_something(handle: &mut impl PrecompileHandle, value: U256) -> EvmResult {
        // Convert U256 to u32 (with bounds check)
        let value: u32 = value
            .try_into()
            .map_err(|_| revert("Value exceeds u32 max"))?;

        // Get the caller's substrate account
        let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

        // Create the dispatch call
        let call = pallet_my_pallet::Call::<Runtime>::do_something { value };

        // Record gas cost based on weight
        let dispatch_info = call.get_dispatch_info();
        RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

        // Emit Ethereum log event
        log1(
            handle.context().address,
            SELECTOR_LOG_SOMETHING_DONE,
            solidity::encode_event_data(value),
        )
        .record(handle)?;

        Ok(())
    }

    /// Solidity signature: getSomething() returns (uint256)
    #[precompile::public("getSomething()")]
    #[precompile::view]
    fn get_something(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        // Read-only call - record storage read cost
        handle.record_db_read::<Runtime>(32)?;

        let value = pallet_my_pallet::Something::<Runtime>::get().unwrap_or_default();
        Ok(U256::from(value))
    }
}

// Event signature: SomethingDone(uint256 indexed value)
const SELECTOR_LOG_SOMETHING_DONE: [u8; 32] = keccak256!("SomethingDone(uint256)");
```

### 4. Create Solidity Interface

```solidity
// precompiles/my-precompile/MyPrecompile.sol
// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @title MyPrecompile Interface
/// @dev Interface for the MyPrecompile precompiled contract
/// @custom:address 0x0000000000000000000000000000000000000820
interface IMyPrecompile {
    /// @dev Emitted when doSomething is called
    /// @param value The value that was set
    event SomethingDone(uint256 indexed value);

    /// @dev Do something with a value
    /// @param value The value to set
    function doSomething(uint256 value) external;

    /// @dev Get the current value
    /// @return The current stored value
    function getSomething() external view returns (uint256);
}
```

### 5. Create Mock Runtime

```rust
// precompiles/my-precompile/src/mock.rs
use super::*;
use frame_support::{derive_impl, parameter_types};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::precompile_set::*;
use sp_core::{H160, U256};
use sp_runtime::BuildStorage;

pub type AccountId = sp_runtime::AccountId32;
pub type Balance = u128;
pub type Block = frame_system::mocking::MockBlock<Runtime>;

frame_support::construct_runtime!(
    pub enum Runtime {
        System: frame_system,
        Balances: pallet_balances,
        Evm: pallet_evm,
        Timestamp: pallet_timestamp,
        MyPallet: pallet_my_pallet,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
}

impl pallet_my_pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// Precompile configuration
pub type Precompiles = PrecompileSetBuilder<
    Runtime,
    (PrecompileAt<AddressU64<0x820>, MyPrecompile<Runtime>>,),
>;

// EVM configuration (simplified for testing)
impl pallet_evm::Config for Runtime {
    // ... EVM config implementation
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(AccountId::from([1u8; 32]), 1_000_000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
```

### 6. Create Tests

```rust
// precompiles/my-precompile/src/tests.rs
use crate::mock::*;
use precompile_utils::testing::*;
use sp_core::H160;

fn precompiles() -> Precompiles {
    PrecompilesValue::get()
}

#[test]
fn test_do_something() {
    new_test_ext().execute_with(|| {
        let alice = H160::from_low_u64_be(1);

        // Call doSomething(42)
        precompiles()
            .prepare_test(alice, H160::from_low_u64_be(0x820), PCall::do_something { value: 42.into() })
            .execute_returns(());

        // Verify the value was set
        assert_eq!(pallet_my_pallet::Something::<Runtime>::get(), Some(42));
    });
}

#[test]
fn test_get_something() {
    new_test_ext().execute_with(|| {
        let alice = H160::from_low_u64_be(1);

        // Set a value first
        pallet_my_pallet::Something::<Runtime>::put(123);

        // Call getSomething()
        precompiles()
            .prepare_test(alice, H160::from_low_u64_be(0x820), PCall::get_something {})
            .execute_returns(U256::from(123));
    });
}

#[test]
fn test_do_something_reverts_on_overflow() {
    new_test_ext().execute_with(|| {
        let alice = H160::from_low_u64_be(1);

        // Try to set a value larger than u32::MAX
        precompiles()
            .prepare_test(
                alice,
                H160::from_low_u64_be(0x820),
                PCall::do_something { value: U256::from(u64::MAX) },
            )
            .execute_reverts(|output| output == b"Value exceeds u32 max");
    });
}
```

### 7. Register in Runtime

```rust
// runtime/moonbase/precompiles.rs
pub type MoonbasePrecompiles<R> = PrecompileSetBuilder<
    R,
    (
        // ... existing precompiles
        PrecompileAt<
            AddressU64<0x820>,
            MyPrecompile<R>,
            CallableByContract<AllExceptXcmTransact<R>>,
        >,
    ),
>;
```

### 8. Add to Workspace

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    # ... existing members
    "precompiles/my-precompile",
]
```

## Checklist

- [ ] Create precompile directory and Cargo.toml
- [ ] Implement precompile with `#[precompile_utils::precompile]` macro
- [ ] Create Solidity interface file
- [ ] Create mock runtime for testing
- [ ] Write comprehensive tests
- [ ] Add to workspace Cargo.toml
- [ ] Register precompile in all runtimes
- [ ] Document the precompile address
- [ ] Add TypeScript helpers in `/test/helpers/`

## Common Patterns

### Selector Calculation
```rust
// keccak256("functionName(type1,type2)")[0:4]
const SELECTOR: u32 = 0x12345678;

// Using the macro
const SELECTOR: [u8; 32] = keccak256!("FunctionName(uint256,address)");
```

### Gas Recording
```rust
// For storage reads
handle.record_db_read::<Runtime>(size)?;

// For computation
handle.record_cost(gas_amount)?;
```

### Dispatch Calls
```rust
// With origin
RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

// Root call
RuntimeHelper::<Runtime>::try_dispatch(handle, frame_support::dispatch::RawOrigin::Root.into(), call, 0)?;
```

### Event Logging
```rust
// log0 - no topics
log0(address, data).record(handle)?;

// log1 - one indexed topic
log1(address, topic0, data).record(handle)?;

// log2, log3, log4 - more indexed topics
```

### Return Types
- `EvmResult` - No return value (void)
- `EvmResult<U256>` - Returns uint256
- `EvmResult<Address>` - Returns address
- `EvmResult<(U256, Address)>` - Returns tuple

## Precompile Address Conventions

| Range       | Usage                         |
|-------------|-------------------------------|
| 0x01-0x09   | Standard Ethereum precompiles |
| 0x100-0x7FF | Frontier precompiles          |
| 0x800-0x8FF | Moonbeam custom precompiles   |
| 0x900+      | Reserved for future use       |

Always check `/runtime/*/precompiles.rs` for existing address assignments.
