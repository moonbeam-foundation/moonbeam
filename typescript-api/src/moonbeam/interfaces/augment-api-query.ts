// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from "@polkadot/api-base/types";
import type { Data } from "@polkadot/types";
import type {
  BTreeMap,
  Bytes,
  Null,
  Option,
  U256,
  U8aFixed,
  Vec,
  bool,
  u128,
  u16,
  u32,
  u64,
} from "@polkadot/types-codec";
import type { AnyNumber, ITuple } from "@polkadot/types-codec/types";
import type {
  AccountId20,
  Call,
  H160,
  H256,
  Perbill,
  Percent,
  Permill,
} from "@polkadot/types/interfaces/runtime";
import type {
  CumulusPalletDmpQueueConfigData,
  CumulusPalletDmpQueuePageIndexData,
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot,
  CumulusPalletXcmpQueueInboundStatus,
  CumulusPalletXcmpQueueOutboundStatus,
  CumulusPalletXcmpQueueQueueConfigData,
  EthereumBlock,
  EthereumReceiptReceiptV3,
  EthereumTransactionTransactionV2,
  FpRpcTransactionStatus,
  FrameSupportWeightsPerDispatchClassU64,
  FrameSystemAccountInfo,
  FrameSystemEventRecord,
  FrameSystemLastRuntimeUpgradeInfo,
  FrameSystemPhase,
  MoonbeamRuntimeAssetType,
  NimbusPrimitivesNimbusCryptoPublic,
  PalletAssetsApproval,
  PalletAssetsAssetBalance,
  PalletAssetsAssetDetails,
  PalletAssetsAssetMetadata,
  PalletAuthorMappingRegistrationInfo,
  PalletBalancesAccountData,
  PalletBalancesBalanceLock,
  PalletBalancesReleases,
  PalletBalancesReserveData,
  PalletCollectiveVotes,
  PalletCrowdloanRewardsRewardInfo,
  PalletDemocracyPreimageStatus,
  PalletDemocracyReferendumInfo,
  PalletDemocracyReleases,
  PalletDemocracyVoteThreshold,
  PalletDemocracyVoteVoting,
  PalletIdentityRegistrarInfo,
  PalletIdentityRegistration,
  PalletProxyAnnouncement,
  PalletProxyProxyDefinition,
  PalletSchedulerReleases,
  PalletSchedulerScheduledV2,
  PalletTransactionPaymentReleases,
  PalletTreasuryProposal,
  PalletXcmQueryStatus,
  PalletXcmVersionMigrationStage,
  ParachainStakingBond,
  ParachainStakingCandidateMetadata,
  ParachainStakingCollator2,
  ParachainStakingCollatorCandidate,
  ParachainStakingCollatorSnapshot,
  ParachainStakingDelayedPayout,
  ParachainStakingDelegations,
  ParachainStakingDelegator,
  ParachainStakingExitQ,
  ParachainStakingInflationInflationInfo,
  ParachainStakingNominator2,
  ParachainStakingParachainBondConfig,
  ParachainStakingRoundInfo,
  ParachainStakingSetOrderedSetBond,
  PolkadotCorePrimitivesOutboundHrmpMessage,
  PolkadotParachainPrimitivesXcmpMessageFormat,
  PolkadotPrimitivesV1AbridgedHostConfiguration,
  PolkadotPrimitivesV1PersistedValidationData,
  PolkadotPrimitivesV1UpgradeRestriction,
  SpRuntimeDigest,
  XcmTransactorRemoteTransactInfoWithMaxWeight,
  XcmV1MultiLocation,
  XcmVersionedMultiLocation,
} from "@polkadot/types/lookup";
import type { Observable } from "@polkadot/types/types";

