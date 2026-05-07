# Cherry-picks for stable2603

> **Draft.** Commit hashes are placeholders (`TBD`) until the `moonbeam-polkadot-stable2603`
> branches are created in each fork. Rows marked **`Verify`** in the Note column require
> confirmation against upstream `stable2603` before finalizing — see
> `.claude/skills/qa-cherry-picks/verify-cherry-picks.md`.

## `polkadot-sdk`

| Applied | Title | Commit | Cherry pick | Status | Upstream PR | Note |
| --- | --- | --- | --- | --- | --- | --- |
| Yes | Add command PrecompileWasmCmd | [moonbeam-foundation/polkadot-sdk@909be1f](https://github.com/moonbeam-foundation/polkadot-sdk/commit/909be1fe0ca) | Included | Upstream PR not merged | [paritytech/polkadot-sdk#1641](https://github.com/paritytech/polkadot-sdk/pull/1641) | This PR should be kept because it improves the time CI takes to run the moonwall tests. <br> Basically the changes allow to start the node with already precompiled runtimes and avoid the compilation step during the node start-up. <br> Adapted to stable2603 `BackendRuntimeCode::new(&state, TryPendingCode::No)` signature change. |
| Yes | Comment log "Unexpected underflow in reducing consumer" | [moonbeam-foundation/polkadot-sdk@834c1d7](https://github.com/moonbeam-foundation/polkadot-sdk/commit/834c1d794d3) | Included | Permanent |  | For moonbeam the consumers make no sense as this counter exist only to know when we can remove the account <br> - And we never remove the account on moonbeam because we can't remove the nonce due to immortal eth transactions. |
| Yes | Bound WildMultiAsset max assets limit to 20 | [moonbeam-foundation/polkadot-sdk@edc21b0](https://github.com/moonbeam-foundation/polkadot-sdk/commit/edc21b0c2d5) | Included | Permanent |  | [MOON-2930](https://opslayer.atlassian.net/browse/MOON-2930). No upstream PR — `WILD_MULTI_ASSET_MAX_LIMIT` is a moonbeam-only constant not present in upstream. |
| Yes | Account for pallet-parameters weight in benchmarks | [moonbeam-foundation/polkadot-sdk@443bfd8](https://github.com/moonbeam-foundation/polkadot-sdk/commit/443bfd80e24) | Included | Temporary | [paritytech/polkadot-sdk#6477](https://github.com/paritytech/polkadot-sdk/pull/6477) | [MOON-2964](https://opslayer.atlassian.net/browse/MOON-2964) <br> Theoretically, this change to pallet parameters is not necessary if all code that fetches these dynamic parameters is properly benchmarked, so it makes sense that they don't want this workaround upstream. <br> [MOON-3294](https://opslayer.atlassian.net/browse/MOON-3294) |
| Yes | Add storage benchmark --keys-limit option |  | Dropped | PR Upstream Merged |  | stable2603 ships `--keys-limit` and `--child-keys-limit` natively in `substrate/utils/frame/benchmarking-cli/src/storage/cmd.rs`. |
| Yes | Remove pallet-revive from pallet-xcm |  | Dropped | PR Upstream Merged |  | stable2603's `polkadot/xcm/pallet-xcm/Cargo.toml` no longer depends on `pallet-revive`. |
| Yes | Fix charge_transaction_payment benchmark |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#10444](https://github.com/paritytech/polkadot-sdk/pull/10444) | Merged into stable2603 as `4b934d0a252`. |
| Yes | ParachainTracingExecuteBlock |  | Dropped but needs refactoring | PR Upstream Merged | [paritytech/polkadot-sdk#9214](https://github.com/paritytech/polkadot-sdk/pull/9214) | Replaced by [paritytech/polkadot-sdk#9871](https://github.com/paritytech/polkadot-sdk/pull/9871) <br> Refactoring on moonbeam repo: we should use the new config cumulus_service::ParachainTracingExecuteBlock (default None config doesn't work) |
| Yes | Backport PR#8108 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#8108](https://github.com/paritytech/polkadot-sdk/pull/8108) |  |
| Yes | Backport PR#10102 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#10102](https://github.com/paritytech/polkadot-sdk/pull/10102) |  |
| Yes | Backport PR#9262 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9262](https://github.com/paritytech/polkadot-sdk/pull/9262) | Can be dropped Polkadot stable2509 |
| Yes | Add supported_version to pallet-xcm genesis config |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9225](https://github.com/paritytech/polkadot-sdk/pull/9225) | Can be dropped Polkadot stable2509 |
| Yes | Stronger WASM compression |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9875](https://github.com/paritytech/polkadot-sdk/pull/9875) |  |
| Yes | Backport PR#9791 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9791](https://github.com/paritytech/polkadot-sdk/pull/9791) |  |
| Yes | Backport PR#9419 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9419](https://github.com/paritytech/polkadot-sdk/pull/9419) |  |
| Yes | Backport PR#8939 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#8939](https://github.com/paritytech/polkadot-sdk/pull/8939) |  |
| Yes | Backport PR#9319 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9319](https://github.com/paritytech/polkadot-sdk/pull/9319) |  |
| Yes | Backport PR#9927 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9927](https://github.com/paritytech/polkadot-sdk/pull/9927) |  |
| Yes | Backport PR#9976 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9976](https://github.com/paritytech/polkadot-sdk/pull/9976) |  |
| Yes | Backport PR#9178 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9178](https://github.com/paritytech/polkadot-sdk/pull/9178) |  |
| Yes | Backport PR#9929 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9929](https://github.com/paritytech/polkadot-sdk/pull/9929) |  |
| Yes | Backport PR#10305 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#10305](https://github.com/paritytech/polkadot-sdk/pull/10305) |  |
| Yes | Backport PR#9703 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9703](https://github.com/paritytech/polkadot-sdk/pull/9703) |  |
| Yes | Backport PR#9990 |  | Dropped | PR Upstream Merged | [paritytech/polkadot-sdk#9990](https://github.com/paritytech/polkadot-sdk/pull/9990) |  |

## `ethereum`

| Applied | Title | Commit | Cherry pick | Status | Upstream PR | Note |
| --- | --- | --- | --- | --- | --- | --- |
| No | Refactor transaction signature validation |  | Dropped | PR Merged | [rust-ethereum/ethereum#75](https://github.com/rust-ethereum/ethereum/pull/75) |  |
| Yes | Add encoded length methods to transactions | TBD | Included | PR not merged | [rust-ethereum/ethereum#77](https://github.com/rust-ethereum/ethereum/pull/77) |  |

## `evm`

| Applied | Title | Commit | Cherry pick | Status | Upstream PR | Note |
| --- | --- | --- | --- | --- | --- | --- |
| Yes | Add support for EIP-7939 (CLZ opcode) | TBD | Included | PR Upstream Merged | [rust-ethereum/evm#400](https://github.com/rust-ethereum/evm/pull/400) | Upstream merged PR#400 but on a different major version (v1.0). Still needed on the moonbeam 0.43.x fork. |

## `frontier`

| Applied | Title | Commit | Cherry pick | Status | Upstream PR | Note |
| --- | --- | --- | --- | --- | --- | --- |
| Yes | Use moonbeam-polkadot-stable2603 branch name for CI | TBD | Included | Permanent |  | To make the CI run on our fork. Branch-name reference must be bumped from stable2512 → stable2603 during cherry-pick. <br> Instead of this cherry-pick we can change our branch name conventions to name them polkadot-v*-moonbeam |
| Yes | Do client side check of withdraw-ability | TBD | Included | Upstream PR not merged | [polkadot-evm/frontier#1546](https://github.com/polkadot-evm/frontier/pull/1546) | [MOON-2968](https://opslayer.atlassian.net/browse/MOON-2968) original issue [MOON-2597](https://opslayer.atlassian.net/browse/MOON-2597) <br> Keep: the SDK fix in [paritytech/polkadot-sdk#2292](https://github.com/paritytech/polkadot-sdk/pull/2292) (for issue [paritytech/polkadot-sdk#1833](https://github.com/paritytech/polkadot-sdk/issues/1833)) did not fully resolve the problem — Eloïs confirmed after investigation that this cherry-pick is still required. |
| Yes | Catch ethereum execution info | TBD | Included | Upstream PR not merged | [polkadot-evm/frontier#1547](https://github.com/polkadot-evm/frontier/pull/1547) | [MOON-2969](https://opslayer.atlassian.net/browse/MOON-2969) |
| Yes | Fix dispatch error legacy decoding | TBD | Included | Permanent |  | This commit should not be pushed upstream since it only impacts projects in production before [paritytech/substrate#10776](https://github.com/paritytech/substrate/pull/10776) and the fix will not be equal for every chain. <br> The changes contained in [paritytech/substrate#10776](https://github.com/paritytech/substrate/pull/10776) changed the scale encoding for variant DispatchError::Module(a breaking change). This commit adds a Legacy type, which is used when decoding the return type of call method in version 4 of EthereumRuntimeRPCApi. |
| No | Account eth tx size only once | TBD | squash | Upstream PR not merged | [polkadot-evm/frontier#1564](https://github.com/polkadot-evm/frontier/pull/1564) | Squashed with [moonbeam-foundation/frontier@b7d6e27](https://github.com/moonbeam-foundation/frontier/commit/b7d6e27d1571c555aea8036b4d3789eedfc34aef) [moonbeam-foundation/frontier#224](https://github.com/moonbeam-foundation/frontier/pull/224) |
| Yes | Effective gas calculation log |  | Dropped | PR Upstream Merged | [moonbeam-foundation/frontier#224](https://github.com/moonbeam-foundation/frontier/pull/224) | [MOON-2976](https://opslayer.atlassian.net/browse/MOON-2976) @Pablo Labarta PR: [moonbeam-foundation/frontier#224](https://github.com/moonbeam-foundation/frontier/pull/224) |
| Yes | Expose lru_cache | TBD | Included | Upstream PR not merged | [polkadot-evm/frontier#1568](https://github.com/polkadot-evm/frontier/pull/1568) | [MOON-2976](https://opslayer.atlassian.net/browse/MOON-2976) |
| Yes | Address POV Underestimations | TBD | Included | Needs PR upstream | [moonbeam-foundation/frontier@fdfdb95](https://github.com/moonbeam-foundation/frontier/commit/fdfdb95057402ff35f69869339262a108ffccf94) | [MOON-3292](https://opslayer.atlassian.net/browse/MOON-3292) |
| Yes | Validate max block range eth_getLogs RPC |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1794](https://github.com/polkadot-evm/frontier/pull/1794) | [MOON-3293](https://opslayer.atlassian.net/browse/MOON-3293) |
| Yes | Fix format of EIP7702 transactions returned by the RPC interface |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1775](https://github.com/polkadot-evm/frontier/pull/1775) |  |
| No | Fix mapping-sync: use import-time is_new_best for newHeads notifications |  | Dropped | Closed | [polkadot-evm/frontier#1781](https://github.com/polkadot-evm/frontier/pull/1781) | NOTE: part of Extend EthereumBlockNotification with reorg info to allow compliance with Ethereum specs |
| Yes | Extend EthereumBlockNotification with reorg info |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1787](https://github.com/polkadot-evm/frontier/pull/1787) |  |
| Yes | Fixes eth_getLogs RPC behavior by filtering out logs with no topics when a filter is provided |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1769](https://github.com/polkadot-evm/frontier/pull/1769) |  |
| Yes | Add support for Osaka fork (EIP-7939) |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1795](https://github.com/polkadot-evm/frontier/pull/1795) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Upgrade frame-metadata | TBD | Included | Temporary |  | Can be dropped once [MOON-3374](https://opslayer.atlassian.net/browse/MOON-3374) has been addressed |
| Yes | Add support for EIP-7883 |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1801](https://github.com/polkadot-evm/frontier/pull/1801) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Add support for EIP-7823 |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1804](https://github.com/polkadot-evm/frontier/pull/1804) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Add support for EIP-7825 |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1799](https://github.com/polkadot-evm/frontier/pull/1799) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Consider block invalid when containing multiple pre-runtime Frontier logs |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1809](https://github.com/polkadot-evm/frontier/pull/1809) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Fix eth_getTransactionReceipt race condition |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1802](https://github.com/polkadot-evm/frontier/pull/1802) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Fix frontier parity-db migration |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1817](https://github.com/polkadot-evm/frontier/pull/1817) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Fix dropping lagging pubsub subscribers |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1814](https://github.com/polkadot-evm/frontier/pull/1814) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Forbid transactions without chain ID by default |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1815](https://github.com/polkadot-evm/frontier/pull/1815) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Fix eth_getBlockByNumber("latest") returning null intermittently |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1816](https://github.com/polkadot-evm/frontier/pull/1816) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Fix inconsistent Eth block visibility caused by missing/stale block-number pointer |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1818](https://github.com/polkadot-evm/frontier/pull/1818) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| No | Simplify and harden re-org handling (single writer path in mapping-sync) |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1820](https://github.com/polkadot-evm/frontier/pull/1820) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Wait for chain head and eth "latest" in createAndFinalizeBlock() |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1855](https://github.com/polkadot-evm/frontier/pull/1855) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Avoid eth_blockNumber returning 0x00 when mapping-sync lags |  | Dropped | PR Upstream Merged | [polkadot-evm/frontier#1832](https://github.com/polkadot-evm/frontier/pull/1832) | Expected to be in `frontier-stable2603`. **Verify** before finalizing. |
| Yes | Validate transaction size | TBD | Included | Upstream PR not merged | [polkadot-evm/frontier#1807](https://github.com/polkadot-evm/frontier/pull/1807) |  |
| Yes | Improve "latest" block resolution on pruned nodes | TBD | Included | Upstream PR not merged | [polkadot-evm/frontier#1856](https://github.com/polkadot-evm/frontier/pull/1856) | Cherry-picked from stable2506 commit [615b2aec](https://github.com/moonbeam-foundation/frontier/commit/615b2aecce141e6d9744b126ac2929e9fc2bddcc). Adds `state_pruning_blocks` param to `MappingSyncWorker::new()` for correct block resolution on pruned nodes. |

## `moonkit`

| Applied | Title | Commit | Cherry pick | Status | Upstream PR | Note |
| --- | --- | --- | --- | --- | --- | --- |
| Yes | Update to polkadot-sdk stable2603 |  | Dropped | Needs PR upstream |  | Tracking row for the moonkit base bump to stable2603. Drop once Moonsong-Labs merges the upgrade PR into `main`, then base `moonbeam-polkadot-stable2603` off the merge commit. |
| Yes | Add using_fake_author logic to author-slot-filter |  | Dropped | PR Merged | [Moonsong-Labs/moonkit#92](https://github.com/Moonsong-Labs/moonkit/pull/92) | Already merged into `main` during the stable2512 cycle. |
