# ADR: AssetHub Support for XCM Transactor Pallet

## Status

DRAFT

## Summary

This ADR proposes extending the XCM Transactor pallet to support AssetHub as a transaction destination, enabling Moonbeam users to execute staking operations and other calls on the Polkadot AssetHub system parachain. The recommended approach is **Option 2: Chain-Specific Encoder Implementations**, which involves:

1. Extending the `Transactors` enum to include `AssetHub`
2. Creating chain-specific encoding logic with `AssetHubIndices`
3. Migrating storage from `RelayIndices` to `ChainIndices<Transactors, ChainIndices>`
4. Comprehensive testing including unit, integration, and chopsticks fork tests

**Key Benefits:**
- Enables DOT staking through AssetHub's delegated staking mechanisms
- Provides extensible architecture for future system parachains (BridgeHub, Collectives)
- Maintains full backwards compatibility with existing V1-V3 precompiles
- Type-safe chain selection with compile-time guarantees

**Estimated Complexity:** Medium (4-6 weeks implementation + testing)

## Context

The XCM Transactor pallet (`pallets/xcm-transactor`) enables Moonbeam parachains to execute remote calls on other chains in the Polkadot ecosystem. Currently, the pallet only supports transactions to the Relay Chain (Polkadot/Kusama/Westend), which is encoded through the `Transactors::Relay` variant.

AssetHub (formerly Statemint/Statemine) is a system parachain that serves as the primary asset management hub in the Polkadot ecosystem. It provides native support for assets, NFTs, and other fungible/non-fungible token operations. Enabling XCM Transactor to interact with AssetHub would unlock important functionality for Moonbeam users, particularly for:

1. **Cross-chain asset operations**: Creating, minting, and managing assets on AssetHub from Moonbeam
2. **Native DOT staking**: Utilizing AssetHub's proxy-staking capabilities to stake DOT on behalf of Moonbeam accounts
3. **NFT operations**: Managing NFTs and collections on AssetHub
4. **Trustless bridging**: Enabling more sophisticated cross-chain workflows

### Current Architecture

The XCM Transactor pallet uses an index-based encoding system to construct extrinsics for remote chains:

```rust
pub struct RelayChainIndices {
    // Pallet indices
    pub staking: u8,
    pub utility: u8,
    pub hrmp: u8,
    // Call indices for each pallet
    pub bond: u8,
    pub bond_extra: u8,
    // ... etc
}
```

The `StakeEncodeCall` trait implementation in `encode.rs` (lines 113-226) encodes staking operations specifically for the Relay Chain using these indices.

### The Challenge with AssetHub

AssetHub has different pallet indices and may have different pallet names/structures compared to the Relay Chain. While staking operations on AssetHub might proxy to the Relay Chain, the encoding indices will differ, requiring chain-specific encoding logic.

## Decision

We will extend the XCM Transactor pallet to support AssetHub as a transaction destination, with specific focus on enabling `StakeEncodeCall` operations that can be used for proxy-based staking through AssetHub.

### Design Options Considered

#### Option 1: Unified Indices Structure (NOT RECOMMENDED)

Create a unified indices structure that works for both Relay Chain and AssetHub:

```rust
pub struct UnifiedChainIndices {
    pub relay: RelayChainIndices,
    pub assethub: AssetHubIndices,
}
```

**Pros:**
- Simple storage model
- Clear separation of concerns

**Cons:**
- Increases storage footprint
- Tightly couples chain-specific logic
- Not extensible for future chains

#### Option 2: Chain-Specific Encoder Implementations (RECOMMENDED)

Extend the `Transactors` enum and implement chain-specific encoding logic with full support for AssetHub integration.

##### Core Architecture Changes

**1. Extend Transactors Enum**

```rust
// primitives/xcm/src/transactor_traits.rs
#[derive(Clone, Copy, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum Transactors {
    Relay,
    AssetHub,
}

impl XcmTransact for Transactors {
    fn destination(self) -> Location {
        match self {
            Transactors::Relay => Location::parent(),
            Transactors::AssetHub => Location {
                parents: 1,
                interior: [Parachain(1000)].into(), // AssetHub para ID
            },
        }
    }
}
```

