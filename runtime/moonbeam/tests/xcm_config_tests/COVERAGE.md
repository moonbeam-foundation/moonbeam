# XCM Config Test Coverage

> Status as of 2026-03-06 — branch `manuel/refactor-xcm-tests`

## Overview

The `xcm_config_tests` suite (Level 1) verifies individual XCM configuration
components in isolation by calling `XcmExecutor::prepare_and_execute()` directly
with the **real** `moonbeam_runtime::xcm_config::XcmExecutorConfig`.

These are fast, deterministic, unit-style tests — no multi-chain harness is
needed.

**48 tests** total across 6 modules.

---

## Test Inventory

### `barriers_test.rs` — 13 tests

| Test                                                | Property                                        | Expected                |
| --------------------------------------------------- | ----------------------------------------------- | ----------------------- |
| `barrier_allows_paid_execution_from_relay`          | WithdrawAsset + BuyExecution from Relay         | Not barrier error       |
| `barrier_allows_paid_execution_from_sibling`        | WithdrawAsset + BuyExecution from Sibling(2000) | Not barrier error       |
| `barrier_allows_paid_execution_from_account_key20`  | WithdrawAsset + BuyExecution from AccountKey20  | Not barrier error       |
| `barrier_passes_unpaid_with_weight_credit`          | DepositAsset with pre-credited weight           | TakeWeightCredit passes |
| `barrier_allows_subscription_messages`              | SubscribeVersion from Relay                     | Not barrier error       |
| `barrier_allows_unsubscribe_messages`               | UnsubscribeVersion from Relay                   | Not barrier error       |
| `barrier_allows_paid_execution_with_descend_origin` | DescendOrigin + paid execution from Relay       | Not barrier error       |
| `barrier_allows_set_topic`                          | Paid execution + SetTopic                       | Not barrier error       |
| `barrier_with_computed_origin_has_depth_limit`      | Multiple DescendOrigin + paid execution         | Not barrier error       |
| `barrier_rejects_unpaid_execution_from_sibling`     | UnpaidExecution from Sibling                    | Barrier error           |
| `barrier_rejects_unpaid_transact_from_sibling`      | UnpaidExecution + Transact from Sibling         | Barrier error           |
| `barrier_allows_known_query_response`               | QueryResponse with registered query_id          | Not barrier error       |
| `barrier_rejects_unknown_query_response`            | QueryResponse with unknown query_id             | Barrier error           |

### `location_test.rs` — 5 tests

| Test                                                       | Property                                           | Expected                         |
| ---------------------------------------------------------- | -------------------------------------------------- | -------------------------------- |
| `location_converts_relay_to_account`                       | `convert(Location::parent())`                      | Relay sovereign (ParentIsPreset) |
| `location_converts_sibling_parachain_to_sovereign_account` | `convert(Sibling(para_id))`                        | Unique per para_id               |
| `location_converts_account_key20_directly`                 | `convert(AccountKey20(key))`                       | `H160(key)`                      |
| `location_converts_only_supported_patterns`                | Complex multi-junction                             | Deterministic or None            |
| `location_converts_bridged_parachain`                      | `convert(GlobalConsensus(Kusama)/Parachain(1000))` | Hashed account                   |

### `reserves_test.rs` — 9 tests

| Test                                                     | Property                                                         | Expected |
| -------------------------------------------------------- | ---------------------------------------------------------------- | -------- |
| `reserves_accepts_dot_from_asset_hub`                    | DOT from Asset Hub via `Case<RelayChainNativeAssetFromAssetHub>` | `true`   |
| `reserves_accepts_bridged_assets_from_asset_hub`         | Bridged asset from Asset Hub                                     | `true`   |
| `reserves_rejects_bridged_assets_from_wrong_origin`      | Bridged asset from non-Asset Hub origin                          | `false`  |
| `reserves_rejects_non_bridged_assets_via_bridged_filter` | Non-bridged asset via bridged filter                             | `false`  |
| `reserves_accepts_self_reserve`                          | GLMR as self-reserve                                             | `true`   |
| `reserves_accepts_sibling_native_asset`                  | Sibling native asset from matching origin                        | `true`   |
| `reserves_rejects_asset_with_mismatched_origin`          | Asset reserve ≠ origin                                           | `false`  |
| `reserves_accepts_dot_from_relay`                        | DOT from relay via MultiNativeAsset                              | `true`   |
| `teleport_always_rejected`                               | `IsTeleporter = ()` rejects all                                  | `false`  |

