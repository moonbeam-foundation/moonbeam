---
name: writing-migrations
description: Writes and tests runtime migrations for state transitions in Moonbeam. Use when handling storage layout changes, renaming or removing storage items, data format changes, pallet index changes, or storage key modifications.
---

# Runtime Migrations

## Contents
- [Migration Lifecycle](#migration-lifecycle)
- [Migration Types](#migration-types)
- [Writing Migrations](#writing-migrations)
- [Registering Migrations](#registering-migrations)
- [Testing Migrations](#testing-migrations)
- [Best Practices](#best-practices)
- [Common Issues](#common-issues)

## Migration Lifecycle

Moonbeam follows a simple migration lifecycle:

1. **Add migration before release**: Write the migration and register it in the runtime
2. **Deploy**: Migration runs once during the runtime upgrade
3. **Remove migration before next release**: Delete the migration code after it has executed

This approach avoids complex storage versioning. Migrations are one-shot: they run once and are removed from the codebase.

```
Release N-1: No migration
     ↓
Add migration code
     ↓
Release N: Migration executes on-chain
     ↓
Remove migration code
     ↓
Release N+1: Clean codebase
```

## Migration Types

### OnRuntimeUpgrade Migrations

Standard migrations that run during runtime upgrade.

```rust
pub struct MigrateStorageFormat<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateStorageFormat<T> {
    fn on_runtime_upgrade() -> Weight {
        // Migration logic - runs once, then this code is removed
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

Migrations that run gradually over multiple blocks. Used for large data sets that cannot be migrated in a single block.

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
    pallet_prelude::*,
    traits::OnRuntimeUpgrade,
    weights::Weight,
};

// Old storage format (define what we're migrating from)
mod old {
    use super::*;

    #[frame_support::storage_alias]
    pub type OldStorage<T: pallet::Config> =
        StorageMap<pallet::Pallet<T>, Blake2_128Concat, AccountId, OldData>;

    #[derive(Decode)]
    pub struct OldData {
        pub value: u32,
    }
}

/// Migration to add extra_field to storage items.
/// Added in runtime XXXX, remove after deployment.
pub struct MigrateStorageFormat<T>(PhantomData<T>);

impl<T: pallet::Config> OnRuntimeUpgrade for MigrateStorageFormat<T> {
    fn on_runtime_upgrade() -> Weight {
        log::info!(target: "migration", "Running MigrateStorageFormat");

        let mut count = 0u64;

        // Iterate over old storage and transform
        for (key, old_data) in old::OldStorage::<T>::drain() {
            let new_data = NewData {
                value: old_data.value,
                extra_field: Default::default(),
            };

            NewStorage::<T>::insert(key, new_data);
            count += 1;
        }

        log::info!(target: "migration", "Migrated {} items", count);

        T::DbWeight::get().reads_writes(count, count)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
        let count = old::OldStorage::<T>::iter().count() as u32;
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

        log::info!(target: "migration", "Post-upgrade: {} items migrated", new_count);
        Ok(())
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
/// Complex migration that updates multiple storage items.
/// Added in runtime XXXX, remove after deployment.
pub struct ComplexMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ComplexMigration<T> {
    fn on_runtime_upgrade() -> Weight {
        log::info!(target: "migration", "Running ComplexMigration");

        let mut weight = Weight::zero();

        // Step 1: Migrate storage A
        weight = weight.saturating_add(migrate_storage_a::<T>());

        // Step 2: Migrate storage B
        weight = weight.saturating_add(migrate_storage_b::<T>());

        // Step 3: Update configuration
        weight = weight.saturating_add(update_config::<T>());

        log::info!(target: "migration", "ComplexMigration complete");
        weight
    }
}
```

## Registering Migrations

### Runtime Executive

```rust
// runtime/moonbase/lib.rs

/// Migrations to run on runtime upgrade.
/// Remove after deployment.
type MoonbaseMigrations = (
    // Run in order
    migrations::MigrateStorageFormat<Runtime>,
    migrations::RemoveDeprecatedStorage<Runtime>,
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

### Migration Order

Migrations run in the order listed in the tuple:

```rust
type MoonbaseMigrations = (
    FirstMigration,   // Runs first
    SecondMigration,  // Runs second
    ThirdMigration,   // Runs third
);
```

### After Deployment

Once migrations have run on all networks (Moonbase Alpha, Moonriver, Moonbeam):

```rust
type MoonbaseMigrations = ();
```

Then remove the migration code from `runtime/common/src/migrations.rs`.

## Testing Migrations

### Unit Tests

```rust
#[test]
fn migration_works() {
    new_test_ext().execute_with(|| {
        // Setup old storage
        old::OldStorage::<Test>::insert(1, old::OldData { value: 42 });
        old::OldStorage::<Test>::insert(2, old::OldData { value: 100 });

        // Run migration
        let weight = MigrateStorageFormat::<Test>::on_runtime_upgrade();

        // Verify migration
        assert!(weight.ref_time() > 0);
        assert!(old::OldStorage::<Test>::iter().count() == 0);
        assert_eq!(NewStorage::<Test>::get(1).unwrap().value, 42);
        assert_eq!(NewStorage::<Test>::get(2).unwrap().value, 100);
    });
}

#[test]
fn migration_handles_empty_storage() {
    new_test_ext().execute_with(|| {
        // No old storage items

        // Run migration - should complete without error
        let weight = MigrateStorageFormat::<Test>::on_runtime_upgrade();

        // Zero reads/writes when nothing to migrate
        assert_eq!(weight, Weight::zero());
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

### 1. Document When to Remove

Add a comment indicating when the migration should be removed:

```rust
/// Migration to fix XYZ issue.
/// Added in runtime 2800, remove after deployment to all networks.
pub struct FixXyzMigration<T>(PhantomData<T>);
```

### 2. Log Migration Progress

```rust
log::info!(
    target: "migration",
    "Running FixXyzMigration"
);

log::info!(
    target: "migration",
    "Migrated {} items, weight: {:?}",
    count,
    weight
);
```

### 3. Handle Empty Storage Gracefully

```rust
fn on_runtime_upgrade() -> Weight {
    let mut count = 0u64;

    for (key, old_data) in OldStorage::<T>::drain() {
        // Process...
        count += 1;
    }

    // Works fine even if storage was empty
    log::info!(target: "migration", "Migrated {} items", count);
    T::DbWeight::get().reads_writes(count, count)
}
```

### 4. Use Lazy Migrations for Large Data Sets

For migrations that could timeout, use the lazy migration pallet:

```rust
// Large unbounded iteration - use lazy migration
// See pallets/moonbeam-lazy-migrations/
```

### 5. Clean Up After Deployment

After the runtime upgrade has been deployed to all networks:
- Remove the migration struct and implementation
- Remove from the `Migrations` tuple in the runtime
- Remove any `old` module definitions

## Common Issues

| Issue                     | Cause                        | Solution                           |
|---------------------------|------------------------------|------------------------------------|
| Migration not removed     | Forgot to clean up           | Remove after deployment to all networks |
| Data loss                 | Incorrect key mapping        | Test with real data before mainnet |
| Timeout                   | Too many items               | Use lazy migration                 |
| Decode error              | Format mismatch              | Define old types correctly         |

## Key Files

- Common Migrations: `runtime/common/src/migrations.rs`
- Runtime Migrations: `runtime/*/migrations.rs`
- Lazy Migrations: `pallets/moonbeam-lazy-migrations/`
- Migration Tests: `*/src/tests.rs`