**2. Chain-Specific Indices Structures**

```rust
// pallets/xcm-transactor/src/relay_indices.rs (rename to chain_indices.rs)

#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct RelayChainIndices {
    // Pallet indices
    pub staking: u8,    // 7
    pub utility: u8,    // 24
    pub hrmp: u8,       // 60
    // Call indices (staking)
    pub bond: u8,       // 0
    pub bond_extra: u8, // 1
    pub unbond: u8,     // 2
    // ... existing fields
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct AssetHubIndices {
    // Pallet indices (from polkadot-fellows/runtimes)
    pub utility: u8,            // 40
    pub proxy: u8,              // 42
    pub staking: u8,            // 89 (delegated staking)
    pub assets: u8,             // 50
    pub nfts: u8,               // 52
    pub nomination_pools: u8,   // 80 (estimate, verify)
    pub delegated_staking: u8,  // 88 (estimate, verify)

    // Utility call indices
    pub as_derivative: u8,      // 1 (standard)
    pub batch: u8,              // 0 (standard)
    pub batch_all: u8,          // 2 (standard)

    // Proxy call indices
    pub proxy: u8,              // 0
    pub add_proxy: u8,          // 1
    pub remove_proxy: u8,       // 2

    // Staking call indices (verify against runtime)
    pub bond: u8,
    pub bond_extra: u8,
    pub unbond: u8,
    pub withdraw_unbonded: u8,
    pub nominate: u8,
    pub chill: u8,
    pub set_payee: u8,
    pub rebond: u8,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum ChainIndices {
    Relay(RelayChainIndices),
    AssetHub(AssetHubIndices),
}

// Storage mapping
#[pallet::storage]
#[pallet::getter(fn chain_indices)]
pub type ChainIndicesStorage<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    Transactors,
    ChainIndices,
    OptionQuery,
>;
```

**3. Updated StakeEncodeCall Trait**

```rust
// primitives/xcm/src/transactor_traits.rs

pub trait StakeEncodeCall {
    /// Encode staking call for a specific chain destination
    fn encode_call(transactor: Transactors, call: AvailableStakeCalls) -> Vec<u8>;
}
```

**4. Chain-Specific Encoding Implementation**

```rust
// pallets/xcm-transactor/src/encode.rs

impl<T: Config> StakeEncodeCall for Pallet<T> {
    fn encode_call(transactor: Transactors, call: AvailableStakeCalls) -> Vec<u8> {
        match transactor {
            Transactors::Relay => Self::encode_relay_stake_call(call),
            Transactors::AssetHub => Self::encode_assethub_stake_call(call),
        }
    }
}

impl<T: Config> Pallet<T> {
    fn encode_relay_stake_call(call: AvailableStakeCalls) -> Vec<u8> {
        // Existing implementation (lines 113-226)
        // Uses RelayIndices storage (legacy path for backwards compat)
        // Or ChainIndicesStorage::<T>::get(Transactors::Relay)
        // ... existing code
    }

    fn encode_assethub_stake_call(call: AvailableStakeCalls) -> Vec<u8> {
        let indices = match ChainIndicesStorage::<T>::get(Transactors::AssetHub) {
            Some(ChainIndices::AssetHub(idx)) => idx,
            _ => return Vec::new(), // or error
        };

        match call {
            AvailableStakeCalls::Bond(amount, payee) => {
                let mut encoded = Vec::new();
                encoded.push(indices.staking);
                encoded.push(indices.bond);
                encoded.append(&mut encode_compact_arg(amount));
                encoded.append(&mut payee.encode());
                encoded
            }
            AvailableStakeCalls::BondExtra(amount) => {
                let mut encoded = Vec::new();
                encoded.push(indices.staking);
                encoded.push(indices.bond_extra);
                encoded.append(&mut encode_compact_arg(amount));
                encoded
            }
            AvailableStakeCalls::Unbond(amount) => {
                let mut encoded = Vec::new();
                encoded.push(indices.staking);
                encoded.push(indices.unbond);
                encoded.append(&mut encode_compact_arg(amount));
                encoded
            }
            // ... implement remaining calls following same pattern
            _ => Vec::new(),
        }
    }
}
```

