# ADR-001: XCM Tests Refactoring

## Status

Proposed (implemented partially; see "Current State")

## Context

The XCM tests in `runtime/{moonbeam,moonriver,moonbase}/tests/xcm_tests.rs` have significant issues:

- **~22,000 lines** of test infrastructure (duplicated across 3 runtimes)
- **Mock runtime diverges from production** - tests pass but production behavior differs
- **Tests do too much** - single tests register assets, fund accounts, do multiple transfers
- **Hard to maintain** - changes must be synchronized across all runtimes

### What Tests Should Validate

1. XCM Barrier configuration
2. Reserve asset acceptance
3. Fee handling (traders, treasury)
4. Asset transactors (native and foreign)
5. Location-to-account conversions
6. Remote execution (XcmTransactor)
7. EVM integration

## Decision

Implement a **hybrid test suite** with two levels, both using **real runtime configuration**:

| Level                          | Tool                                           | Purpose                                                        | Speed             |
| ------------------------------ | ---------------------------------------------- | -------------------------------------------------------------- | ----------------- |
| **Level 1: Config Tests**      | Direct `XcmExecutor::execute()`                | Test XCM config components                                     | Fast (~seconds)   |
| **Level 2: Integration Tests** | `xcm-emulator` **plus** `xcm-simulator`        | End-to-end flows through pallets + executor-level coverage     | Slower (~minutes) |

Both levels run on every commit.

---

## Level 1: XCM Config Tests

Test XCM configuration by calling `XcmExecutor::execute()` directly with the **real `XcmConfig`**:

```rust
use moonbeam_runtime::xcm_config::XcmConfig;
use xcm_executor::XcmExecutor;

#[test]
fn barrier_rejects_unpaid_execution_from_sibling() {
    new_test_ext().execute_with(|| {
        let origin = Location::new(1, [Parachain(2000)]);
        let message = Xcm(vec![
            UnpaidExecution { weight_limit: Unlimited, check_origin: None },
            Transact { /* ... */ },
        ]);

        let outcome = XcmExecutor::<XcmConfig>::prepare_and_execute(
            origin, message, &mut [0u8; 32], Weight::MAX, Weight::zero(),
        );

        assert!(matches!(outcome, Outcome::Error { error: XcmError::Barrier, .. }));
    });
}
```

**Tests**: Barriers, Traders, Weigher, IsReserve, AssetTransactor, LocationToAccountId

---

## Level 2: XCM Integration Tests

Test full cross-chain flows using `xcm-emulator` with:
- **Moonbeam**: Real runtime
- **Relay/AssetHub**: Minimal mocks

```rust
#[test]
fn transfer_dot_from_relay_to_moonbeam() {
    PolkadotMoonbeamNet::reset();

    Polkadot::execute_with(|| {
        assert_ok!(XcmPallet::reserve_transfer_assets(/* ... */));
    });

    Moonbeam::execute_with(|| {
        assert!(EvmForeignAssets::balance(DOT_ASSET_ID, ALITH).unwrap() > 0);
    });
}
```

**Tests**: Transfers, XcmTransactor, HRMP channels, EVM integration, fee collection

---

## Project Structure

```
moonbeam/
├── test-utils/xcm-test-utils/     # Shared helper utilities
│   └── src/
│       ├── accounts.rs
│       ├── locations.rs
│       └── xcm_helpers.rs
│
├── runtime/moonbeam/tests/
│   ├── xcm_config_tests/          # Level 1: Fast config tests (39 tests)
│   ├── xcm_integration_tests/     # Level 2B: xcm-simulator tests (32 tests)
│   └── xcm_emulator_tests/        # Level 2A: xcm-emulator tests (4 tests, incl. e2e transfer)
│       ├── emulator_relay.rs      #   Westend relay genesis
│       ├── emulator_network.rs    #   Network + chain declarations
│       └── emulator_transfer_tests.rs
│
├── runtime/moonbase/tests/
│   ├── xcm_config_tests/          # Level 1: Fast config tests (39 tests)
│   └── xcm_integration_tests/     # Level 2B: xcm-simulator tests (32 tests)
│
└── runtime/moonriver/tests/
    ├── xcm_config_tests/          # Level 1: Fast config tests (39 tests)
    └── xcm_integration_tests/     # Level 2B: xcm-simulator tests (32 tests)
```

---

## Implementation Plan

