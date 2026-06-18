# WIP — Upgrade plan: polkadot-sdk → stable2603-1

> **Temporary working doc.** Tracks the upgrade from `moonbeam-polkadot-stable2512`
> to `moonbeam-polkadot-stable2603` against upstream
> [`polkadot-stable2603-1`](https://github.com/paritytech/polkadot-sdk/releases/tag/polkadot-stable2603-1).
> Delete (or archive into `docs/adr/`) once the upgrade has shipped.

## Context

- **Current pin (Cargo.toml):** `moonbeam-polkadot-stable2512` for polkadot-sdk, frontier, evm, ethereum, moonkit.
- **Target upstream:** `polkadot-stable2603-1` (paritytech/polkadot-sdk, released 2026-05-01).
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

## Phase 0.5 — Audit fork branches for undocumented cherry-picks

> **Why this exists.** `polkadot-sdk-stable2603.md` was assembled from the prior
> cycle's tracker plus deltas we knew about. It may not capture commits landed
> directly on a fork's `moonbeam-polkadot-stable2512` branch without a tracker
> entry. Run this audit before Phase 1 so any missing cherry-picks are added to
> the tracker and re-applied to the new branch.

For each fork, list every commit on `moonbeam-polkadot-stable2512` that is not
on its upstream base, then reconcile with the corresponding table in
`polkadot-sdk-stable2603.md`. Parallelize via one sub-agent per fork.

> Across all five checkouts, `origin` already points to the moonbeam-controlled
> remote (moonbeam-foundation for sdk/frontier/evm/ethereum, Moonsong-Labs for
> moonkit), and `upstream` points to the canonical upstream. Use `origin/...`
> refs in commands.

- [x] **polkadot-sdk** — in `~/Workspace/polkadot-sdk`: `git log upstream/stable2512..origin/moonbeam-polkadot-stable2512 --no-merges --oneline`. Cross-check every commit against the `polkadot-sdk` table.
- [x] **frontier** — in `~/Workspace/frontier`: `git log upstream/stable2512..origin/moonbeam-polkadot-stable2512 --no-merges --oneline`. Cross-check against the `frontier` table.
- [x] **evm** — in `~/Workspace/evm`: diff against the upstream 0.43.x base the fork branched from (`git log $(git merge-base upstream/master origin/moonbeam-polkadot-stable2512)..origin/moonbeam-polkadot-stable2512 --no-merges --oneline`). Cross-check against the `evm` table.
- [x] **ethereum** — in `~/Workspace/ethereum`: `git log $(git merge-base upstream/master origin/moonbeam-polkadot-stable2512)..origin/moonbeam-polkadot-stable2512 --no-merges --oneline`. Cross-check against the `ethereum` table.
- [x] **moonkit** — in `~/Workspace/moonkit`: `git log $(git merge-base upstream/main origin/moonbeam-polkadot-stable2512)..origin/moonbeam-polkadot-stable2512 --no-merges --oneline`. Cross-check against the `moonkit` table. (Note: moonkit's `origin` is `Moonsong-Labs/moonkit` — the moonbeam fork branches live there.)

> **Methodology note (learned during this audit):** raw SHA-set-difference is misleading because the same upstream PR can land on both branches with different SHAs (we cherry-picked many backports from upstream master; upstream/stable2512 later got equivalent commits via its own backport workflow). The correct method is **PR-number set-difference**: extract `(#NNNNN)` refs from each side and take the difference. Commits without a PR ref are moonbeam-authored or admin and need manual classification.

For each commit not represented in the tracker:
- [x] Add a row to `polkadot-sdk-stable2603.md` matching the existing schema (Applied / Title / Commit / Cherry pick / Status / Upstream PR / Note).
- [x] Decide Included vs Dropped using the same rule as Phase 0: search upstream stable2603 (`git log --grep` on PR number, title, or a stable code fragment) for an equivalent merge before deciding. — *Resolved in Phase 1.5 (frontier) + Phase 2 verification (2026-06-18): all frontier `Verify` rows confirmed present in `upstream/stable2603`.*
- [x] If Included, add it to the appropriate Phase 1.x re-cherry-pick list below. — *Resolved: no TBD row flipped to Included; the only untracked moonbeam-only commit found was polkadot-sdk `ddba2453` (added as a tracker row, not a new re-cherry-pick).*

### Phase 0.5 results (2026-05-08)

Diff sizes against `upstream/stable<release>` for each fork (after pruning), then PR-number set-difference vs upstream:

| Fork | Commits ours-only | PRs ours-only (true moonbeam) | Tracker corrections | New tracker rows |
| --- | --- | --- | --- | --- |
| polkadot-sdk | 75 | 2 (#7, #8 — both moonbeam-foundation-internal, already tracked) | — | 3 (weight reclaim logs, xcm-emulator BlockProducer, bridges GRANDPA experiment+revert) |
| frontier | 35 | 21 — many already tracked, plus PR #1881, #247 missing; PR #1794 / #1787 in tracker were typos for #1824 / #1862 | Row 79 (`#1820`) `Applied: No → Yes`; Row 62 PR `#1794 → #1824`; Row 65 PR `#1787 → #1862` | 6 (logs journal #1881, CI triggers #247, canonical hash mapping repair, Saturate U256, configurable tx gas-limit cap, MBF ethereum dep + pin) |
| evm | 32 (mostly upstream 0.43.x content baked into the fork — the original upstream branch has been deleted from rust-ethereum/evm, so the merge-base diff overstates) | 1 moonbeam-authored (depend on MBF ethereum fork) | — | 1 |
| ethereum | 1 | 0 — already tracked (#77) | — | 0 |
| moonkit | 11 (most are Cargo.lock bumps) | 1 (#94 make relay offset dynamic) | — | 1 |

### Phase 0.5 follow-ups (TBD rows)

The new rows added with `Cherry pick: TBD` / `Status: TBD` need a Phase 2 decision (Included vs Dropped) once we can grep `frontier-stable2603` and the new moonkit `main` for equivalent merges:

- frontier `#1824` (block range checks), `#1862` (reorg-out logs), `#1881` (logs journal memory bound) — verify against `frontier-stable2603`.
- frontier `Fix canonical hash mapping repair`, `Saturate U256 to u64::MAX`, `Make tx gas limit cap configurable` — moonbeam-authored, default Permanent unless an equivalent landed upstream.
- moonkit `#94` — ✅ resolved (2026-06-08): inherited from `main` (`c0cf857`); flipped to Dropped. See Phase 1.4.

## Phase 1 — Fork branches + cherry-pick re-application

Order: **polkadot-sdk → (evm, ethereum) → (moonkit, frontier)**.

### 1.1 polkadot-sdk ✅
- [x] Create `moonbeam-polkadot-stable2603` from the `polkadot-stable2603-1` tag, then rebase onto `upstream/stable2603` head (`afb51b7a8c6`, 2026-05-08) to absorb 4 upstream backports (#11964, #11856, #11987, #12017).
- [x] Re-cherry-pick `Add command PrecompileWasmCmd` → `898357f0b74` (adapted to `BackendRuntimeCode::new(state, TryPendingCode::No)`).
- [x] Re-cherry-pick `Comment log "Unexpected underflow in reducing consumer"` → `c9cac9e684e`.
- [x] Re-cherry-pick `Bound WildMultiAsset max assets limit to 20` → `32c83c66988`.
- [x] Re-cherry-pick `Account for pallet-parameters weight in benchmarks` → `96bdb0d604f`.
- [x] Re-cherry-pick (Phase 0.5) `improve weight reclaim logs (call metadata, warn level)` → `b0b4fd52a9e`.
- [x] Re-cherry-pick (Phase 0.5) `xcm-emulator: make slot/digest producer overridable for non-Aura parachains` → `bb5b47f6c66`. Trivial additive conflict with stable2603's `native_total_supply_tracker` macro arm — resolved by keeping both.
- [x] **Drop** `Add storage benchmark --keys-limit option` — flags ship natively in stable2603's `benchmarking-cli/src/storage/cmd.rs`.
- [x] **Drop** `Remove pallet-revive from pallet-xcm` — `pallet-xcm/Cargo.toml` no longer depends on `pallet-revive` in stable2603.
- [x] **Drop** `Fix charge_transaction_payment benchmark` — merged into stable2603 as `4b934d0a252`.
- [x] **Drop** `bridges: retry outdated grandpa justifications with newer headers` — was reverted on stable2512 itself; do not re-apply.
- [x] Force-push `moonbeam-polkadot-stable2603` to `moonbeam-foundation/polkadot-sdk` (history rewritten by rebase).
- [x] Update commit hashes in [`polkadot-sdk-stable2603.md`](./polkadot-sdk-stable2603.md).

### 1.2 evm ✅
- [x] Create `moonbeam-polkadot-stable2603` from current 0.43.x base. `rust-ethereum/evm` master has moved to v1.0; we stay on the 0.43.x fork. The new branch points at the same SHA as `stable2512` (`bb9cdde4`) — there is no upstream advancement to absorb.
- [x] `Add support for EIP-7939 (CLZ opcode)` — **inherited, not a cherry-pick**. PR [#400](https://github.com/rust-ethereum/evm/pull/400) merged into `rust-ethereum/evm:v0.x` on 2026-01-12 as `a656db90`, the upstream-base HEAD the branch is cut from. Tracker row corrected to `Dropped`.
- [x] Push branch and update tracker.

### 1.3 ethereum ✅
- [x] Create `moonbeam-polkadot-stable2603` from `upstream/master` head. `upstream/master` (`d7bdf2888253a30f160d434688e378636e253870`) is exactly the merge-base with `moonbeam-polkadot-stable2512`, so the new branch points at the same SHA as `stable2512` (`58a5a8a1eeec58fa8a9ca8c6fab20a028ddcc1e5`) — no upstream advancement to absorb.
- [x] Re-cherry-pick `Add encoded length methods to transactions` (rust-ethereum/ethereum#77 still open). Already on the branch as `58a5a8a` — inherited as-is.
- [x] Push branch and update tracker.

### 1.4 moonkit — interim branch cut off PR #95 (reconciled — #95 merged to `main` 2026-06-18) ✅
- [x] **Interim (unblocks moonbeam Phase 3):** created `moonbeam-polkadot-stable2603` at the head of the open base-bump PR branch `mb/polkadot-sdk-stable2603` (`adfff2d`) and pushed to `Moonsong-Labs/moonkit`. PR [#95](https://github.com/Moonsong-Labs/moonkit/pull/95) is `main` + 11 base-bump commits (linear — `main` is a direct ancestor), so the release branch already equals `main` + base-bump, i.e. exactly what it would be if cut from the eventual merge commit. Original SHAs preserved; the branch is its own ref so it survives the PR branch being deleted on merge.
- [x] Confirmed `using_fake_author` (#92) **and** `make relay offset dynamic` (#94) are already in `main` (and therefore on this branch) — no extra moonbeam-only cherry-picks needed. Resolves the Phase 0.5 `#94` follow-up: inherited from `main`, flip tracker row to Dropped.
- [x] **[#95](https://github.com/Moonsong-Labs/moonkit/pull/95) merged to `main` 2026-06-18** (squash commit `4088d76`). Rebased `moonbeam-polkadot-stable2603` onto `main` via `git rebase --onto origin/main adfff2d` (new head `9d71129` = `main` + the dep-redirect commit). Content parity confirmed (tree hash identical to pre-rebase `ba06fb0`). Moonbeam `Cargo.lock` re-pinned `ba06fb0` → `9d71129`; base-bump row removed from the tracker (not a cherry-pick).

### 1.4a moonkit upstream base-bump PR (we author) — merged 2026-06-18 ✅
- [x] In `~/Workspace/moonkit`, branch off `main` as `mb/polkadot-sdk-stable2603`.
- [x] Bump polkadot-sdk dependencies in moonkit's `Cargo.toml` from `stable2512` → `stable2603`.
- [x] Compile-fix loop until `cargo check --workspace --all-features` is clean.
- [x] Open PR on `Moonsong-Labs/moonkit`; coordinate review with Moonsong-Labs → [moonkit#95](https://github.com/Moonsong-Labs/moonkit/pull/95). CI green; reviewed (CodeRabbit + manual). Review feedback applied: `b9c467e` (drop dead `Option` from the vendored `ProposerInterface`, debug-log unfound parent, `test_helpers` TODO) and `56e4a92` (linkspector npmjs ignore).
- [x] Merged 2026-06-18 (squash commit `4088d76`); Phase 1.4 reconciliation completed.

### 1.5a frontier upstream base-bump (we own — B1) ✅ — done by upstream
- [x] In `~/Workspace/frontier`, branch off a chosen `upstream/master` SHA (e.g. `mb/polkadot-sdk-stable2603`). Pick the most recent stable master commit that doesn't introduce unrelated risk.
- [x] Bump polkadot-sdk dependencies in frontier's `Cargo.toml` from `stable2512` → `stable2603`.
- [x] Compile-fix loop until `cargo check --workspace` is clean.
- [x] Push to `moonbeam-foundation/frontier` (this is *our* effective upstream base for Phase 1.5).
- [x] When `polkadot-evm/frontier` officially cuts `stable2603`, rebase the work onto the official branch. **Done by upstream:** our DIY base-bump branch was opened upstream as [polkadot-evm/frontier#1892](https://github.com/polkadot-evm/frontier/pull/1892) and squash-merged on 2026-05-08; upstream then cut `stable2603` from the resulting master commit (`baf505d8f`). Local `mb/polkadot-sdk-stable2603` was deleted (redundant), and Phase 1.5 below uses `upstream/stable2603` directly as the base.

### 1.5 frontier (moonbeam fork) ✅
- [x] Create `moonbeam-polkadot-stable2603` from `upstream/stable2603` (`baf505d8feeaaa0ba636a003faa49e4ad897b592`).
- [x] Cherry-pick the moonbeam-only commits (12 in total):
  - [x] Update CI triggers (#247) → `1921dd848`
  - [x] Do client side check of withdraw-ability (#1546) → `4df28dadc`
  - [x] Catch ethereum execution info (#1547) → `0210cb242`
  - [x] Fix dispatch error legacy decoding (#203, merge commit) → `3d7eb4665`
  - [x] Account eth tx size only once (squashed with #224) → `dff25ac36`
  - [x] Address POV Underestimations (#244) → `75a85bdc9`. Conflict with stable2603's WeightInfo error-handling refactor — resolved by keeping upstream's `match` block + adding `mut`.
  - [x] Expose lru_cache (#1568) → `6727696bb`
  - [x] Fix frontier parity-db migration (#252) → `d6a7e3c41`. Trivial conflict on `block_number_u64` syntax — took upstream's `*` deref form.
  - [x] Upgrade frame-metadata → `fc5e6f75c`
  - [x] Validate transaction size (#254) → `d6693687f`
  - [x] Saturate U256 to u64::MAX → `3397496a9`
  - [x] Fix canonical hash mapping repair → `d6a7e3c41`
- [x] Manually update `Cargo.toml` to redirect deps (polkadot-sdk → moonbeam-foundation fork, ethereum → MBF, evm → MBF) → `5264e3ae0`. This consolidates the prior cycle's separate "Use moonbeam-polkadot-stable*" CI ref and "Depend on MBF ethereum fork" cherry-picks into one commit.
- [x] Confirmed inherited from `upstream/stable2603` (rows in tracker now `Dropped, PR Upstream Merged`):
  - EIP-7939 (#1795), EIP-7883 (#1801), EIP-7823 (#1804), EIP-7825 (#1799)
  - Multi pre-runtime Frontier logs invalid (#1809)
  - eth_getTransactionReceipt race (#1802)
  - Drop lagging pubsub subscribers (#1814)
  - Forbid txs without chain ID (#1815)
  - eth_getBlockByNumber("latest") null fix (#1816)
  - Inconsistent Eth block visibility (#1818)
  - Single-writer mapping-sync (#1820)
  - RPC filter range checks (#1824)
  - Avoid eth_blockNumber 0x00 lag (#1832)
  - Wait for chain head & eth latest in createAndFinalizeBlock (#1855)
  - Improve "latest" block resolution on pruned nodes (#1856)
  - Reorged-out logs (#1862)
  - Bound logs journal memory (#1881)
  - Make tx gas limit cap configurable (upstream `b2088f29b`, no PR ref)
  - Frontier parity-db migration: also has equivalent in upstream — keep our cherry-pick for the additional moonbeam-foundation/frontier#252 fixes.
- [x] Cargo.lock regenerated; `cargo check --workspace` is clean (one harmless unused-const warning from the dispatch-error cherry-pick).
- [x] Pushed `moonbeam-polkadot-stable2603` to `moonbeam-foundation/frontier` at `a89b62d5e`.
- [x] Update tracker (`polkadot-sdk-stable2603.md`).
- [x] Side effect: bumped `moonbeam-foundation/evm:moonbeam-polkadot-stable2603` to `7dd6ecc6` so its `ethereum` dep points at the stable2603 branch (was stable2512). Required to avoid two-versions-of-`ethereum` cargo conflict in frontier.

## Phase 2 — Tracking doc finalization

- [x] Draft `polkadot-sdk-stable2603.md` (TBD placeholders for commits).
- [x] Run merge-base verification per repo (sub-agents, dispatched in parallel). 2026-06-18: frontier + polkadot-sdk verified via agents; evm/ethereum/moonkit have no `Verify` rows and were settled in Phase 0.5/1 (evm = 1 moonbeam-only commit, ethereum = #77 only, moonkit = reconciled in Phase 1.4).
- [x] Update tracker: real commit hashes for Included rows, finalize Cherry-pick column for every Verify row. All 13 frontier Included SHAs confirmed present + moonbeam-only; all 18 frontier `Verify` rows confirmed in `upstream/stable2603` and the `**Verify**` flags removed; all 9 polkadot-sdk Included SHAs confirmed.
- [x] Reconcile any agent-reported discrepancies. (1) polkadot-sdk `ddba2453` was untracked → row added. (2) frontier #1856 note cited the stable2512 cherry-pick SHA `54396433…` instead of the upstream commit `46cf7a43e` → corrected.

## Phase 3 — Moonbeam repository upgrade ✅ (Rust compile; TS fixtures pending)

- [x] `Cargo.toml`: replace `moonbeam-polkadot-stable2512` → `moonbeam-polkadot-stable2603` (180 occurrences across the 5 forks).
- [x] Refresh the lockfile (re-resolved to stable2603).
- [x] Compile-fix loop — green across the **realistic feature matrix** (default, `runtime-benchmarks`, `try-runtime`). Notable drift handled:
  - **Dropped crate** `cumulus-client-consensus-proposer` (upstream [#9947](https://github.com/paritytech/polkadot-sdk/pull/9947) folded it into sp-consensus/sc-basic-authorship); was an unused dep.
  - **`num_enum`** bumped to 0.7.6 (frontier `fp-evm` now needs `^0.7.6`).
  - **`RuntimeDebug` → `Debug`** (upstream [#10582](https://github.com/paritytech/polkadot-sdk/pull/10582) removed it from sp_core/sp_runtime/frame_support) — 51 sites / 14 files.
  - **Duplicate polkadot-sdk** root cause: moonkit's stable2603 base-bump pointed at canonical paritytech/frontier; added a redirect commit on `Moonsong-Labs/moonkit:moonbeam-polkadot-stable2603` (`ba06fb0`) to the MBF forks, unifying the tree.
  - **EVM `Runner::call`** gained a `state_override` arg; **`fp_rpc` `call`** gained `state_override`; **`SessionKeys::generate_session_keys`** now `(owner, seed) -> OpaqueGeneratedSessionKeys`.
  - **`cumulus_pallet_parachain_system::WeightInfo`** gained 3 methods (added to weight files).
  - **XCM executor `Config`** dropped `AssetClaims` (merged into `AssetTrap`, which now needs `ClaimAssets`).
  - **node/service**: `BuildNetworkParams::spawn_essential_handle`, `new_full_parts_record_import` pruning-filters arg, `CallExecutor::runtime_version`/`Backend::set_block_data` new params, moonkit `_phantom`/`relay_parent_offset` field changes, and the proof-recording refactor ([#9947](https://github.com/paritytech/polkadot-sdk/pull/9947)) in the lazy-loading manual-seal.
  - **runtime-benchmarks**: weight-reclaim benchmark needed the `GetCallMetadata` bound the logging cherry-pick added (fixed in the fork at `ddba2453`); `pallet_transaction_payment::benchmarking` is now private (use `Pallet` + `BenchmarkConfig`); `worst_case_holding` returns `AssetsInHolding` (built via `generate_holding_assets`).
- [x] **XCM credit-based holding migration** (the substantive piece): stable2603 reworked `AssetsInHolding` to carry `fungible::Credit` imbalances. Real impls for `erc20-xcm-bridge` / `moonbeam-foreign-assets` (`deposit_asset`/`withdraw_asset` via a notional credit, since erc20 isn't a Substrate `fungible`) and `xcm-weight-trader` (`buy_weight`/`refund_weight`/`Drop`).
- [x] No `[patch.crates-io]` needed — the fork branches resolve directly from the moonbeam-foundation remotes.
- [x] Update `tests/` TS fixtures referencing changed types/RPCs. — regenerated `typescript-api/` types and updated the dev fixtures (`test-block-mocked-relay.ts`, `test-transactional-outcomes.ts`, `test-precompile-relay-verifier.ts`) for the new relay / credit-model XCM events. *Running* them against a built node binary is deferred to Phase 6.

> **Note — `--all-features` is NOT a valid check for moonbeam.** It forces the mutually-exclusive `disable-genesis-builder` feature on, which strips `genesis_config_preset` (needed by the node's chain_spec). Use the realistic feature matrix (default / `runtime-benchmarks` / `try-runtime`) instead. Pallet unit tests can't be run in isolation either (`sp-session`/`sp-authority-discovery` fail to build no_std standalone — pre-existing infra quirk); run via the full workspace test.
>
> **Fork-side fixes pushed this cycle** (need recording in `polkadot-sdk-stable2603.md` / Phase 1 trackers): moonkit `ba06fb0` → rebased to `9d71129` after [moonkit#95](https://github.com/Moonsong-Labs/moonkit/pull/95) merged (redirect deps to MBF forks) and polkadot-sdk `ddba2453` (weight-reclaim benchmark `GetCallMetadata` bound — completes the weight-reclaim-logs cherry-pick `b0b4fd52a9e`).

## Phase 4 — Runtime, weights, migrations

- [ ] Re-run benchmarks for any pallet whose weight signature changed (`benchmarking-pallets` skill).
- [ ] Run `analyzing-weights` over the diff; flag regressions in ref_time / proof_size / DB reads.
- [ ] Author migrations for any pallet whose `StorageVersion` advanced upstream (`writing-migrations` skill).
- [ ] Run `try-runtime` against current production state for moonbase, moonriver, moonbeam.

## Phase 5 — Bridge maintenance

- [ ] Per `bridge-maintenance` skill, rebuild `substrate-relay` against stable2603.
- [ ] Regenerate zombienet relay chain specs.
- [ ] Run bridge integration tests; fix relayer regressions.

## Phase 6 — Verification

- [ ] `cargo check --workspace`
- [ ] `cargo check --workspace --features runtime-benchmarks`
- [ ] `cargo check --workspace --features try-runtime`
- [ ] Rust unit tests: `cargo test --workspace`
- [ ] TypeScript integration tests (`testing-moonbeam` skill).
- [ ] Smoke tests against moonbase/moonriver chain specs.
- [ ] XCM cross-chain test scenarios.
- [ ] Spot-check for cherry-pick regressions: confirm each Included row's behaviour still works (e.g. PrecompileWasmCmd, WildMultiAsset bound, POV underestimation fix).

## Risks & open questions

- **moonkit dependency** — ✅ resolved. [moonkit#95](https://github.com/Moonsong-Labs/moonkit/pull/95) merged to `main` 2026-06-18 (squash `4088d76`); the `moonbeam-polkadot-stable2603` release branch was rebased onto `main` (head `9d71129`) and moonbeam re-pinned its `Cargo.lock` to it. See Phase 1.4.
- **EVM divergence** — anything new in upstream v1.0 (new EIPs, gas changes) must be backported manually onto the 0.43.x fork.
- **Deferred EIPs** — confirm no EIP from stable2603 needs implementation work in our fork (`implementing-eips` skill).
- **Verification cost** — Phase 1.5 frontier verification has 14 rows; parallelize via sub-agents.

## Drop / archive checklist

When this upgrade ships:
- [ ] Move (or delete) this file. The cherry-pick tracker `polkadot-sdk-stable2603.md` stays.
- [ ] Confirm `polkadot-sdk-stable2512.md` and `polkadot-sdk-stable2506.md` remain for historical reference.
