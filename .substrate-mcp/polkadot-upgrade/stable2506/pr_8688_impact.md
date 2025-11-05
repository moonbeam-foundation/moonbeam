# PR 8688 Impact Analysis

## PR Details
- **Title**: bound trusted local cache to shared limits sizes
- **URL**: https://github.com/paritytech/polkadot-sdk/pull/8688
- **Audience**: Node Dev
- **Bump**: Patch

## Summary
This PR improves the trie cache implementation by bounding the "trusted" local cache sizes to reasonable limits instead of allowing unlimited growth. Previously, trusted cache configurations used `usize::MAX` for cache limits, which could theoretically lead to unbounded memory usage. This change bounds the trusted local cache to the shared cache limits.

## Changed Crates
- `sp-trie` (patch bump)

## Changes Description

### Key Changes:
1. **LocalNodeCacheConfig::trusted()**:
   - Now takes `local_node_cache_max_heap_size` and `local_node_cache_max_inline_size` parameters
   - Uses `max(provided_limit, default_limit)` instead of `usize::MAX`
   - Prevents unbounded cache growth while still allowing flexible limits

2. **LocalValueCacheConfig::trusted()**:
   - Similarly updated with `local_value_cache_max_heap_size` and `local_value_cache_max_inline_size` parameters
   - Applies the same bounded approach

3. **SharedTrieCache**:
   - Now stores the trusted cache configurations
   - Passes shared cache limits to local trusted caches
   - Ensures local caches are bounded by shared cache limits

### Technical Details:
- Files changed:
  - `substrate/primitives/trie/src/cache/mod.rs` - Updated config constructors
  - `substrate/primitives/trie/src/cache/shared_cache.rs` - Stores and uses bounded configs

### Rationale:
The "trusted" path is used during block authoring and importing where operations are already bounded by other constraints. However, using `usize::MAX` was identified as a potential issue. The new approach:
- Maintains the flexibility of trusted paths
- Prevents theoretical unbounded growth
- Bounds local cache to shared cache limits (which makes sense since items are promoted to shared cache anyway)

## Impact on Moonbeam

**IMPACT LEVEL: LOW (Transparent Improvement)**

### Analysis
1. **sp-trie Usage**: Moonbeam uses `sp-trie` extensively:
   - Used in `client/rpc/debug` for proof size extensions
   - Used in `node/service/src/lazy_loading/substrate_backend.rs` for PrefixedMemoryDB
   - Used in `primitives/storage-proof` for storage proofs and trie operations
   - Dependency in root `Cargo.toml`

2. **Cache Configuration**:
   - Moonbeam does NOT have custom trie cache configuration
   - Uses default Substrate cache settings
   - No usage of `SharedTrieCache`, `local_trie_cache`, or cache size parameters found in codebase

3. **Type of Change**:
   - This is a node-level optimization, not a runtime change
   - The API remains the same (parameters are added to internal methods)
   - Changes are transparent to existing code
   - Improves safety without changing behavior

4. **Performance Impact**:
   - POSITIVE: Better memory bounds prevent potential memory issues
   - NEUTRAL: No performance regression expected (cache limits are still generous)
   - The trusted cache paths (block authoring/importing) now have reasonable bounds

### Verification
```bash
# Confirmed sp-trie usage:
rg "sp-trie" --type toml  # Found in client/rpc/debug and root Cargo.toml

# Confirmed no custom cache configuration:
rg "SharedTrieCache|local_trie_cache|node_cache_max|value_cache_max"  # No custom config
```

## Recommendation
**NO ACTION REQUIRED - Transparent Upgrade**

This is a transparent safety improvement that will be automatically picked up when updating to stable2506. The changes:
- Do not require any code modifications in Moonbeam
- Do not change any public APIs
- Improve memory safety
- Have no performance regression

## Benefits for Moonbeam
1. **Improved Memory Safety**: Local caches now have reasonable bounds even on trusted paths
2. **Better Resource Management**: Prevents theoretical unbounded cache growth
3. **Aligned with Best Practices**: Cache sizes are now properly bounded relative to shared limits
4. **No Breaking Changes**: All changes are internal to sp-trie

## Notes
- This is a patch-level change (no breaking changes)
- The change was motivated by a code review comment on PR #7556
- The trusted cache is used during block authoring and importing operations
- Moonbeam will benefit from this safety improvement with no code changes required
