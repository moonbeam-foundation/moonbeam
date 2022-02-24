// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from "@polkadot/api-base/types";
import type {
  Bytes,
  Null,
  Option,
  Result,
  U256,
  U8aFixed,
  Vec,
  bool,
  u128,
  u16,
  u32,
  u64,
  u8,
} from "@polkadot/types-codec";
import type { ITuple } from "@polkadot/types-codec/types";
import type {
  AccountId20,
  H160,
  H256,
  Perbill,
  Percent,
  Permill,
} from "@polkadot/types/interfaces/runtime";
import type {
  EthereumLog,
  EvmCoreErrorExitReason,
  FrameSupportTokensMiscBalanceStatus,
  FrameSupportWeightsDispatchInfo,
  MoonbeamRuntimeAssetRegistrarMetadata,
  MoonbeamRuntimeAssetType,
  MoonbeamRuntimeCurrencyId,
  MoonbeamRuntimeProxyType,
  NimbusPrimitivesNimbusCryptoPublic,
  PalletDemocracyVoteAccountVote,
  PalletDemocracyVoteThreshold,
  ParachainStakingDelegationRequest,
  ParachainStakingDelegatorAdded,
  SpRuntimeDispatchError,
  XcmTransactorRemoteTransactInfoWithMaxWeight,
  XcmV1MultiAsset,
  XcmV1MultiLocation,
  XcmV2Response,
  XcmV2TraitsError,
  XcmV2TraitsOutcome,
  XcmV2Xcm,
  XcmVersionedMultiAssets,
  XcmVersionedMultiLocation,
} from "@polkadot/types/lookup";