##### Precompile Integration

**5. Use Existing Precompiles**

The existing XCM Transactor precompiles (V1-V3) will continue to work without modification. The pallet-level changes to support AssetHub will be accessed through:
- Direct pallet calls from Substrate transactions
- Existing precompile interfaces by encoding AssetHub-specific calls in the `innerCall` parameter
- The existing `transact_through_derivative` and `transact_through_signed` methods already accept arbitrary destinations via `Multilocation`

**Note:** No new precompile version is required. The AssetHub support is purely a pallet-level enhancement that extends the encoding capabilities and destination support.

##### Compatibility Analysis with Polkadot Fellows Runtime

**6. AssetHub Staking Capabilities**

Based on analysis of `polkadot-fellows/runtimes`, AssetHub (Polkadot) runtime v2.0.2 includes:

**Available Pallets:**
- `pallet_staking` (index 89)
- `pallet_nomination_pools` (index ~80, TBD)
- `pallet_delegated_staking` (index ~88, TBD)
- `pallet_utility` (index 40)
- `pallet_proxy` (index 42)

**Supported Stake Operations:**

| AvailableStakeCalls | Relay Chain | AssetHub Direct | AssetHub via Proxy |
|---------------------|-------------|-----------------|-------------------|
| Bond | ✅ | ✅ (index TBD) | ✅ |
| BondExtra | ✅ | ✅ (index TBD) | ✅ |
| Unbond | ✅ | ✅ (index TBD) | ✅ |
| WithdrawUnbonded | ✅ | ✅ (index TBD) | ✅ |
| Validate | ✅ | ❌ (N/A on AssetHub) | ⚠️ (proxied to Relay) |
| Nominate | ✅ | ✅ (index TBD) | ✅ |
| Chill | ✅ | ✅ (index TBD) | ✅ |
| SetPayee | ✅ | ✅ (index TBD) | ✅ |
| SetController | ✅ | ⚠️ (deprecated) | ⚠️ |
| Rebond | ✅ | ✅ (index TBD) | ✅ |

**Action Required:**
1. Verify exact pallet indices from AssetHub runtime metadata
2. Test which staking calls work directly vs require relay proxy
3. Consider if some calls should route Relay->AssetHub or direct to Relay

**7. Index Verification Strategy**

```bash
# Extract runtime metadata from AssetHub
polkadot-js-api query.system.properties --ws wss://polkadot-asset-hub-rpc.polkadot.io

# Decode metadata to find indices
subxt metadata --url wss://polkadot-asset-hub-rpc.polkadot.io > assethub-metadata.scale
subxt codegen --file assethub-metadata.scale | grep "pallet_index"
```

##### Manual Testing Guidelines

**8. Testing on Live Networks**

After deploying AssetHub support, manual testing should be performed to verify functionality:

**Prerequisites:**
1. Ensure AssetHub chain indices are configured in runtime
2. Verify AssetHub transactor info is registered with correct weight/fee parameters
3. Have test accounts with sufficient balance for fees

**Test 1: Verify Configuration**

```bash
# Check AssetHub indices are configured
# Via Polkadot.js Apps or API:
api.query.xcmTransactor.chainIndices({ AssetHub: null })

# Check transactor info is set
api.query.xcmTransactor.transactorInfo({ AssetHub: null })
```

**Test 2: Simple Bond Operation**

```javascript
// 1. Register derivative account (if not already done)
api.tx.xcmTransactor.register(accountId, derivativeIndex)

// 2. Encode a simple bond call for AssetHub
// Use the encoding helper or manually encode with AssetHub indices
const bondCall = encodeBondCall(amount, rewardDestination)

// 3. Execute transact to AssetHub
const assetHubDest = {
  parents: 1,
  interior: { X1: [{ Parachain: 1000 }] }
}

api.tx.xcmTransactor.transactThroughDerivative(
  assetHubDest,
  derivativeIndex,
  {
    currency: { AsMultiLocation: assetHubDest },
    feeAmount: 1_000_000_000 // Adjust based on fee configuration
  },
  bondCall,
  {
    transactRequiredWeightAtMost: { refTime: 1_000_000_000, proofSize: 64_000 },
    overallWeight: { refTime: 2_000_000_000, proofSize: 128_000 }
  },
  true // refund
).signAndSend(account)
```

