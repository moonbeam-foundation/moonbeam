# ADR: AssetHub Support for XCM Transactor Pallet

## Status

DRAFT

## Summary

This ADR proposes extending the XCM Transactor pallet to support AssetHub as a transaction destination, enabling Moonbeam users to execute staking operations and other calls on the Polkadot AssetHub system parachain. The recommended approach is **Option 2: Chain-Specific Encoder Implementations**, which involves:

1. Extending the `Transactors` enum to include `AssetHub`
2. Creating chain-specific encoding logic with `AssetHubIndices`
3. Implementing a new V4 precompile at address `0x0818` with unified `Transactor` parameter
4. Migrating storage from `RelayIndices` to `ChainIndices<Transactors, ChainIndices>`
5. Comprehensive testing including unit, integration, and chopsticks fork tests

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

**5. V3 Extension vs V4 New Version**

Two approaches are viable:

**Approach A: Extend V3 (Minimal Changes)**
- Add `transactThroughDerivativeAssetHub()` and similar methods to existing V3
- Pro: No new precompile address needed
- Con: Bloats V3 interface, breaks semantic versioning

**Approach B: Create V4 (Recommended)**
- Create new `precompiles/xcm-transactor/src/v4/` directory
- New precompile address: `0x0000000000000000000000000000000000000818` (AddressU64<2072>)
- Pro: Clean separation, semantic versioning, backwards compatible
- Con: Additional maintenance overhead

**Recommended: V4 with Unified Interface**

```solidity
// precompiles/xcm-transactor/src/v4/XcmTransactorV4.sol

pragma solidity >=0.8.3;

/// @title XCM Transactor Interface V4
/// @notice Interface for cross-chain transactions with AssetHub support
/// @dev Precompiled contract at address 0x0000000000000000000000000000000000000818
interface XcmTransactorV4 {
    /// @dev Transactor destination chains
    enum Transactor {
        Relay,      // 0: Polkadot/Kusama/Westend Relay Chain
        AssetHub    // 1: AssetHub system parachain
    }

    struct Multilocation {
        uint8 parents;
        bytes[] interior;
    }

    struct Weight {
        uint64 refTime;
        uint64 proofSize;
    }

    // ============ Enhanced Transact Methods ============

    /// @notice Transact through derivative account with chain selection
    /// @param transactor Target chain (Relay or AssetHub)
    /// @param index Derivative account index
    /// @param feeAsset Asset to use for fees
    /// @param transactRequiredWeightAtMost Weight limit for the remote call
    /// @param innerCall SCALE-encoded call to execute remotely
    /// @param feeAmount Maximum fee willing to pay
    /// @param overallWeight Total weight limit including XCM overhead
    /// @param refund Whether to refund unused fees
    function transactThroughDerivative(
        Transactor transactor,
        uint16 index,
        Multilocation memory feeAsset,
        Weight memory transactRequiredWeightAtMost,
        bytes memory innerCall,
        uint256 feeAmount,
        Weight memory overallWeight,
        bool refund
    ) external;

    /// @notice Transact through signed origin with chain selection
    /// @param transactor Target chain (Relay or AssetHub)
    /// @param dest Destination location for the transaction
    /// @param feeAsset Asset to use for fees
    /// @param transactRequiredWeightAtMost Weight limit
    /// @param innerCall SCALE-encoded call
    /// @param feeAmount Maximum fee
    /// @param overallWeight Total weight limit
    /// @param refund Whether to refund unused fees
    function transactThroughSigned(
        Transactor transactor,
        Multilocation memory dest,
        Multilocation memory feeAsset,
        Weight memory transactRequiredWeightAtMost,
        bytes memory innerCall,
        uint256 feeAmount,
        Weight memory overallWeight,
        bool refund
    ) external;

    // ============ Query Methods ============

    /// @notice Get account address for a derivative index
    /// @param index The derivative account index
    /// @return The account address
    function indexToAccount(uint16 index) external view returns (address);

    /// @notice Get transaction weight info for a destination
    /// @param transactor Target chain
    /// @param multilocation The destination location
    /// @return extraWeight Additional weight added by XCM
    /// @return maxWeight Maximum allowed weight
    function transactInfoWithSigned(
        Transactor transactor,
        Multilocation memory multilocation
    ) external view returns (Weight memory, Weight memory);

    /// @notice Get fee per second for an asset on a destination
    /// @param transactor Target chain
    /// @param multilocation The asset location
    /// @return Fee per second of execution
    function feePerSecond(
        Transactor transactor,
        Multilocation memory multilocation
    ) external view returns (uint256);

    // ============ Encoding Utilities ============

    /// @notice Encode a utility.asDerivative call
    /// @param transactor Target chain (affects pallet indices)
    /// @param index Derivative index
    /// @param innerCall The call to wrap
    /// @return SCALE-encoded asDerivative call
    function encodeUtilityAsDerivative(
        Transactor transactor,
        uint8 index,
        bytes memory innerCall
    ) external pure returns (bytes memory);
}
```