declare module "@polkadot/api-base/types/storage" {
  export interface AugmentedQueries<ApiType extends ApiTypes> {
    assetManager: {
      /**
       * Mapping from an asset id to asset type. This is mostly used when
       * receiving transaction specifying an asset directly, like transferring
       * an asset from this chain to another.
       */
      assetIdType: AugmentedQuery<
        ApiType,
        (arg: u128 | AnyNumber | Uint8Array) => Observable<Option<MoonbeamRuntimeAssetType>>,
        [u128]
      > &
        QueryableStorageEntry<ApiType, [u128]>;
      /**
       * Reverse mapping of AssetIdType. Mapping from an asset type to an asset
       * id. This is mostly used when receiving a multilocation XCM message to
       * retrieve the corresponding asset in which tokens should me minted.
       */
      assetTypeId: AugmentedQuery<
        ApiType,
        (
          arg: MoonbeamRuntimeAssetType | { Xcm: any } | string | Uint8Array
        ) => Observable<Option<u128>>,
        [MoonbeamRuntimeAssetType]
      > &
        QueryableStorageEntry<ApiType, [MoonbeamRuntimeAssetType]>;
      /**
       * Stores the units per second for local execution for a AssetType. This
       * is used to know how to charge for XCM execution in a particular asset
       * Not all assets might contain units per second, hence the different storage
       */
      assetTypeUnitsPerSecond: AugmentedQuery<
        ApiType,
        (
          arg: MoonbeamRuntimeAssetType | { Xcm: any } | string | Uint8Array
        ) => Observable<Option<u128>>,
        [MoonbeamRuntimeAssetType]
      > &
        QueryableStorageEntry<ApiType, [MoonbeamRuntimeAssetType]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    assets: {
      /**
       * The number of units of assets held by any given account.
       */
      account: AugmentedQuery<
        ApiType,
        (
          arg1: u128 | AnyNumber | Uint8Array,
          arg2: AccountId20 | string | Uint8Array
        ) => Observable<PalletAssetsAssetBalance>,
        [u128, AccountId20]
      > &
        QueryableStorageEntry<ApiType, [u128, AccountId20]>;
      /**
       * Approved balance transfers. First balance is the amount approved for
       * transfer. Second is the amount of `T::Currency` reserved for storing
       * this. First key is the asset ID, second key is the owner and third key
       * is the delegate.
       */
      approvals: AugmentedQuery<
        ApiType,
        (
          arg1: u128 | AnyNumber | Uint8Array,
          arg2: AccountId20 | string | Uint8Array,
          arg3: AccountId20 | string | Uint8Array
        ) => Observable<Option<PalletAssetsApproval>>,
        [u128, AccountId20, AccountId20]
      > &
        QueryableStorageEntry<ApiType, [u128, AccountId20, AccountId20]>;
      /**
       * Details of an asset.
       */
      asset: AugmentedQuery<
        ApiType,
        (arg: u128 | AnyNumber | Uint8Array) => Observable<Option<PalletAssetsAssetDetails>>,
        [u128]
      > &
        QueryableStorageEntry<ApiType, [u128]>;
      /**
       * Metadata of an asset.
       */
      metadata: AugmentedQuery<
        ApiType,
        (arg: u128 | AnyNumber | Uint8Array) => Observable<PalletAssetsAssetMetadata>,
        [u128]
      > &
        QueryableStorageEntry<ApiType, [u128]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    authorFilter: {
      /**
       * The percentage of active authors that will be eligible at each height.
       */
      eligibleRatio: AugmentedQuery<ApiType, () => Observable<Percent>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    authorInherent: {
      /**
       * Author of current block.
       */
      author: AugmentedQuery<ApiType, () => Observable<Option<AccountId20>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    authorMapping: {
      /**
       * We maintain a mapping from the NimbusIds used in the consensus layer to
       * the AccountIds runtime (including this staking pallet).
       */
      mappingWithDeposit: AugmentedQuery<
        ApiType,
        (
          arg: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array
        ) => Observable<Option<PalletAuthorMappingRegistrationInfo>>,
        [NimbusPrimitivesNimbusCryptoPublic]
      > &
        QueryableStorageEntry<ApiType, [NimbusPrimitivesNimbusCryptoPublic]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    balances: {
      /**
       * The balance of an account.
       *
       * NOTE: This is only used in the case that this pallet is used to store balances.
       */
      account: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<PalletBalancesAccountData>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Any liquidity locks on some account balances. NOTE: Should only be
       * accessed when setting, changing and freeing a lock.
       */
      locks: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Vec<PalletBalancesBalanceLock>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Named reserves on some account balances.
       */
      reserves: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Vec<PalletBalancesReserveData>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Storage version of the pallet.
       *
       * This is set to v2.0.0 for new networks.
       */
      storageVersion: AugmentedQuery<ApiType, () => Observable<PalletBalancesReleases>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The total units issued in the system.
       */
      totalIssuance: AugmentedQuery<ApiType, () => Observable<u128>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    baseFee: {
      baseFeePerGas: AugmentedQuery<ApiType, () => Observable<U256>, []> &
        QueryableStorageEntry<ApiType, []>;
      elasticity: AugmentedQuery<ApiType, () => Observable<Permill>, []> &
        QueryableStorageEntry<ApiType, []>;
      isActive: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    councilCollective: {
      /**
       * The current members of the collective. This is stored sorted (just by value).
       */
      members: AugmentedQuery<ApiType, () => Observable<Vec<AccountId20>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The prime member that helps determine the default vote behavior in case
       * of absentations.
       */
      prime: AugmentedQuery<ApiType, () => Observable<Option<AccountId20>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Proposals so far.
       */
      proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Actual proposal for a given hash, if it's current.
       */
      proposalOf: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<Option<Call>>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The hashes of the active proposals.
       */
      proposals: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Votes on a given proposal, if it is ongoing.
       */
      voting: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<Option<PalletCollectiveVotes>>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    crowdloanRewards: {
      accountsPayable: AugmentedQuery<
        ApiType,
        (
          arg: AccountId20 | string | Uint8Array
        ) => Observable<Option<PalletCrowdloanRewardsRewardInfo>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      claimedRelayChainIds: AugmentedQuery<
        ApiType,
        (arg: U8aFixed | string | Uint8Array) => Observable<Option<Null>>,
        [U8aFixed]
      > &
        QueryableStorageEntry<ApiType, [U8aFixed]>;
      /**
       * Vesting block height at the initialization of the pallet
       */
      endRelayBlock: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      initialized: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Total initialized amount so far. We store this to make pallet funds ==
       * contributors reward check easier and more efficient
       */
      initializedRewardAmount: AugmentedQuery<ApiType, () => Observable<u128>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Vesting block height at the initialization of the pallet
       */
      initRelayBlock: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Total number of contributors to aid hinting benchmarking
       */
      totalContributors: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      unassociatedContributions: AugmentedQuery<
        ApiType,
        (
          arg: U8aFixed | string | Uint8Array
        ) => Observable<Option<PalletCrowdloanRewardsRewardInfo>>,
        [U8aFixed]
      > &
        QueryableStorageEntry<ApiType, [U8aFixed]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    democracy: {
      /**
       * A record of who vetoed what. Maps proposal hash to a possible existent
       * block number (until when it may not be resubmitted) and who vetoed it.
       */
      blacklist: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<Option<ITuple<[u32, Vec<AccountId20>]>>>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Record of all proposals that have been subject to emergency cancellation.
       */
      cancellations: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<bool>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Those who have locked a deposit.
       *
       * TWOX-NOTE: Safe, as increasing integer keys are safe.
       */
      depositOf: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<ITuple<[Vec<AccountId20>, u128]>>>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * True if the last referendum tabled was submitted externally. False if
       * it was a public proposal.
       */
      lastTabledWasExternal: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Accounts for which there are locks in action which may be removed at
       * some point in the future. The value is the block number at which the
       * lock expires and may be removed.
       *
       * TWOX-NOTE: OK ― `AccountId` is a secure hash.
       */
      locks: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Option<u32>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * The lowest referendum index representing an unbaked referendum. Equal
       * to `ReferendumCount` if there isn't a unbaked referendum.
       */
      lowestUnbaked: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The referendum to be tabled whenever it would be valid to table an
       * external proposal. This happens when a referendum needs to be tabled
       * and one of two conditions are met:
       *
       * - `LastTabledWasExternal` is `false`; or
       * - `PublicProps` is empty.
       */
      nextExternal: AugmentedQuery<
        ApiType,
        () => Observable<Option<ITuple<[H256, PalletDemocracyVoteThreshold]>>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Map of hashes to the proposal preimage, along with who registered it
       * and their deposit. The block number is the block at which it was deposited.
       */
      preimages: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<Option<PalletDemocracyPreimageStatus>>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The number of (public) proposals that have been made so far.
       */
      publicPropCount: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The public proposals. Unsorted. The second item is the proposal's hash.
       */
      publicProps: AugmentedQuery<
        ApiType,
        () => Observable<Vec<ITuple<[u32, H256, AccountId20]>>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The next free referendum index, aka the number of referenda started so far.
       */
      referendumCount: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Information concerning any given referendum.
       *
       * TWOX-NOTE: SAFE as indexes are not under an attacker’s control.
       */
      referendumInfoOf: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletDemocracyReferendumInfo>>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Storage version of the pallet.
       *
       * New networks start with last version.
       */
      storageVersion: AugmentedQuery<
        ApiType,
        () => Observable<Option<PalletDemocracyReleases>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * All votes for a particular voter. We store the balance for the number
       * of votes that we have recorded. The second item is the total amount of
       * delegations, that will be added.
       *
       * TWOX-NOTE: SAFE as `AccountId`s are crypto hashes anyway.
       */
      votingOf: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<PalletDemocracyVoteVoting>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    dmpQueue: {
      /**
       * The configuration.
       */
      configuration: AugmentedQuery<
        ApiType,
        () => Observable<CumulusPalletDmpQueueConfigData>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The overweight messages.
       */
      overweight: AugmentedQuery<
        ApiType,
        (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<ITuple<[u32, Bytes]>>>,
        [u64]
      > &
        QueryableStorageEntry<ApiType, [u64]>;
      /**
       * The page index.
       */
      pageIndex: AugmentedQuery<ApiType, () => Observable<CumulusPalletDmpQueuePageIndexData>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The queue pages.
       */
      pages: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Vec<ITuple<[u32, Bytes]>>>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    ethereum: {
      blockHash: AugmentedQuery<
        ApiType,
        (arg: U256 | AnyNumber | Uint8Array) => Observable<H256>,
        [U256]
      > &
        QueryableStorageEntry<ApiType, [U256]>;
      /**
       * The current Ethereum block.
       */
      currentBlock: AugmentedQuery<ApiType, () => Observable<Option<EthereumBlock>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The current Ethereum receipts.
       */
      currentReceipts: AugmentedQuery<
        ApiType,
        () => Observable<Option<Vec<EthereumReceiptReceiptV3>>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The current transaction statuses.
       */
      currentTransactionStatuses: AugmentedQuery<
        ApiType,
        () => Observable<Option<Vec<FpRpcTransactionStatus>>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Current building block's transactions and receipts.
       */
      pending: AugmentedQuery<
        ApiType,
        () => Observable<
          Vec<
            ITuple<
              [EthereumTransactionTransactionV2, FpRpcTransactionStatus, EthereumReceiptReceiptV3]
            >
          >
        >,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    ethereumChainId: {
      chainId: AugmentedQuery<ApiType, () => Observable<u64>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    evm: {
      accountCodes: AugmentedQuery<
        ApiType,
        (arg: H160 | string | Uint8Array) => Observable<Bytes>,
        [H160]
      > &
        QueryableStorageEntry<ApiType, [H160]>;
      accountStorages: AugmentedQuery<
        ApiType,
        (arg1: H160 | string | Uint8Array, arg2: H256 | string | Uint8Array) => Observable<H256>,
        [H160, H256]
      > &
        QueryableStorageEntry<ApiType, [H160, H256]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    identity: {
      /**
       * Information that is pertinent to identify the entity behind an account.
       *
       * TWOX-NOTE: OK ― `AccountId` is a secure hash.
       */
      identityOf: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Option<PalletIdentityRegistration>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * The set of registrars. Not expected to get very big as can only be
       * added through a special origin (likely a council motion).
       *
       * The index into this can be cast to `RegistrarIndex` to get a valid value.
       */
      registrars: AugmentedQuery<
        ApiType,
        () => Observable<Vec<Option<PalletIdentityRegistrarInfo>>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Alternative "sub" identities of this account.
       *
       * The first item is the deposit, the second is a vector of the accounts.
       *
       * TWOX-NOTE: OK ― `AccountId` is a secure hash.
       */
      subsOf: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<ITuple<[u128, Vec<AccountId20>]>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * The super-identity of an alternative "sub" identity together with its
       * name, within that context. If the account is not some other account's
       * sub-identity, then just `None`.
       */
      superOf: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Option<ITuple<[AccountId20, Data]>>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    maintenanceMode: {
      /**
       * Whether the site is in maintenance mode
       */
      maintenanceMode: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    migrations: {
      /**
       * True if all required migrations have completed
       */
      fullyUpgraded: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * MigrationState tracks the progress of a migration. Maps name (Vec<u8>)
       * -> whether or not migration has been completed (bool)
       */
      migrationState: AugmentedQuery<
        ApiType,
        (arg: Bytes | string | Uint8Array) => Observable<bool>,
        [Bytes]
      > &
        QueryableStorageEntry<ApiType, [Bytes]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    parachainInfo: {
      parachainId: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    parachainStaking: {
      /**
       * Snapshot of collator delegation stake at the start of the round
       */
      atStake: AugmentedQuery<
        ApiType,
        (
          arg1: u32 | AnyNumber | Uint8Array,
          arg2: AccountId20 | string | Uint8Array
        ) => Observable<ParachainStakingCollatorSnapshot>,
        [u32, AccountId20]
      > &
        QueryableStorageEntry<ApiType, [u32, AccountId20]>;
      /**
       * Points for each collator per round
       */
      awardedPts: AugmentedQuery<
        ApiType,
        (
          arg1: u32 | AnyNumber | Uint8Array,
          arg2: AccountId20 | string | Uint8Array
        ) => Observable<u32>,
        [u32, AccountId20]
      > &
        QueryableStorageEntry<ApiType, [u32, AccountId20]>;
      /**
       * Bottom delegations for collator candidate
       */
      bottomDelegations: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Option<ParachainStakingDelegations>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Get collator candidate info associated with an account if account is
       * candidate else None
       */
      candidateInfo: AugmentedQuery<
        ApiType,
        (
          arg: AccountId20 | string | Uint8Array
        ) => Observable<Option<ParachainStakingCandidateMetadata>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * The pool of collator candidates, each with their total backing stake
       */
      candidatePool: AugmentedQuery<ApiType, () => Observable<Vec<ParachainStakingBond>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * DEPRECATED Get collator candidate state associated with an account if
       * account is a candidate else None
       */
      candidateState: AugmentedQuery<
        ApiType,
        (
          arg: AccountId20 | string | Uint8Array
        ) => Observable<Option<ParachainStakingCollatorCandidate>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Commission percent taken off of rewards for all collators
       */
      collatorCommission: AugmentedQuery<ApiType, () => Observable<Perbill>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * DEPRECATED in favor of CandidateState Get collator state associated
       * with an account if account is collating else None
       */
      collatorState2: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Option<ParachainStakingCollator2>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Delayed payouts
       */
      delayedPayouts: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<ParachainStakingDelayedPayout>>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Get delegator state associated with an account if account is delegating else None
       */
      delegatorState: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Option<ParachainStakingDelegator>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * DEPRECATED, to be removed in future runtime upgrade but necessary for
       * runtime migration A queue of collators and nominators awaiting exit
       */
      exitQueue2: AugmentedQuery<ApiType, () => Observable<ParachainStakingExitQ>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Inflation configuration
       */
      inflationConfig: AugmentedQuery<
        ApiType,
        () => Observable<ParachainStakingInflationInflationInfo>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * DEPRECATED in favor of DelegatorState Get nominator state associated
       * with an account if account is nominating else None
       */
      nominatorState2: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Option<ParachainStakingNominator2>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Parachain bond config info { account, percent_of_inflation }
       */
      parachainBondInfo: AugmentedQuery<
        ApiType,
        () => Observable<ParachainStakingParachainBondConfig>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Total points awarded to collators for block production in the round
       */
      points: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<u32>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Current round index and next round scheduled transition
       */
      round: AugmentedQuery<ApiType, () => Observable<ParachainStakingRoundInfo>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The collator candidates selected for the current round
       */
      selectedCandidates: AugmentedQuery<ApiType, () => Observable<Vec<AccountId20>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Total counted stake for selected candidates in the round
       */
      staked: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<u128>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Top delegations for collator candidate
       */
      topDelegations: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<Option<ParachainStakingDelegations>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Total capital locked by this staking pallet
       */
      total: AugmentedQuery<ApiType, () => Observable<u128>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The total candidates selected every round
       */
      totalSelected: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    parachainSystem: {
      /**
       * The number of HRMP messages we observed in `on_initialize` and thus
       * used that number for announcing the weight of `on_initialize` and `on_finalize`.
       */
      announcedHrmpMessagesPerCandidate: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The next authorized upgrade, if there is one.
       */
      authorizedUpgrade: AugmentedQuery<ApiType, () => Observable<Option<H256>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Were the validation data set to notify the relay chain?
       */
      didSetValidationCode: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The parachain host configuration that was obtained from the relay parent.
       *
       * This field is meant to be updated each block with the validation data
       * inherent. Therefore, before processing of the inherent, e.g. in
       * `on_initialize` this data may be stale.
       *
       * This data is also absent from the genesis.
       */
      hostConfiguration: AugmentedQuery<
        ApiType,
        () => Observable<Option<PolkadotPrimitivesV1AbridgedHostConfiguration>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * HRMP messages that were sent in a block.
       *
       * This will be cleared in `on_initialize` of each new block.
       */
      hrmpOutboundMessages: AugmentedQuery<
        ApiType,
        () => Observable<Vec<PolkadotCorePrimitivesOutboundHrmpMessage>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * HRMP watermark that was set in a block.
       *
       * This will be cleared in `on_initialize` of each new block.
       */
      hrmpWatermark: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The last downward message queue chain head we have observed.
       *
       * This value is loaded before and saved after processing inbound downward
       * messages carried by the system inherent.
       */
      lastDmqMqcHead: AugmentedQuery<ApiType, () => Observable<H256>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The message queue chain heads we have observed per each channel incoming channel.
       *
       * This value is loaded before and saved after processing inbound downward
       * messages carried by the system inherent.
       */
      lastHrmpMqcHeads: AugmentedQuery<ApiType, () => Observable<BTreeMap<u32, H256>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Validation code that is set by the parachain and is to be communicated
       * to collator and consequently the relay-chain.
       *
       * This will be cleared in `on_initialize` of each new block if no other
       * pallet already set the value.
       */
      newValidationCode: AugmentedQuery<ApiType, () => Observable<Option<Bytes>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Upward messages that are still pending and not yet send to the relay chain.
       */
      pendingUpwardMessages: AugmentedQuery<ApiType, () => Observable<Vec<Bytes>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * In case of a scheduled upgrade, this storage field contains the
       * validation code to be applied.
       *
       * As soon as the relay chain gives us the go-ahead signal, we will
       * overwrite the `:code` which will result the next block process with the
       * new validation code. This concludes the upgrade process.
       */
      pendingValidationCode: AugmentedQuery<ApiType, () => Observable<Bytes>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Number of downward messages processed in a block.
       *
       * This will be cleared in `on_initialize` of each new block.
       */
      processedDownwardMessages: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The snapshot of some state related to messaging relevant to the current
       * parachain as per the relay parent.
       *
       * This field is meant to be updated each block with the validation data
       * inherent. Therefore, before processing of the inherent, e.g. in
       * `on_initialize` this data may be stale.
       *
       * This data is also absent from the genesis.
       */
      relevantMessagingState: AugmentedQuery<
        ApiType,
        () => Observable<
          Option<CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot>
        >,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The weight we reserve at the beginning of the block for processing DMP
       * messages. This overrides the amount set in the Config trait.
       */
      reservedDmpWeightOverride: AugmentedQuery<ApiType, () => Observable<Option<u64>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The weight we reserve at the beginning of the block for processing XCMP
       * messages. This overrides the amount set in the Config trait.
       */
      reservedXcmpWeightOverride: AugmentedQuery<ApiType, () => Observable<Option<u64>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * An option which indicates if the relay-chain restricts signalling a
       * validation code upgrade. In other words, if this is `Some` and
       * [`NewValidationCode`] is `Some` then the produced candidate will be invalid.
       *
       * This storage item is a mirror of the corresponding value for the
       * current parachain from the relay-chain. This value is ephemeral which
       * means it doesn't hit the storage. This value is set after the inherent.
       */
      upgradeRestrictionSignal: AugmentedQuery<
        ApiType,
        () => Observable<Option<PolkadotPrimitivesV1UpgradeRestriction>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Upward messages that were sent in a block.
       *
       * This will be cleared in `on_initialize` of each new block.
       */
      upwardMessages: AugmentedQuery<ApiType, () => Observable<Vec<Bytes>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The [`PersistedValidationData`] set for this block. This value is
       * expected to be set only once per block and it's never stored in the trie.
       */
      validationData: AugmentedQuery<
        ApiType,
        () => Observable<Option<PolkadotPrimitivesV1PersistedValidationData>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    polkadotXcm: {
      /**
       * The existing asset traps.
       *
       * Key is the blake2 256 hash of (origin, versioned `MultiAssets`) pair.
       * Value is the number of times this pair has been trapped (usually just 1
       * if it exists at all).
       */
      assetTraps: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<u32>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The current migration's stage, if any.
       */
      currentMigration: AugmentedQuery<
        ApiType,
        () => Observable<Option<PalletXcmVersionMigrationStage>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The ongoing queries.
       */
      queries: AugmentedQuery<
        ApiType,
        (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<PalletXcmQueryStatus>>,
        [u64]
      > &
        QueryableStorageEntry<ApiType, [u64]>;
      /**
       * The latest available query index.
       */
      queryCounter: AugmentedQuery<ApiType, () => Observable<u64>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Default version to encode XCM when latest version of destination is
       * unknown. If `None`, then the destinations whose XCM version is unknown
       * are considered unreachable.
       */
      safeXcmVersion: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The Latest versions that we know various locations support.
       */
      supportedVersion: AugmentedQuery<
        ApiType,
        (
          arg1: u32 | AnyNumber | Uint8Array,
          arg2: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array
        ) => Observable<Option<u32>>,
        [u32, XcmVersionedMultiLocation]
      > &
        QueryableStorageEntry<ApiType, [u32, XcmVersionedMultiLocation]>;
      /**
       * Destinations whose latest XCM version we would like to know. Duplicates
       * not allowed, and the `u32` counter is the number of times that a send
       * to the destination has been attempted, which is used as a prioritization.
       */
      versionDiscoveryQueue: AugmentedQuery<
        ApiType,
        () => Observable<Vec<ITuple<[XcmVersionedMultiLocation, u32]>>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * All locations that we have requested version notifications from.
       */
      versionNotifiers: AugmentedQuery<
        ApiType,
        (
          arg1: u32 | AnyNumber | Uint8Array,
          arg2: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array
        ) => Observable<Option<u64>>,
        [u32, XcmVersionedMultiLocation]
      > &
        QueryableStorageEntry<ApiType, [u32, XcmVersionedMultiLocation]>;
      /**
       * The target locations that are subscribed to our version changes, as
       * well as the most recent of our versions we informed them of.
       */
      versionNotifyTargets: AugmentedQuery<
        ApiType,
        (
          arg1: u32 | AnyNumber | Uint8Array,
          arg2: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array
        ) => Observable<Option<ITuple<[u64, u64, u32]>>>,
        [u32, XcmVersionedMultiLocation]
      > &
        QueryableStorageEntry<ApiType, [u32, XcmVersionedMultiLocation]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    proxy: {
      /**
       * The announcements made by the proxy (key).
       */
      announcements: AugmentedQuery<
        ApiType,
        (
          arg: AccountId20 | string | Uint8Array
        ) => Observable<ITuple<[Vec<PalletProxyAnnouncement>, u128]>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * The set of account proxies. Maps the account which has delegated to the
       * accounts which are being delegated to, together with the amount held on deposit.
       */
      proxies: AugmentedQuery<
        ApiType,
        (
          arg: AccountId20 | string | Uint8Array
        ) => Observable<ITuple<[Vec<PalletProxyProxyDefinition>, u128]>>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    randomnessCollectiveFlip: {
      /**
       * Series of block headers from the last 81 blocks that acts as random
       * seed material. This is arranged as a ring buffer with `block_number %
       * 81` being the index into the `Vec` of the oldest hash.
       */
      randomMaterial: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    scheduler: {
      /**
       * Items to be executed, indexed by the block number that they should be
       * executed on.
       */
      agenda: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Vec<Option<PalletSchedulerScheduledV2>>>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Lookup from identity to the block number and index of the task.
       */
      lookup: AugmentedQuery<
        ApiType,
        (arg: Bytes | string | Uint8Array) => Observable<Option<ITuple<[u32, u32]>>>,
        [Bytes]
      > &
        QueryableStorageEntry<ApiType, [Bytes]>;
      /**
       * Storage version of the pallet.
       *
       * New networks start with last version.
       */
      storageVersion: AugmentedQuery<ApiType, () => Observable<PalletSchedulerReleases>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    system: {
      /**
       * The full account information for a particular account ID.
       */
      account: AugmentedQuery<
        ApiType,
        (arg: AccountId20 | string | Uint8Array) => Observable<FrameSystemAccountInfo>,
        [AccountId20]
      > &
        QueryableStorageEntry<ApiType, [AccountId20]>;
      /**
       * Total length (in bytes) for all extrinsics put together, for the current block.
       */
      allExtrinsicsLen: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Map of block numbers to block hashes.
       */
      blockHash: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<H256>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * The current weight for the block.
       */
      blockWeight: AugmentedQuery<
        ApiType,
        () => Observable<FrameSupportWeightsPerDispatchClassU64>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Digest of the current block, also part of the block header.
       */
      digest: AugmentedQuery<ApiType, () => Observable<SpRuntimeDigest>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The number of events in the `Events<T>` list.
       */
      eventCount: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Events deposited for the current block.
       *
       * NOTE: This storage item is explicitly unbounded since it is never
       * intended to be read from within the runtime.
       */
      events: AugmentedQuery<ApiType, () => Observable<Vec<FrameSystemEventRecord>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Mapping between a topic (represented by T::Hash) and a vector of
       * indexes of events in the `<Events<T>>` list.
       *
       * All topic vectors have deterministic storage locations depending on the
       * topic. This allows light-clients to leverage the changes trie storage
       * tracking mechanism and in case of changes fetch the list of events of interest.
       *
       * The value has the type `(T::BlockNumber, EventIndex)` because if we
       * used only just the `EventIndex` then in case if the topic has the same
       * contents on the next block no notification will be triggered thus the
       * event might be lost.
       */
      eventTopics: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<Vec<ITuple<[u32, u32]>>>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The execution phase of the block.
       */
      executionPhase: AugmentedQuery<ApiType, () => Observable<Option<FrameSystemPhase>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Total extrinsics count for the current block.
       */
      extrinsicCount: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Extrinsics data for the current block (maps an extrinsic's index to its data).
       */
      extrinsicData: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Bytes>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Stores the `spec_version` and `spec_name` of when the last runtime
       * upgrade happened.
       */
      lastRuntimeUpgrade: AugmentedQuery<
        ApiType,
        () => Observable<Option<FrameSystemLastRuntimeUpgradeInfo>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The current block number being processed. Set by `execute_block`.
       */
      number: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Hash of the previous block.
       */
      parentHash: AugmentedQuery<ApiType, () => Observable<H256>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * True if we have upgraded so that AccountInfo contains three types of
       * `RefCount`. False (default) if not.
       */
      upgradedToTripleRefCount: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * True if we have upgraded so that `type RefCount` is `u32`. False
       * (default) if not.
       */
      upgradedToU32RefCount: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    techCommitteeCollective: {
      /**
       * The current members of the collective. This is stored sorted (just by value).
       */
      members: AugmentedQuery<ApiType, () => Observable<Vec<AccountId20>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The prime member that helps determine the default vote behavior in case
       * of absentations.
       */
      prime: AugmentedQuery<ApiType, () => Observable<Option<AccountId20>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Proposals so far.
       */
      proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Actual proposal for a given hash, if it's current.
       */
      proposalOf: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<Option<Call>>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The hashes of the active proposals.
       */
      proposals: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Votes on a given proposal, if it is ongoing.
       */
      voting: AugmentedQuery<
        ApiType,
        (arg: H256 | string | Uint8Array) => Observable<Option<PalletCollectiveVotes>>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    timestamp: {
      /**
       * Did the timestamp get updated in this block?
       */
      didUpdate: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Current time for the current block.
       */
      now: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    transactionPayment: {
      nextFeeMultiplier: AugmentedQuery<ApiType, () => Observable<u128>, []> &
        QueryableStorageEntry<ApiType, []>;
      storageVersion: AugmentedQuery<
        ApiType,
        () => Observable<PalletTransactionPaymentReleases>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    treasury: {
      /**
       * Proposal indices that have been approved but not yet awarded.
       */
      approvals: AugmentedQuery<ApiType, () => Observable<Vec<u32>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Number of proposals that have been made.
       */
      proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Proposals that have been made.
       */
      proposals: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletTreasuryProposal>>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    xcmpQueue: {
      /**
       * Inbound aggregate XCMP messages. It can only be one per ParaId/block.
       */
      inboundXcmpMessages: AugmentedQuery<
        ApiType,
        (
          arg1: u32 | AnyNumber | Uint8Array,
          arg2: u32 | AnyNumber | Uint8Array
        ) => Observable<Bytes>,
        [u32, u32]
      > &
        QueryableStorageEntry<ApiType, [u32, u32]>;
      /**
       * Status of the inbound XCMP channels.
       */
      inboundXcmpStatus: AugmentedQuery<
        ApiType,
        () => Observable<
          Vec<
            ITuple<
              [
                u32,
                CumulusPalletXcmpQueueInboundStatus,
                Vec<ITuple<[u32, PolkadotParachainPrimitivesXcmpMessageFormat]>>
              ]
            >
          >
        >,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The messages outbound in a given XCMP channel.
       */
      outboundXcmpMessages: AugmentedQuery<
        ApiType,
        (
          arg1: u32 | AnyNumber | Uint8Array,
          arg2: u16 | AnyNumber | Uint8Array
        ) => Observable<Bytes>,
        [u32, u16]
      > &
        QueryableStorageEntry<ApiType, [u32, u16]>;
      /**
       * The non-empty XCMP channels in order of becoming non-empty, and the
       * index of the first and last outbound message. If the two indices are
       * equal, then it indicates an empty queue and there must be a non-`Ok`
       * `OutboundStatus`. We assume queues grow no greater than 65535 items.
       * Queue indices for normal messages begin at one; zero is reserved in
       * case of the need to send a high-priority signal message this block. The
       * bool is true if there is a signal message waiting to be sent.
       */
      outboundXcmpStatus: AugmentedQuery<
        ApiType,
        () => Observable<Vec<ITuple<[u32, CumulusPalletXcmpQueueOutboundStatus, bool, u16, u16]>>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The configuration which controls the dynamics of the outbound queue.
       */
      queueConfig: AugmentedQuery<
        ApiType,
        () => Observable<CumulusPalletXcmpQueueQueueConfigData>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Any signal messages waiting to be sent.
       */
      signalMessages: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Bytes>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    xcmTransactor: {
      indexToAccount: AugmentedQuery<
        ApiType,
        (arg: u16 | AnyNumber | Uint8Array) => Observable<Option<AccountId20>>,
        [u16]
      > &
        QueryableStorageEntry<ApiType, [u16]>;
      transactInfoWithWeightLimit: AugmentedQuery<
        ApiType,
        (
          arg: XcmV1MultiLocation | { parents?: any; interior?: any } | string | Uint8Array
        ) => Observable<Option<XcmTransactorRemoteTransactInfoWithMaxWeight>>,
        [XcmV1MultiLocation]
      > &
        QueryableStorageEntry<ApiType, [XcmV1MultiLocation]>;
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    xTokens: {
      /**
       * Generic query
       */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
  } // AugmentedQueries
} // declare module
