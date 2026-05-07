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

- [x] Fetch upstream tag `polkadot-stable2603-1` in `~/Workspace/polkadot-sdk` and confirm the corresponding `stable2603` branch sha.
- [x] Fetch `frontier-stable2603` from `polkadot-evm/frontier`.
- [x] Confirm rust-ethereum/evm and rust-ethereum/ethereum upstream commits to base on (no version bump expected for evm — staying on 0.43.x).
- [x] Confirm moonkit base-bump PR status (Moonsong-Labs). **Blocker** for moonkit Phase 1.

### Resolved bases

| Fork | Base | SHA | Notes |
| --- | --- | --- | --- |
| polkadot-sdk | tag `polkadot-stable2603-1` | `f8cfbb96055978d5031f60153c9dfbff33814183` | Released 2026-05-01. `upstream/stable2603` branch HEAD = `d3dfb281ee0f62c3b8bcda36c35c46f82f2d8878`. Use the tag. |
| frontier | **no stable2603 cut upstream yet** | `upstream/stable2512` = `9d49e36ed5bac38241594f8ba055fdb94991483a` | polkadot-evm/frontier has not branched stable2603. `upstream/master` is 43 commits ahead and already contains several PRs we listed as Included (e.g. #1856 latest-block-resolution on pruned nodes, #1855, #1832). **Decision needed** — see blockers. |
| evm | current `moonbeam-polkadot-stable2512` | `bb9cdde4` | Upstream `rust-ethereum/evm` has moved to v1.0; we stay on the 0.43.x fork. No new upstream commits to pull. The new branch is effectively a rename of stable2512. |
| ethereum | upstream master | `d7bdf28` | Master HEAD is `Refactor transaction signature validation (#75)`, which is already absorbed in our stable2512 line. Fork stable2512 head `58a5a8a` adds the unmerged #77 cherry-pick. |
| moonkit | **no stable2603 base-bump PR yet** | latest merged was #89 stable2512 (2026-01-14) | Most recent merged PR is #94 (2026-05-04, dynamic relay-parent offset). **Blocker** — see below. |

### New blockers found in Phase 0

- **B1 (frontier upstream)** — `polkadot-evm/frontier` has not yet branched `stable2603`. **Resolution:** we own this. Branch from a chosen `upstream/master` SHA in `~/Workspace/frontier`, bump polkadot-sdk dependencies stable2512 → stable2603 ourselves, and use that as the upstream base for our `moonbeam-polkadot-stable2603` cherry-picks. When polkadot-evm cuts an official `stable2603`, rebase our work onto it. Tracked as Phase 1.5a below.
- **B2 (moonkit upstream)** — No PR in `Moonsong-Labs/moonkit` titled or scoped to "stable2603". **Resolution:** author the upstream base-bump PR ourselves on `Moonsong-Labs/moonkit` and coordinate with Moonsong-Labs for review. Tracked as Phase 1.4a below.
- **F1 (frontier #1856 actually merged)** — `Improve "latest" block resolution on pruned nodes` (#1856) appears in `upstream/master` (commit `46cf7a43e`) even though the stable2512 tracker marked it "Upstream PR not merged". When verifying frontier cherry-picks, confirm this and update the tracker (move row from Included → Dropped if our chosen frontier base contains it).

## Phase 1 — Fork branches + cherry-pick re-application

Order: **polkadot-sdk → (evm, ethereum) → (moonkit, frontier)**.

### 1.1 polkadot-sdk ✅
- [x] Create `moonbeam-polkadot-stable2603` from the `polkadot-stable2603-1` tag.
- [x] Re-cherry-pick `Add command PrecompileWasmCmd` → `909be1fe0ca` (adapted to `BackendRuntimeCode::new(state, TryPendingCode::No)`).
- [x] Re-cherry-pick `Comment log "Unexpected underflow in reducing consumer"` → `834c1d794d3`.
- [x] Re-cherry-pick `Bound WildMultiAsset max assets limit to 20` → `edc21b0c2d5`.
- [x] Re-cherry-pick `Account for pallet-parameters weight in benchmarks` → `443bfd80e24`.
- [x] **Drop** `Add storage benchmark --keys-limit option` — flags ship natively in stable2603's `benchmarking-cli/src/storage/cmd.rs`.
- [x] **Drop** `Remove pallet-revive from pallet-xcm` — `pallet-xcm/Cargo.toml` no longer depends on `pallet-revive` in stable2603.
- [x] **Drop** `Fix charge_transaction_payment benchmark` — merged into stable2603 as `4b934d0a252`.
- [x] Push `moonbeam-polkadot-stable2603` to `moonbeam-foundation/polkadot-sdk`.
- [x] Update commit hashes in [`polkadot-sdk-stable2603.md`](./polkadot-sdk-stable2603.md).

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

### 1.4a moonkit upstream base-bump PR (we author)
- [ ] In `~/Workspace/moonkit`, branch off `upstream/main` (e.g. `mb/polkadot-sdk-stable2603`).
- [ ] Bump polkadot-sdk dependencies in moonkit's `Cargo.toml` from `stable2512` → `stable2603`.
- [ ] Compile-fix loop until `cargo check --workspace` is clean.
- [ ] Open PR on `Moonsong-Labs/moonkit`; coordinate review with Moonsong-Labs.
- [ ] Once merged, proceed to Phase 1.4.

### 1.5a frontier upstream base-bump (we own — B1)
- [ ] In `~/Workspace/frontier`, branch off a chosen `upstream/master` SHA (e.g. `mb/polkadot-sdk-stable2603`). Pick the most recent stable master commit that doesn't introduce unrelated risk.
- [ ] Bump polkadot-sdk dependencies in frontier's `Cargo.toml` from `stable2512` → `stable2603`.
- [ ] Compile-fix loop until `cargo check --workspace` is clean.
- [ ] Push to `moonbeam-foundation/frontier` (this is *our* effective upstream base for Phase 1.5).
- [ ] When `polkadot-evm/frontier` officially cuts `stable2603`, rebase the work onto the official branch.

### 1.5 frontier (moonbeam fork)
- [ ] Create `moonbeam-polkadot-stable2603` from the Phase 1.5a base in `moonbeam-foundation/frontier`.
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
