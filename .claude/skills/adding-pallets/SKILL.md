---
name: adding-pallets
description: Creates and integrates new FRAME pallets into the Moonbeam runtime. Use when adding on-chain functionality, implementing Substrate-native features, creating new storage and extrinsics, or extending runtime capabilities.
---

# Adding a New Pallet

## Contents
- [Pallet Creation Workflow](#pallet-creation-workflow)
- [Checklist](#checklist)
- [Common Patterns](#common-patterns)

## Pallet Creation Workflow

### 1. Create Pallet Structure

```bash
# Create pallet directory
mkdir -p pallets/my-pallet/src
```

### 2. Create Cargo.toml

```toml
# pallets/my-pallet/Cargo.toml
[package]
name = "pallet-my-pallet"
version = "0.1.0"
edition = "2021"
description = "Description of the pallet"

[dependencies]
# Parity
parity-scale-codec = { workspace = true }
scale-info = { workspace = true }

# Substrate
frame-support = { workspace = true }
frame-system = { workspace = true }
sp-runtime = { workspace = true }
sp-std = { workspace = true }

# Benchmarking
frame-benchmarking = { workspace = true, optional = true }

[dev-dependencies]
sp-io = { workspace = true, features = ["std"] }
sp-core = { workspace = true, features = ["std"] }

[features]
default = ["std"]
std = [
    "parity-scale-codec/std",
    "scale-info/std",
    "frame-support/std",
    "frame-system/std",
    "sp-runtime/std",
    "sp-std/std",
]
runtime-benchmarks = [
    "frame-benchmarking/runtime-benchmarks",
    "frame-support/runtime-benchmarks",
    "frame-system/runtime-benchmarks",
]
try-runtime = [
    "frame-support/try-runtime",
    "frame-system/try-runtime",
]
```

### 3. Create Pallet Implementation

```rust
// pallets/my-pallet/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Weight information for extrinsics.
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type Something<T: Config> = StorageValue<_, u32, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Something was done.
        SomethingDone { value: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value is invalid.
        InvalidValue,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn do_something(origin: OriginFor<T>, value: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(value > 0, Error::<T>::InvalidValue);

            Something::<T>::put(value);
            Self::deposit_event(Event::SomethingDone { value });

            Ok(())
        }
    }
}
```

### 4. Create Weight Trait

```rust
// pallets/my-pallet/src/weights.rs
use frame_support::weights::Weight;

pub trait WeightInfo {
    fn do_something() -> Weight;
}

impl WeightInfo for () {
    fn do_something() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}
```

### 5. Create Mock Runtime

```rust
// pallets/my-pallet/src/mock.rs
use crate as pallet_my_pallet;
use frame_support::{derive_impl, parameter_types};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        MyPallet: pallet_my_pallet,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

impl pallet_my_pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
```

### 6. Create Tests

```rust
// pallets/my-pallet/src/tests.rs
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn do_something_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(MyPallet::do_something(RuntimeOrigin::signed(1), 42));
        assert_eq!(MyPallet::something(), Some(42));

        System::assert_last_event(Event::SomethingDone { value: 42 }.into());
    });
}

#[test]
fn do_something_fails_with_zero() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            MyPallet::do_something(RuntimeOrigin::signed(1), 0),
            Error::<Test>::InvalidValue
        );
    });
}
```

### 7. Add to Workspace

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    # ... existing members
    "pallets/my-pallet",
]
```

### 8. Integrate into Runtime

```rust
// runtime/moonbase/lib.rs (and moonbeam, moonriver)

// Add the pallet to construct_runtime!
construct_runtime!(
    pub enum Runtime {
        // ... existing pallets
        MyPallet: pallet_my_pallet = 100, // Choose unique index
    }
);

// Implement Config for the runtime
impl pallet_my_pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_my_pallet::weights::SubstrateWeight<Runtime>;
}
```

### 9. Add Benchmarks (Optional but Recommended)

```rust
// pallets/my-pallet/src/benchmarking.rs
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn do_something() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        do_something(RawOrigin::Signed(caller), 42);

        assert_eq!(Something::<T>::get(), Some(42));
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
```

### 10. Run Benchmarks

```bash
# Generate weights
./scripts/run-benches-for-runtime.sh moonbase release pallet_my_pallet
```

## Checklist

- [ ] Create pallet directory and Cargo.toml
- [ ] Implement pallet with Config, Storage, Events, Errors, Calls
- [ ] Create weight trait with placeholder weights
- [ ] Create mock runtime for testing
- [ ] Write comprehensive tests
- [ ] Add to workspace Cargo.toml
- [ ] Add to all three runtimes (moonbase, moonbeam, moonriver)
- [ ] Implement benchmarks
- [ ] Generate production weights
- [ ] Add any necessary migrations
- [ ] Update runtime version if breaking changes

## Common Patterns

### Storage Types
- `StorageValue<_, T>` - Single value
- `StorageMap<_, Blake2_128Concat, K, V>` - Key-value map
- `StorageDoubleMap<_, ..., K1, K2, V>` - Double key map
- `CountedStorageMap<_, ..., K, V>` - Map with item count

### Origin Checking
- `ensure_signed(origin)?` - Must be signed transaction
- `ensure_root(origin)?` - Must be sudo/governance
- `T::SomeOrigin::ensure_origin(origin)?` - Custom origin

### Weight Patterns
- Use benchmarks for accurate weights
- Consider worst-case scenarios
- Include DB read/write counts
