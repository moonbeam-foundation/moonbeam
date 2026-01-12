---
name: writing-migrations
description: Writes and tests runtime migrations for state transitions in Moonbeam. Use when handling storage layout changes, renaming or removing storage items, data format changes, pallet index changes, or storage key modifications.
---

# Runtime Migrations

## Contents
- [Migration Types](#migration-types)
- [Writing Migrations](#writing-migrations)
- [Registering Migrations](#registering-migrations)
- [Testing Migrations](#testing-migrations)
- [Best Practices](#best-practices)
- [Common Issues](#common-issues)

## Migration Types

### OnRuntimeUpgrade Migrations

Standard migrations that run during runtime upgrade.

```rust
pub struct MigrateV1ToV2<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateV1ToV2<T> {
    fn on_runtime_upgrade() -> Weight {
        // Migration logic
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
        // Pre-migration checks
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), DispatchError> {
        // Post-migration verification
    }
}
```

### Lazy Migrations

Migrations that run gradually over multiple blocks.

```rust
// pallets/moonbeam-lazy-migrations/
pub struct LazyMigration<T> {
    cursor: Option<Vec<u8>>,
    _marker: PhantomData<T>,
}

impl<T: Config> LazyMigration<T> {
    pub fn step(&mut self, limit: u32) -> (u32, bool) {
        // Process up to `limit` items
        // Return (processed, is_finished)
    }
}
```

## Writing Migrations

### Basic Migration Structure

```rust
// runtime/common/src/migrations.rs
use frame_support::{
    migration::storage_key_iter,
    pallet_prelude::*,
    traits::OnRuntimeUpgrade,
    weights::Weight,
};

pub mod v2 {
    use super::*;

    // Old storage format
    mod v1 {
        use super::*;

        #[frame_support::storage_alias]
        pub type OldStorage<T: Config> =
            StorageMap<Pallet<T>, Blake2_128Concat, AccountId, OldData>;

        #[derive(Decode)]
        pub struct OldData {
            pub value: u32,
        }
    }

    pub struct MigrateToV2<T>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for MigrateToV2<T> {
        fn on_runtime_upgrade() -> Weight {
            let on_chain_version = Pallet::<T>::on_chain_storage_version();

            if on_chain_version < 2 {
                log::info!(target: "migration", "Migrating pallet to v2");

                let mut count = 0u64;

                // Iterate over old storage
                for (key, old_data) in v1::OldStorage::<T>::drain() {
                    // Transform to new format
                    let new_data = NewData {
                        value: old_data.value,
                        extra_field: Default::default(),
                    };

                    // Write to new storage
                    NewStorage::<T>::insert(key, new_data);
                    count += 1;
                }

                // Update storage version
                StorageVersion::new(2).put::<Pallet<T>>();

                log::info!(target: "migration", "Migrated {} items", count);

                T::DbWeight::get().reads_writes(count + 1, count + 1)
            } else {
                log::info!(target: "migration", "No migration needed");
                T::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
            let count = v1::OldStorage::<T>::iter().count() as u32;
            log::info!(target: "migration", "Pre-upgrade: {} items to migrate", count);
            Ok(count.encode())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), DispatchError> {
            let old_count: u32 = Decode::decode(&mut &state[..])
                .map_err(|_| "Failed to decode state")?;

            let new_count = NewStorage::<T>::iter().count() as u32;

            ensure!(
                old_count == new_count,
                "Migration count mismatch: old={}, new={}",
                old_count,
                new_count
            );

            let version = Pallet::<T>::on_chain_storage_version();
            ensure!(version >= 2, "Storage version not updated");

            log::info!(target: "migration", "Post-upgrade: {} items migrated", new_count);
            Ok(())
        }
    }
}
```

### Removing Storage

```rust
pub struct RemoveDeprecatedStorage<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for RemoveDeprecatedStorage<T> {
    fn on_runtime_upgrade() -> Weight {
        // Clear the deprecated storage
        let removed = DeprecatedStorage::<T>::clear(u32::MAX, None);

        log::info!(
            target: "migration",
            "Removed {} deprecated storage items",
            removed.unique
        );

        T::DbWeight::get().writes(removed.unique as u64)
    }
}
```

### Killing Storage Prefix

```rust
use frame_support::migration::clear_storage_prefix;

pub struct KillOldPalletStorage;

impl OnRuntimeUpgrade for KillOldPalletStorage {
    fn on_runtime_upgrade() -> Weight {
        // Kill all storage under a prefix
        let result = clear_storage_prefix(
            b"OldPallet",  // Pallet name
            b"Storage",    // Storage name
            b"",           // Prefix to clear (empty = all)
            None,          // Limit
            None,          // Cursor
        );

        Weight::from_parts(0, 0)
            .saturating_add(T::DbWeight::get().writes(result.unique as u64))
    }
}
```

### Multi-Step Migration

```rust
pub struct ComplexMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ComplexMigration<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut weight = Weight::zero();

        // Step 1: Migrate storage A
        weight = weight.saturating_add(migrate_storage_a::<T>());

        // Step 2: Migrate storage B
        weight = weight.saturating_add(migrate_storage_b::<T>());

        // Step 3: Update configuration
        weight = weight.saturating_add(update_config::<T>());

        // Update version
        StorageVersion::new(3).put::<Pallet<T>>();

        weight
    }
}
```

## Registering Migrations

### Runtime Executive

```rust
// runtime/moonbase/lib.rs

/// Migrations to run on runtime upgrade
pub type Migrations = (
    // Run in order
    pallet_a::migrations::v2::MigrateToV2<Runtime>,
    pallet_b::migrations::RemoveOldStorage<Runtime>,
    // Add more migrations here
);

pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    Migrations,  // <-- Migrations tuple
>;
```

### Migration Order

Migrations run in the order listed in the tuple:

```rust
pub type Migrations = (
    FirstMigration,   // Runs first
    SecondMigration,  // Runs second
    ThirdMigration,   // Runs third
);
```

## Testing Migrations

### Unit Tests

```rust
#[test]
fn migration_v1_to_v2_works() {
    new_test_ext().execute_with(|| {
        // Setup old storage
        v1::OldStorage::<Test>::insert(1, v1::OldData { value: 42 });
        v1::OldStorage::<Test>::insert(2, v1::OldData { value: 100 });

        // Set old version
        StorageVersion::new(1).put::<Pallet<Test>>();

        // Run migration
        let weight = MigrateToV2::<Test>::on_runtime_upgrade();

        // Verify migration
        assert!(weight.ref_time() > 0);
        assert!(v1::OldStorage::<Test>::iter().count() == 0);
        assert_eq!(NewStorage::<Test>::get(1).unwrap().value, 42);
        assert_eq!(NewStorage::<Test>::get(2).unwrap().value, 100);
        assert_eq!(Pallet::<Test>::on_chain_storage_version(), 2);
    });
}

#[test]
fn migration_skips_if_already_done() {
    new_test_ext().execute_with(|| {
        // Set current version
        StorageVersion::new(2).put::<Pallet<Test>>();

        // Run migration
        let weight = MigrateToV2::<Test>::on_runtime_upgrade();

        // Should only read version
        assert_eq!(weight, <Test as frame_system::Config>::DbWeight::get().reads(1));
    });
}
```

### Try-Runtime Testing

```bash
# Build with try-runtime
cargo build --release --features try-runtime

# Test against live state
try-runtime \
    --runtime target/release/wbuild/moonbase-runtime/moonbase_runtime.wasm \
    on-runtime-upgrade \
    --checks all \
    live --uri wss://wss.api.moonbase.moonbeam.network

# Test against specific block
try-runtime \
    --runtime target/release/wbuild/moonbase-runtime/moonbase_runtime.wasm \
    on-runtime-upgrade \
    live --uri wss://wss.api.moonbase.moonbeam.network \
    --at 0x1234...
```

### Pre/Post Upgrade Checks

```rust
#[cfg(feature = "try-runtime")]
fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
    // Capture state before migration
    let state = PreMigrationState {
        count: OldStorage::<T>::iter().count() as u32,
        total_value: OldStorage::<T>::iter()
            .map(|(_, v)| v.value)
            .sum(),
    };

    Ok(state.encode())
}

#[cfg(feature = "try-runtime")]
fn post_upgrade(state: Vec<u8>) -> Result<(), DispatchError> {
    let pre_state: PreMigrationState = Decode::decode(&mut &state[..])
        .map_err(|_| "Failed to decode")?;

    // Verify count matches
    let new_count = NewStorage::<T>::iter().count() as u32;
    ensure!(
        pre_state.count == new_count,
        "Item count mismatch"
    );

    // Verify data integrity
    let new_total: u64 = NewStorage::<T>::iter()
        .map(|(_, v)| v.value)
        .sum();
    ensure!(
        pre_state.total_value == new_total,
        "Data integrity check failed"
    );

    Ok(())
}
```

## Best Practices

### 1. Always Use Storage Versions

```rust
#[pallet::pallet]
#[pallet::storage_version(STORAGE_VERSION)]
pub struct Pallet<T>(_);

const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);
```

### 2. Check Version Before Migrating

```rust
fn on_runtime_upgrade() -> Weight {
    let on_chain = Pallet::<T>::on_chain_storage_version();
    let current = Pallet::<T>::current_storage_version();

    if on_chain < current {
        // Run migration
    } else {
        // Skip
    }
}
```

### 3. Log Migration Progress

```rust
log::info!(
    target: "migration",
    "Starting migration from v{} to v{}",
    on_chain_version,
    target_version
);

log::info!(
    target: "migration",
    "Migrated {} items, weight: {:?}",
    count,
    weight
);
```

### 4. Handle Empty Storage

```rust
fn on_runtime_upgrade() -> Weight {
    if OldStorage::<T>::iter().next().is_none() {
        // No data to migrate, just update version
        StorageVersion::new(2).put::<Pallet<T>>();
        return T::DbWeight::get().writes(1);
    }
    // Continue with migration
}
```

### 5. Bounded Iterations

```rust
// Avoid unbounded iteration
let (cursor, processed) = OldStorage::<T>::clear(MAX_ITEMS, cursor);

// Or use drain with limit
for (key, value) in OldStorage::<T>::drain().take(MAX_ITEMS) {
    // Process
}
```

## Common Issues

| Issue                | Cause                 | Solution                              |
|----------------------|-----------------------|---------------------------------------|
| Migration runs twice | Missing version check | Always check on_chain_storage_version |
| Data loss            | Incorrect key mapping | Test with real data before mainnet    |
| Timeout              | Too many items        | Use lazy migration or pagination      |
| Decode error         | Format mismatch       | Define old types correctly            |

## Key Files

- Common Migrations: `runtime/common/src/migrations.rs`
- Runtime Migrations: `runtime/*/migrations.rs`
- Lazy Migrations: `pallets/moonbeam-lazy-migrations/`
- Migration Tests: `*/src/tests.rs`
