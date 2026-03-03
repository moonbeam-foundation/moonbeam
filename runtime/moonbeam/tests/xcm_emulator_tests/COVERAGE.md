# XCM Emulator Test Coverage

> Status as of 2026-03-03 — branch `manuel/refactor-xcm-tests`

## Overview

The new xcm-emulator test suite uses **real runtimes** (Westend relay + Moonbeam parachain)
via `xcm-emulator`, replacing the legacy `xcm_tests.rs` which used `xcm-simulator` with
mock chains. Both suites coexist temporarily to allow incremental PR splitting.

**28 emulator tests** total (8 pre-existing + 20 new).

---

## Test Inventory

### `emulator_transact_tests.rs` — 17 tests

| Test | Legacy equivalent | Notes |
|------|-------------------|-------|
| `transact_through_sovereign_to_relay` | `transact_through_sovereign` | Remark via sovereign account |
| `transact_through_sovereign_fee_payer_none` | `transact_through_sovereign` | `fee_payer = None` — no local withdraw |
| `transact_through_sovereign_custom_fee_weight` | `transact_through_sovereign_with_custom_fee_weight` | Explicit fee & weight |
| `transact_through_sovereign_custom_fee_weight_refund` | `transact_through_sovereign_with_custom_fee_weight_refund` | Refund surplus to sovereign |
| `transact_through_derivative_to_relay` | `transact_through_derivative_multilocation` | `as_derivative` sub-account on relay |
| `transact_through_derivative_custom_fee_weight` | `transact_through_derivative_with_custom_fee_weight` | Explicit fee & weight |
| `transact_through_derivative_custom_fee_weight_refund` | `transact_through_derivative_with_custom_fee_weight_refund` | Refund surplus to sovereign |
| `transact_through_signed_to_relay` | `transact_through_signed_multilocation` | `DescendOrigin` + hashed account |
| `transact_through_signed_custom_fee_weight` | `transact_through_signed_multilocation_custom_fee_and_weight` | Explicit fee & weight |
| `transact_through_signed_custom_fee_weight_refund` | `transact_through_signed_multilocation_custom_fee_and_weight_refund` | Refund surplus to derived account |
| `transact_through_signed_para_to_para` | `transact_through_signed_multilocation_para_to_para` | HRMP, DOT fees on sibling |
| `transact_through_signed_para_to_para_refund` | `transact_through_signed_multilocation_para_to_para` + refund | 8B ref_time for appendix |
| `transact_through_signed_para_to_para_ethereum` | `transact_through_signed_multilocation_para_to_para_ethereum` | EVM value transfer via `EthereumXcm::transact` |
| `transact_through_signed_para_to_para_ethereum_no_proxy_fails` | `…_ethereum_no_proxy_fails` | No proxy → no balance change |
| `transact_through_signed_para_to_para_ethereum_proxy_succeeds` | `…_ethereum_proxy_succeeds` | ALITH delegates to derived account |
| `hrmp_init_accept_close_via_xcm_transactor` | `hrmp_init_accept_close_via_xcm_transactor` | Full lifecycle: init → accept → close |
| `hrmp_close_via_xcm_transactor` | `hrmp_close_works` | Close a force-opened channel |

### `emulator_transfer_tests.rs` — 11 tests

| Test | Legacy equivalent | Notes |
|------|-------------------|-------|
| `transfer_dot_from_relay_to_moonbeam` | `receive_relay_asset_with_trader` | DMP, DOT via `LocalReserve` |
| `transfer_dot_from_moonbeam_to_relay` | (implicit in legacy) | UMP, DOT back to relay |
| `error_when_not_paying_enough_fees` | (implicit in legacy) | Insufficient DOT → no deposit |
| `fees_collected_by_treasury` | `receive_relay_asset_with_trader` (fee part) | Treasury receives XCM fees |
| `receive_asset_for_non_existent_account` | `receive_assets_with_sufficients_true_…` | Fresh H160 can receive DOT |
| `transfer_dot_from_moonbeam_to_sibling` | (implicit in legacy) | HRMP, DOT via relay reserve |
| `evm_account_receives_foreign_asset` | `evm_account_receiving_assets_should_handle_sufficients_ref_count` | Adapted for EVM foreign assets |
| `foreign_assets_survive_native_balance_drain` | `empty_account_should_not_be_reset` | Adapted: no `sufficients` in EVM foreign assets |
| `transfer_glmr_from_moonbeam_to_sibling` | `send_para_a_asset_to_para_b` | GLMR as reserve-backed foreign asset |
| `transfer_glmr_roundtrip_moonbeam_sibling` | `send_para_a_asset_to_para_b_and_back_to_para_a` | Moonbeam → Sibling → Moonbeam |
| `transfer_glmr_to_sibling_with_trader_fees` | `send_para_a_asset_to_para_b_with_trader` | Fee deduction + treasury collection |

---

## Emulator Network Topology

```
WestendRelay (real westend-runtime)
├── MoonbeamPara (para 2004, real moonbeam-runtime)
└── SiblingPara  (para 2005, real moonbeam-runtime)
```

- **HRMP channels**: opened on demand via `open_hrmp_channels()` helper
- **DOT**: registered as foreign asset (id=1) on both paras via `register_dot_asset()`
- **GLMR**: registered as foreign asset (id=2) on sibling via `register_glmr_on_sibling()`

### Test Accounts