**6. Precompile Implementation (Rust)**

```rust
// precompiles/xcm-transactor/src/v4/mod.rs

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    TransactThroughDerivative = "transactThroughDerivative(uint8,uint16,(uint8,bytes[]),(uint64,uint64),bytes,uint256,(uint64,uint64),bool)",
    TransactThroughSigned = "transactThroughSigned(uint8,(uint8,bytes[]),(uint8,bytes[]),(uint64,uint64),bytes,uint256,(uint64,uint64),bool)",
    IndexToAccount = "indexToAccount(uint16)",
    TransactInfoWithSigned = "transactInfoWithSigned(uint8,(uint8,bytes[]))",
    FeePerSecond = "feePerSecond(uint8,(uint8,bytes[]))",
    EncodeUtilityAsDerivative = "encodeUtilityAsDerivative(uint8,uint8,bytes)",
}

impl<Runtime> XcmTransactorPrecompileV4<Runtime>
where
    Runtime: pallet_xcm_transactor::Config + pallet_evm::Config + frame_system::Config,
{
    fn transact_through_derivative(
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        // Read input
        read_args!(handle, {
            transactor_u8: u8,
            index: u16,
            fee_asset: MultiLocation,
            weight: EvmWeight,
            inner_call: BoundedBytes<GetDataLimit>,
            fee_amount: U256,
            overall_weight: EvmWeight,
            refund: bool
        });

        // Convert u8 to Transactors enum
        let transactor = match transactor_u8 {
            0 => Transactors::Relay,
            1 => Transactors::AssetHub,
            _ => return Err(revert("Invalid transactor")),
        };

        // Call shared implementation
        XcmTransactorWrapper::<Runtime>::transact_through_derivative_v4(
            handle,
            transactor,
            index,
            fee_asset,
            weight,
            inner_call,
            fee_amount,
            overall_weight,
            refund,
        )
    }
}
```

**7. Shared Implementation Updates**

```rust
// precompiles/xcm-transactor/src/functions.rs

impl<Runtime> XcmTransactorWrapper<Runtime>
where
    Runtime: pallet_xcm_transactor::Config + pallet_evm::Config,
{
    pub fn transact_through_derivative_v4(
        handle: &mut impl PrecompileHandle,
        transactor: Transactors,
        index: u16,
        // ... other params
    ) -> EvmResult<PrecompileOutput> {
        // Validate transactor is registered
        let destination = transactor.destination();

        // Build call to pallet
        let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
            dest: transactor,
            index,
            fee: CurrencyPayment {
                currency: Currency::AsMultiLocation(Box::new(fee_asset.into())),
                fee_amount: Some(fee_amount),
            },
            inner_call: inner_call.into(),
            weight_info: TransactWeights {
                transact_required_weight_at_most: weight.into(),
                overall_weight: Some(overall_weight.into()),
            },
            refund,
        };

        RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;
        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }
}
```

##### Compatibility Analysis with Polkadot Fellows Runtime

**8. AssetHub Staking Capabilities**

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

**9. Index Verification Strategy**

```bash
# Extract runtime metadata from AssetHub
polkadot-js-api query.system.properties --ws wss://polkadot-asset-hub-rpc.polkadot.io

# Decode metadata to find indices
subxt metadata --url wss://polkadot-asset-hub-rpc.polkadot.io > assethub-metadata.scale
subxt codegen --file assethub-metadata.scale | grep "pallet_index"
```

##### Testing Strategy

**10. Comprehensive Test Plan**

**Phase 1: Unit Tests (Rust)**