1. **Phase 1**: Level 1 infrastructure + barrier/trader/reserve tests ✅
2. **Phase 2**: Level 2 infrastructure + basic transfer test ✅
3. **Phase 3**: Migrate remaining tests, simplify (one behavior per test) ✅
4. **Phase 4**: Add Moonriver/Moonbase support ✅
5. **Phase 5**: Remove old `xcm_mock/` and `xcm_tests.rs` ✅
6. **Phase 6**: Add `xcm-emulator` network + tests ✅ (Moonbeam, incl. e2e DMP transfer)

---

## Consequences

**Positive**:
- No mock divergence - both levels use real XcmConfig/runtime
- Fast feedback from Level 1, comprehensive coverage from Level 2
- Clear separation of concerns
- Shared infrastructure across runtimes

**Negative**:
- Two test systems to maintain
- Initial setup effort

---

## Current State (Feb 2026)

**Observed implementation**:
- Level 1 (config) and Level 2 (integration) tests exist for all three runtimes:
  Moonbeam, Moonbase, and Moonriver.
- Level 2 uses `xcm-simulator` with a custom message bus.
- `moonbeam-xcm-test-utils` is shared across all three runtimes.
- Old `xcm_tests.rs` and `xcm_mock/` were removed and replaced with the new structure.
- A minimal `xcm-emulator` harness exists for Moonbeam only (3 tests, partial coverage).
- Test counts: 39 config + 32 integration per runtime (× 3 runtimes), + 4 emulator (Moonbeam).

**`xcm-emulator` integration status (Feb 2026)**:
A minimal `xcm-emulator` harness has been added at `runtime/moonbeam/tests/xcm_emulator_tests/`.
It uses the real `westend_runtime` as the relay chain and the real `moonbeam_runtime` as the
parachain. Three passing tests validate network initialization, sovereign account funding, and
reserve-transfer-type classification.

### Blockers encountered and resolved during `xcm-emulator` integration

1. **Mandatory inherents** ✅ — Moonbeam has two pallets (`pallet_author_inherent`,
   `pallet_randomness`) that assert in `on_finalize` that their mandatory inherents were
   dispatched during the block. The emulator only dispatches `set_validation_data` and
   `pallet_timestamp::set`. **Fix**: `satisfy_moonbeam_inherents()` patches the
   storage items (`Author`, `InherentIncluded`) directly and clears `NotFirstBlock` so
   VRF verification is skipped. Must be called in every `MoonbeamPara::execute_with`.

2. **VRF pre-digest** ✅ — `pallet_randomness` requires a VRF pre-runtime digest in the
   block header from block 2 onward. The emulator's `DigestProvider` (defaulting to `()`)
   produces an empty digest. **Fix**: clear `Randomness::NotFirstBlock` after
   `Parachain::init()` via `ext_wrapper` so every block takes the genesis (first-block) path.

3. **`ParachainHost` runtime API** ✅ — The emulator's `decl_test_relay_chains!` macro
   calls `Runtime::dmq_contents(para_id)`, which requires the `ParachainHost` runtime API
   (version 13). **Fix**: use `westend_runtime` as the relay chain (implements the full API).

4. **DMP routing requires parachain registration** ✅ — The relay's `ChildParachainRouter`
   checks `paras::Heads::contains_key(para)` before routing DMP. **Fix**: insert a dummy
   `HeadData` for Moonbeam's `ParaId` directly into relay genesis storage.
   No need for full validator/session setup or `emulated-integration-tests-common`.

5. **Westend Asset Hub Migration guard** ✅ — `pallet_xcm::limited_reserve_transfer_assets`
   and `transfer_assets` are blocked for network-native assets on Westend. **Fix**: use
   `transfer_assets_using_type_and_then` with explicit `TransferType::LocalReserve`.

6. **XCM fee pricing (TooExpensive)** ✅ — The `XcmWeightTrader` relative_price must
   account for the decimal difference between DOT (10 decimals) and GLMR (18 decimals).
   A value of `10^28` provides sufficient headroom for test XCM execution fees.

### Result

All blockers have been overcome. A full end-to-end DMP transfer test
(`transfer_dot_from_relay_to_moonbeam`) passes: DOT is sent from the Westend relay,
routed via DMP, deposited as a foreign ERC20 asset on Moonbeam, and the beneficiary's
balance is verified.

---

## Updated Testing Strategy (Using Both `xcm-emulator` and `xcm-simulator`)

### Level 2A: `xcm-emulator` (Preferred, pallet-level)
Use for full end-to-end flows that go through pallets and message queues:
- `pallet_xcm` and `xcm_pallet` flows
- `XTokens`, `XcmTransactor`, HRMP management
- Fee accounting in Treasury
- Multi-hop routing and message queue interactions

### Level 2B: `xcm-simulator` (Fallback, executor-level)
Use when:
- The emulator network isn’t wired yet for a given runtime
- A test needs fast, deterministic, executor-only behavior
- You want direct control over message routing without pallet queues

