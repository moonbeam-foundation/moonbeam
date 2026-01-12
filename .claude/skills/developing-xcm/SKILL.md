---
name: developing-xcm
description: Develops XCM (Cross-Consensus Messaging) functionality for cross-chain communication between Moonbeam and other parachains/relay chains. Use when implementing cross-chain asset transfers, adding support for remote chains, debugging XCM message failures, configuring XCM fees, or registering foreign assets.
---

# XCM and Cross-Chain Development

## Contents
- [XCM Architecture Overview](#xcm-architecture-overview)
- [Common Tasks](#common-tasks)
- [XCM Precompile Development](#xcm-precompile-development)
- [Testing XCM](#testing-xcm)
- [Debugging XCM](#debugging-xcm)

## XCM Architecture Overview

### Key Components

| Component            | Location                           | Purpose                        |
|----------------------|------------------------------------|--------------------------------|
| XCM Config           | `runtime/*/xcm_config.rs`          | XCM configuration and barriers |
| XCM Transactor       | `pallets/xcm-transactor/`          | Remote chain execution         |
| XTokens Precompile   | `precompiles/xtokens/`             | EVM interface for transfers    |
| XCM Utils Precompile | `precompiles/xcm-utils/`           | XCM utility functions          |
| Foreign Assets       | `pallets/moonbeam-foreign-assets/` | Foreign asset management       |

### Message Flow

```
Moonbeam EVM Contract
        │
        ▼
XTokens Precompile (0x804)
        │
        ▼
pallet-xtokens
        │
        ▼
XCM Router
        │
        ▼
Relay Chain / Destination Parachain
```

## Common Tasks

### 1. Register a Foreign Asset

```rust
// Via governance or sudo
pallet_moonbeam_foreign_assets::Call::create_foreign_asset {
    asset_id: xcm::v4::Location {
        parents: 1,
        interior: Junctions::X3([
            Parachain(1000), // Source parachain
            PalletInstance(50), // Assets pallet
            GeneralIndex(1984), // Asset ID
        ].into()),
    },
    decimals: 12,
    symbol: "DOT".into(),
    name: "Polkadot".into(),
    // ...
}
```

### 2. Configure XCM Barriers

```rust
// runtime/moonbase/xcm_config.rs
pub type Barrier = TrailingSetTopicAsId<(
    TakeWeightCredit,
    // Allow parent (relay) to execute
    AllowTopLevelPaidExecutionFrom<ParentLocation>,
    // Allow sibling parachains
    AllowTopLevelPaidExecutionFrom<SiblingParachains>,
    // Allow trusted reserves
    AllowKnownQueryResponses<PolkadotXcm>,
    WithComputedOrigin<
        (
            AllowTopLevelPaidExecutionFrom<Everything>,
            AllowSubscriptionsFrom<Everything>,
        ),
        UniversalLocation,
        ConstU32<8>,
    >,
)>;
```

### 3. Add a New Reserve Asset

```rust
// In xcm_config.rs
pub struct IsTrustedReserve;
impl ContainsPair<Asset, Location> for IsTrustedReserve {
    fn contains(asset: &Asset, origin: &Location) -> bool {
        // Check if origin is trusted reserve for asset
        matches!(
            (asset.id.0.unpack(), origin.unpack()),
            // DOT from relay chain
            ((1, []), (1, [])) |
            // Asset from parachain 1000
            ((1, [Parachain(1000), ..]), (1, [Parachain(1000)])) |
            // Add new reserve patterns here
        )
    }
}
```

### 4. Configure Fee Handling

```rust
// Weight to fee conversion for XCM
pub struct XcmWeightToFee;
impl WeightToFee for XcmWeightToFee {
    type Balance = Balance;

    fn weight_to_fee(weight: &Weight) -> Self::Balance {
        // Convert weight to native token fee
        let base_fee: Balance = 1_000_000_000; // 1 GLMR base
        let per_second: Balance = WEIGHT_REF_TIME_PER_SECOND as Balance;

        base_fee.saturating_add(
            weight.ref_time().saturating_mul(WEIGHT_FEE) / per_second
        )
    }
}
```

### 5. XCM Transactor for Remote Calls

```rust
// Execute a call on another chain
pallet_xcm_transactor::Call::transact_through_sovereign {
    dest: Location::new(1, [Parachain(1000)]),
    fee_location: Box::new(VersionedLocation::V4(Location::parent())),
    fee: CurrencyPayment {
        currency: Currency::AsMultiLocation(Box::new(VersionedLocation::V4(Location::parent()))),
        fee_amount: Some(1_000_000_000),
    },
    call: remote_call.encode(),
    origin_kind: OriginKind::SovereignAccount,
    weight_info: TransactWeights {
        require_weight_at_most: Weight::from_parts(1_000_000_000, 65536),
        transact_extra_weight: Weight::zero(),
        overall_weight: None,
        transact_extra_weight_signed: None,
    },
}
```

## XCM Precompile Development

### XTokens Precompile (Asset Transfers)

```rust
// precompiles/xtokens/src/lib.rs
#[precompile::public("transfer(address,uint256,(uint8,bytes[]),uint64)")]
fn transfer(
    handle: &mut impl PrecompileHandle,
    currency_address: Address,
    amount: U256,
    destination: Location,
    weight: u64,
) -> EvmResult {
    // Convert EVM address to currency ID
    let currency_id = Self::address_to_currency_id(currency_address)?;

    // Execute XCM transfer
    pallet_xtokens::Pallet::<Runtime>::transfer(
        origin,
        currency_id,
        amount.try_into()?,
        Box::new(VersionedLocation::V4(destination)),
        WeightLimit::Limited(Weight::from_parts(weight, 65536)),
    )?;

    Ok(())
}
```

### XCM Utils Precompile

```rust
// precompiles/xcm-utils/src/lib.rs
#[precompile::public("weightMessage(bytes)")]
#[precompile::view]
fn weight_message(
    handle: &mut impl PrecompileHandle,
    message: Vec<u8>,
) -> EvmResult<u64> {
    let msg: VersionedXcm<()> = VersionedXcm::decode(&mut &message[..])
        .map_err(|_| revert("Failed to decode XCM"))?;

    let weight = T::Weigher::weight(&mut msg.try_into()?)
        .map_err(|_| revert("Failed to weight XCM"))?;

    Ok(weight.ref_time())
}
```

## Testing XCM

### Unit Tests

```rust
#[test]
fn test_xcm_transfer_to_relay() {
    new_test_ext().execute_with(|| {
        let dest = Location::parent();
        let beneficiary = Location::new(0, [AccountId32 { network: None, id: [1u8; 32] }]);
        let amount = 1_000_000_000_000;

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            CurrencyId::SelfReserve,
            amount,
            Box::new(VersionedLocation::V4(dest)),
            WeightLimit::Unlimited,
        ));
    });
}
```

### Integration Tests (Zombienet)

```typescript
// test/suites/zombie/test-xcm-transfer.ts
describeSuite({
  id: "Z010101",
  title: "XCM Transfer Test",
  foundationMethods: "zombie",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Should transfer DOT from relay to Moonbeam",
      test: async () => {
        // Execute XCM transfer from relay chain
        await context.relayApi.tx.xcmPallet.limitedReserveTransferAssets(
          // ...
        ).signAndSend(alice);

        // Verify receipt on Moonbeam
        const balance = await context.moonbeamApi.query.assets.account(
          DOT_ASSET_ID,
          aliceMoonbeam
        );
        expect(balance.balance.toBigInt()).toBeGreaterThan(0n);
      },
    });
  },
});
```

### Local Testing with Chopsticks

```typescript
// test/suites/chopsticks/test-xcm-fork.ts
// Test XCM on forked mainnet state
const config = {
  moonbeam: {
    endpoint: "wss://wss.api.moonbeam.network",
    block: 5000000,
  },
  polkadot: {
    endpoint: "wss://rpc.polkadot.io",
    block: 20000000,
  },
};
```

## Debugging XCM

### Enable XCM Logging

```bash
RUST_LOG=xcm=trace,xcm_executor=trace ./target/release/moonbeam --dev
```

### Common XCM Errors

| Error                      | Cause                  | Solution                        |
|----------------------------|------------------------|---------------------------------|
| `TooExpensive`             | Insufficient fee       | Increase fee or adjust weight   |
| `UntrustedReserveLocation` | Asset not trusted      | Add to trusted reserves         |
| `AssetNotFound`            | Asset not registered   | Register the foreign asset      |
| `Barrier`                  | Message blocked        | Update barrier configuration    |
| `FailedToDecode`           | Invalid message format | Check XCM version compatibility |

### Trace XCM Message

```rust
// Add to xcm_config.rs for debugging
impl xcm_executor::Config for XcmConfig {
    type AssetTransactor = AssetTransactor;
    type OriginConverter = OriginConverter;
    type IsReserve = IsTrustedReserve;
    type IsTeleporter = ();
    type Barrier = Barrier;
    // Add message notifier for debugging
    type ResponseHandler = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
    // ...
}
```

## XCM Version Management

```rust
// Ensure version compatibility
impl GetVersion for PolkadotXcm {
    fn get_version(dest: &Location) -> Option<XcmVersion> {
        // Return the XCM version supported by destination
        match dest.unpack() {
            (1, []) => Some(4), // Relay chain supports v4
            (1, [Parachain(1000)]) => Some(4), // AssetHub supports v4
            _ => Some(3), // Default to v3 for unknown
        }
    }
}
```

## Resources

- XCM Documentation: https://wiki.polkadot.network/docs/learn-xcm
- XCM Format: https://github.com/paritytech/xcm-format
- Moonbeam XCM Guide: https://docs.moonbeam.network/builders/interoperability/xcm/
