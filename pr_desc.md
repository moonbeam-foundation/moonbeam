## ⚠️ Breaking Changes ⚠️

Brief summary of changes that might impact apps and tools that interact with the chain:
- **Extrinsics**:
  - `pallet-xcm-transactor` extrinsics `set_fee_per_second` and `remove_fee_per_second` are removed from the callable API (their call indices remain reserved as commented holes for SCALE compatibility).
- **On-chain storage**:
  - The `DestinationAssetFeePerSecond` storage map in `pallet-xcm-transactor` is removed.
  - XCM fee pricing data is now sourced exclusively from `pallet-xcm-weight-trader::SupportedAssets` instead of a dedicated map in `pallet-xcm-transactor`.
- **Events and errors**:
  - `DestFeePerSecondChanged` and `DestFeePerSecondRemoved` events are removed (kept as commented placeholders with their original codec indices).
  - `Error::FeePerSecondNotSet` is removed; callers now see `UnableToWithdrawAsset` / `AssetHasNoReserve` / `AssetIsNotReserveInDestination` depending on the failure mode. All error and event variants after the removed ones are explicitly annotated with `#[codec(index = N)]` to preserve their original SCALE indices.

### What does it do?

This PR refactors XCM fee handling so that:
- `pallet-xcm-transactor` no longer stores or manages fee-per-second data; instead, it delegates fee computation to a new trait `xcm_primitives::XcmFeeTrader`.
- `pallet-xcm-weight-trader` implements `XcmFeeTrader` and becomes the single source of truth for XCM execution fee pricing.
- `pallet-xcm-transactor`’s internal `calculate_fee`:
  - Calls `T::FeeTrader::compute_fee` to obtain the fee amount for a given `weight` and `fee_location`.
  - Validates that the fee asset is a valid reserve for the XCM destination via `T::ReserveProvider`, returning `AssetHasNoReserve` or `AssetIsNotReserveInDestination` when invalid.
- `FeeTraderSetter` is scoped to `pallet-xcm-transactor` tests/benchmarks only and is no longer implemented in the runtimes; runtime governance should now use the xcm-weight-trader pallet extrinsics directly for fee configuration.

### What important points should reviewers know?

- **Codec and call index compatibility**:
  - Removed calls keep their original `#[pallet::call_index]` positions as commented-out stubs, leaving intentional “holes” so existing call indices remain stable.
  - `Error` and `Event` enums are carefully annotated with `#[codec(index = N)]` for all variants after the removed ones, preserving the historical SCALE indices used on-chain.
- **Trait placement and implementations**:
  - `XcmFeeTrader` now lives in `primitives/xcm/src/fee_trader.rs` and is re-exported from `primitives/xcm/src/lib.rs`.
  - `FeeTraderSetter` remains defined in `xcm-primitives` but is only wired into `pallet-xcm-transactor::Config` under `#[cfg(any(test, feature = "runtime-benchmarks"))]` and is implemented by the pallet’s `MockFeeTrader` for tests/benchmarks.
- **Reserve validation location**:
  - Reserve validation is _not_ done inside `XcmFeeTrader::compute_fee`; it is explicitly done in `pallet-xcm-transactor::calculate_fee`, which uses `T::ReserveProvider` and maps failures to pallet errors.
- **Tests and precompiles**:
  - `pallet-xcm-transactor` unit tests and benchmarks are updated to:
    - Use `FeeTraderSetter` in the pallet’s `MockFeeTrader` to configure fees in tests/benches.
    - Expect the new error paths (`UnableToWithdrawAsset`, reserve-related errors) instead of the previous `FeePerSecondNotSet`.
  - `pallet-evm-precompile-xcm-transactor` tests use a local `MockFeeTrader` with simple in-memory setters; they no longer depend on `pallet-xcm-weight-trader` or runtime-level fee setters.
- **Migrations**:
  - `runtime/common/src/migrations.rs` adds a new `DestinationAssetFeePerSecondStorageName` constant and a `ResetStorage` entry for `pallet_xcm_transactor::Pallet<Runtime>`’s `DestinationAssetFeePerSecond` map in `MultiBlockMigrations`, ensuring the old map is wiped clean on all runtimes.

### Is there something left for follow-up PRs?

No

### What alternative implementations were considered?

N/A

### Are there relevant PRs or issues in other repositories (Substrate, Polkadot, Frontier, Cumulus)?

No

### What value does it bring to the blockchain users?

- **Single source of truth for XCM fees**: All XCM execution fee pricing is now centralized in `pallet-xcm-weight-trader`, reducing configuration drift and simplifying audits.