**Goal**: Migrate tests to `xcm-emulator` where it adds value, keep `xcm-simulator` for fast or low-level validation.

---

## Formal Test Specification

### Types

```
TYPE Location
  STRUCTURE: { parents: u8, interior: Junctions }
  INVARIANT: parents ≤ 255 ∧ interior.len() ≤ 8

TYPE Origin
  VARIANTS:
    | Relay                                    // Parent chain
    | Sibling(ParaId)                          // Sibling parachain
    | AccountKey20(H160)                       // 20-byte account
    | ForeignConsensus(NetworkId, Location)   // Cross-consensus

TYPE XcmOutcome
  VARIANTS:
    | Complete(Weight)                // Fully executed
    | Incomplete(Weight, XcmError)    // Partially executed (including barrier rejection in some configs)
    | Error(XcmError)                 // Failed at barrier or other errors
```

---

### Level 1: XCM Config Test Cases

#### Barrier Tests

| Test                                               | Property                                                                | Expected       |
| -------------------------------------------------- | ----------------------------------------------------------------------- | -------------- |
| `barrier_allows_paid_execution_from_relay`         | `∀ assets. WithdrawAsset + BuyExecution from Relay`                     | `Ok`           |
| `barrier_allows_paid_execution_from_sibling`       | `∀ para_id, assets. WithdrawAsset + BuyExecution from Sibling(para_id)` | `Ok`           |
| `barrier_allows_paid_execution_from_account_key20` | `∀ key, assets. WithdrawAsset + BuyExecution from AccountKey20(key)`    | `Ok`           |
| `barrier_rejects_unpaid_execution_from_sibling`    | `∀ para_id. UnpaidExecution from Sibling(para_id)`                      | `Err(Barrier)` |
| `barrier_rejects_unpaid_transact_from_sibling`     | `∀ para_id, call. UnpaidExecution + Transact from Sibling(para_id)`     | `Err(Barrier)` |
| `barrier_allows_subscription_from_any_origin`      | `∀ origin. SubscribeVersion from origin`                                | `Ok`           |
| `barrier_allows_known_query_response`              | `∀ origin, query_id. QueryResponse where query_id is expected`          | `Ok`           |
| `barrier_rejects_unknown_query_response`           | `∀ origin, query_id. QueryResponse where query_id is NOT expected`      | `Err(Barrier)` |

#### Reserve Tests

| Test                                           | Property                                                               | Expected |
| ---------------------------------------------- | ---------------------------------------------------------------------- | -------- |
| `reserve_accepts_dot_from_relay`               | `is_reserve(DOT, Relay)`                                               | `true`   |
| `reserve_accepts_dot_from_asset_hub`           | `is_reserve(DOT, Sibling(1000))`                                       | `true`   |
| `reserve_rejects_dot_from_other_sibling`       | `∀ para_id ≠ 1000. is_reserve(DOT, Sibling(para_id))`                  | `false`  |
| `reserve_accepts_bridged_asset_from_asset_hub` | `∀ bridged. is_reserve(bridged, Sibling(1000))`                        | `true`   |
| `reserve_accepts_bridged_asset_from_moonriver` | `∀ bridged. is_reserve(bridged, Moonriver)`                            | `true`   |
| `reserve_accepts_self_reserve_asset`           | `∀ origin, asset. reserve(asset) = origin → is_reserve(asset, origin)` | `true`   |
| `teleport_always_rejected`                     | `∀ asset, origin. is_teleporter(asset, origin)`                        | `false`  |

#### Trader Tests

| Test                                              | Property                                                                               | Expected            |
| ------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------- |
| `trader_accepts_native_asset_for_fees`            | `buy_execution(GLMR, weight)`                                                          | `Ok`                |
| `trader_accepts_registered_foreign_asset`         | `∀ asset_id. registered(asset_id) → buy_execution(asset_id, weight)`                   | `Ok`                |
| `trader_rejects_unregistered_foreign_asset`       | `∀ asset_id. ¬registered(asset_id) → buy_execution(asset_id, weight)`                  | `Err(TooExpensive)` |
| `trader_rejects_insufficient_fees`                | `∀ asset, weight. amount < required_fee(weight, asset) → buy_execution(asset, weight)` | `Err(TooExpensive)` |
| `trader_calculates_native_fee_correctly`          | `fee(GLMR, weight) = weight_to_fee(weight)`                                            | exact match         |
| `trader_calculates_foreign_fee_using_price_ratio` | `fee(foreign, weight) = weight_to_fee(weight) * native_price / foreign_price`          | exact match         |
| `trader_deposits_fees_to_treasury`                | `∀ execution. treasury_balance_after ≥ treasury_balance_before + fees_paid`            | `true`              |
| `trader_refunds_unused_weight`                    | `∀ bought, used. used < bought → refund ≈ (bought - used) * fee_rate`                  | within 1%           |