**Test 3: Verify on AssetHub**

After sending the transaction:
1. Check XCM message was sent via `xcmpQueue.outboundXcmpMessages`
2. Monitor AssetHub for the transaction execution
3. Verify the staking operation succeeded on AssetHub
4. Check derivative account state on AssetHub

**Test 4: Other Staking Operations**

Repeat similar tests for:
- `bond_extra` - Add more to existing bond
- `unbond` - Unbond tokens
- `nominate` - Set validator nominations
- `chill` - Stop nominating

**Network-Specific Testing:**
- **Moonbase Alpha** (Westend): Test with Westend AssetHub
- **Moonriver** (Kusama): Test with Kusama AssetHub
- **Moonbeam** (Polkadot): Test with Polkadot AssetHub

**Expected Behaviors:**
- Transaction succeeds without errors
- XCM message appears in outbound queue
- Fees are deducted correctly
- AssetHub executes the encoded call
- Derivative account state updates on AssetHub

##### Migration Strategy

**9. Storage Migration**

```rust
// pallets/xcm-transactor/src/migrations.rs

pub mod v2 {
    use super::*;

    pub struct MigrateToChainIndices<T>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for MigrateToChainIndices<T> {
        fn on_runtime_upgrade() -> Weight {
            let mut weight = T::DbWeight::get().reads(1);

            // Read old RelayIndices
            if let Some(old_indices) = RelayIndices::<T>::get() {
                // Store as ChainIndices for Relay
                ChainIndicesStorage::<T>::insert(
                    Transactors::Relay,
                    ChainIndices::Relay(old_indices),
                );
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }

            // Initialize AssetHub indices (from genesis config or default)
            let assethub_indices = AssetHubIndices::default(); // or from config
            ChainIndicesStorage::<T>::insert(
                Transactors::AssetHub,
                ChainIndices::AssetHub(assethub_indices),
            );
            weight = weight.saturating_add(T::DbWeight::get().writes(1));

            log::info!("XcmTransactor storage migrated to v2");
            weight
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            let old_count = RelayIndices::<T>::get().is_some() as u32;
            Ok(old_count.encode())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
            let old_count: u32 = Decode::decode(&mut &state[..])
                .expect("pre_upgrade provides a valid state; qed");

            // Verify migration
            if old_count > 0 {
                assert!(ChainIndicesStorage::<T>::get(Transactors::Relay).is_some());
            }
            assert!(ChainIndicesStorage::<T>::get(Transactors::AssetHub).is_some());

            Ok(())
        }
    }
}
```

##### Documentation Updates

**10. Pallet Documentation**

Update technical documentation at `pallets/xcm-transactor/README.md`:

```markdown
# XCM Transactor Pallet

## Overview

The XCM Transactor pallet enables Moonbeam parachains to execute remote calls on other chains via XCM (Cross-Consensus Messaging).

## Supported Destinations

- **Relay Chain**: Polkadot/Kusama/Westend Relay Chain
- **AssetHub**: AssetHub system parachain (Polkadot/Kusama/Westend)

## Key Features

### AssetHub Support

AssetHub support enables staking, asset management, and proxy operations on the AssetHub system parachain through chain-specific encoding:

```rust
// Example: Bond DOT on AssetHub
let call = AvailableStakeCalls::Bond(10_000_000_000, RewardDestination::Staked);
let encoded_call = XcmTransactor::encode_call(Transactors::AssetHub, call);