```rust
// pallets/xcm-transactor/src/tests.rs

#[test]
fn encode_assethub_bond_produces_correct_bytes() {
    ExtBuilder::default()
        .with_assethub_indices(AssetHubIndices {
            staking: 89,
            bond: 0,
            // ... etc
        })
        .build()
        .execute_with(|| {
            let call = AvailableStakeCalls::Bond(
                1_000_000_000_000, // 1 DOT
                RewardDestination::Staked,
            );

            let encoded = XcmTransactor::encode_call(
                Transactors::AssetHub,
                call
            );

            // Verify pallet index
            assert_eq!(encoded[0], 89);
            // Verify call index
            assert_eq!(encoded[1], 0);
            // Verify compact encoding of amount
            // ... assertions
        });
}

#[test]
fn transact_to_assethub_via_derivative_succeeds() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE, 1000 * UNIT)])
        .build()
        .execute_with(|| {
            // Register AssetHub transactor info
            assert_ok!(XcmTransactor::set_transact_info(
                RuntimeOrigin::root(),
                Box::new(xcm::VersionedLocation::V4(Transactors::AssetHub.destination())),
                // ... weight and fee config
            ));

            // Register derivative account
            assert_ok!(XcmTransactor::register(
                RuntimeOrigin::signed(ALICE),
                ALICE,
                0,
            ));

            // Encode bond call
            let inner_call = XcmTransactor::encode_call(
                Transactors::AssetHub,
                AvailableStakeCalls::Bond(10 * UNIT, RewardDestination::Staked),
            );

            // Execute transact
            assert_ok!(XcmTransactor::transact_through_derivative(
                RuntimeOrigin::signed(ALICE),
                Transactors::AssetHub,
                0,
                // ... fee and weight params
                inner_call,
            ));

            // Verify XCM message sent
            // ... assertions on XCM queue
        });
}
```

**Phase 2: Precompile Tests**

```rust
// precompiles/xcm-transactor/src/tests.rs

#[test]
fn test_v4_selectors() {
    use sha3::{Digest, Keccak256};

    assert_eq!(
        &Keccak256::digest(b"transactThroughDerivative(uint8,uint16,(uint8,bytes[]),(uint64,uint64),bytes,uint256,(uint64,uint64),bool)")[0..4],
        PCallV4::transact_through_derivative_selectors()
    );
}

#[test]
fn test_v4_transact_assethub_derivative() {
    ExtBuilder::default()
        .with_balances(vec![(Alice.into(), 1000 * UNIT)])
        .build()
        .execute_with(|| {
            let input = EvmDataWriter::new_with_selector(
                Action::TransactThroughDerivative
            )
            .write(1u8) // AssetHub
            .write(0u16) // index
            .write(Multilocation { parents: 1, interior: vec![...] })
            // ... params
            .build();

            precompiles()
                .prepare_test(Alice, Precompile, input)
                .execute_returns(EvmDataWriter::new().write(true).build());
        });
}
```

**Phase 3: Integration Tests (TypeScript/Moonwall)**

```typescript
// test/suites/dev/moonbase/test-xcm-transactor/test-xcm-assethub.ts

describeSuite({
  id: "D0305",
  title: "XCM Transactor V4 - AssetHub Staking",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should bond DOT on AssetHub via derivative",
      test: async () => {
        const XCM_TRANSACTOR_V4 = "0x0000000000000000000000000000000000000818";
        const Transactor = { Relay: 0, AssetHub: 1 };

        // Encode bond call using relay-encoder or manual SCALE
        const bondCall = encodeBondCall(10n * GLMR);

        // Create fee asset multilocation for AssetHub
        const feeAsset = {
          parents: 1,
          interior: [{ Parachain: 1000 }],
        };

        // Call precompile
        const { result } = await context.createBlock(
          context.polkadotJs().tx.ethereum.transact({
            to: XCM_TRANSACTOR_V4,
            data: encodeFunctionData({
              abi: XcmTransactorV4ABI,
              functionName: "transactThroughDerivative",
              args: [
                Transactor.AssetHub,
                0, // index
                feeAsset,
                { refTime: 1_000_000_000, proofSize: 64_000 },
                bondCall,
                1000000n,
                { refTime: 2_000_000_000, proofSize: 128_000 },
                true, // refund
              ],
            }),
            // ... gas params
          })
        );

        expect(result?.successful).to.be.true;

        // Verify XCM message in outbound queue
        const messages = await context.polkadotJs().query.xcmpQueue.outboundXcmpMessages.entries();
        expect(messages.length).to.be.greaterThan(0);

        // Could also use chopsticks to verify on AssetHub side
      },
    });

    it({
      id: "T02",
      title: "should nominate validators on AssetHub",
      test: async () => {
        // Similar test for nominate call
        // ...
      },
    });
  },
});
```

