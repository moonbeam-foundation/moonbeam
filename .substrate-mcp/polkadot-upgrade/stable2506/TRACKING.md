# Polkadot SDK stable2506 Upgrade Tracking for Moonbeam

## Project Context

**Project**: Moonbeam - Ethereum-compatible parachain built with Polkadot SDK
**Release**: stable2506
**Total PRs**: 134
**Analysis Date**: 2025-10-22

### Moonbeam Runtime Architecture

Moonbeam is an Ethereum-compatible parachain with three runtime variants:
- **moonbeam**: Production runtime for Polkadot
- **moonriver**: Production runtime for Kusama
- **moonbase**: TestNet runtime for Westend

### Key Pallets in Use

**Standard Substrate/FRAME pallets**:
System, Utility, Timestamp, Balances, Sudo, Scheduler, Treasury, Proxy, Identity, Multisig, Preimage, Whitelist, Parameters

**Parachain-specific (Cumulus)**:
ParachainSystem, ParachainInfo, XcmpQueue, CumulusXcm, MessageQueue, WeightReclaim

**XCM-related**:
PolkadotXcm, XcmTransactor, XcmWeightTrader, EthereumXcm, Erc20XcmBridge, EmergencyParaXcm

**Governance**:
ConvictionVoting, Referenda, Origins (custom), TreasuryCouncilCollective, OpenTechCommitteeCollective, RootTesting

**EVM/Ethereum**:
EVM, Ethereum, EthereumChainId

**Moonbeam-specific**:
ParachainStaking, AuthorInherent, AuthorFilter, AuthorMapping, CrowdloanRewards, MaintenanceMode, MoonbeamOrbiters, Randomness, ProxyGenesisCompanion, AsyncBacking, MoonbeamLazyMigrations, RelayStorageRoots, EvmForeignAssets, MultiBlockMigrations

---

## PR Tracking

**Total PRs to Analyze**: 134
**Status**: ✅ **ALL ANALYSES COMPLETE** (Updated: 2025-10-22)