### `traders_test.rs` — 8 tests

| Test                                                    | Property                                       | Expected                       |
| ------------------------------------------------------- | ---------------------------------------------- | ------------------------------ |
| `trader_accepts_native_token`                           | `buy_weight(GLMR, weight)`                     | `Ok`                           |
| `trader_computes_native_fee_correctly`                  | `query_weight_to_asset_fee(GLMR, w)`           | `> 0`                          |
| `trader_rejects_unsupported_asset`                      | `buy_weight(unknown, weight)`                  | `Err(AssetNotFound)`           |
| `trader_accepts_registered_foreign_asset`               | `buy_weight(DOT, weight)` after registration   | `Ok`                           |
| `trader_computes_foreign_asset_fee_with_relative_price` | `query_weight_to_asset_fee(DOT, w)`            | `> 0`                          |
| `trader_cannot_buy_weight_twice`                        | Two `buy_weight` calls                         | Second fails `NotWithdrawable` |
| `trader_refunds_unused_weight`                          | `refund_weight(unused)` after buy              | Non-zero refund                |
| `trader_handles_insufficient_payment`                   | `buy_weight` with tiny amount for large weight | `Err(TooExpensive)`            |

### `transactors_test.rs` — 8 tests

| Test                                                   | Property                               | Expected          |
| ------------------------------------------------------ | -------------------------------------- | ----------------- |
| `local_transactor_deposits_native_token`               | Deposit GLMR to Bob                    | Balance increased |
| `local_transactor_withdraws_native_token`              | Withdraw GLMR from Alice               | Balance decreased |
| `local_transactor_fails_withdraw_insufficient_balance` | Withdraw > balance                     | `Err`             |
| `foreign_asset_transactor_deposits_registered_asset`   | Deposit DOT (registered)               | `Ok`              |
| `transactor_withdraws_registered_foreign_asset`        | Withdraw DOT (registered, funded)      | `Ok`              |
| `transactor_fails_for_unregistered_asset`              | Deposit unregistered asset             | `Err`             |
| `transactor_handles_erc20_bridge_asset`                | Deposit via ERC20 bridge (no contract) | No panic          |
| `transactor_handles_relay_sovereign_account`           | Withdraw GLMR from relay sovereign     | `Ok`              |

### `weigher_test.rs` — 5 tests

| Test                                              | Property                           | Expected       |
| ------------------------------------------------- | ---------------------------------- | -------------- |
| `weigher_calculates_weight_for_simple_message`    | `weight(ClearOrigin)`              | `> 0`          |
| `weigher_calculates_weight_for_transfer_message`  | `weight(Withdraw + Buy + Deposit)` | `> 0`          |
| `weigher_weight_increases_with_more_instructions` | 1 vs 5 ClearOrigin                 | More → heavier |
| `weigher_respects_max_instructions`               | 150 instructions (max = 100)       | `Err`          |
| `weigher_handles_transact_instruction`            | `weight(Transact)`                 | `Ok`           |

---

## Barrier Configuration Under Test

```rust
pub type XcmBarrier = TrailingSetTopicAsId<(
    TakeWeightCredit,
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

---

## File Structure

```
runtime/moonbeam/tests/xcm_config_tests/
├── main.rs              # Test binary entry point
├── xcm_common.rs        # Shared helpers (execute_xcm, is_barrier_error, …)
├── barriers_test.rs     # 13 barrier tests
├── location_test.rs     # 5 location conversion tests
├── reserves_test.rs     # 9 reserve / teleport tests
├── traders_test.rs      # 8 trader tests
├── transactors_test.rs  # 8 asset transactor tests
├── weigher_test.rs      # 5 weigher tests
└── COVERAGE.md          # ← this file
```
