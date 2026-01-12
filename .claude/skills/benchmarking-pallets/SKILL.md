---
name: benchmarking-pallets
description: Creates and runs benchmarks for pallets and precompiles to generate accurate weight functions for the Moonbeam runtime. Use when adding new extrinsics, creating precompiles, optimizing functionality, updating weights after logic changes, or validating weight accuracy.
---

# Benchmarking

## Contents
- [Benchmarking Overview](#benchmarking-overview)
- [Pallet Benchmarking](#pallet-benchmarking)
- [Running Benchmarks](#running-benchmarks)
- [Generated Weight File](#generated-weight-file)
- [Precompile Benchmarking](#precompile-benchmarking)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Benchmarking Overview

### Why Benchmarking Matters

- **Block Weight Limits**: Ensures blocks don't exceed computational limits
- **Fee Accuracy**: Users pay proportional to actual resource usage
- **DoS Prevention**: Prevents underpriced operations from being exploited
- **Network Stability**: Predictable block production times

### Weight Components

```rust
Weight {
    ref_time: u64,      // Computational time
    proof_size: u64,    // Storage proof size
}
```

## Pallet Benchmarking

### 1. Add Benchmark Feature

```toml
# pallets/my-pallet/Cargo.toml
[dependencies]
frame-benchmarking = { workspace = true, optional = true }

[features]
runtime-benchmarks = [
    "frame-benchmarking/runtime-benchmarks",
    "frame-support/runtime-benchmarks",
    "frame-system/runtime-benchmarks",
]
```

### 2. Create Benchmark Module

```rust
// pallets/my-pallet/src/benchmarking.rs
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    /// Benchmark for `do_something` extrinsic
    #[benchmark]
    fn do_something() {
        // Setup: Create any required state
        let caller: T::AccountId = whitelisted_caller();
        let value = 42u32;

        // Optional: Fund the account if needed
        // T::Currency::make_free_balance_be(&caller, 1_000_000_000_000u128.into());

        #[extrinsic_call]
        do_something(RawOrigin::Signed(caller.clone()), value);

        // Verify: Assert the expected state change
        assert_eq!(Something::<T>::get(), Some(value));
    }

    /// Benchmark with linear complexity
    #[benchmark]
    fn do_something_complex(n: Linear<1, 100>) {
        let caller: T::AccountId = whitelisted_caller();

        // Setup scales with n
        for i in 0..n {
            SomeStorage::<T>::insert(i, i);
        }

        #[extrinsic_call]
        do_something_complex(RawOrigin::Signed(caller), n);

        assert!(SomeStorage::<T>::iter().count() as u32 >= n);
    }

    /// Benchmark for storage-heavy operation
    #[benchmark]
    fn clear_storage(n: Linear<1, 1000>) {
        let caller: T::AccountId = whitelisted_caller();

        // Populate storage
        for i in 0..n {
            SomeStorage::<T>::insert(i, vec![0u8; 100]);
        }

        #[extrinsic_call]
        clear_storage(RawOrigin::Signed(caller), n);

        assert_eq!(SomeStorage::<T>::iter().count(), 0);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
```

### 3. Define Weight Trait

```rust
// pallets/my-pallet/src/weights.rs
use frame_support::weights::Weight;

pub trait WeightInfo {
    fn do_something() -> Weight;
    fn do_something_complex(n: u32) -> Weight;
    fn clear_storage(n: u32) -> Weight;
}

/// Placeholder weights for development
impl WeightInfo for () {
    fn do_something() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn do_something_complex(n: u32) -> Weight {
        Weight::from_parts(10_000 + n as u64 * 1_000, 0)
    }

    fn clear_storage(n: u32) -> Weight {
        Weight::from_parts(10_000 + n as u64 * 5_000, 0)
    }
}
```

### 4. Include in Pallet

```rust
// pallets/my-pallet/src/lib.rs
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    #[pallet::weight(T::WeightInfo::do_something())]
    pub fn do_something(origin: OriginFor<T>, value: u32) -> DispatchResult {
        // ...
    }

    #[pallet::call_index(1)]
    #[pallet::weight(T::WeightInfo::do_something_complex(*n))]
    pub fn do_something_complex(origin: OriginFor<T>, n: u32) -> DispatchResult {
        // ...
    }
}
```

### 5. Add to Runtime Benchmarks

```rust
// runtime/moonbase/lib.rs
#[cfg(feature = "runtime-benchmarks")]
mod benches {
    frame_benchmarking::define_benchmarks!(
        // Existing benchmarks...
        [pallet_my_pallet, MyPallet]
    );
}
```

## Running Benchmarks

### Using the Script

```bash
# Run all benchmarks for a runtime
./scripts/run-benches-for-runtime.sh moonbase release

# Run specific pallet benchmarks
./scripts/run-benches-for-runtime.sh moonbase release pallet_my_pallet
```

### Manual Benchmark Execution

```bash
# Build with benchmarks
cargo build --release --features runtime-benchmarks

# Run frame-omni-bencher
frame-omni-bencher v1 benchmark pallet \
    --runtime target/release/wbuild/moonbase-runtime/moonbase_runtime.wasm \
    --pallet pallet_my_pallet \
    --extrinsic "*" \
    --steps 50 \
    --repeat 20 \
    --wasm-execution compiled \
    --output runtime/moonbase/src/weights/pallet_my_pallet.rs \
    --template benchmarking/frame-weight-template.hbs
```

### Benchmark Parameters

| Parameter          | Purpose                 | Typical Value |
|--------------------|-------------------------|---------------|
| `--steps`          | Number of sample points | 50            |
| `--repeat`         | Repetitions per step    | 20            |
| `--heap-pages`     | WASM heap size          | 4096          |
| `--wasm-execution` | Execution mode          | compiled      |

## Generated Weight File

The benchmark generates a weights file like:

```rust
// runtime/moonbase/src/weights/pallet_my_pallet.rs
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_my_pallet::WeightInfo for SubstrateWeight<T> {
    fn do_something() -> Weight {
        Weight::from_parts(12_345_000, 3456)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn do_something_complex(n: u32) -> Weight {
        Weight::from_parts(10_000_000, 1000)
            .saturating_add(Weight::from_parts(500_000, 100).saturating_mul(n as u64))
            .saturating_add(T::DbWeight::get().reads(1 + n as u64))
            .saturating_add(T::DbWeight::get().writes(n as u64))
    }
}
```

## Precompile Benchmarking

### 1. Create Benchmark Module

```rust
// precompiles/my-precompile/src/benchmarking.rs
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn do_something() {
        let caller = H160::from_low_u64_be(1);
        let value = U256::from(42);

        #[block]
        {
            MyPrecompile::<T>::do_something(caller, value).unwrap();
        }
    }
}
```

### 2. Precompile Gas Costs

```rust
// In the precompile implementation
const BASE_GAS_COST: u64 = 200;
const PER_ITEM_GAS_COST: u64 = 100;

fn gas_cost(items: u64) -> u64 {
    BASE_GAS_COST.saturating_add(PER_ITEM_GAS_COST.saturating_mul(items))
}
```

## Best Practices

### 1. Worst Case Scenarios

Always benchmark the worst case:

```rust
#[benchmark]
fn bounded_operation(n: Linear<1, MAX_ITEMS>) {
    // Setup with maximum complexity
    for i in 0..n {
        HeavyStorage::<T>::insert(i, vec![0u8; MAX_SIZE]);
    }

    #[extrinsic_call]
    bounded_operation(RawOrigin::Root, n);
}
```

### 2. Include All DB Operations

```rust
// Weight should reflect actual DB access
Weight::from_parts(compute_time, proof_size)
    .saturating_add(T::DbWeight::get().reads(read_count))
    .saturating_add(T::DbWeight::get().writes(write_count))
```

### 3. Test Benchmarks

```bash
# Run benchmark tests to verify they work
cargo test -p pallet-my-pallet --features runtime-benchmarks
```

### 4. Validate Results

After generating weights:
- Compare with expected complexity
- Check for outliers
- Verify linear relationships match code

## Checklist

- [ ] Add runtime-benchmarks feature to Cargo.toml
- [ ] Create benchmarking.rs with all extrinsics
- [ ] Define WeightInfo trait in weights.rs
- [ ] Use weight trait in pallet calls
- [ ] Add pallet to runtime benchmark list
- [ ] Run benchmarks and generate weight file
- [ ] Copy generated weights to runtime
- [ ] Verify benchmark tests pass
- [ ] Review generated weights for accuracy

## Troubleshooting

### Benchmark Fails

```bash
# Check benchmark compiles
cargo check --release -p moonbase-runtime --features runtime-benchmarks

# Run with more verbose output
RUST_LOG=runtime::benchmark=trace frame-omni-bencher ...
```

### Unrealistic Weights

- Check benchmark setup creates realistic state
- Verify worst-case is actually benchmarked
- Look for missing storage operations

### Memory Issues

```bash
# Increase heap pages
--heap-pages 8192
```