**Phase 4: Chopsticks Fork Testing**

```typescript
// test/helpers/assethub-fork.ts

import { setup } from "@acala-network/chopsticks";

export async function setupAssetHubFork() {
  const assetHub = await setup({
    endpoint: "wss://polkadot-asset-hub-rpc.polkadot.io",
    db: "./db.sqlite",
    port: 8001,
  });

  const moonbeam = await setup({
    endpoint: "wss://wss.api.moonbeam.network",
    db: "./db-moonbeam.sqlite",
    port: 8000,
  });

  // Connect via HRMP
  await setupHrmp(moonbeam, assetHub);

  return { assetHub, moonbeam };
}

// Use in tests to verify end-to-end flow
```

**Phase 5: Smoke Tests**

```typescript
// test/suites/smoke/moonbeam/test-xcm-transactor-assethub.ts

describeSuite({
  id: "S0305",
  title: "XCM Transactor AssetHub - Smoke Test",
  foundationMethods: "read_only",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "V4 precompile should be deployed",
      test: async () => {
        const code = await context.viem().getBytecode({
          address: "0x0000000000000000000000000000000000000818",
        });
        expect(code).to.not.equal("0x");
      },
    });

    it({
      id: "T02",
      title: "AssetHub transactor info should be configured",
      test: async () => {
        const info = await context.polkadotJs().query.xcmTransactor.transactorInfo(
          { AssetHub: null }
        );
        expect(info.isSome).to.be.true;
      },
    });
  },
});
```

**Testing Matrix:**

| Test Type | Coverage | Tools | Priority |
|-----------|----------|-------|----------|
| Unit - Encoding | All AvailableStakeCalls | Rust cargo test | P0 |
| Unit - Storage Migration | Old→New format | Rust + try-runtime | P0 |
| Unit - Pallet Logic | Transact calls | Rust cargo test | P0 |
| Precompile - Selectors | V4 function sigs | Rust cargo test | P0 |
| Precompile - Modifiers | View/state-changing | Rust cargo test | P0 |
| Integration - XCM Messages | Message formation | Moonwall dev tests | P1 |
| Integration - End-to-end | Full flow | Chopsticks | P1 |
| Smoke - Deployment | Runtime config | Moonwall smoke | P1 |
| Fuzz - Invalid Inputs | Edge cases | Rust proptest | P2 |

##### Migration Strategy

**11. Storage Migration**

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

**12. Solidity Documentation**

Update technical documentation at `precompiles/xcm-transactor/src/v4/README.md`:

```markdown
# XCM Transactor V4 Precompile

Address: `0x0000000000000000000000000000000000000818`

## Overview

The XCM Transactor V4 precompile enables Ethereum-style contracts on Moonbeam
to execute calls on remote chains via XCM (Cross-Consensus Messaging).

Version 4 adds support for **AssetHub** as a transaction destination, enabling
staking, asset management, and proxy operations on the Polkadot AssetHub
system parachain.

## Supported Destinations

- **Relay (0)**: Polkadot/Kusama/Westend Relay Chain
- **AssetHub (1)**: Polkadot/Kusama AssetHub system parachain

## Key Features

### AssetHub Staking

Execute DOT staking operations through AssetHub's delegated staking:

solidity
// Bond 10 DOT on AssetHub
bytes memory bondCall = encodeAssetHubBond(10 ether, REWARD_STAKED);

xcmTransactorV4.transactThroughDerivative(
    XcmTransactorV4.Transactor.AssetHub,
    0, // derivative index
    assetHubFeeAsset,
    Weight(1_000_000_000, 64_000),
    bondCall,
    1_000_000,
    Weight(2_000_000_000, 128_000),
    true // refund
);


## Migration from V3

V3 users can upgrade to V4 by:
1. Changing precompile address to `0x0818`
2. Adding `Transactor` enum parameter (use `Transactor.Relay` for same behavior)
3. Updating function signatures (selectors changed)

## Examples

See `examples/` directory for complete integration examples.
```