declare module "@polkadot/api-base/types/events" {
  export interface AugmentedEvents<ApiType extends ApiTypes> {
    assetManager: {
      AssetRegistered: AugmentedEvent<
        ApiType,
        [u128, MoonbeamRuntimeAssetType, MoonbeamRuntimeAssetRegistrarMetadata]
      >;
      AssetTypeChanged: AugmentedEvent<ApiType, [u128, MoonbeamRuntimeAssetType]>;
      UnitsPerSecondChanged: AugmentedEvent<ApiType, [MoonbeamRuntimeAssetType, u128]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    assets: {
      /**
       * An approval for account `delegate` was cancelled by `owner`.
       */
      ApprovalCancelled: AugmentedEvent<ApiType, [u128, AccountId20, AccountId20]>;
      /**
       * (Additional) funds have been approved for transfer to a destination account.
       */
      ApprovedTransfer: AugmentedEvent<ApiType, [u128, AccountId20, AccountId20, u128]>;
      /**
       * Some asset `asset_id` was frozen.
       */
      AssetFrozen: AugmentedEvent<ApiType, [u128]>;
      /**
       * An asset has had its attributes changed by the `Force` origin.
       */
      AssetStatusChanged: AugmentedEvent<ApiType, [u128]>;
      /**
       * Some asset `asset_id` was thawed.
       */
      AssetThawed: AugmentedEvent<ApiType, [u128]>;
      /**
       * Some assets were destroyed.
       */
      Burned: AugmentedEvent<ApiType, [u128, AccountId20, u128]>;
      /**
       * Some asset class was created.
       */
      Created: AugmentedEvent<ApiType, [u128, AccountId20, AccountId20]>;
      /**
       * An asset class was destroyed.
       */
      Destroyed: AugmentedEvent<ApiType, [u128]>;
      /**
       * Some asset class was force-created.
       */
      ForceCreated: AugmentedEvent<ApiType, [u128, AccountId20]>;
      /**
       * Some account `who` was frozen.
       */
      Frozen: AugmentedEvent<ApiType, [u128, AccountId20]>;
      /**
       * Some assets were issued.
       */
      Issued: AugmentedEvent<ApiType, [u128, AccountId20, u128]>;
      /**
       * Metadata has been cleared for an asset.
       */
      MetadataCleared: AugmentedEvent<ApiType, [u128]>;
      /**
       * New metadata has been set for an asset.
       */
      MetadataSet: AugmentedEvent<ApiType, [u128, Bytes, Bytes, u8, bool]>;
      /**
       * The owner changed.
       */
      OwnerChanged: AugmentedEvent<ApiType, [u128, AccountId20]>;
      /**
       * The management team changed.
       */
      TeamChanged: AugmentedEvent<ApiType, [u128, AccountId20, AccountId20, AccountId20]>;
      /**
       * Some account `who` was thawed.
       */
      Thawed: AugmentedEvent<ApiType, [u128, AccountId20]>;
      /**
       * Some assets were transferred.
       */
      Transferred: AugmentedEvent<ApiType, [u128, AccountId20, AccountId20, u128]>;
      /**
       * An `amount` was transferred in its entirety from `owner` to
       * `destination` by the approved `delegate`.
       */
      TransferredApproved: AugmentedEvent<
        ApiType,
        [u128, AccountId20, AccountId20, AccountId20, u128]
      >;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    authorFilter: {
      /**
       * The amount of eligible authors for the filter to select has been changed.
       */
      EligibleUpdated: AugmentedEvent<ApiType, [Percent]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    authorMapping: {
      /**
       * An NimbusId has been de-registered, and its AccountId mapping removed.
       */
      AuthorDeRegistered: AugmentedEvent<ApiType, [NimbusPrimitivesNimbusCryptoPublic]>;
      /**
       * A NimbusId has been registered and mapped to an AccountId.
       */
      AuthorRegistered: AugmentedEvent<ApiType, [NimbusPrimitivesNimbusCryptoPublic, AccountId20]>;
      /**
       * An NimbusId has been registered, replacing a previous registration and
       * its mapping.
       */
      AuthorRotated: AugmentedEvent<ApiType, [NimbusPrimitivesNimbusCryptoPublic, AccountId20]>;
      /**
       * An NimbusId has been forcibly deregistered after not being rotated or
       * cleaned up. The reporteing account has been rewarded accordingly.
       */
      DefunctAuthorBusted: AugmentedEvent<
        ApiType,
        [NimbusPrimitivesNimbusCryptoPublic, AccountId20]
      >;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    balances: {
      /**
       * A balance was set by root.
       */
      BalanceSet: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Some amount was deposited (e.g. for transaction fees).
       */
      Deposit: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * An account was removed whose balance was non-zero but below
       * ExistentialDeposit, resulting in an outright loss.
       */
      DustLost: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * An account was created with some free balance.
       */
      Endowed: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Some balance was reserved (moved from free to reserved).
       */
      Reserved: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Some balance was moved from the reserve of the first account to the
       * second account. Final argument indicates the destination balance type.
       */
      ReserveRepatriated: AugmentedEvent<
        ApiType,
        [AccountId20, AccountId20, u128, FrameSupportTokensMiscBalanceStatus]
      >;
      /**
       * Some amount was removed from the account (e.g. for misbehavior).
       */
      Slashed: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Transfer succeeded.
       */
      Transfer: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * Some balance was unreserved (moved from reserved to free).
       */
      Unreserved: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Some amount was withdrawn from the account (e.g. for transaction fees).
       */
      Withdraw: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    baseFee: {
      BaseFeeOverflow: AugmentedEvent<ApiType, []>;
      IsActive: AugmentedEvent<ApiType, [bool]>;
      NewBaseFeePerGas: AugmentedEvent<ApiType, [U256]>;
      NewElasticity: AugmentedEvent<ApiType, [Permill]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    councilCollective: {
      /**
       * A motion was approved by the required threshold.
       */
      Approved: AugmentedEvent<ApiType, [H256]>;
      /**
       * A proposal was closed because its threshold was reached or after its
       * duration was up.
       */
      Closed: AugmentedEvent<ApiType, [H256, u32, u32]>;
      /**
       * A motion was not approved by the required threshold.
       */
      Disapproved: AugmentedEvent<ApiType, [H256]>;
      /**
       * A motion was executed; result will be `Ok` if it returned without error.
       */
      Executed: AugmentedEvent<ApiType, [H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A single member did some action; result will be `Ok` if it returned
       * without error.
       */
      MemberExecuted: AugmentedEvent<ApiType, [H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A motion (given hash) has been proposed (by given account) with a
       * threshold (given `MemberCount`).
       */
      Proposed: AugmentedEvent<ApiType, [AccountId20, u32, H256, u32]>;
      /**
       * A motion (given hash) has been voted on by given account, leaving a
       * tally (yes votes and no votes given respectively as `MemberCount`).
       */
      Voted: AugmentedEvent<ApiType, [AccountId20, H256, bool, u32, u32]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    crowdloanRewards: {
      /**
       * When initializing the reward vec an already initialized account was found
       */
      InitializedAccountWithNotEnoughContribution: AugmentedEvent<
        ApiType,
        [U8aFixed, Option<AccountId20>, u128]
      >;
      /**
       * When initializing the reward vec an already initialized account was found
       */
      InitializedAlreadyInitializedAccount: AugmentedEvent<
        ApiType,
        [U8aFixed, Option<AccountId20>, u128]
      >;
      /**
       * The initial payment of InitializationPayment % was paid
       */
      InitialPaymentMade: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Someone has proven they made a contribution and associated a native
       * identity with it. Data is the relay account, native account and the
       * total amount of *rewards* that will be paid
       */
      NativeIdentityAssociated: AugmentedEvent<ApiType, [U8aFixed, AccountId20, u128]>;
      /**
       * A contributor has updated the reward address.
       */
      RewardAddressUpdated: AugmentedEvent<ApiType, [AccountId20, AccountId20]>;
      /**
       * A contributor has claimed some rewards. Data is the account getting
       * paid and the amount of rewards paid.
       */
      RewardsPaid: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    cumulusXcm: {
      /**
       * Downward message executed with the given outcome. [ id, outcome ]
       */
      ExecutedDownward: AugmentedEvent<ApiType, [U8aFixed, XcmV2TraitsOutcome]>;
      /**
       * Downward message is invalid XCM. [ id ]
       */
      InvalidFormat: AugmentedEvent<ApiType, [U8aFixed]>;
      /**
       * Downward message is unsupported version of XCM. [ id ]
       */
      UnsupportedVersion: AugmentedEvent<ApiType, [U8aFixed]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    democracy: {
      /**
       * A proposal_hash has been blacklisted permanently.
       */
      Blacklisted: AugmentedEvent<ApiType, [H256]>;
      /**
       * A referendum has been cancelled.
       */
      Cancelled: AugmentedEvent<ApiType, [u32]>;
      /**
       * An account has delegated their vote to another account.
       */
      Delegated: AugmentedEvent<ApiType, [AccountId20, AccountId20]>;
      /**
       * A proposal has been enacted.
       */
      Executed: AugmentedEvent<ApiType, [u32, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * An external proposal has been tabled.
       */
      ExternalTabled: AugmentedEvent<ApiType, []>;
      /**
       * A proposal has been rejected by referendum.
       */
      NotPassed: AugmentedEvent<ApiType, [u32]>;
      /**
       * A proposal has been approved by referendum.
       */
      Passed: AugmentedEvent<ApiType, [u32]>;
      /**
       * A proposal could not be executed because its preimage was invalid.
       */
      PreimageInvalid: AugmentedEvent<ApiType, [H256, u32]>;
      /**
       * A proposal could not be executed because its preimage was missing.
       */
      PreimageMissing: AugmentedEvent<ApiType, [H256, u32]>;
      /**
       * A proposal's preimage was noted, and the deposit taken.
       */
      PreimageNoted: AugmentedEvent<ApiType, [H256, AccountId20, u128]>;
      /**
       * A registered preimage was removed and the deposit collected by the reaper.
       */
      PreimageReaped: AugmentedEvent<ApiType, [H256, AccountId20, u128, AccountId20]>;
      /**
       * A proposal preimage was removed and used (the deposit was returned).
       */
      PreimageUsed: AugmentedEvent<ApiType, [H256, AccountId20, u128]>;
      /**
       * A motion has been proposed by a public account.
       */
      Proposed: AugmentedEvent<ApiType, [u32, u128]>;
      /**
       * An account has secconded a proposal
       */
      Seconded: AugmentedEvent<ApiType, [AccountId20, u32]>;
      /**
       * A referendum has begun.
       */
      Started: AugmentedEvent<ApiType, [u32, PalletDemocracyVoteThreshold]>;
      /**
       * A public proposal has been tabled for referendum vote.
       */
      Tabled: AugmentedEvent<ApiType, [u32, u128, Vec<AccountId20>]>;
      /**
       * An account has cancelled a previous delegation operation.
       */
      Undelegated: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * An external proposal has been vetoed.
       */
      Vetoed: AugmentedEvent<ApiType, [AccountId20, H256, u32]>;
      /**
       * An account has voted in a referendum
       */
      Voted: AugmentedEvent<ApiType, [AccountId20, u32, PalletDemocracyVoteAccountVote]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    dmpQueue: {
      /**
       * Downward message executed with the given outcome. [ id, outcome ]
       */
      ExecutedDownward: AugmentedEvent<ApiType, [U8aFixed, XcmV2TraitsOutcome]>;
      /**
       * Downward message is invalid XCM. [ id ]
       */
      InvalidFormat: AugmentedEvent<ApiType, [U8aFixed]>;
      /**
       * Downward message is overweight and was placed in the overweight queue.
       * [ id, index, required ]
       */
      OverweightEnqueued: AugmentedEvent<ApiType, [U8aFixed, u64, u64]>;
      /**
       * Downward message from the overweight queue was executed. [ index, used ]
       */
      OverweightServiced: AugmentedEvent<ApiType, [u64, u64]>;
      /**
       * Downward message is unsupported version of XCM. [ id ]
       */
      UnsupportedVersion: AugmentedEvent<ApiType, [U8aFixed]>;
      /**
       * The weight limit for handling downward messages was reached. [ id,
       * remaining, required ]
       */
      WeightExhausted: AugmentedEvent<ApiType, [U8aFixed, u64, u64]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    ethereum: {
      /**
       * An ethereum transaction was successfully executed. [from,
       * to/contract_address, transaction_hash, exit_reason]
       */
      Executed: AugmentedEvent<ApiType, [H160, H160, H256, EvmCoreErrorExitReason]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    evm: {
      /**
       * A deposit has been made at a given address. [sender, address, value]
       */
      BalanceDeposit: AugmentedEvent<ApiType, [AccountId20, H160, U256]>;
      /**
       * A withdrawal has been made from a given address. [sender, address, value]
       */
      BalanceWithdraw: AugmentedEvent<ApiType, [AccountId20, H160, U256]>;
      /**
       * A contract has been created at given [address].
       */
      Created: AugmentedEvent<ApiType, [H160]>;
      /**
       * A [contract] was attempted to be created, but the execution failed.
       */
      CreatedFailed: AugmentedEvent<ApiType, [H160]>;
      /**
       * A [contract] has been executed successfully with states applied.
       */
      Executed: AugmentedEvent<ApiType, [H160]>;
      /**
       * A [contract] has been executed with errors. States are reverted with
       * only gas fees applied.
       */
      ExecutedFailed: AugmentedEvent<ApiType, [H160]>;
      /**
       * Ethereum events from contracts.
       */
      Log: AugmentedEvent<ApiType, [EthereumLog]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    identity: {
      /**
       * A name was cleared, and the given balance returned.
       */
      IdentityCleared: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * A name was removed and the given balance slashed.
       */
      IdentityKilled: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * A name was set or reset (which will remove all judgements).
       */
      IdentitySet: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * A judgement was given by a registrar.
       */
      JudgementGiven: AugmentedEvent<ApiType, [AccountId20, u32]>;
      /**
       * A judgement was asked from a registrar.
       */
      JudgementRequested: AugmentedEvent<ApiType, [AccountId20, u32]>;
      /**
       * A judgement request was retracted.
       */
      JudgementUnrequested: AugmentedEvent<ApiType, [AccountId20, u32]>;
      /**
       * A registrar was added.
       */
      RegistrarAdded: AugmentedEvent<ApiType, [u32]>;
      /**
       * A sub-identity was added to an identity and the deposit paid.
       */
      SubIdentityAdded: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * A sub-identity was removed from an identity and the deposit freed.
       */
      SubIdentityRemoved: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * A sub-identity was cleared, and the given deposit repatriated from the
       * main identity account to the sub-identity account.
       */
      SubIdentityRevoked: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    maintenanceMode: {
      /**
       * The chain was put into Maintenance Mode
       */
      EnteredMaintenanceMode: AugmentedEvent<ApiType, []>;
      /**
       * The chain returned to its normal operating state
       */
      NormalOperationResumed: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    migrations: {
      MigrationCompleted: AugmentedEvent<ApiType, [Bytes, u64]>;
      MigrationStarted: AugmentedEvent<ApiType, [Bytes]>;
      RuntimeUpgradeCompleted: AugmentedEvent<ApiType, [u64]>;
      RuntimeUpgradeStarted: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    parachainStaking: {
      /**
       * Set blocks per round [current_round, first_block, old, new,
       * new_per_round_inflation]
       */
      BlocksPerRoundSet: AugmentedEvent<ApiType, [u32, u32, u32, u32, Perbill, Perbill, Perbill]>;
      /**
       * Candidate, Amount, Round at which could be executed
       */
      CancelledCandidateBondLess: AugmentedEvent<ApiType, [AccountId20, u128, u32]>;
      /**
       * Candidate
       */
      CancelledCandidateExit: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * Delegator, Cancelled Request
       */
      CancelledDelegationRequest: AugmentedEvent<
        ApiType,
        [AccountId20, ParachainStakingDelegationRequest]
      >;
      /**
       * Candidate
       */
      CandidateBackOnline: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * Candidate, Amount, New Bond
       */
      CandidateBondedLess: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Candidate, Amount, New Bond Total
       */
      CandidateBondedMore: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Candidate, Amount To Decrease, Round at which request can be executed by caller
       */
      CandidateBondLessRequested: AugmentedEvent<ApiType, [AccountId20, u128, u32]>;
      /**
       * Ex-Candidate, Amount Unlocked, New Total Amt Locked
       */
      CandidateLeft: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Round At Which Exit Is Allowed, Candidate, Scheduled Exit
       */
      CandidateScheduledExit: AugmentedEvent<ApiType, [u32, AccountId20, u32]>;
      /**
       * Candidate
       */
      CandidateWentOffline: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * Round, Collator Account, Total Exposed Amount (includes all delegations)
       */
      CollatorChosen: AugmentedEvent<ApiType, [u32, AccountId20, u128]>;
      /**
       * Set collator commission to this value [old, new]
       */
      CollatorCommissionSet: AugmentedEvent<ApiType, [Perbill, Perbill]>;
      /**
       * Delegator, Amount Locked, Candidate, Delegator Position with New Total
       * Counted if in Top
       */
      Delegation: AugmentedEvent<
        ApiType,
        [AccountId20, u128, AccountId20, ParachainStakingDelegatorAdded]
      >;
      DelegationDecreased: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, bool]>;
      /**
       * Delegator, Candidate, Amount to be decreased, Round at which can be executed
       */
      DelegationDecreaseScheduled: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, u32]>;
      DelegationIncreased: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, bool]>;
      /**
       * Delegator, Candidate, Amount Unstaked
       */
      DelegationKicked: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * Round, Delegator, Candidate, Scheduled Exit
       */
      DelegationRevocationScheduled: AugmentedEvent<ApiType, [u32, AccountId20, AccountId20, u32]>;
      /**
       * Delegator, Candidate, Amount Unstaked
       */
      DelegationRevoked: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * Delegator
       */
      DelegatorExitCancelled: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * Round, Delegator, Scheduled Exit
       */
      DelegatorExitScheduled: AugmentedEvent<ApiType, [u32, AccountId20, u32]>;
      /**
       * Delegator, Amount Unstaked
       */
      DelegatorLeft: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Delegator, Candidate, Amount Unstaked, New Total Amt Staked for Candidate
       */
      DelegatorLeftCandidate: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, u128]>;
      /**
       * Annual inflation input (first 3) was used to derive new per-round
       * inflation (last 3)
       */
      InflationSet: AugmentedEvent<ApiType, [Perbill, Perbill, Perbill, Perbill, Perbill, Perbill]>;
      /**
       * Account, Amount Locked, New Total Amt Locked
       */
      JoinedCollatorCandidates: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Starting Block, Round, Number of Collators Selected, Total Balance
       */
      NewRound: AugmentedEvent<ApiType, [u32, u32, u32, u128]>;
      /**
       * Account (re)set for parachain bond treasury [old, new]
       */
      ParachainBondAccountSet: AugmentedEvent<ApiType, [AccountId20, AccountId20]>;
      /**
       * Percent of inflation reserved for parachain bond (re)set [old, new]
       */
      ParachainBondReservePercentSet: AugmentedEvent<ApiType, [Percent, Percent]>;
      /**
       * Transferred to account which holds funds reserved for parachain bond
       */
      ReservedForParachainBond: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Paid the account (delegator or collator) the balance as liquid rewards
       */
      Rewarded: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Staking expectations set
       */
      StakeExpectationsSet: AugmentedEvent<ApiType, [u128, u128, u128]>;
      /**
       * Set total selected candidates to this value [old, new]
       */
      TotalSelectedSet: AugmentedEvent<ApiType, [u32, u32]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    parachainSystem: {
      /**
       * Downward messages were processed using the given weight. [ weight_used,
       * result_mqc_head ]
       */
      DownwardMessagesProcessed: AugmentedEvent<ApiType, [u64, H256]>;
      /**
       * Some downward messages have been received and will be processed. [ count ]
       */
      DownwardMessagesReceived: AugmentedEvent<ApiType, [u32]>;
      /**
       * An upgrade has been authorized.
       */
      UpgradeAuthorized: AugmentedEvent<ApiType, [H256]>;
      /**
       * The validation function was applied as of the contained relay chain block number.
       */
      ValidationFunctionApplied: AugmentedEvent<ApiType, [u32]>;
      /**
       * The relay-chain aborted the upgrade process.
       */
      ValidationFunctionDiscarded: AugmentedEvent<ApiType, []>;
      /**
       * The validation function has been scheduled to apply.
       */
      ValidationFunctionStored: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    polkadotXcm: {
      /**
       * Some assets have been placed in an asset trap.
       *
       * [ hash, origin, assets ]
       */
      AssetsTrapped: AugmentedEvent<ApiType, [H256, XcmV1MultiLocation, XcmVersionedMultiAssets]>;
      /**
       * Execution of an XCM message was attempted.
       *
       * [ outcome ]
       */
      Attempted: AugmentedEvent<ApiType, [XcmV2TraitsOutcome]>;
      /**
       * Expected query response has been received but the origin location of
       * the response does not match that expected. The query remains registered
       * for a later, valid, response to be received and acted upon.
       *
       * [ origin location, id, expected location ]
       */
      InvalidResponder: AugmentedEvent<
        ApiType,
        [XcmV1MultiLocation, u64, Option<XcmV1MultiLocation>]
      >;
      /**
       * Expected query response has been received but the expected origin
       * location placed in storage by this runtime previously cannot be
       * decoded. The query remains registered.
       *
       * This is unexpected (since a location placed in storage in a previously
       * executing runtime should be readable prior to query timeout) and
       * dangerous since the possibly valid response will be dropped. Manual
       * governance intervention is probably going to be needed.
       *
       * [ origin location, id ]
       */
      InvalidResponderVersion: AugmentedEvent<ApiType, [XcmV1MultiLocation, u64]>;
      /**
       * Query response has been received and query is removed. The registered
       * notification has been dispatched and executed successfully.
       *
       * [ id, pallet index, call index ]
       */
      Notified: AugmentedEvent<ApiType, [u64, u8, u8]>;
      /**
       * Query response has been received and query is removed. The dispatch was
       * unable to be decoded into a `Call`; this might be due to dispatch
       * function having a signature which is not `(origin, QueryId, Response)`.
       *
       * [ id, pallet index, call index ]
       */
      NotifyDecodeFailed: AugmentedEvent<ApiType, [u64, u8, u8]>;
      /**
       * Query response has been received and query is removed. There was a
       * general error with dispatching the notification call.
       *
       * [ id, pallet index, call index ]
       */
      NotifyDispatchError: AugmentedEvent<ApiType, [u64, u8, u8]>;
      /**
       * Query response has been received and query is removed. The registered
       * notification could not be dispatched because the dispatch weight is
       * greater than the maximum weight originally budgeted by this runtime for
       * the query result.
       *
       * [ id, pallet index, call index, actual weight, max budgeted weight ]
       */
      NotifyOverweight: AugmentedEvent<ApiType, [u64, u8, u8, u64, u64]>;
      /**
       * A given location which had a version change subscription was dropped
       * owing to an error migrating the location to our new XCM format.
       *
       * [ location, query ID ]
       */
      NotifyTargetMigrationFail: AugmentedEvent<ApiType, [XcmVersionedMultiLocation, u64]>;
      /**
       * A given location which had a version change subscription was dropped
       * owing to an error sending the notification to it.
       *
       * [ location, query ID, error ]
       */
      NotifyTargetSendFail: AugmentedEvent<ApiType, [XcmV1MultiLocation, u64, XcmV2TraitsError]>;
      /**
       * Query response has been received and is ready for taking with
       * `take_response`. There is no registered notification call.
       *
       * [ id, response ]
       */
      ResponseReady: AugmentedEvent<ApiType, [u64, XcmV2Response]>;
      /**
       * Received query response has been read and removed.
       *
       * [ id ]
       */
      ResponseTaken: AugmentedEvent<ApiType, [u64]>;
      /**
       * A XCM message was sent.
       *
       * [ origin, destination, message ]
       */
      Sent: AugmentedEvent<ApiType, [XcmV1MultiLocation, XcmV1MultiLocation, XcmV2Xcm]>;
      /**
       * The supported version of a location has been changed. This might be
       * through an automatic notification or a manual intervention.
       *
       * [ location, XCM version ]
       */
      SupportedVersionChanged: AugmentedEvent<ApiType, [XcmV1MultiLocation, u32]>;
      /**
       * Query response received which does not match a registered query. This
       * may be because a matching query was never registered, it may be because
       * it is a duplicate response, or because the query timed out.
       *
       * [ origin location, id ]
       */
      UnexpectedResponse: AugmentedEvent<ApiType, [XcmV1MultiLocation, u64]>;
      /**
       * An XCM version change notification message has been attempted to be sent.
       *
       * [ destination, result ]
       */
      VersionChangeNotified: AugmentedEvent<ApiType, [XcmV1MultiLocation, u32]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    proxy: {
      /**
       * An announcement was placed to make a call in the future.
       */
      Announced: AugmentedEvent<ApiType, [AccountId20, AccountId20, H256]>;
      /**
       * Anonymous account has been created by new proxy with given
       * disambiguation index and proxy type.
       */
      AnonymousCreated: AugmentedEvent<
        ApiType,
        [AccountId20, AccountId20, MoonbeamRuntimeProxyType, u16]
      >;
      /**
       * A proxy was added.
       */
      ProxyAdded: AugmentedEvent<
        ApiType,
        [AccountId20, AccountId20, MoonbeamRuntimeProxyType, u32]
      >;
      /**
       * A proxy was executed correctly, with the given.
       */
      ProxyExecuted: AugmentedEvent<ApiType, [Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    scheduler: {
      /**
       * Canceled some task. [when, index]
       */
      Canceled: AugmentedEvent<ApiType, [u32, u32]>;
      /**
       * Dispatched some task. [task, id, result]
       */
      Dispatched: AugmentedEvent<
        ApiType,
        [ITuple<[u32, u32]>, Option<Bytes>, Result<Null, SpRuntimeDispatchError>]
      >;
      /**
       * Scheduled some task. [when, index]
       */
      Scheduled: AugmentedEvent<ApiType, [u32, u32]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    system: {
      /**
       * `:code` was updated.
       */
      CodeUpdated: AugmentedEvent<ApiType, []>;
      /**
       * An extrinsic failed. [error, info]
       */
      ExtrinsicFailed: AugmentedEvent<
        ApiType,
        [SpRuntimeDispatchError, FrameSupportWeightsDispatchInfo]
      >;
      /**
       * An extrinsic completed successfully. [info]
       */
      ExtrinsicSuccess: AugmentedEvent<ApiType, [FrameSupportWeightsDispatchInfo]>;
      /**
       * An [account] was reaped.
       */
      KilledAccount: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * A new [account] was created.
       */
      NewAccount: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * On on-chain remark happened. [origin, remark_hash]
       */
      Remarked: AugmentedEvent<ApiType, [AccountId20, H256]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    techCommitteeCollective: {
      /**
       * A motion was approved by the required threshold.
       */
      Approved: AugmentedEvent<ApiType, [H256]>;
      /**
       * A proposal was closed because its threshold was reached or after its
       * duration was up.
       */
      Closed: AugmentedEvent<ApiType, [H256, u32, u32]>;
      /**
       * A motion was not approved by the required threshold.
       */
      Disapproved: AugmentedEvent<ApiType, [H256]>;
      /**
       * A motion was executed; result will be `Ok` if it returned without error.
       */
      Executed: AugmentedEvent<ApiType, [H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A single member did some action; result will be `Ok` if it returned
       * without error.
       */
      MemberExecuted: AugmentedEvent<ApiType, [H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A motion (given hash) has been proposed (by given account) with a
       * threshold (given `MemberCount`).
       */
      Proposed: AugmentedEvent<ApiType, [AccountId20, u32, H256, u32]>;
      /**
       * A motion (given hash) has been voted on by given account, leaving a
       * tally (yes votes and no votes given respectively as `MemberCount`).
       */
      Voted: AugmentedEvent<ApiType, [AccountId20, H256, bool, u32, u32]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    treasury: {
      /**
       * Some funds have been allocated. [proposal_index, award, beneficiary]
       */
      Awarded: AugmentedEvent<ApiType, [u32, u128, AccountId20]>;
      /**
       * Some of our funds have been burnt. [burn]
       */
      Burnt: AugmentedEvent<ApiType, [u128]>;
      /**
       * Some funds have been deposited. [deposit]
       */
      Deposit: AugmentedEvent<ApiType, [u128]>;
      /**
       * New proposal. [proposal_index]
       */
      Proposed: AugmentedEvent<ApiType, [u32]>;
      /**
       * A proposal was rejected; funds were slashed. [proposal_index, slashed]
       */
      Rejected: AugmentedEvent<ApiType, [u32, u128]>;
      /**
       * Spending has finished; this is the amount that rolls over until next
       * spend. [budget_remaining]
       */
      Rollover: AugmentedEvent<ApiType, [u128]>;
      /**
       * We have ended a spend period and will now allocate funds. [budget_remaining]
       */
      Spending: AugmentedEvent<ApiType, [u128]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    utility: {
      /**
       * Batch of dispatches completed fully with no error.
       */
      BatchCompleted: AugmentedEvent<ApiType, []>;
      /**
       * Batch of dispatches did not complete fully. Index of first failing
       * dispatch given, as well as the error.
       */
      BatchInterrupted: AugmentedEvent<ApiType, [u32, SpRuntimeDispatchError]>;
      /**
       * A call was dispatched. [result]
       */
      DispatchedAs: AugmentedEvent<ApiType, [Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A single item within a Batch of dispatches has completed with no error.
       */
      ItemCompleted: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    xcmpQueue: {
      /**
       * Bad XCM format used.
       */
      BadFormat: AugmentedEvent<ApiType, [Option<H256>]>;
      /**
       * Bad XCM version used.
       */
      BadVersion: AugmentedEvent<ApiType, [Option<H256>]>;
      /**
       * Some XCM failed.
       */
      Fail: AugmentedEvent<ApiType, [Option<H256>, XcmV2TraitsError]>;
      /**
       * Some XCM was executed ok.
       */
      Success: AugmentedEvent<ApiType, [Option<H256>]>;
      /**
       * An upward message was sent to the relay chain.
       */
      UpwardMessageSent: AugmentedEvent<ApiType, [Option<H256>]>;
      /**
       * An HRMP message was sent to a sibling parachain.
       */
      XcmpMessageSent: AugmentedEvent<ApiType, [Option<H256>]>;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    xcmTransactor: {
      RegisterdDerivative: AugmentedEvent<ApiType, [AccountId20, u16]>;
      TransactedDerivative: AugmentedEvent<ApiType, [AccountId20, XcmV1MultiLocation, Bytes, u16]>;
      TransactedSovereign: AugmentedEvent<ApiType, [AccountId20, XcmV1MultiLocation, Bytes]>;
      TransactFailed: AugmentedEvent<ApiType, [XcmV2TraitsError]>;
      TransactInfoChanged: AugmentedEvent<
        ApiType,
        [XcmV1MultiLocation, XcmTransactorRemoteTransactInfoWithMaxWeight]
      >;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
    xTokens: {
      /**
       * Transferred. [sender, currency_id, amount, dest]
       */
      Transferred: AugmentedEvent<
        ApiType,
        [AccountId20, MoonbeamRuntimeCurrencyId, u128, XcmV1MultiLocation]
      >;
      /**
       * Transferred `MultiAsset`. [sender, asset, dest]
       */
      TransferredMultiAsset: AugmentedEvent<
        ApiType,
        [AccountId20, XcmV1MultiAsset, XcmV1MultiLocation]
      >;
      /**
       * Transferred `MultiAsset` with fee. [sender, asset, fee, dest]
       */
      TransferredMultiAssetWithFee: AugmentedEvent<
        ApiType,
        [AccountId20, XcmV1MultiAsset, XcmV1MultiAsset, XcmV1MultiLocation]
      >;
      /**
       * Transferred with fee. [sender, currency_id, amount, fee, dest]
       */
      TransferredWithFee: AugmentedEvent<
        ApiType,
        [AccountId20, MoonbeamRuntimeCurrencyId, u128, u128, XcmV1MultiLocation]
      >;
      /**
       * Generic event
       */
      [key: string]: AugmentedEvent<ApiType>;
    };
  } // AugmentedEvents
} // declare module