#### Asset Transactor Tests

| Test                                            | Property                                                                 | Expected                |
| ----------------------------------------------- | ------------------------------------------------------------------------ | ----------------------- |
| `transactor_withdraws_native_asset`             | `withdraw(GLMR, account, amount) where balance ≥ amount`                 | `Ok`, balance decreased |
| `transactor_deposits_native_asset`              | `deposit(GLMR, account, amount)`                                         | `Ok`, balance increased |
| `transactor_withdraws_foreign_asset`            | `withdraw(foreign, account, amount) where registered ∧ balance ≥ amount` | `Ok`, balance decreased |
| `transactor_deposits_foreign_asset`             | `deposit(foreign, account, amount) where registered`                     | `Ok`, balance increased |
| `transactor_rejects_unregistered_foreign_asset` | `withdraw(unregistered, account, amount)`                                | `Err(AssetNotFound)`    |
| `transactor_handles_erc20_bridge_asset`         | `withdraw/deposit(erc20_bridge_asset, account, amount)`                  | `Ok`                    |

#### Location Conversion Tests

| Test                                           | Property                                                            | Expected                     |
| ---------------------------------------------- | ------------------------------------------------------------------- | ---------------------------- |
| `location_converts_relay_to_sovereign`         | `convert(Location::parent())`                                       | `RELAY_SOVEREIGN`            |
| `location_converts_sibling_to_sovereign`       | `∀ para_id. convert(Sibling(para_id))`                              | `sibling_sovereign(para_id)` |
| `location_converts_account_key20_to_h160`      | `∀ key. convert(AccountKey20(key))`                                 | `H160(key)`                  |
| `location_converts_foreign_consensus_via_hash` | `∀ network, interior. convert(GlobalConsensus(network) / interior)` | `hashed_description(...)`    |
| `location_rejects_invalid_location`            | `convert(invalid_location)`                                         | `None`                       |

#### Weigher Tests

| Test                                          | Property                           | Expected |
| --------------------------------------------- | ---------------------------------- | -------- |
| `weigher_calculates_weight_for_withdraw`      | `weigh(WithdrawAsset)`             | `> 0`    |
| `weigher_calculates_weight_for_deposit`       | `weigh(DepositAsset)`              | `> 0`    |
| `weigher_calculates_weight_for_buy_execution` | `weigh(BuyExecution)`              | `> 0`    |
| `weigher_calculates_weight_for_transact`      | `weigh(Transact)`                  | `> 0`    |
| `weigher_rejects_too_many_instructions`       | `weigh(xcm) where xcm.len() > 100` | `Err`    |

---

### Level 2: Integration Test Cases

#### Transfer Tests

| Test                                      | Precondition              | Action                                             | Postcondition                    |
| ----------------------------------------- | ------------------------- | -------------------------------------------------- | -------------------------------- |
| `transfer_dot_from_relay_to_moonbeam`     | Relay: Alice has DOT      | `reserve_transfer(Alice, Moonbeam(Alith), amount)` | Moonbeam: Alith has DOT - fees   |
| `transfer_dot_from_moonbeam_to_relay`     | Moonbeam: Alith has DOT   | `xtokens_transfer(Alith, Relay(Alice), amount)`    | Relay: Alice has more DOT        |
| `transfer_dot_from_moonbeam_to_asset_hub` | Moonbeam: Alith has DOT   | `xtokens_transfer(Alith, AssetHub(Alice), amount)` | AssetHub: Alice has DOT - fees   |
| `transfer_asset_from_moonbeam_to_sibling` | Moonbeam: Alith has asset | `xtokens_transfer(Alith, Sibling(Bob), amount)`    | Sibling: Bob has asset - fees    |
| `transfer_asset_from_sibling_to_moonbeam` | Sibling: Bob has asset    | `xcm_transfer(Bob, Moonbeam(Alith), amount)`       | Moonbeam: Alith has asset - fees |
| `transfer_roundtrip_preserves_supply`     | Moonbeam: initial balance | send to relay, receive back                        | balance ≈ initial (within fees)  |

#### XcmTransactor Tests