| Name | Key | Description |
|------|-----|-------------|
| `ALITH` | `[1u8; 20]` | Primary Moonbeam user (H160) |
| `BALTATHAR` | `[2u8; 20]` | Secondary Moonbeam user |
| `CHARLETH` | `[3u8; 20]` | Third Moonbeam user |
| `RELAY_ALICE` | `[1u8; 32]` | Relay chain user (AccountId32) |

### Constants

| Name | Value | Notes |
|------|-------|-------|
| `ONE_DOT` | `10_000_000_000` | 10 decimals |
| `MOONBEAM_PARA_ID` | `2004` | |
| `SIBLING_PARA_ID` | `2005` | |
| Westend Staking index | `6` | |
| Westend Utility index | `16` | |
| Westend HRMP index | `51` | |
| Westend Balances index | `4` | |
| Moonbeam Balances index | `10` | `PalletInstance(10)` for GLMR location |

---

## Technical Discoveries

### 1. Account derivation differs between `Account32Hash` and Westend's `LocationConverter`

The relay's `LocationConverter` uses `HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>`
which hashes differently from `xcm_builder::Account32Hash`. For signed transact tests, always use
the relay's actual converter:

```rust
westend_runtime::xcm_config::LocationConverter::convert_location(&location)
```

For the sibling (Moonbeam runtime), use:

```rust
moonbeam_runtime::xcm_config::LocationToAccountId::convert_location(&location)
```

### 2. `MoonbeamCall` CallDispatcher intercepts EthereumXcm calls

Defined in `runtime/common/src/impl_moonbeam_xcm_call.rs`. When the XCM executor dispatches
`EthereumXcm::transact` with a `Signed(AccountId)` origin, `MoonbeamCall` re-dispatches it
with `pallet_ethereum_xcm::Origin::XcmEthereumTransaction(account_id.into())`. Without this,
`EthereumXcm::transact` would fail with `BadOrigin` since it requires `XcmEthereumTransaction`.

### 3. Para-to-para refund needs higher weight limit

With `refund=true`, the XCM message includes a `SetAppendix([RefundSurplus, DepositAsset])`
instruction. On the sibling, the full execution (including appendix) uses ~5.6B ref_time.
Use `overall_weight: Some(Limited(8_000_000_000u64.into()))` for refund tests vs 4B for basic.

### 4. Derivative account derivation

`pallet_utility::derivative_account_id` uses the prefix `b"modlpy/utilisuba"` (not
`b"modlpy/teleport"` as sometimes referenced). The derivative is computed from the
parachain's sovereign account on the relay:

```rust
pallet_utility::Pallet::<westend_runtime::Runtime>::derivative_account_id(sovereign, index)
```

### 5. `remark` vs `remark_with_event`

`frame_system::Call::remark` is a no-op that emits no event. Always use `remark_with_event`
to get a `Remarked` event for assertion.

### 6. EVM foreign assets don't use frame `sufficients`

Moonbeam's `pallet_moonbeam_foreign_assets` stores balances in EVM contract storage, not
through frame's `AccountInfo::sufficients`. Tests adapted to check EVM balances directly
instead of `sufficients` ref counts.

### 7. XCM fee re-anchoring for sibling-to-sibling

When sending from Moonbeam to Sibling, `Location::parent()` (DOT) remains
`Location::parent()` after re-anchoring because both parachains share the same relay parent.
The pallet's `transact_message()` handles re-anchoring via `asset.reanchored(dest, universal_location)`.

---

## Deferred Tests (Future PRs)

### Asset Hub / xtokens (Group 6) — 6-7 tests

Requires adding an Asset Hub chain to the emulator network. Tests include:
- `test_statemint_like`
- `send_statemint_asset_from_para_a_to_statemint_with_relay_fee`
- 5× `send_dot_…_via_xtokens_transfer*`

### Versioning (Group 7) — 2 tests

Requires runtime upgrade simulation in the emulator:
- `test_automatic_versioning_on_runtime_upgrade_with_relay`
- `test_automatic_versioning_on_runtime_upgrade_with_para_b`

### 3-chain multi-hop (Group 5 partial) — 1 test

Requires a 3rd parachain in the emulator network:
- `send_para_a_asset_from_para_b_to_para_c`

---

## Known Issues

- **Pre-existing failure**: `xcm_config_tests::barriers_test::barrier_passes_unpaid_with_weight_credit`
  fails on moonbeam-runtime — unrelated to this work.
- **Cargo `[patch]` limitation**: Cannot point `[patch]` to remote `polkadot-sdk` branch
  (same-source error). Local path patches in `Cargo.toml` remain for the `force-xcm-processor`
  feature on `westend-runtime`.

---

## File Structure

```
runtime/moonbeam/tests/
├── xcm_emulator_tests/
│   ├── mod.rs                      # Test binary entry point
│   ├── emulator_network.rs         # Network topology, helpers, genesis
│   ├── emulator_relay.rs           # Relay genesis config
│   ├── emulator_transact_tests.rs  # 17 transact + HRMP tests
│   └── emulator_transfer_tests.rs  # 11 transfer tests
│   └── COVERAGE.md                 # ← this file
├── xcm_tests.rs                    # Legacy suite (45 tests, temporary)
└── xcm_mock/                       # Legacy mock chains (temporary)
    ├── mod.rs
    ├── parachain.rs
    └── relay_chain.rs
```