**Quick Links:**
- [Critical PRs requiring action](#critical-prs-requiring-action)
- [Medium Priority PRs](#medium-priority-prs)
- [Analysis Progress Summary](#analysis-progress)

| PR | GitHub | Title | Status | Category | Analysis |
| --- | --- | --- | --- | --- | --- |
| [pr_3811.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_3811.prdoc) | [#3811](https://github.com/paritytech/polkadot-sdk/pull/3811) | Implicit chill when full unbounding in [pallet_staking] | ✅ Complete | Low Impact | [View Analysis](pr_3811.md) |
| [pr_5620.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_5620.prdoc) | [#5620](https://github.com/paritytech/polkadot-sdk/pull/5620) | New NFT traits: granular and abstract interface | ✅ Complete | No Impact | [View Analysis](pr_5620.md) |
| [pr_5884.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_5884.prdoc) | [#5884](https://github.com/paritytech/polkadot-sdk/pull/5884) | Set PoV size limit to 10 Mb | ✅ Complete | Inherited | [View Analysis](pr_5884.md) |
| [pr_6010.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_6010.prdoc) | [#6010](https://github.com/paritytech/polkadot-sdk/pull/6010) | Proof Of Possession for public keys | ✅ Complete | No Impact | [View Analysis](pr_6010.md) |
| [pr_6137.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_6137.prdoc) | [#6137](https://github.com/paritytech/polkadot-sdk/pull/6137) | cumulus: ParachainBlockData support multiple blocks | ✅ Complete | Medium Impact | [View Analysis](pr_6137.md) |
| [pr_6312.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_6312.prdoc) | [#6312](https://github.com/paritytech/polkadot-sdk/pull/6312) | DeprecationInfo propagate or automatically add allow(deprecated) attributes in the generated code | ✅ Complete | Low Impact | [View Analysis](pr_6312.md) |
| [pr_6324.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_6324.prdoc) | [#6324](https://github.com/paritytech/polkadot-sdk/pull/6324) | Introduce #[pallet::authorize(...)] macro attribute and AuthorizeCall system transaction extension | ✅ Complete | Analyzed | [View Analysis](pr_6324.md) |
| [pr_6827.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_6827.prdoc) | [#6827](https://github.com/paritytech/polkadot-sdk/pull/6827) | Introduction of Approval Slashes | ✅ Complete | Analyzed | [View Analysis](pr_6827.md) |
| [pr_7220.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7220.prdoc) | [#7220](https://github.com/paritytech/polkadot-sdk/pull/7220) | Yet Another Parachain Runtime | ✅ Complete | Analyzed | [View Analysis](pr_7220.md) |
| [pr_7229.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7229.prdoc) | [#7229](https://github.com/paritytech/polkadot-sdk/pull/7229) | FRAME: Deprecate RuntimeEvent associated type from Config trait | ✅ Complete | Analyzed | [View Analysis](pr_7229.md) |
| [pr_7375.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7375.prdoc) | [#7375](https://github.com/paritytech/polkadot-sdk/pull/7375) | Refactor the host <-> runtime interface machinery (the #[runtime_interface] macro) and the way host functions are defined | ✅ Complete | Analyzed | [View Analysis](pr_7375.md) |
| [pr_7556.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7556.prdoc) | [#7556](https://github.com/paritytech/polkadot-sdk/pull/7556) | Add trie cache warmup | ✅ Complete | Analyzed | [View Analysis](pr_7556.md) |
| [pr_7592.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7592.prdoc) | [#7592](https://github.com/paritytech/polkadot-sdk/pull/7592) | Add Paras authorize_code_hash + apply_authorized_code feature | ✅ Complete | Analyzed | [View Analysis](pr_7592.md) |
| [pr_7597.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7597.prdoc) | [#7597](https://github.com/paritytech/polkadot-sdk/pull/7597) | Introduce CreateBare, deprecated CreateInherent | ✅ Complete | Analyzed | [View Analysis](pr_7597.md) |
| [pr_7666.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7666.prdoc) | [#7666](https://github.com/paritytech/polkadot-sdk/pull/7666) | Migrate 0009-approval-voting-coalescing.zndsl to zombienet-sdk | ✅ Complete | Analyzed | [View Analysis](pr_7666.md) |
| [pr_7682.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7682.prdoc) | [#7682](https://github.com/paritytech/polkadot-sdk/pull/7682) | Make SharedTrieCache/LocalTrieCache work with entire state in memory | ✅ Complete | Analyzed | [View Analysis](pr_7682.md) |
| [pr_7719.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7719.prdoc) | [#7719](https://github.com/paritytech/polkadot-sdk/pull/7719) | Add export-chain-spec substrate CLI command | ✅ Complete | Analyzed | [View Analysis](pr_7719.md) |
| [pr_7720.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7720.prdoc) | [#7720](https://github.com/paritytech/polkadot-sdk/pull/7720) | Clamp Core Fellowship Benchmarks to Runtime MaxRank Configuration | ✅ Complete | Analyzed | [View Analysis](pr_7720.md) |
| [pr_7730.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7730.prdoc) | [#7730](https://github.com/paritytech/polkadot-sdk/pull/7730) | Nest errors in pallet-xcm | ✅ Complete | Analyzed | [View Analysis](pr_7730.md) |
| [pr_7762.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7762.prdoc) | [#7762](https://github.com/paritytech/polkadot-sdk/pull/7762) | ERC20 Asset Transactor | ✅ Complete | Analyzed | [View Analysis](pr_7762.md) |
| [pr_7833.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7833.prdoc) | [#7833](https://github.com/paritytech/polkadot-sdk/pull/7833) | add poke_deposit extrinsic to pallet-society | ✅ Complete | Analyzed | [View Analysis](pr_7833.md) |
| [pr_7857.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7857.prdoc) | [#7857](https://github.com/paritytech/polkadot-sdk/pull/7857) | Add new host APIs set_storage_or_clear and get_storage_or_zero | ✅ Complete | Analyzed | [View Analysis](pr_7857.md) |
| [pr_7867.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7867.prdoc) | [#7867](https://github.com/paritytech/polkadot-sdk/pull/7867) | benchmark/storage Make read/write benchmarks more accurate | ✅ Complete | Analyzed | [View Analysis](pr_7867.md) |
| [pr_7882.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7882.prdoc) | [#7882](https://github.com/paritytech/polkadot-sdk/pull/7882) | add poke_deposit extrinsic to pallet-recovery | ✅ Complete | Analyzed | [View Analysis](pr_7882.md) |
| [pr_7936.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7936.prdoc) | [#7936](https://github.com/paritytech/polkadot-sdk/pull/7936) | Replace Validator FullIdentification from Exposure to Existence | ✅ Complete | Analyzed | [View Analysis](pr_7936.md) |
| [pr_7944.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7944.prdoc) | [#7944](https://github.com/paritytech/polkadot-sdk/pull/7944) | Allow to set a worst case buy execution fee asset and weight | ✅ Complete | Analyzed | [View Analysis](pr_7944.md) |
| [pr_7955.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7955.prdoc) | [#7955](https://github.com/paritytech/polkadot-sdk/pull/7955) | Add ApprovedPeer UMP signal | ✅ Complete | Analyzed | [View Analysis](pr_7955.md) |
| [pr_7960.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7960.prdoc) | [#7960](https://github.com/paritytech/polkadot-sdk/pull/7960) | Stabilize pallet view functions | ✅ Complete | Analyzed | [View Analysis](pr_7960.md) |
| [pr_7980.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7980.prdoc) | [#7980](https://github.com/paritytech/polkadot-sdk/pull/7980) | fatxpool: optimize txs prunning based on inactive views provides tags | ✅ Complete | Analyzed | [View Analysis](pr_7980.md) |
| [pr_7995.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_7995.prdoc) | [#7995](https://github.com/paritytech/polkadot-sdk/pull/7995) | Add PureKilled event to pallet-proxy | ✅ Complete | Analyzed | [View Analysis](pr_7995.md) |
| [pr_8001.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8001.prdoc) | [#8001](https://github.com/paritytech/polkadot-sdk/pull/8001) | Structured Logging for transaction pool | ✅ Complete | Analyzed | [View Analysis](pr_8001.md) |
| [pr_8021.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8021.prdoc) | [#8021](https://github.com/paritytech/polkadot-sdk/pull/8021) | XCMP: use batching when enqueuing inbound messages | ✅ Complete | Analyzed | [View Analysis](pr_8021.md) |
| [pr_8038.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8038.prdoc) | [#8038](https://github.com/paritytech/polkadot-sdk/pull/8038) | Fix penpal runtime | ✅ Complete | Analyzed | [View Analysis](pr_8038.md) |
| [pr_8069.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8069.prdoc) | [#8069](https://github.com/paritytech/polkadot-sdk/pull/8069) | Benchmark storage access on block validation | ✅ Complete | Analyzed | [View Analysis](pr_8069.md) |
| [pr_8072.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8072.prdoc) | [#8072](https://github.com/paritytech/polkadot-sdk/pull/8072) | RFC-0008: Store parachain bootnodes in the relay chain DHT | ✅ Complete | Analyzed | [View Analysis](pr_8072.md) |
| [pr_8102.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8102.prdoc) | [#8102](https://github.com/paritytech/polkadot-sdk/pull/8102) | Make min_peers_to_start_warp_sync configurable | ✅ Complete | Analyzed | [View Analysis](pr_8102.md) |
| [pr_8103.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8103.prdoc) | [#8103](https://github.com/paritytech/polkadot-sdk/pull/8103) | [pallet-revive] Add genesis config | ✅ Complete | Analyzed | [View Analysis](pr_8103.md) |
| [pr_8118.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8118.prdoc) | [#8118](https://github.com/paritytech/polkadot-sdk/pull/8118) | Safer conversions in polkadot-runtime-parachains | ✅ Complete | Analyzed | [View Analysis](pr_8118.md) |
| [pr_8122.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8122.prdoc) | [#8122](https://github.com/paritytech/polkadot-sdk/pull/8122) | Accommodate small changes to unstable V16 metadata format | ✅ Complete | Analyzed | [View Analysis](pr_8122.md) |
| [pr_8127.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8127.prdoc) | [#8127](https://github.com/paritytech/polkadot-sdk/pull/8127) | [AHM] Async Staking module across AH and RC | ✅ Complete | Analyzed | [View Analysis](pr_8127.md) |
| [pr_8130.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8130.prdoc) | [#8130](https://github.com/paritytech/polkadot-sdk/pull/8130) | rpc v2: move archive MethodResult to the archive mod | ✅ Complete | Analyzed | [View Analysis](pr_8130.md) |
| [pr_8134.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8134.prdoc) | [#8134](https://github.com/paritytech/polkadot-sdk/pull/8134) | separate validation and collation protocols | ✅ Complete | Analyzed | [View Analysis](pr_8134.md) |
| [pr_8148.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8148.prdoc) | [#8148](https://github.com/paritytech/polkadot-sdk/pull/8148) | [revive] eth-rpc refactoring | ✅ Complete | Analyzed | [View Analysis](pr_8148.md) |
| [pr_8153.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8153.prdoc) | [#8153](https://github.com/paritytech/polkadot-sdk/pull/8153) | Introduce SelectCore digest in Cumulus | ✅ Complete | Analyzed | [View Analysis](pr_8153.md) |
| [pr_8163.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8163.prdoc) | [#8163](https://github.com/paritytech/polkadot-sdk/pull/8163) | chore: idiomatic rust cleanup | ✅ Complete | Analyzed | [View Analysis](pr_8163.md) |
| [pr_8164.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8164.prdoc) | [#8164](https://github.com/paritytech/polkadot-sdk/pull/8164) | [PoP] Add personhood tracking pallets | ✅ Complete | Analyzed | [View Analysis](pr_8164.md) |
| [pr_8171.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8171.prdoc) | [#8171](https://github.com/paritytech/polkadot-sdk/pull/8171) | Add vested transfer event emission | ✅ Complete | Analyzed | [View Analysis](pr_8171.md) |
| [pr_8179.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8179.prdoc) | [#8179](https://github.com/paritytech/polkadot-sdk/pull/8179) | Do not make pallet-identity benchmarks signature-dependent | ✅ Complete | Analyzed | [View Analysis](pr_8179.md) |
| [pr_8197.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8197.prdoc) | [#8197](https://github.com/paritytech/polkadot-sdk/pull/8197) | [pallet-revive] add fee_history | ✅ Complete | Analyzed | [View Analysis](pr_8197.md) |
| [pr_8208.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8208.prdoc) | [#8208](https://github.com/paritytech/polkadot-sdk/pull/8208) | Omni Node: Enable OCW http | ✅ Complete | Analyzed | [View Analysis](pr_8208.md) |
| [pr_8212.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8212.prdoc) | [#8212](https://github.com/paritytech/polkadot-sdk/pull/8212) | [pallet-revive] fix bn128 benchmark | ✅ Complete | Analyzed | [View Analysis](pr_8212.md) |
| [pr_8230.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8230.prdoc) | [#8230](https://github.com/paritytech/polkadot-sdk/pull/8230) | add parachain block validation latency metrics and logs | ✅ Complete | Analyzed | [View Analysis](pr_8230.md) |
| [pr_8234.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8234.prdoc) | [#8234](https://github.com/paritytech/polkadot-sdk/pull/8234) | Set a memory limit when decoding an UncheckedExtrinsic | ✅ Complete | Analyzed | [View Analysis](pr_8234.md) |
| [pr_8238.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8238.prdoc) | [#8238](https://github.com/paritytech/polkadot-sdk/pull/8238) | Add checked_sqrt to the FixedPointNumber trait | ✅ Complete | Analyzed | [View Analysis](pr_8238.md) |
| [pr_8248.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8248.prdoc) | [#8248](https://github.com/paritytech/polkadot-sdk/pull/8248) | Frame: Authorize pallet::error int discriminant | ✅ Complete | Analyzed | [View Analysis](pr_8248.md) |
| [pr_8254.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8254.prdoc) | [#8254](https://github.com/paritytech/polkadot-sdk/pull/8254) | Introduce remove_upgrade_cooldown | ✅ Complete | Analyzed | [View Analysis](pr_8254.md) |
| [pr_8262.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8262.prdoc) | [#8262](https://github.com/paritytech/polkadot-sdk/pull/8262) | pallet_revive: Replace adhoc pre-compiles with pre-compile framework | ✅ Complete | Analyzed | [View Analysis](pr_8262.md) |
| [pr_8271.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8271.prdoc) | [#8271](https://github.com/paritytech/polkadot-sdk/pull/8271) | Snowbridge - Message reward topups | ✅ Complete | Analyzed | [View Analysis](pr_8271.md) |
| [pr_8273.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8273.prdoc) | [#8273](https://github.com/paritytech/polkadot-sdk/pull/8273) | pallet-revive: Add net-listening rpc | ✅ Complete | Analyzed | [View Analysis](pr_8273.md) |
| [pr_8274.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8274.prdoc) | [#8274](https://github.com/paritytech/polkadot-sdk/pull/8274) | [pallet-revive] add get_storage_var_key for variable-sized keys | ✅ Complete | Analyzed | [View Analysis](pr_8274.md) |
| [pr_8281.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8281.prdoc) | [#8281](https://github.com/paritytech/polkadot-sdk/pull/8281) | workaround: XcmPaymentApi::query_weight_to_asset_fee simple common impl | ✅ Complete | Analyzed | [View Analysis](pr_8281.md) |
| [pr_8289.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8289.prdoc) | [#8289](https://github.com/paritytech/polkadot-sdk/pull/8289) | Extract create_pool_with_native_on macro to common crate | ✅ Complete | Analyzed | [View Analysis](pr_8289.md) |
| [pr_8299.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8299.prdoc) | [#8299](https://github.com/paritytech/polkadot-sdk/pull/8299) | Collator: Support building on older relay parents | ✅ Complete | Analyzed | [View Analysis](pr_8299.md) |
| [pr_8310.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8310.prdoc) | [#8310](https://github.com/paritytech/polkadot-sdk/pull/8310) | staking-async: add missing new_session_genesis | ✅ Complete | Analyzed | [View Analysis](pr_8310.md) |
| [pr_8311.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8311.prdoc) | [#8311](https://github.com/paritytech/polkadot-sdk/pull/8311) | [pallet-revive] update tracing rpc methods parameters | ✅ Complete | Analyzed | [View Analysis](pr_8311.md) |
| [pr_8314.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8314.prdoc) | [#8314](https://github.com/paritytech/polkadot-sdk/pull/8314) | Add RPCs in the statement store to get the statements and not just the statement data | ✅ Complete | Analyzed | [View Analysis](pr_8314.md) |
| [pr_8316.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8316.prdoc) | [#8316](https://github.com/paritytech/polkadot-sdk/pull/8316) | Remove slashing spans from pallet-staking-async | ✅ Complete | Analyzed | [View Analysis](pr_8316.md) |
| [pr_8323.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8323.prdoc) | [#8323](https://github.com/paritytech/polkadot-sdk/pull/8323) | Allow genesis-presets to be patched and remove native runtime calls from the | ✅ Complete | Analyzed | [View Analysis](pr_8323.md) |
| [pr_8327.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8327.prdoc) | [#8327](https://github.com/paritytech/polkadot-sdk/pull/8327) | Update to the latest unstable V16 metadata | ✅ Complete | Analyzed | [View Analysis](pr_8327.md) |
| [pr_8332.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8332.prdoc) | [#8332](https://github.com/paritytech/polkadot-sdk/pull/8332) | parachain informant | ✅ Complete | Analyzed | [View Analysis](pr_8332.md) |
| [pr_8337.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8337.prdoc) | [#8337](https://github.com/paritytech/polkadot-sdk/pull/8337) | add staking/election related view functions | ✅ Complete | Analyzed | [View Analysis](pr_8337.md) |
| [pr_8339.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8339.prdoc) | [#8339](https://github.com/paritytech/polkadot-sdk/pull/8339) | [AHM] add election-provider-multi-block::minimum-score to genesis config | ✅ Complete | Analyzed | [View Analysis](pr_8339.md) |
| [pr_8344.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8344.prdoc) | [#8344](https://github.com/paritytech/polkadot-sdk/pull/8344) | XCMP weight metering: account for the MQ page position | ✅ Complete | Analyzed | [View Analysis](pr_8344.md) |
| [pr_8345.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8345.prdoc) | [#8345](https://github.com/paritytech/polkadot-sdk/pull/8345) | tx/metrics: Add metrics for the RPC v2 transactionWatch_v1_submitAndWatch | ✅ Complete | Analyzed | [View Analysis](pr_8345.md) |
| [pr_8369.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8369.prdoc) | [#8369](https://github.com/paritytech/polkadot-sdk/pull/8369) | Enhancements to macros for trusted teleporter scenarios | ✅ Complete | Analyzed | [View Analysis](pr_8369.md) |
| [pr_8370.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8370.prdoc) | [#8370](https://github.com/paritytech/polkadot-sdk/pull/8370) | fix unneeded collator connection issue | ✅ Complete | Analyzed | [View Analysis](pr_8370.md) |
| [pr_8376.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8376.prdoc) | [#8376](https://github.com/paritytech/polkadot-sdk/pull/8376) | Remove TakeFirstAssetTrader from AH Westend and Rococo | ✅ Complete | Analyzed | [View Analysis](pr_8376.md) |
| [pr_8382.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8382.prdoc) | [#8382](https://github.com/paritytech/polkadot-sdk/pull/8382) | add poke_deposit extrinsic to pallet-bounties | ✅ Complete | Analyzed | [View Analysis](pr_8382.md) |
| [pr_8387.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8387.prdoc) | [#8387](https://github.com/paritytech/polkadot-sdk/pull/8387) | Update tests-evm.yml | ✅ Complete | Analyzed | [View Analysis](pr_8387.md) |
| [pr_8409.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8409.prdoc) | [#8409](https://github.com/paritytech/polkadot-sdk/pull/8409) | check XCM size in VMP routing | ✅ Complete | Analyzed | [View Analysis](pr_8409.md) |
| [pr_8422.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8422.prdoc) | [#8422](https://github.com/paritytech/polkadot-sdk/pull/8422) | [AHM] Staking async fixes for XCM and election planning | ✅ Complete | Analyzed | [View Analysis](pr_8422.md) |
| [pr_8441.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8441.prdoc) | [#8441](https://github.com/paritytech/polkadot-sdk/pull/8441) | Update prdoc in 8327 to fix release issue | ✅ Complete | Analyzed | [View Analysis](pr_8441.md) |
| [pr_8443.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8443.prdoc) | [#8443](https://github.com/paritytech/polkadot-sdk/pull/8443) | Stabilize V16 metadata | ✅ Complete | Analyzed | [View Analysis](pr_8443.md) |
| [pr_8445.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8445.prdoc) | [#8445](https://github.com/paritytech/polkadot-sdk/pull/8445) | Fix the clearing of gap sync on known imported blocks | ✅ Complete | Analyzed | [View Analysis](pr_8445.md) |
| [pr_8461.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8461.prdoc) | [#8461](https://github.com/paritytech/polkadot-sdk/pull/8461) | Use litep2p as the default network backend | ✅ Complete | Analyzed | [View Analysis](pr_8461.md) |
| [pr_8470.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8470.prdoc) | [#8470](https://github.com/paritytech/polkadot-sdk/pull/8470) | Stabilize the FRAME umbrella crate | ✅ Complete | Analyzed | [View Analysis](pr_8470.md) |
| [pr_8473.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8473.prdoc) | [#8473](https://github.com/paritytech/polkadot-sdk/pull/8473) | Snowbridge: Remove asset location check | ✅ Complete | Analyzed | [View Analysis](pr_8473.md) |
| [pr_8477.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8477.prdoc) | [#8477](https://github.com/paritytech/polkadot-sdk/pull/8477) | FeeTracker deduplications | ✅ Complete | Analyzed | [View Analysis](pr_8477.md) |
| [pr_8500.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8500.prdoc) | [#8500](https://github.com/paritytech/polkadot-sdk/pull/8500) | txpool: fix tx removal from unlocks set | ✅ Complete | Analyzed | [View Analysis](pr_8500.md) |
| [pr_8504.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8504.prdoc) | [#8504](https://github.com/paritytech/polkadot-sdk/pull/8504) | Fix generated address returned by Substrate RPC runtime call | ✅ Complete | Analyzed | [View Analysis](pr_8504.md) |
| [pr_8528.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8528.prdoc) | [#8528](https://github.com/paritytech/polkadot-sdk/pull/8528) | FeeTracker: remove get_min_fee_factor() | ✅ Complete | Analyzed | [View Analysis](pr_8528.md) |
| [pr_8531.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8531.prdoc) | [#8531](https://github.com/paritytech/polkadot-sdk/pull/8531) | Added OnNewHead to pallet-bridge-parachains | ✅ Complete | Analyzed | [View Analysis](pr_8531.md) |
| [pr_8533.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8533.prdoc) | [#8533](https://github.com/paritytech/polkadot-sdk/pull/8533) | fatxpool: add fallback for ready at light | ✅ Complete | Analyzed | [View Analysis](pr_8533.md) |
| [pr_8535.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8535.prdoc) | [#8535](https://github.com/paritytech/polkadot-sdk/pull/8535) | Make WeightBounds return XcmError to surface failures | ✅ Complete | Analyzed | [View Analysis](pr_8535.md) |
| [pr_8545.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8545.prdoc) | [#8545](https://github.com/paritytech/polkadot-sdk/pull/8545) | [pallet-revive] eth-rpc improved healthcheck | ✅ Complete | Analyzed | [View Analysis](pr_8545.md) |
| [pr_8547.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8547.prdoc) | [#8547](https://github.com/paritytech/polkadot-sdk/pull/8547) | Disable check-runtime-migration | ✅ Complete | Analyzed | [View Analysis](pr_8547.md) |
| [pr_8554.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8554.prdoc) | [#8554](https://github.com/paritytech/polkadot-sdk/pull/8554) | pallet-assets ERC20 precompile | ✅ Complete | Analyzed | [View Analysis](pr_8554.md) |
| [pr_8559.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8559.prdoc) | [#8559](https://github.com/paritytech/polkadot-sdk/pull/8559) | [pallet-revive] rename DepositLimit::Unchecked & minor code cleanup | ✅ Complete | Analyzed | [View Analysis](pr_8559.md) |
| [pr_8584.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8584.prdoc) | [#8584](https://github.com/paritytech/polkadot-sdk/pull/8584) | Remove all XCM dependencies from pallet-revive | ✅ Complete | Analyzed | [View Analysis](pr_8584.md) |
| [pr_8587.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8587.prdoc) | [#8587](https://github.com/paritytech/polkadot-sdk/pull/8587) | [pallet-revive] make subscription task panic on error | ✅ Complete | Analyzed | [View Analysis](pr_8587.md) |
| [pr_8594.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8594.prdoc) | [#8594](https://github.com/paritytech/polkadot-sdk/pull/8594) | omni-node: fix benchmark pallet to work with --runtime | ✅ Complete | Analyzed | [View Analysis](pr_8594.md) |
| [pr_8599.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8599.prdoc) | [#8599](https://github.com/paritytech/polkadot-sdk/pull/8599) | Snowbridge: Unpaid execution when bridging to Ethereum | ✅ Complete | Analyzed | [View Analysis](pr_8599.md) |
| [pr_8606.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8606.prdoc) | [#8606](https://github.com/paritytech/polkadot-sdk/pull/8606) | Use hashbrown hashmap/hashset in validation context | ✅ Complete | Analyzed | [View Analysis](pr_8606.md) |
| [pr_8630.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8630.prdoc) | [#8630](https://github.com/paritytech/polkadot-sdk/pull/8630) | Broker: Introduce min price and adjust renewals to lower market | ✅ Complete | Analyzed | [View Analysis](pr_8630.md) |
| [pr_8633.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8633.prdoc) | [#8633](https://github.com/paritytech/polkadot-sdk/pull/8633) | Staking (EPMB): update the semantics of elect() and Phase::Extract(N) | ✅ Complete | Analyzed | [View Analysis](pr_8633.md) |
| [pr_8650.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8650.prdoc) | [#8650](https://github.com/paritytech/polkadot-sdk/pull/8650) | litep2p/peerset: Reject non-reserved peers in the reserved-only mode | ✅ Complete | Analyzed | [View Analysis](pr_8650.md) |
| [pr_8652.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8652.prdoc) | [#8652](https://github.com/paritytech/polkadot-sdk/pull/8652) | [pallet-revive] impl_revive_api macro | ✅ Complete | Analyzed | [View Analysis](pr_8652.md) |
| [pr_8662.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8662.prdoc) | [#8662](https://github.com/paritytech/polkadot-sdk/pull/8662) | [pallet-revive] update dry-run logic | ✅ Complete | Analyzed | [View Analysis](pr_8662.md) |
| [pr_8664.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8664.prdoc) | [#8664](https://github.com/paritytech/polkadot-sdk/pull/8664) | [pallet-revive] Fix rpc-types | ✅ Complete | Analyzed | [View Analysis](pr_8664.md) |
| [pr_8667.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8667.prdoc) | [#8667](https://github.com/paritytech/polkadot-sdk/pull/8667) | revive: Simplify the storage meter | ✅ Complete | Analyzed | [View Analysis](pr_8667.md) |
| [pr_8669.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8669.prdoc) | [#8669](https://github.com/paritytech/polkadot-sdk/pull/8669) | cumulus-aura: Improve equivocation checks | ✅ Complete | Analyzed | [View Analysis](pr_8669.md) |
| [pr_8679.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8679.prdoc) | [#8679](https://github.com/paritytech/polkadot-sdk/pull/8679) | Shared Add ethereum-standards crate | ✅ Complete | Analyzed | [View Analysis](pr_8679.md) |
| [pr_8687.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8687.prdoc) | [#8687](https://github.com/paritytech/polkadot-sdk/pull/8687) | Staking (EPMB): Add defensive error handling to voter snapshot creation and solution verification | ✅ Complete | Analyzed | [View Analysis](pr_8687.md) |
| [pr_8688.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8688.prdoc) | [#8688](https://github.com/paritytech/polkadot-sdk/pull/8688) | bound trusted local cache to shared limits sizes | ✅ Complete | Analyzed | [View Analysis](pr_8688.md) |
| [pr_8700.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8700.prdoc) | [#8700](https://github.com/paritytech/polkadot-sdk/pull/8700) | transfer_assets benchmarking and weights for people chains | ✅ Complete | Analyzed | [View Analysis](pr_8700.md) |
| [pr_8702.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8702.prdoc) | [#8702](https://github.com/paritytech/polkadot-sdk/pull/8702) | [AHM] Relax the requirement for RC-Client to receive +1 session reports | ✅ Complete | Analyzed | [View Analysis](pr_8702.md) |
| [pr_8704.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8704.prdoc) | [#8704](https://github.com/paritytech/polkadot-sdk/pull/8704) | [AHM] Repot the weights of epmb pallet to expose kusama and polkadot weights | ✅ Complete | Analyzed | [View Analysis](pr_8704.md) |
| [pr_8708.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8708.prdoc) | [#8708](https://github.com/paritytech/polkadot-sdk/pull/8708) | feat: add collator peer ID to ParachainInherentData | ✅ Complete | Analyzed | [View Analysis](pr_8708.md) |
| [pr_8715.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8715.prdoc) | [#8715](https://github.com/paritytech/polkadot-sdk/pull/8715) | [AHM] Prepare For Westend Cleanup | ✅ Complete | Analyzed | [View Analysis](pr_8715.md) |
| [pr_8718.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8718.prdoc) | [#8718](https://github.com/paritytech/polkadot-sdk/pull/8718) | Record ed as part of the storage deposit | ✅ Complete | Analyzed | [View Analysis](pr_8718.md) |
| [pr_8724.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8724.prdoc) | [#8724](https://github.com/paritytech/polkadot-sdk/pull/8724) | Implement detailed logging for XCM failures | ✅ Complete | Analyzed | [View Analysis](pr_8724.md) |
| [pr_8725.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8725.prdoc) | [#8725](https://github.com/paritytech/polkadot-sdk/pull/8725) | Snowbridge: register polkadot native asset with fee | ✅ Complete | Analyzed | [View Analysis](pr_8725.md) |
| [pr_8734.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8734.prdoc) | [#8734](https://github.com/paritytech/polkadot-sdk/pull/8734) | [pallet-revive] contract's nonce starts at 1 | ✅ Complete | Analyzed | [View Analysis](pr_8734.md) |
| [pr_8745.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8745.prdoc) | [#8745](https://github.com/paritytech/polkadot-sdk/pull/8745) | Use correct relay parent offset in YAP parachain | ✅ Complete | Analyzed | [View Analysis](pr_8745.md) |
| [pr_8750.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8750.prdoc) | [#8750](https://github.com/paritytech/polkadot-sdk/pull/8750) | Move Transaction depth limit checks | ✅ Complete | Analyzed | [View Analysis](pr_8750.md) |
| [pr_8860.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8860.prdoc) | [#8860](https://github.com/paritytech/polkadot-sdk/pull/8860) | XCMP and DMP improvements | ✅ Complete | Analyzed | [View Analysis](pr_8860.md) |
| [pr_8891.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_8891.prdoc) | [#8891](https://github.com/paritytech/polkadot-sdk/pull/8891) | RuntimeAllocator: Align returned pointers | ✅ Complete | Analyzed | [View Analysis](pr_8891.md) |
| [pr_9094.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_9094.prdoc) | [#9094](https://github.com/paritytech/polkadot-sdk/pull/9094) | bitfield_distribution: fix subsystem clogged at begining of a session | ✅ Complete | Analyzed | [View Analysis](pr_9094.md) |
| [pr_9102.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_9102.prdoc) | [#9102](https://github.com/paritytech/polkadot-sdk/pull/9102) | polkadot-omni-node: pass timestamp inherent data for block import | ✅ Complete | Analyzed | [View Analysis](pr_9102.md) |
| [pr_9127.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_9127.prdoc) | [#9127](https://github.com/paritytech/polkadot-sdk/pull/9127) | add block hashes to the randomness used by hashmaps and friends in validation | ✅ Complete | Analyzed | [View Analysis](pr_9127.md) |
| [pr_9137.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_9137.prdoc) | [#9137](https://github.com/paritytech/polkadot-sdk/pull/9137) | Pallet XCM - transfer_assets pre-ahm patch | ✅ Complete | Analyzed | [View Analysis](pr_9137.md) |
| [pr_9139.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_9139.prdoc) | [#9139](https://github.com/paritytech/polkadot-sdk/pull/9139) | Expose more constants for pallet-xcm | ✅ Complete | Analyzed | [View Analysis](pr_9139.md) |
| [pr_9202.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_9202.prdoc) | [#9202](https://github.com/paritytech/polkadot-sdk/pull/9202) | apply_authorized_force_set_current_code does not need to consume the whole block | ✅ Complete | Analyzed | [View Analysis](pr_9202.md) |
| [pr_9264.prdoc](/Users/manuelmauro/.substrate-mcp/moonbeam/releases/stable2506/pr-docs/pr_9264.prdoc) | [#9264](https://github.com/paritytech/polkadot-sdk/pull/9264) | gossip-support: make low connectivity message an error | ✅ Complete | Analyzed | [View Analysis](pr_9264.md) |

---

## Analysis Progress

- **Analyzed**: ✅ **134 / 134** (100%)
- **Completion Date**: 2025-10-22
- **Analysis Time**: ~4 batches (77 PRs analyzed in parallel)

---

## Critical PRs Requiring Action

### 🔴 HIGH PRIORITY

| PR | Title | Action Required | Estimated Effort |
|---|---|---|---|
| [#8531](https://github.com/paritytech/polkadot-sdk/pull/8531) | Added OnNewHead to pallet-bridge-parachains | Add `type OnNewHead = ();` to bridge configs | < 5 min |
| [#8708](https://github.com/paritytech/polkadot-sdk/pull/8708) | Add collator peer ID to ParachainInherentData | Add `collator_peer_id: None` field (4 locations) | ~10 min |
| [#8860](https://github.com/paritytech/polkadot-sdk/pull/8860) | XCMP and DMP improvements | Coordinate with relay chain upgrade timing | High complexity |

**Files to Update:**
- `runtime/moonbeam/src/bridge_config.rs` (line 93)
- `runtime/moonriver/src/bridge_config.rs` (line 93)
- `node/service/src/rpc.rs` (line 271)
- `runtime/moonbase/tests/common/mod.rs` (line 411)
- `runtime/moonriver/tests/common/mod.rs` (line 445)
- `runtime/moonbeam/tests/common/mod.rs` (line 445)

---

## Medium Priority PRs

| PR | Title | Impact | Action |
|---|---|---|---|
| [#8535](https://github.com/paritytech/polkadot-sdk/pull/8535) | WeightBounds error handling | Test mock update | Update `pallets/xcm-transactor/src/mock.rs` |
| [#8594](https://github.com/paritytech/polkadot-sdk/pull/8594) | Benchmarking CLI changes | Major version bump | Verify benchmarking workflow |

---

## Beneficial Changes (Low Priority)

### Performance Improvements
- **#8370**: Reduces unnecessary collator connections
- **#8445**: Fixes sync gap clearing bug
- **#8533**: Improves transaction pool reliability
- **#8606**: Faster validation with hashbrown (~40% read speedup)
- **#8891**: **Critical** u128 alignment fix (affects Balance types)
- **#9094**: Bitfield distribution optimization

### Enhanced Debugging
- **#8724**: Detailed XCM failure logging
- **#8001**: Structured transaction pool logging

### Metadata Improvements
- **#8443**: Stabilizes V16 metadata format
- **#9139**: Exposes more pallet-xcm constants

---

## No Impact PRs

### pallet-revive Related (23 PRs)
Moonbeam uses `pallet-evm`, not `pallet-revive`:
#8103, #8148, #8197, #8212, #8262, #8273, #8274, #8311, #8545, #8559, #8584, #8587, #8652, #8662, #8664, #8667, #8718, #8734

### Snowbridge Related (5 PRs)
Different bridge architecture:
#8271, #8473, #8599, #8725

### Relay Chain Only (15+ PRs)
Election providers, async staking, broker pallet:
#3811, #6827, #7720, #7833, #7882, #7936, #8127, #8310, #8316, #8339, #8382, #8422, #8630, #8633, #8687, #8702, #8704, #9202

### CI/Test Infrastructure
#7666, #8038, #8387, #8547

---

## Next Steps

1. **Address Critical PRs** - Focus on #8531, #8708, #8860
2. **Test on Moonbase Alpha** - Deploy and verify XCM operations
3. **Run Benchmarks** - Verify #8594 doesn't break workflow
4. **Update TypeScript Bindings** - After upgrade completes
5. **Monitor Performance** - Track improvements from #8606, #8891, etc.

---

## Analysis Metadata

- **Total PRs in Release**: 134
- **Analysis Method**: AI-assisted with manual review required
- **Analysis Files**: Individual reports in `/pr_*.md` files
- **High Priority Issues**: 3 PRs requiring code changes
- **Medium Priority Issues**: 2 PRs requiring verification
- **No Impact**: ~80% of PRs (different pallets/architecture)