// Use with transact_through_derivative
XcmTransactor::transact_through_derivative(
    origin,
    Transactors::AssetHub,
    derivative_index,
    fee_payment,
    encoded_call,
    weight_info,
    refund,
)?;
```

## Storage

### ChainIndices

Stores pallet and call indices for each supported chain destination.

## Examples

See integration tests in `test/suites/dev/moonbase/test-xcm-transactor/` for usage examples.
```

**Pros:**
- Extensible to additional chains (BridgeHub, Collectives, etc.)
- Clear separation of encoding logic per chain
- Configurable indices per chain via runtime configuration
- Maintains full backwards compatibility (existing precompiles unchanged)
- Type-safe chain selection
- Future-proof architecture for multi-chain support
- Comprehensive testing coverage
- Verified compatibility with AssetHub runtime
- No new precompile required - simpler upgrade path

**Cons:**
- Moderate implementation complexity
- Need to update trait signatures across the codebase
- Storage migration required
- Requires maintaining AssetHub pallet indices on runtime upgrades
- Additional testing burden (Relay + AssetHub compatibility)

#### Option 3: Dynamic Runtime Configuration (FUTURE CONSIDERATION)

Use runtime configuration to dynamically define supported pallets and calls:

```rust
pub struct ChainCallConfig {
    pub chain: Transactors,
    pub pallet_name: Vec<u8>,
    pub pallet_index: u8,
    pub call_configs: Vec<CallConfig>,
}
```

**Pros:**
- Maximum flexibility
- No runtime upgrade needed for new call types

**Cons:**
- Significantly more complex
- Higher storage overhead
- More error-prone (runtime configuration errors)
- Overkill for current requirements

### Recommended Approach: Option 2

We recommend **Option 2** for the following reasons:

1. **Extensibility**: Easily add support for other system parachains (e.g., BridgeHub, Collectives)
2. **Type Safety**: Compile-time guarantees about supported chains and calls
3. **Clarity**: Clear separation of chain-specific encoding logic
4. **Migration Path**: Can be migrated to Option 3 if dynamic configuration becomes necessary

## Implementation Plan

### Phase 1: Core Infrastructure

1. **Extend the `Transactors` enum**
   - Add `AssetHub` variant to `primitives/xcm/src/transactor_traits.rs`
   - Update `XcmTransact::destination()` implementations

2. **Create AssetHub indices structure**
   - Define `AssetHubIndices` in `pallets/xcm-transactor/src/relay_indices.rs`
   - Initially support: `utility`, `proxy`, and potentially direct staking pallets

3. **Update storage**
   - Migrate `RelayIndices` storage to `ChainIndices<Transactors, IndicesData>`
   - Create migration code for existing Relay Chain configurations

### Phase 2: Encoding Implementation

4. **Refactor `StakeEncodeCall` trait**
   - Update signature: `fn encode_call(transactor: Transactors, call: AvailableStakeCalls) -> Vec<u8>`
   - Split implementation into chain-specific functions
   - Implement AssetHub-specific encoding

5. **Update `UtilityEncodeCall` trait**
   - Follow same pattern as `StakeEncodeCall`
   - Support AssetHub's utility pallet indices

6. **Determine AssetHub pallet indices**
   - Research AssetHub runtime to identify correct pallet and call indices
   - Document indices for Polkadot AssetHub, Kusama AssetHub, and Westend AssetHub
   - Create configuration for each network

### Phase 3: Testing & Integration

7. **Unit tests**
   - Test encoding for AssetHub staking calls
   - Verify correct SCALE encoding
   - Test migration from old storage format

8. **Integration tests**
   - Test actual XCM messages to AssetHub
   - Verify staking operations work end-to-end
   - Test derivative account management

9. **Precompile updates**
   - Update relay-encoder precompile to support AssetHub
   - Add Solidity interfaces for AssetHub operations

### Phase 4: Documentation & Deployment

10. **Documentation**
    - Update pallet documentation
    - Create migration guide
    - Document AssetHub-specific call patterns

11. **Runtime integration**
    - Configure AssetHub indices for each runtime (moonbeam, moonriver, moonbase)
    - Add AssetHub to `TransactorAssetIdToLocation` mapping
    - Configure XCM fees for AssetHub transactions

