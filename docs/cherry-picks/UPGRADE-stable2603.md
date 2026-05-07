# WIP — Upgrade plan: polkadot-sdk → stable2603-1

> **Temporary working doc.** Tracks the upgrade from `moonbeam-polkadot-stable2512`
> to `moonbeam-polkadot-stable2603` against upstream
> [`polkadot-stable2603-1`](https://github.com/paritytech/polkadot-sdk/releases/tag/polkadot-stable2603-1).
> Delete (or archive into `docs/adr/`) once the upgrade has shipped.

## Context

- **Current pin (Cargo.toml):** `moonbeam-polkadot-stable2512` for polkadot-sdk, frontier, evm, ethereum, moonkit.
- **Target upstream:** `polkadot-stable2603-1` (paritytech/polkadot-sdk, released 2026-05-04).
- **Sibling upstream bases:**
  - frontier → tag `frontier-stable2603` (polkadot-evm/frontier)
  - evm → 0.43.x line (rust-ethereum/evm) — diverged from upstream v1.0
  - ethereum → master (rust-ethereum/ethereum)
  - moonkit → `main` (Moonsong-Labs/moonkit) once the stable2603 base-bump PR lands
- **No fork has a `moonbeam-polkadot-stable2603` branch yet** (verified via `git branch -r` in each `~/Workspace/<fork>` checkout).

## Branch

- Working branch: `chore-update-polkadot-sdk-stable2603` (off `master`).
- Cherry-pick tracking doc: [`polkadot-sdk-stable2603.md`](./polkadot-sdk-stable2603.md).

---

## Phase 0 — Preparation

- [ ] Fetch upstream tag `polkadot-stable2603-1` in `~/Workspace/polkadot-sdk` and confirm the corresponding `stable2603` branch sha.
- [ ] Fetch `frontier-stable2603` from `polkadot-evm/frontier`.
- [ ] Confirm rust-ethereum/evm and rust-ethereum/ethereum upstream commits to base on (no version bump expected for evm — staying on 0.43.x).
- [ ] Confirm moonkit base-bump PR status (Moonsong-Labs). **Blocker** for moonkit Phase 1.

## Phase 1 — Fork branches + cherry-pick re-application

Order: **polkadot-sdk → (evm, ethereum) → (moonkit, frontier)**.

### 1.1 polkadot-sdk
- [ ] Create `moonbeam-polkadot-stable2603` from `upstream/stable2603` (or the `polkadot-stable2603-1` tag).
- [ ] Re-cherry-pick `Add command PrecompileWasmCmd` (PR #1641 still open).
- [ ] Re-cherry-pick `Comment log "Unexpected underflow in reducing consumer"` (Permanent).
- [ ] Re-cherry-pick `Bound WildMultiAsset max assets limit to 20` (Permanent).
- [ ] Re-cherry-pick `Account for pallet-parameters weight in benchmarks` (PR #6477 open).
- [ ] **Verify**: is [paritytech/polkadot-sdk#7835](https://github.com/paritytech/polkadot-sdk/pull/7835) in stable2603? If yes → **drop** `Add storage benchmark --keys-limit option`. If no → re-cherry-pick.
- [ ] **Verify**: is pallet-revive still a dependency of pallet-xcm in stable2603? If no → **drop** `Remove pallet-revive from pallet-xcm`. If yes → re-cherry-pick.
- [ ] **Verify**: confirm [paritytech/polkadot-sdk#10444](https://github.com/paritytech/polkadot-sdk/pull/10444) is in stable2603 — drop `Fix charge_transaction_payment benchmark` (pre-marked Dropped in tracker).
- [ ] Push `moonbeam-polkadot-stable2603` to `moonbeam-foundation/polkadot-sdk`.
- [ ] Update commit hashes in [`polkadot-sdk-stable2603.md`](./polkadot-sdk-stable2603.md).

### 1.2 evm
- [ ] Create `moonbeam-polkadot-stable2603` from current 0.43.x base.
- [ ] Re-cherry-pick `Add support for EIP-7939 (CLZ opcode)` (still needed; upstream merged on v1.0 only).
- [ ] Push branch and update tracker.

### 1.3 ethereum
- [ ] Create `moonbeam-polkadot-stable2603` from `upstream/master` head.
- [ ] Re-cherry-pick `Add encoded length methods to transactions` (rust-ethereum/ethereum#77 still open).
- [ ] Push branch and update tracker.

### 1.4 moonkit (blocked on Moonsong-Labs)
- [ ] Wait for / coordinate the stable2603 base-bump PR into `main`.
- [ ] Create `moonbeam-polkadot-stable2603` from the merge commit.
- [ ] Confirm `using_fake_author` (#92) is in `main` — should be no-op.
- [ ] Push branch and update tracker.

### 1.5 frontier
- [ ] Create `moonbeam-polkadot-stable2603` from upstream tag `frontier-stable2603`.
- [ ] Re-cherry-pick (still required):
  - [ ] CI branch name (rename `stable2512` → `stable2603` during cherry-pick)
  - [ ] Do client side check of withdraw-ability (#1546)
  - [ ] Catch ethereum execution info (#1547)
  - [ ] Fix dispatch error legacy decoding (Permanent)
  - [ ] Account eth tx size only once (squashed with #224)
  - [ ] Expose lru_cache (#1568)
  - [ ] Address POV Underestimations
  - [ ] Upgrade frame-metadata
  - [ ] Validate transaction size (#1807)
  - [ ] Improve "latest" block resolution on pruned nodes (#1856)
- [ ] **Verify** each row pre-marked Dropped in the tracker is actually present in `frontier-stable2603` (use `git log --grep` on PR number). If any are missing, re-cherry-pick and flip Dropped → Included in the tracker:
  - [ ] EIP-7939 (#1795)
  - [ ] EIP-7883 (#1801)
  - [ ] EIP-7823 (#1804)
  - [ ] EIP-7825 (#1799)
  - [ ] Multi pre-runtime Frontier logs invalid (#1809)
  - [ ] eth_getTransactionReceipt race (#1802)
  - [ ] Frontier parity-db migration (#1817)
  - [ ] Drop lagging pubsub subscribers (#1814)
  - [ ] Forbid txs without chain ID (#1815)
  - [ ] eth_getBlockByNumber("latest") null fix (#1816)
  - [ ] Inconsistent Eth block visibility (#1818)
  - [ ] Single-writer mapping-sync (#1820)
  - [ ] Wait for chain head & eth latest in createAndFinalizeBlock (#1855)
  - [ ] Avoid eth_blockNumber 0x00 lag (#1832)
- [ ] Push branch and update tracker.

## Phase 2 — Tracking doc finalization

- [x] Draft `polkadot-sdk-stable2603.md` (TBD placeholders for commits).
- [ ] Run merge-base verification per repo (one `cherry-pick-specialist` agent per fork, dispatched in parallel).
- [ ] Update tracker: real commit hashes for Included rows, finalize Cherry-pick column for every Verify row.
- [ ] Reconcile any agent-reported discrepancies.

## Phase 3 — Moonbeam repository upgrade

- [ ] `Cargo.toml`: replace `moonbeam-polkadot-stable2512` → `moonbeam-polkadot-stable2603` (~120 occurrences).
- [ ] `cargo update -p sp-core` (and friends) to refresh the lockfile.
- [ ] Compile-fix loop (`cargo check --workspace --all-features`):
  - [ ] FRAME pallet trait drift (Config additions, `WeightInfo` signature changes).
  - [ ] `sc-*` client crate restructure / renames.
  - [ ] XCM v5+ changes if upstream advanced.
  - [ ] Cumulus parachain service config (`ParachainTracingExecuteBlock` plumbing — see stable2512 doc note about [#9871](https://github.com/paritytech/polkadot-sdk/pull/9871)).
- [ ] During iteration, patch `[patch.crates-io]` to local checkouts per the `patching-dependencies` skill.
- [ ] Update `tests/` TS fixtures referencing changed types/RPCs.

## Phase 4 — Runtime, weights, version bump

- [ ] Re-run benchmarks for any pallet whose weight signature changed (`benchmarking-pallets` skill).
- [ ] Run `analyzing-weights` over the diff; flag regressions in ref_time / proof_size / DB reads.
- [ ] Bump `spec_version` in each runtime (last bump: commit b4f02de7b5).
- [ ] Author migrations for any pallet whose `StorageVersion` advanced upstream (`writing-migrations` skill).
- [ ] Run `try-runtime` against current production state for moonbase, moonriver, moonbeam.

## Phase 5 — Bridge maintenance

- [ ] Per `bridge-maintenance` skill, rebuild `substrate-relay` against stable2603.
- [ ] Regenerate zombienet relay chain specs.
- [ ] Run bridge integration tests; fix relayer regressions.

## Phase 6 — Verification

- [ ] `cargo check --workspace --all-features`
- [ ] Rust unit tests: `cargo test --workspace`
- [ ] TypeScript integration tests (`testing-moonbeam` skill).
- [ ] Smoke tests against moonbase/moonriver chain specs.
- [ ] XCM cross-chain test scenarios.
- [ ] Spot-check for cherry-pick regressions: confirm each Included row's behaviour still works (e.g. PrecompileWasmCmd, WildMultiAsset bound, POV underestimation fix).

## Risks & open questions

- **moonkit dependency** — Phase 1.4 blocks on Moonsong-Labs base bump. Consider opening / nudging the upstream PR early.
- **EVM divergence** — anything new in upstream v1.0 (new EIPs, gas changes) must be backported manually onto the 0.43.x fork.
- **Deferred EIPs** — confirm no EIP from stable2603 needs implementation work in our fork (`implementing-eips` skill).
- **Verification cost** — Phase 1.5 frontier verification has 14 rows; parallelize via sub-agents.

## Drop / archive checklist

When this upgrade ships:
- [ ] Move (or delete) this file. The cherry-pick tracker `polkadot-sdk-stable2603.md` stays.
- [ ] Confirm `polkadot-sdk-stable2512.md` and `polkadot-sdk-stable2506.md` remain for historical reference.