---

## What's solved in this change and what features are modified?

- Brief summary of issue that should be resolved (if any)
    - Remove duplicated/parallel fee configuration between `pallet-xcm-transactor` and `pallet-xcm-weight-trader`.
    - Eliminate the `DestinationAssetFeePerSecond` map and the need for transactor-specific fee extrinsics.
    - Simplify the runtime configuration surface for XCM by wiring `pallet-xcm-weight-trader` directly into `pallet-xcm-transactor` via the `XcmFeeTrader` trait.

- High-level summary of feature changes or specifications of new feature
    - `XcmFeeTrader` is promoted to a shared primitive trait in `primitives/xcm`, implemented by `pallet-xcm-weight-trader`.
    - `pallet-xcm-transactor` calculates fees exclusively via `XcmFeeTrader`, then enforces reserve validity through `ReserveProvider`.
    - `FeeTraderSetter` is limited to tests/benchmarks and is not part of the runtime API surface.

## What changes to storage structures, processes or high-level assumptions have been made?

- High-level summary of any core assumptions that are modified (e.g permissions, invariants).
    - Fee pricing for XCM execution is assumed to be defined and maintained by `pallet-xcm-weight-trader`’s `SupportedAssets` storage, not by `pallet-xcm-transactor`.
    - The invariants around fee asset validity now explicitly include “asset must have a reserve and must be a reserve for the XCM destination,” enforced at the transactor layer.

Low-level summary of any processes or storage structures that are changed.
- **Removed storage**:
  - `pallet-xcm-transactor::DestinationAssetFeePerSecond` map is removed, and a `ResetStorage` multi-block migration clears any existing keys on-chain.
- **New / reused storage**:
  - `pallet-xcm-weight-trader::SupportedAssets` becomes the authoritative store for per-asset pricing and is used by the `XcmFeeTrader` implementation.
- **Process changes**:
  - Fee calculation path:
    1. `pallet-xcm-transactor` computes the weight for an XCM call.
    2. `calculate_fee` calls `T::FeeTrader::compute_fee(weight, &fee_location, explicit_amount)` to get the fee amount.
    3. The transactor validates reserve location with `T::ReserveProvider` and builds the fee `Asset`.
  - Tests and benchmarks configure prices via `MockFeeTrader` implementing `FeeTraderSetter`.

## Are there additional mechanisms or storage structures indirectly affected by these changes?

- Describe any known side-effects of this change (not directly visible in diff).
    - Any tooling, dashboards, or scripts that previously queried `XcmTransactor::DestinationAssetFeePerSecond` will need to be updated to query `pallet-xcm-weight-trader` instead.
    - Governance procedures that used `XcmTransactor::set_fee_per_second` / `remove_fee_per_second` must now use `pallet-xcm-weight-trader` extrinsics.

- Describe any relationships, high-level constraints, or assumptions that this aims to change.
    - Establishes a clear separation of concerns:
        - `pallet-xcm-weight-trader`: defines pricing.
        - `pallet-xcm-transactor`: computes required weight and enforces reserve correctness, but does not own pricing data.
    - Assumes `pallet-xcm-weight-trader` is present and correctly configured in all runtimes using `pallet-xcm-transactor`.

## What risks have already been internally considered or handled?

- Describe any internal concerns related to these changes.
    - **Risk: Migration correctness**:
        - If the `ResetStorage` migration for `DestinationAssetFeePerSecond` is misconfigured (wrong storage name or pallet), stale data could remain or, worse, unrelated keys could be cleared.
    - **Risk: Misconfigured fee assets**:
        - If `SupportedAssets` is not configured for commonly used fee assets, transactor calls will fail with `UnableToWithdrawAsset` or reserve-related errors.
    - **Risk: Compatibility with existing tooling**:
        - External consumers relying on removed extrinsics/events or the old storage map may need updates.

- Describe how these risks were handled (e.g tests, design-decisions, rationale).
    - Added a generic `ResetStorage` migration for the `DestinationAssetFeePerSecond` map, reusing the existing multi-block migration framework to avoid block-weight issues on chains with many entries.
    -  Kept call and codec indices stable and added explicit annotations to avoid subtle SCALE compatibility regressions.
    - Updated:
        - Pallet unit tests, including error-path coverage for missing fee configuration and invalid reserves.
        - Precompile tests, ensuring EVM users still see consistent behavior and errors.
        - Runtime XCM integration tests and benchmarks to use the new fee path and weight-trader-backed pricing.

## What runtime is this intended for?

Runtime 4200