## Consequences

### Positive

- **Enhanced functionality**: Users can interact with AssetHub directly from Moonbeam
- **Staking flexibility**: Enable DOT staking through AssetHub proxies
- **Future-proof**: Architecture supports additional parachains easily
- **Better UX**: Users can manage cross-chain assets without leaving Moonbeam

### Negative

- **Increased complexity**: More code paths to maintain
- **Storage migration**: Requires runtime upgrade with storage migration
- **Testing burden**: Need to test multiple chain destinations
- **Index maintenance**: Must track indices for multiple chains

### Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| AssetHub runtime changes indices | High | Document current indices, create monitoring for AssetHub upgrades, version the indices configuration |
| Storage migration fails | Critical | Extensive testing on testnets, try-runtime testing, rollback plan |
| Encoding errors produce invalid calls | High | Comprehensive unit tests against actual AssetHub metadata, integration testing |
| Increased attack surface | Medium | Thorough security review, limit initial operations to well-tested pallets |

## Open Questions

1. **Proxy staking mechanics** ⚠️ PARTIALLY ANSWERED
   - AssetHub includes `pallet_staking` (index 89), `pallet_nomination_pools`, and `pallet_delegated_staking`
   - Most staking operations (Bond, Nominate, Chill, etc.) appear supported
   - **NEEDS VERIFICATION**: Whether `Validate` operation requires proxying to Relay Chain
   - **NEEDS VERIFICATION**: Exact call indices for staking operations in AssetHub
   - **ACTION**: Test each `AvailableStakeCalls` variant on AssetHub testnet

2. **Fee configuration**
   - What are the appropriate XCM fee configurations for AssetHub transactions?
   - Should fees be paid in DOT/KSM or can AssetHub accept other assets?
   - **ACTION**: Benchmark typical AssetHub XCM transaction weights
   - **ACTION**: Consult AssetHub documentation for recommended fee structures

3. **Which pallets to support initially?**
   - **Phase 1 (MVP)**: `utility` (40), `proxy` (42), `staking` (89)
   - **Phase 2**: `nomination_pools` (~80), `delegated_staking` (~88)
   - **Phase 3**: `assets` (50), `nfts` (52) - requires new encoding traits
   - **DECISION NEEDED**: Should we support assets/NFTs in initial release or defer?

4. **Pallet index verification** ⚠️ CRITICAL
   - Current indices from `polkadot-fellows/runtimes` are estimates
   - **ACTION REQUIRED**: Extract exact indices from live AssetHub metadata
   - **ACTION REQUIRED**: Verify indices match across Polkadot/Kusama/Westend AssetHubs
   - **SCRIPT**: Use `subxt metadata` to decode and verify (see section 9 in Option 2)

5. **Backwards compatibility**
   - **RECOMMENDATION**: Keep old `RelayIndices` storage as read-only fallback
   - Migration should be automatic and transparent
   - Runtime upgrade must include try-runtime tests
   - **DECISION**: Keep legacy storage for 2-3 runtime versions, then deprecate

6. **Precompile interface** ✅ ANSWERED
   - **DECISION**: No new precompile version required
   - AssetHub support is a pallet-level enhancement
   - Existing precompiles (V1-V3) continue to work without modification
   - Users access AssetHub via existing methods by passing AssetHub destination
   - Relay-encoder precompile may need AssetHub encoding support separately

7. **Multi-network support**
   - AssetHub exists on Polkadot (para 1000), Kusama (para 1000), Westend (para 1000)
   - **OPTION A**: Single `AssetHub` variant with runtime-specific para ID configuration
   - **OPTION B**: Separate variants (`AssetHubPolkadot`, `AssetHubKusama`, `AssetHubWestend`)
   - **RECOMMENDATION**: Option A with runtime config - simpler and less enum bloat
   - Para ID 1000 is consistent, so `Location { parents: 1, interior: [Parachain(1000)] }` works

