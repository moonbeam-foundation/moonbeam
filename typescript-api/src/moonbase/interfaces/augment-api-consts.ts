// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/consts";

import type { ApiTypes, AugmentedConst } from "@polkadot/api-base/types";
import type { Bytes, Option, Vec, bool, u128, u16, u32, u64, u8 } from "@polkadot/types-codec";
import type { Codec, ITuple } from "@polkadot/types-codec/types";
import type { Perbill, Permill } from "@polkadot/types/interfaces/runtime";
import type {
  FrameSupportPalletId,
  FrameSystemLimitsBlockLength,
  FrameSystemLimitsBlockWeights,
  PalletReferendaTrackInfo,
  SpVersionRuntimeVersion,
  SpWeightsRuntimeDbWeight,
  SpWeightsWeightV2Weight,
  StagingXcmV3MultiLocation,
} from "@polkadot/types/lookup";

export type __AugmentedConst<ApiType extends ApiTypes> = AugmentedConst<ApiType>;

declare module "@polkadot/api-base/types/consts" {
  interface AugmentedConsts<ApiType extends ApiTypes> {
    assetManager: {
      /** The basic amount of funds that must be reserved for a local asset. */
      localAssetDeposit: u128 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    assets: {
      /** The amount of funds that must be reserved when creating a new approval. */
      approvalDeposit: u128 & AugmentedConst<ApiType>;
      /** The amount of funds that must be reserved for a non-provider asset account to be maintained. */
      assetAccountDeposit: u128 & AugmentedConst<ApiType>;
      /** The basic amount of funds that must be reserved for an asset. */
      assetDeposit: u128 & AugmentedConst<ApiType>;
      /** The basic amount of funds that must be reserved when adding metadata to your asset. */
      metadataDepositBase: u128 & AugmentedConst<ApiType>;
      /** The additional funds that must be reserved for the number of bytes you store in your metadata. */
      metadataDepositPerByte: u128 & AugmentedConst<ApiType>;
      /**
       * Max number of items to destroy per `destroy_accounts` and `destroy_approvals` call.
       *
       * Must be configured to result in a weight that makes each call fit in a block.
       */
      removeItemsLimit: u32 & AugmentedConst<ApiType>;
      /** The maximum length of a name or symbol stored on-chain. */
      stringLimit: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    balances: {
      /**
       * The minimum amount required to keep an account open. MUST BE GREATER THAN ZERO!
       *
       * If you _really_ need it to be zero, you can enable the feature `insecure_zero_ed` for this
       * pallet. However, you do so at your own risk: this will open up a major DoS vector. In case
       * you have multiple sources of provider references, you may also get unexpected behaviour if
       * you set this to zero.
       *
       * Bottom line: Do yourself a favour and make it at least one!
       */
      existentialDeposit: u128 & AugmentedConst<ApiType>;
      /** The maximum number of individual freeze locks that can exist on an account at any time. */
      maxFreezes: u32 & AugmentedConst<ApiType>;
      /** The maximum number of holds that can exist on an account at any time. */
      maxHolds: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of locks that should exist on an account. Not strictly enforced, but
       * used for weight estimation.
       */
      maxLocks: u32 & AugmentedConst<ApiType>;
      /** The maximum number of named reserves that can exist on an account. */
      maxReserves: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    convictionVoting: {
      /**
       * The maximum number of concurrent votes an account may have.
       *
       * Also used to compute weight, an overly large value can lead to extrinsics with large weight
       * estimation: see `delegate` for instance.
       */
      maxVotes: u32 & AugmentedConst<ApiType>;
      /**
       * The minimum period of vote locking.
       *
       * It should be no shorter than enactment period to ensure that in the case of an approval,
       * those successful voters are locked into the consequences that their votes entail.
       */
      voteLockingPeriod: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    councilCollective: {
      /** The maximum weight of a dispatch call that can be proposed and executed. */
      maxProposalWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    crowdloanRewards: {
      /** Percentage to be payed at initialization */
      initializationPayment: Perbill & AugmentedConst<ApiType>;
      maxInitContributors: u32 & AugmentedConst<ApiType>;
      /**
       * A fraction representing the percentage of proofs that need to be presented to change a
       * reward address through the relay keys
       */
      rewardAddressRelayVoteThreshold: Perbill & AugmentedConst<ApiType>;
      /**
       * Network Identifier to be appended into the signatures for reward address change/association
       * Prevents replay attacks from one network to the other
       */
      signatureNetworkIdentifier: Bytes & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    democracy: {
      /** Period in blocks where an external proposal may not be re-submitted after being vetoed. */
      cooloffPeriod: u32 & AugmentedConst<ApiType>;
      /**
       * The period between a proposal being approved and enacted.
       *
       * It should generally be a little more than the unstake period to ensure that voting stakers
       * have an opportunity to remove themselves from the system in the case where they are on the
       * losing side of a vote.
       */
      enactmentPeriod: u32 & AugmentedConst<ApiType>;
      /** Minimum voting period allowed for a fast-track referendum. */
      fastTrackVotingPeriod: u32 & AugmentedConst<ApiType>;
      /**
       * Indicator for whether an emergency origin is even allowed to happen. Some chains may want
       * to set this permanently to `false`, others may want to condition it on things such as an
       * upgrade having happened recently.
       */
      instantAllowed: bool & AugmentedConst<ApiType>;
      /** How often (in blocks) new public referenda are launched. */
      launchPeriod: u32 & AugmentedConst<ApiType>;
      /** The maximum number of items which can be blacklisted. */
      maxBlacklisted: u32 & AugmentedConst<ApiType>;
      /** The maximum number of deposits a public proposal may have at any time. */
      maxDeposits: u32 & AugmentedConst<ApiType>;
      /** The maximum number of public proposals that can exist at any time. */
      maxProposals: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of votes for an account.
       *
       * Also used to compute weight, an overly big value can lead to extrinsic with very big
       * weight: see `delegate` for instance.
       */
      maxVotes: u32 & AugmentedConst<ApiType>;
      /** The minimum amount to be used as a deposit for a public referendum proposal. */
      minimumDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * The minimum period of vote locking.
       *
       * It should be no shorter than enactment period to ensure that in the case of an approval,
       * those successful voters are locked into the consequences that their votes entail.
       */
      voteLockingPeriod: u32 & AugmentedConst<ApiType>;
      /** How often (in blocks) to check for new votes. */
      votingPeriod: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    identity: {
      /** The amount held on deposit for a registered identity */
      basicDeposit: u128 & AugmentedConst<ApiType>;
      /** The amount held on deposit per additional field for a registered identity. */
      fieldDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * Maximum number of additional fields that may be stored in an ID. Needed to bound the I/O
       * required to access an identity, but can be pretty high.
       */
      maxAdditionalFields: u32 & AugmentedConst<ApiType>;
      /**
       * Maxmimum number of registrars allowed in the system. Needed to bound the complexity of,
       * e.g., updating judgements.
       */
      maxRegistrars: u32 & AugmentedConst<ApiType>;
      /** The maximum number of sub-accounts allowed per identified account. */
      maxSubAccounts: u32 & AugmentedConst<ApiType>;
      /**
       * The amount held on deposit for a registered subaccount. This should account for the fact
       * that one storage item's value will increase by the size of an account ID, and there will be
       * another trie item whose value is the size of an account ID plus 32 bytes.
       */
      subAccountDeposit: u128 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    localAssets: {
      /** The amount of funds that must be reserved when creating a new approval. */
      approvalDeposit: u128 & AugmentedConst<ApiType>;
      /** The amount of funds that must be reserved for a non-provider asset account to be maintained. */
      assetAccountDeposit: u128 & AugmentedConst<ApiType>;
      /** The basic amount of funds that must be reserved for an asset. */
      assetDeposit: u128 & AugmentedConst<ApiType>;
      /** The basic amount of funds that must be reserved when adding metadata to your asset. */
      metadataDepositBase: u128 & AugmentedConst<ApiType>;
      /** The additional funds that must be reserved for the number of bytes you store in your metadata. */
      metadataDepositPerByte: u128 & AugmentedConst<ApiType>;
      /**
       * Max number of items to destroy per `destroy_accounts` and `destroy_approvals` call.
       *
       * Must be configured to result in a weight that makes each call fit in a block.
       */
      removeItemsLimit: u32 & AugmentedConst<ApiType>;
      /** The maximum length of a name or symbol stored on-chain. */
      stringLimit: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    moonbeamOrbiters: {
      /** Maximum number of orbiters per collator. */
      maxPoolSize: u32 & AugmentedConst<ApiType>;
      /** Maximum number of round to keep on storage. */
      maxRoundArchive: u32 & AugmentedConst<ApiType>;
      /**
       * Number of rounds before changing the selected orbiter. WARNING: when changing
       * `RotatePeriod`, you need a migration code that sets `ForceRotation` to true to avoid holes
       * in `OrbiterPerRound`.
       */
      rotatePeriod: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    multisig: {
      /**
       * The base amount of currency needed to reserve for creating a multisig execution or to store
       * a dispatch call for later.
       *
       * This is held for an additional storage item whose value size is `4 + sizeof((BlockNumber,
       * Balance, AccountId))` bytes and whose key size is `32 + sizeof(AccountId)` bytes.
       */
      depositBase: u128 & AugmentedConst<ApiType>;
      /**
       * The amount of currency needed per unit threshold when creating a multisig execution.
       *
       * This is held for adding 32 bytes more into a pre-existing storage value.
       */
      depositFactor: u128 & AugmentedConst<ApiType>;
      /** The maximum amount of signatories allowed in the multisig. */
      maxSignatories: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    openTechCommitteeCollective: {
      /** The maximum weight of a dispatch call that can be proposed and executed. */
      maxProposalWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    parachainStaking: {
      /** Number of rounds candidate requests to decrease self-bond must wait to be executable */
      candidateBondLessDelay: u32 & AugmentedConst<ApiType>;
      /** Number of rounds that delegation less requests must wait before executable */
      delegationBondLessDelay: u32 & AugmentedConst<ApiType>;
      /** Number of rounds that candidates remain bonded before exit request is executable */
      leaveCandidatesDelay: u32 & AugmentedConst<ApiType>;
      /** Number of rounds that delegators remain bonded before exit request is executable */
      leaveDelegatorsDelay: u32 & AugmentedConst<ApiType>;
      /** Maximum bottom delegations (not counted) per candidate */
      maxBottomDelegationsPerCandidate: u32 & AugmentedConst<ApiType>;
      /** Maximum candidates */
      maxCandidates: u32 & AugmentedConst<ApiType>;
      /** Maximum delegations per delegator */
      maxDelegationsPerDelegator: u32 & AugmentedConst<ApiType>;
      /**
       * If a collator doesn't produce any block on this number of rounds, it is notified as
       * inactive. This value must be less than or equal to RewardPaymentDelay.
       */
      maxOfflineRounds: u32 & AugmentedConst<ApiType>;
      /** Maximum top delegations counted per candidate */
      maxTopDelegationsPerCandidate: u32 & AugmentedConst<ApiType>;
      /** Minimum number of blocks per round */
      minBlocksPerRound: u32 & AugmentedConst<ApiType>;
      /** Minimum stake required for any account to be a collator candidate */
      minCandidateStk: u128 & AugmentedConst<ApiType>;
      /** Minimum stake for any registered on-chain account to delegate */
      minDelegation: u128 & AugmentedConst<ApiType>;
      /** Minimum number of selected candidates every round */
      minSelectedCandidates: u32 & AugmentedConst<ApiType>;
      /** Number of rounds that delegations remain bonded before revocation request is executable */
      revokeDelegationDelay: u32 & AugmentedConst<ApiType>;
      /** Number of rounds after which block authors are rewarded */
      rewardPaymentDelay: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    proxy: {
      /**
       * The base amount of currency needed to reserve for creating an announcement.
       *
       * This is held when a new storage item holding a `Balance` is created (typically 16 bytes).
       */
      announcementDepositBase: u128 & AugmentedConst<ApiType>;
      /**
       * The amount of currency needed per announcement made.
       *
       * This is held for adding an `AccountId`, `Hash` and `BlockNumber` (typically 68 bytes) into
       * a pre-existing storage value.
       */
      announcementDepositFactor: u128 & AugmentedConst<ApiType>;
      /** The maximum amount of time-delayed announcements that are allowed to be pending. */
      maxPending: u32 & AugmentedConst<ApiType>;
      /** The maximum amount of proxies allowed for a single account. */
      maxProxies: u32 & AugmentedConst<ApiType>;
      /**
       * The base amount of currency needed to reserve for creating a proxy.
       *
       * This is held for an additional storage item whose value size is `sizeof(Balance)` bytes and
       * whose key size is `sizeof(AccountId)` bytes.
       */
      proxyDepositBase: u128 & AugmentedConst<ApiType>;
      /**
       * The amount of currency needed per proxy added.
       *
       * This is held for adding 32 bytes plus an instance of `ProxyType` more into a pre-existing
       * storage value. Thus, when configuring `ProxyDepositFactor` one should take into account `32
       *
       * - Proxy_type.encode().len()` bytes of data.
       */
      proxyDepositFactor: u128 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    randomness: {
      /** Local requests expire and can be purged from storage after this many blocks/epochs */
      blockExpirationDelay: u32 & AugmentedConst<ApiType>;
      /** The amount that should be taken as a security deposit when requesting randomness. */
      deposit: u128 & AugmentedConst<ApiType>;
      /** Babe requests expire and can be purged from storage after this many blocks/epochs */
      epochExpirationDelay: u64 & AugmentedConst<ApiType>;
      /**
       * Local per-block VRF requests must be at most this many blocks after the block in which they
       * were requested
       */
      maxBlockDelay: u32 & AugmentedConst<ApiType>;
      /** Maximum number of random words that can be requested per request */
      maxRandomWords: u8 & AugmentedConst<ApiType>;
      /**
       * Local per-block VRF requests must be at least this many blocks after the block in which
       * they were requested
       */
      minBlockDelay: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    referenda: {
      /**
       * Quantization level for the referendum wakeup scheduler. A higher number will result in
       * fewer storage reads/writes needed for smaller voters, but also result in delays to the
       * automatic referendum status changes. Explicit servicing instructions are unaffected.
       */
      alarmInterval: u32 & AugmentedConst<ApiType>;
      /** Maximum size of the referendum queue for a single track. */
      maxQueued: u32 & AugmentedConst<ApiType>;
      /** The minimum amount to be used as a deposit for a public referendum proposal. */
      submissionDeposit: u128 & AugmentedConst<ApiType>;
      /** Information concerning the different referendum tracks. */
      tracks: Vec<ITuple<[u16, PalletReferendaTrackInfo]>> & AugmentedConst<ApiType>;
      /**
       * The number of blocks after submission that a referendum must begin being decided by. Once
       * this passes, then anyone may cancel the referendum.
       */
      undecidingTimeout: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    scheduler: {
      /** The maximum weight that may be scheduled per block for any dispatchables. */
      maximumWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /**
       * The maximum number of scheduled calls in the queue for a single block.
       *
       * NOTE:
       *
       * - Dependent pallets' benchmarks might require a higher limit for the setting. Set a higher
       *   limit under `runtime-benchmarks` feature.
       */
      maxScheduledPerBlock: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    system: {
      /** Maximum number of block number to block hash mappings to keep (oldest pruned first). */
      blockHashCount: u32 & AugmentedConst<ApiType>;
      /** The maximum length of a block (in bytes). */
      blockLength: FrameSystemLimitsBlockLength & AugmentedConst<ApiType>;
      /** Block & extrinsics weights: base values and limits. */
      blockWeights: FrameSystemLimitsBlockWeights & AugmentedConst<ApiType>;
      /** The weight of runtime database operations the runtime can invoke. */
      dbWeight: SpWeightsRuntimeDbWeight & AugmentedConst<ApiType>;
      /**
       * The designated SS58 prefix of this chain.
       *
       * This replaces the "ss58Format" property declared in the chain spec. Reason is that the
       * runtime should know about the prefix in order to make use of it as an identifier of the chain.
       */
      ss58Prefix: u16 & AugmentedConst<ApiType>;
      /** Get the chain's current version. */
      version: SpVersionRuntimeVersion & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    techCommitteeCollective: {
      /** The maximum weight of a dispatch call that can be proposed and executed. */
      maxProposalWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    timestamp: {
      /**
       * The minimum period between blocks.
       *
       * Be aware that this is different to the _expected_ period that the block production
       * apparatus provides. Your chosen consensus system will generally work with this to determine
       * a sensible block time. For example, in the Aura pallet it will be double this period on
       * default settings.
       */
      minimumPeriod: u64 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    transactionPayment: {
      /**
       * A fee mulitplier for `Operational` extrinsics to compute "virtual tip" to boost their `priority`
       *
       * This value is multipled by the `final_fee` to obtain a "virtual tip" that is later added to
       * a tip component in regular `priority` calculations. It means that a `Normal` transaction
       * can front-run a similarly-sized `Operational` extrinsic (with no tip), by including a tip
       * value greater than the virtual tip.
       *
       * ```rust,ignore
       * // For `Normal`
       * let priority = priority_calc(tip);
       *
       * // For `Operational`
       * let virtual_tip = (inclusion_fee + tip) * OperationalFeeMultiplier;
       * let priority = priority_calc(tip + virtual_tip);
       * ```
       *
       * Note that since we use `final_fee` the multiplier applies also to the regular `tip` sent
       * with the transaction. So, not only does the transaction get a priority bump based on the
       * `inclusion_fee`, but we also amplify the impact of tips applied to `Operational` transactions.
       */
      operationalFeeMultiplier: u8 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    treasury: {
      /** Percentage of spare funds (if any) that are burnt per spend period. */
      burn: Permill & AugmentedConst<ApiType>;
      /**
       * The maximum number of approvals that can wait in the spending queue.
       *
       * NOTE: This parameter is also used within the Bounties Pallet extension if enabled.
       */
      maxApprovals: u32 & AugmentedConst<ApiType>;
      /** The treasury's pallet id, used for deriving its sovereign account ID. */
      palletId: FrameSupportPalletId & AugmentedConst<ApiType>;
      /** The period during which an approved treasury spend has to be claimed. */
      payoutPeriod: u32 & AugmentedConst<ApiType>;
      /**
       * Fraction of a proposal's value that should be bonded in order to place the proposal. An
       * accepted proposal gets these back. A rejected proposal does not.
       */
      proposalBond: Permill & AugmentedConst<ApiType>;
      /** Maximum amount of funds that should be placed in a deposit for making a proposal. */
      proposalBondMaximum: Option<u128> & AugmentedConst<ApiType>;
      /** Minimum amount of funds that should be placed in a deposit for making a proposal. */
      proposalBondMinimum: u128 & AugmentedConst<ApiType>;
      /** Period between successive spends. */
      spendPeriod: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    treasuryCouncilCollective: {
      /** The maximum weight of a dispatch call that can be proposed and executed. */
      maxProposalWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    utility: {
      /** The limit on the number of batched calls. */
      batchedCallsLimit: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    xcmTransactor: {
      /** The actual weight for an XCM message is `T::BaseXcmWeight + T::Weigher::weight(&msg)`. */
      baseXcmWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /** Self chain location. */
      selfLocation: StagingXcmV3MultiLocation & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    xTokens: {
      /**
       * Base XCM weight.
       *
       * The actually weight for an XCM message is `T::BaseXcmWeight + T::Weigher::weight(&msg)`.
       */
      baseXcmWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /** Self chain location. */
      selfLocation: StagingXcmV3MultiLocation & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
  } // AugmentedConsts
} // declare module