| Test                                     | Precondition                    | Action                                               | Postcondition                              |
| ---------------------------------------- | ------------------------------- | ---------------------------------------------------- | ------------------------------------------ |
| `transact_derivative_to_relay`           | derivative registered for index | `transact_through_derivative(Relay, index, call)`    | Relay: call executed by derivative account |
| `transact_derivative_to_asset_hub`       | derivative registered for index | `transact_through_derivative(AssetHub, index, call)` | AssetHub: call executed                    |
| `transact_derivative_with_custom_fee`    | derivative registered           | `transact_through_derivative(..., fee_amount)`       | fee_used ≤ fee_amount                      |
| `transact_derivative_with_custom_weight` | derivative registered           | `transact_through_derivative(..., weight)`           | weight_used ≤ weight                       |
| `transact_derivative_refunds_unused`     | derivative registered           | execute with excess weight                           | refund received                            |
| `transact_sovereign_to_relay`            | sovereign funded                | `transact_through_sovereign(Relay, call)`            | Relay: call executed by Moonbeam sovereign |
| `transact_sovereign_with_fee_payer`      | fee_payer funded                | `transact_through_sovereign(..., fee_payer)`         | fee deducted from fee_payer                |

#### HRMP Channel Tests

| Test                                   | Precondition                | Action                                 | Postcondition                  |
| -------------------------------------- | --------------------------- | -------------------------------------- | ------------------------------ |
| `hrmp_init_channel_via_root`           | sovereign has relay balance | `hrmp_manage(Root, InitOpen(para_id))` | Relay: channel request pending |
| `hrmp_accept_channel_via_root`         | pending request exists      | `hrmp_manage(Root, Accept(para_id))`   | Relay: channel opened          |
| `hrmp_close_channel_via_root`          | channel open                | `hrmp_manage(Root, Close(para_id))`    | Relay: channel closing         |
| `hrmp_fails_with_insufficient_balance` | sovereign underfunded       | `hrmp_manage(Root, InitOpen(para_id))` | `Err`                          |

#### EVM Integration Tests

| Test                             | Precondition                         | Action                                       | Postcondition           |
| -------------------------------- | ------------------------------------ | -------------------------------------------- | ----------------------- |
| `evm_sees_foreign_asset_balance` | Alith received foreign asset via XCM | `evm_call(precompile.balanceOf(Alith))`      | returns correct balance |
| `evm_can_transfer_foreign_asset` | Alith has foreign asset              | `evm_call(precompile.transfer(Bob, amount))` | Bob's balance increased |
| `xcm_transact_triggers_evm_call` | valid XCM Transact with EVM call     | XCM execution                                | EVM contract called     |
| `evm_precompile_xcm_interaction` | XCM precompile available             | `evm_call(xcm_precompile.send(...))`         | XCM message sent        |

#### Fee Collection Tests

| Test                         | Precondition                   | Action       | Postcondition              |
| ---------------------------- | ------------------------------ | ------------ | -------------------------- |
| `fees_collected_by_treasury` | any XCM execution              | execute XCM  | Treasury balance increased |
| `fee_proportional_to_weight` | two XCM with different weights | execute both | higher weight → higher fee |

#### Error Scenario Tests

| Test                             | Precondition                   | Action                         | Postcondition          |
| -------------------------------- | ------------------------------ | ------------------------------ | ---------------------- |
| `trapped_assets_are_claimable`   | assets trapped from failed XCM | `claim_assets(origin, assets)` | assets returned        |
| `failed_execution_reverts_state` | any failing XCM                | execute XCM                    | state unchanged        |
| `oversized_message_rejected`     | XCM > MAX_MESSAGE_SIZE         | send XCM                       | `Err(MessageTooLarge)` |

---

## Unresolved Questions

| Question                                          | Proposed Default                            |
| ------------------------------------------------- | ------------------------------------------- |
| Exact fee calculation formula for foreign assets? | `native_fee * native_price / foreign_price` |
| Should barrier allow UnpaidExecution from relay?  | No, require paid execution from all origins |
| Maximum XCM message size?                         | 103 * 1024 bytes (MessageQueueHeapSize)     |
| How long are trapped assets claimable?            | Indefinitely                                |
| EVM gas limits for XCM calls?                     | 400,000 gas                                 |
| Behavior when price oracle returns zero?          | Reject fee payment with that asset          |
| Rate limits on XCM processing?                    | 25% of block weight                         |

---

## References

- [xcm-emulator](https://github.com/paritytech/polkadot-sdk/tree/master/cumulus/xcm/xcm-emulator)
- [xcm-simulator](https://github.com/paritytech/polkadot-sdk/tree/master/polkadot/xcm/xcm-simulator) - "If you just wish to test execution of various XCM instructions against the XCM VM then the xcm-simulator is the perfect tool"