8. **Staking call compatibility matrix**
   - **NEEDS RESEARCH**: Does AssetHub's `SetController` exist or is it deprecated?
   - **NEEDS TESTING**: Which calls work identically vs have different behavior?
   - **ACTION**: Create compatibility test suite comparing Relay vs AssetHub staking calls

9. **Error handling**
   - How should encoding failures be handled? Empty Vec, Result type, or panic?
   - **RECOMMENDATION**: Return `Result<Vec<u8>, EncodeError>` for better error messages
   - Requires updating `StakeEncodeCall` trait signature

10. **Runtime API updates**
    - Should we add a runtime API to query supported chains and their capabilities?
    - **FUTURE ENHANCEMENT**: `fn supported_transactors() -> Vec<(Transactors, Vec<PalletInfo>)>`

## References

- XCM Transactor Pallet: `pallets/xcm-transactor/`
- XCM Primitives: `primitives/xcm/src/transactor_traits.rs`
- AssetHub Runtime: https://github.com/polkadot-fellows/runtimes
- Relay Encoder Precompile: `precompiles/relay-encoder/`

## Next Steps

### Immediate Actions (Before Implementation)

1. **Index Verification** (Priority: P0)
   ```bash
   # Extract AssetHub metadata
   subxt metadata --url wss://polkadot-asset-hub-rpc.polkadot.io > assethub-polkadot.scale
   subxt metadata --url wss://kusama-asset-hub-rpc.polkadot.io > assethub-kusama.scale

   # Decode and extract pallet indices
   subxt codegen --file assethub-polkadot.scale | grep -A 5 "pallet_staking\|pallet_utility\|pallet_proxy"
   ```
   - Document exact pallet and call indices for Polkadot, Kusama, and Westend AssetHubs
   - Create a reference table comparing indices across networks
   - **Owner**: TBD | **Deadline**: Before Phase 1 implementation

2. **Team Review & Approval** (Priority: P0)
   - Schedule ADR review meeting with core team
   - Present Option 2 design and get consensus
   - Discuss open questions and get decisions on:
     - Which pallets to support in Phase 1
     - Error handling approach (Vec vs Result)
     - Multi-network configuration strategy
   - **Owner**: TBD | **Deadline**: Week of [DATE]

3. **Technical Feasibility Spike** (Priority: P0)
   - Create proof-of-concept for AssetHub staking call encoding
   - Test a simple `Bond` operation on AssetHub testnet
   - Verify XCM message formation and execution
   - **Owner**: TBD | **Deadline**: 1 week after approval

4. **Security Review Planning** (Priority: P1)
   - Identify security-critical components (encoding, storage migration)
   - Schedule internal security review
   - Determine if external audit is needed
   - **Owner**: TBD | **Deadline**: Before Phase 2 implementation

### Implementation Roadmap

**Phase 1: Core Infrastructure** (2 weeks)
- [ ] Extend `Transactors` enum in `xcm-primitives`
- [ ] Create `AssetHubIndices` structure
- [ ] Implement storage migration with try-runtime tests
- [ ] Update `XcmTransact::destination()` implementation
- [ ] Document verified pallet indices in code comments

**Phase 2: Encoding Logic** (1-2 weeks)
- [ ] Refactor `StakeEncodeCall` trait signature
- [ ] Implement `encode_assethub_stake_call()`
- [ ] Update `UtilityEncodeCall` for AssetHub
- [ ] Create comprehensive unit tests for encoding

**Phase 3: Testing & Integration** (2 weeks)
- [ ] Unit tests (encoding, storage, pallet logic)
- [ ] Moonwall integration tests (dev and smoke)
- [ ] Chopsticks fork testing with live AssetHub
- [ ] Documentation updates (inline, README, migration guide)

**Phase 4: Deployment** (1 week)
- [ ] Moonbase Alpha testnet deployment
- [ ] Community testing period
- [ ] Runtime upgrade to Moonriver (Kusama)
- [ ] Runtime upgrade to Moonbeam (Polkadot)

**Total Estimated Timeline:** 4-6 weeks

### Success Criteria

