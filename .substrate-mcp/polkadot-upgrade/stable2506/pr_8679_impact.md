# PR 8679 Impact Analysis

## PR Details
- **Title**: Shared Add ethereum-standards crate
- **URL**: https://github.com/paritytech/polkadot-sdk/pull/8679
- **Audience**: Runtime Dev
- **Bump**: Minor

## Summary
This PR adds a new `ethereum-standards` crate to the Polkadot SDK ecosystem. The crate provides standard Ethereum interfaces (like IERC20) that can be used by other pallets and components. This is a preparatory PR for upcoming changes in #7762 and #8365.

## Changed Crates
- `ethereum-standards` (new crate)
- `pallet-revive` (minor bump)
- `snowbridge-pallet-inbound-queue` (minor bump)
- `snowbridge-inbound-queue-primitives` (minor bump)
- `snowbridge-outbound-queue-primitives` (minor bump)

## Changes Description
The PR introduces a new crate at `substrate/primitives/ethereum-standards` that contains:
- IERC20 Solidity interface definition
- Rust bindings for Ethereum standards

The following components were updated to use this new crate:
- `pallet-revive`: Updated precompiles to use ethereum-standards
- Snowbridge components: Updated envelope and message handling to use ethereum-standards

## Impact on Moonbeam

**IMPACT LEVEL: NONE**

### Analysis
1. **pallet-revive**: Not used by Moonbeam. This is a new experimental pallet for running Ethereum contracts on Polkadot substrate chains using a different approach than pallet-evm.

2. **Snowbridge components**:
   - Moonbeam has `snowbridge-core` as a transitive dependency in Cargo.lock
   - However, the affected crates (`snowbridge-pallet-inbound-queue`, `snowbridge-inbound-queue-primitives`, `snowbridge-outbound-queue-primitives`) are NOT part of Moonbeam's dependency tree
   - No direct usage of snowbridge components found in Moonbeam's codebase

3. **ethereum-standards crate**: This is a new crate that Moonbeam does not currently depend on.

### Verification
```bash
# Confirmed that Moonbeam does not use:
rg "pallet-revive|snowbridge-.*-queue" --type toml  # No results
rg "snowbridge" --type rust  # No usage in code
```

## Recommendation
**NO ACTION REQUIRED**

This PR introduces infrastructure that Moonbeam does not currently use. The changes are isolated to components that are not part of Moonbeam's runtime or dependencies.

## Notes
- This is a preparatory PR for future changes
- If Moonbeam decides to integrate with Snowbridge bridge in the future, this ethereum-standards crate will provide useful Ethereum interfaces
- The new crate follows similar patterns to what Moonbeam already implements for Ethereum compatibility via pallet-evm