**Pros:**
- Extensible to additional chains (BridgeHub, Collectives, etc.)
- Clear separation of encoding logic per chain
- Configurable indices per chain via runtime configuration
- Maintains backwards compatibility (V3 still works)
- Type-safe chain selection
- Future-proof architecture for multi-chain support
- Comprehensive testing coverage
- Verified compatibility with AssetHub runtime

**Cons:**
- Requires creating new precompile version (V4)
- Moderate implementation complexity
- Need to update trait signatures across the codebase
- Storage migration required
- Requires maintaining AssetHub pallet indices on runtime upgrades
- Additional testing burden (3x test matrix: Relay, AssetHub, compatibility)

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
   - **DECISION**: Create new V4 precompile at `0x0818`
   - Unified interface with `Transactor` enum parameter
   - V1, V2, V3 remain unchanged and functional (backwards compatibility)
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

**Phase 3: Precompile V4** (1-2 weeks)
- [ ] Create `precompiles/xcm-transactor/src/v4/` structure
- [ ] Implement V4 precompile with `Transactor` parameter
- [ ] Write Solidity interface `XcmTransactorV4.sol`
- [ ] Register V4 at address `0x0818` in runtime
- [ ] Update shared `functions.rs` implementation

**Phase 4: Testing & Integration** (2 weeks)
- [ ] Unit tests (encoding, storage, pallet logic)
- [ ] Precompile tests (selectors, modifiers, integration)
- [ ] Moonwall integration tests (dev and smoke)
- [ ] Chopsticks fork testing with live AssetHub
- [ ] Documentation updates (inline, README, migration guide)

**Phase 5: Deployment** (1 week)
- [ ] Moonbase Alpha testnet deployment
- [ ] Community testing period
- [ ] Runtime upgrade to Moonriver (Kusama)
- [ ] Runtime upgrade to Moonbeam (Polkadot)

**Total Estimated Timeline:** 6-8 weeks

### Success Criteria

- [ ] All `AvailableStakeCalls` successfully encode for AssetHub
- [ ] Storage migration tested with try-runtime on all runtimes
- [ ] V4 precompile deployed and functional on all networks
- [ ] End-to-end test: Bond DOT on AssetHub via Moonbeam derivative account
- [ ] Zero breaking changes to existing V1-V3 precompile users
- [ ] Documentation complete and reviewed
- [ ] Security review passed (if required)

### Rollback Plan

If critical issues are discovered:
1. V4 precompile can be disabled without affecting V1-V3
2. Storage migration includes rollback capability via try-runtime
3. AssetHub `Transactor` variant can be feature-flagged
4. Worst case: Runtime upgrade to remove AssetHub support, restore legacy behavior

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Moonbeam EVM Layer                          │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ V1 Precompile│  │ V2 Precompile│  │ V3 Precompile│             │
│  │   (0x0806)   │  │   (0x080D)   │  │   (0x0817)   │             │
│  │ Relay Only   │  │ Relay Only   │  │ Relay Only   │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │                 │                 │                       │
│         └─────────────────┴─────────────────┘                       │
│                           │                                         │
│  ┌────────────────────────▼──────────────────────────┐             │
│  │          V4 Precompile (0x0818) NEW!              │             │
│  │                                                    │             │
│  │  function transactThroughDerivative(              │             │
│  │    Transactor transactor,  // Relay | AssetHub   │             │
│  │    ...                                            │             │
│  │  )                                                │             │
│  └────────────────────────┬──────────────────────────┘             │
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

1. Solidity Contract calls V4 Precompile:
   xcmTransactorV4.transactThroughDerivative(
     Transactor.AssetHub,  // Target: AssetHub
     0,                     // Derivative index
     feeAsset,
     weight,
     bondCall,              // SCALE-encoded Bond(10 DOT)
     ...
   )

2. V4 Precompile converts to pallet call:
   pallet_xcm_transactor::transact_through_derivative(
     dest: Transactors::AssetHub,
     ...
   )

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