- [ ] All `AvailableStakeCalls` successfully encode for AssetHub
- [ ] Storage migration tested with try-runtime on all runtimes
- [ ] End-to-end test: Bond DOT on AssetHub via Moonbeam derivative account
- [ ] Zero breaking changes to existing precompile users
- [ ] Documentation complete and reviewed
- [ ] Security review passed (if required)

### Rollback Plan

If critical issues are discovered:
1. Storage migration includes rollback capability via try-runtime
2. AssetHub `Transactor` variant can be feature-flagged
3. Worst case: Runtime upgrade to remove AssetHub support, restore legacy behavior

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Moonbeam EVM Layer                          │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ V1 Precompile│  │ V2 Precompile│  │ V3 Precompile│             │
│  │   (0x0806)   │  │   (0x080D)   │  │   (0x0817)   │             │
│  │  Unchanged   │  │  Unchanged   │  │  Unchanged   │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │                 │                 │                       │
│         └─────────────────┴─────────────────┘                       │
│                           │                                         │
│     All precompiles now support AssetHub via destination parameter │
│                           │                                         │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Substrate Pallet Layer                           │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │           pallet-xcm-transactor (Enhanced)                   │  │
│  │                                                              │  │
│  │  Storage:                                                    │  │
│  │  ┌────────────────────────────────────────────────┐         │  │
│  │  │ ChainIndices<Transactors, ChainIndices>        │         │  │
│  │  │  - Relay -> RelayChainIndices                  │         │  │
│  │  │  - AssetHub -> AssetHubIndices NEW!            │         │  │
│  │  └────────────────────────────────────────────────┘         │  │
│  │                                                              │  │
│  │  Encoding:                                                   │  │
│  │  ┌──────────────────┐  ┌──────────────────┐                 │  │
│  │  │ StakeEncodeCall  │  │ UtilityEncodeCall│                 │  │
│  │  │                  │  │                  │                 │  │
│  │  │ ├─ Relay         │  │ ├─ Relay         │                 │  │
│  │  │ └─ AssetHub NEW! │  │ └─ AssetHub NEW! │                 │  │
│  │  └──────────────────┘  └──────────────────┘                 │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                           │                                         │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        XCM Message Layer                            │
│                                                                     │
│  ┌──────────────────┐              ┌──────────────────┐            │
│  │  Relay Chain     │              │    AssetHub      │            │
│  │  (Polkadot)      │              │  (Para 1000)     │            │
│  │                  │              │                  │            │
│  │  Staking: idx 7  │              │  Staking: idx 89 │            │
│  │  Utility: idx 24 │              │  Utility: idx 40 │            │
│  │  HRMP: idx 60    │              │  Proxy: idx 42   │            │
│  │                  │              │  Assets: idx 50  │            │
│  └──────────────────┘              └──────────────────┘            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Flow Example: Bond DOT via AssetHub
====================================

1. User calls existing precompile or Substrate tx:
   // Via Substrate
   xcmTransactor.transactThroughDerivative(
     dest: { parents: 1, interior: { X1: { Parachain: 1000 } } },  // AssetHub
     index: 0,
     fee: { currency: {...}, feeAmount: 1000000 },
     innerCall: encodedBondCall,  // SCALE-encoded Bond(10 DOT)
     weightInfo: {...},
     refund: true
   )

   // Or via existing V3 precompile with AssetHub destination
   xcmTransactorV3.transactThroughDerivative(...)

2. Pallet receives call and determines destination is AssetHub

3. Pallet encodes call using AssetHub indices:
   [89, 0, ...] // Pallet 89 (staking), Call 0 (bond)

4. XCM message sent to AssetHub:
   Transact {
     origin_kind: SovereignAccount,
     require_weight_at_most: weight,
     call: [89, 0, ...],
   }

5. AssetHub executes Bond operation
   → DOT staked on Relay Chain via delegation
```

## Decision Log

- **2025-11-13**: Initial draft created with comprehensive Option 2 design
- **TBD**: Team review and feedback incorporation
- **TBD**: Technical feasibility spike completed
- **TBD**: AssetHub pallet indices verified
- **TBD**: Final decision and approval
- **TBD**: Implementation kickoff
