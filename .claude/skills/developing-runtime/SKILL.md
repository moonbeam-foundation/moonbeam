---
name: developing-runtime
description: Develops and modifies the Moonbeam runtime using general patterns and best practices. Use when modifying runtime configuration, adding pallets to the runtime, implementing migrations, updating runtime APIs, or managing runtime versioning.
---

# Runtime Development

## Contents
- [Runtime Structure](#runtime-structure)
- [Adding a Pallet to Runtime](#adding-a-pallet-to-runtime)
- [Runtime Migrations](#runtime-migrations)
- [Runtime APIs](#runtime-apis)
- [Runtime Versioning](#runtime-versioning)
- [Common Runtime Patterns](#common-runtime-patterns)
- [Building and Testing](#building-and-testing)

## Runtime Structure

### Three Runtime Variants

| Runtime   | Chain ID | Network  | Location             |
|-----------|----------|----------|----------------------|
| moonbase  | 1287     | TestNet  | `runtime/moonbase/`  |
| moonbeam  | 1284     | Polkadot | `runtime/moonbeam/`  |
| moonriver | 1285     | Kusama   | `runtime/moonriver/` |

### Key Files Per Runtime

```
runtime/moonbase/
├── lib.rs                    # Main runtime definition
├── precompiles.rs           # Precompile registry
├── xcm_config.rs            # XCM configuration
├── asset_config.rs          # Asset configuration
├── governance/              # Governance structs
├── weights/                 # Benchmark weights
├── migrations.rs            # Runtime migrations
└── runtime_params.rs        # Runtime parameters
```

### Shared Code

```
runtime/common/src/
├── lib.rs                   # Common exports
├── apis.rs                  # Runtime API implementations
├── types.rs                 # Shared types
├── migrations.rs            # Common migrations
└── impl_*.rs               # Trait implementations
```

## Adding a Pallet to Runtime

### 1. Add Dependency

```toml
# runtime/moonbase/Cargo.toml
[dependencies]
pallet-my-pallet = { workspace = true }

[features]
std = [
    # ...
    "pallet-my-pallet/std",
]
runtime-benchmarks = [
    # ...
    "pallet-my-pallet/runtime-benchmarks",
]
try-runtime = [
    # ...
    "pallet-my-pallet/try-runtime",
]
```

### 2. Configure the Pallet

```rust
// runtime/moonbase/lib.rs

// Parameter types
parameter_types! {
    pub const MyPalletParameter: u32 = 100;
}

// Implement Config
impl pallet_my_pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_my_pallet::weights::SubstrateWeight<Runtime>;
    type MyParameter = MyPalletParameter;
}
```

### 3. Add to construct_runtime!

```rust
// runtime/moonbase/lib.rs
construct_runtime!(
    pub enum Runtime {
        // System pallets
        System: frame_system = 0,
        // ...

        // Custom pallets (use unique indices)
        MyPallet: pallet_my_pallet = 100,
    }
);
```

### 4. Add Benchmarks (if applicable)

```rust
// runtime/moonbase/lib.rs
#[cfg(feature = "runtime-benchmarks")]
mod benches {
    frame_benchmarking::define_benchmarks!(
        // ...existing benchmarks
        [pallet_my_pallet, MyPallet]
    );
}
```

## Runtime Migrations

### Migration Lifecycle

Moonbeam follows a simple migration lifecycle:

1. **Add migration before release**: Write the migration and register it
2. **Deploy**: Migration runs once during the runtime upgrade
3. **Remove migration before next release**: Delete the migration code

Migrations are one-shot: they run once and are removed from the codebase.

### When Migrations Are Needed

- Storage layout changes
- Pallet index changes
- Configuration changes that affect storage
- Data transformations

### Writing a Migration

```rust
// runtime/common/src/migrations.rs

/// Migration to update storage format.
/// Added in runtime XXXX, remove after deployment to all networks.
pub struct MigrateStorageFormat<T>(PhantomData<T>);

impl<T: pallet_my_pallet::Config> OnRuntimeUpgrade for MigrateStorageFormat<T> {
    fn on_runtime_upgrade() -> Weight {
        log::info!(target: "migration", "Running MigrateStorageFormat");

        let count = migrate_storage::<T>();

        log::info!(target: "migration", "Migrated {} items", count);
        T::DbWeight::get().reads_writes(count, count)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
        let count = OldStorage::<T>::iter().count() as u32;
        Ok(count.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), DispatchError> {
        let old_count: u32 = Decode::decode(&mut &state[..]).unwrap();
        let new_count = NewStorage::<T>::iter().count() as u32;
        ensure!(old_count == new_count, "Migration count mismatch");
        Ok(())
    }
}
```

### Registering Migrations

```rust
// runtime/moonbase/lib.rs

/// Migrations to run on runtime upgrade.
/// Remove after deployment.
type MoonbaseMigrations = (
    migrations::MigrateStorageFormat<Runtime>,
);

pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    MoonbaseMigrations,
>;
```

Each runtime has its own migrations type:
- `MoonbaseMigrations` in `runtime/moonbase/lib.rs`
- `MoonriverMigrations` in `runtime/moonriver/lib.rs`
- `MoonbeamMigrations` in `runtime/moonbeam/lib.rs`

### After Deployment

Once migrations have run on all networks, clean up:

```rust
type MoonbaseMigrations = ();
```

Then remove the migration code from `runtime/common/src/migrations.rs`.

## Runtime APIs

### Implementing a Runtime API

```rust
// runtime/common/src/apis.rs
impl_runtime_apis! {
    impl my_pallet_runtime_api::MyPalletApi<Block> for Runtime {
        fn get_something() -> Option<u32> {
            pallet_my_pallet::Something::<Runtime>::get()
        }

        fn calculate_fee(amount: u128) -> u128 {
            // Computation that shouldn't be an extrinsic
            pallet_my_pallet::Pallet::<Runtime>::calculate_fee(amount)
        }
    }
}
```

### Defining the API

```rust
// primitives/rpc/my-api/src/lib.rs
sp_api::decl_runtime_api! {
    pub trait MyPalletApi {
        fn get_something() -> Option<u32>;
        fn calculate_fee(amount: u128) -> u128;
    }
}
```

## Runtime Versioning

### When to Bump Versions

| Change Type               | Bump                  |
|---------------------------|-----------------------|
| Breaking storage change   | `spec_version`        |
| New pallet                | `spec_version`        |
| Runtime logic change      | `spec_version`        |
| Transaction format change | `transaction_version` |
| State version change      | `state_version`       |

### Version Location

```rust
// runtime/moonbase/lib.rs
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("moonbase"),
    impl_name: create_runtime_str!("moonbase"),
    authoring_version: 4,
    spec_version: 3200,  // Bump for breaking changes
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 2,
    state_version: 1,
};
```

## Common Runtime Patterns

### Configurable Origins

```rust
// Define custom origin
pub type EnsureRootOrHalfCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 1, 2>,
>;

impl pallet_my_pallet::Config for Runtime {
    type AdminOrigin = EnsureRootOrHalfCouncil;
}
```

### Currency Configuration

```rust
impl pallet_my_pallet::Config for Runtime {
    type Currency = Balances;
    type MinimumDeposit = ConstU128<1_000_000_000_000>; // 1 GLMR
}
```

### Event Filtering

```rust
// Filter events for specific pallets
impl frame_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    // Events from all pallets
}
```

### Weight Configuration

```rust
impl pallet_my_pallet::Config for Runtime {
    // Use benchmarked weights
    type WeightInfo = pallet_my_pallet::weights::SubstrateWeight<Runtime>;

    // Or use placeholder for development
    // type WeightInfo = ();
}
```

## Building and Testing

### Build Runtime

```bash
# Build specific runtime
cargo build --release -p moonbase-runtime

# Build all runtimes
cargo build --release -p moonbeam-runtime -p moonriver-runtime -p moonbase-runtime
```

### Test Runtime

```bash
# Run runtime tests
cargo test -p moonbase-runtime

# Test migrations with try-runtime
cargo build --release --features try-runtime
try-runtime --runtime target/release/wbuild/moonbase-runtime/moonbase_runtime.wasm \
    on-runtime-upgrade --checks all \
    live --uri wss://wss.api.moonbase.moonbeam.network
```

### Generate Weights

```bash
# Run benchmarks for a pallet
./scripts/run-benches-for-runtime.sh moonbase release pallet_my_pallet
```

## Checklist for Runtime Changes

- [ ] Add pallet to all three runtimes (moonbase, moonbeam, moonriver)
- [ ] Configure pallet parameters appropriately per network
- [ ] Add to Cargo.toml with all feature flags
- [ ] Add to construct_runtime! with unique index
- [ ] Implement required Config traits
- [ ] Add benchmarks and generate weights
- [ ] Write migration if storage changes
- [ ] Bump spec_version
- [ ] Update runtime API if needed
- [ ] Test with try-runtime
- [ ] Run full test suite
