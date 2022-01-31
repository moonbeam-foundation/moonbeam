// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

declare module '@polkadot/types/lookup' {
  import type { Data } from '@polkadot/types';
  import type { BTreeMap, Bytes, Compact, Enum, Null, Option, Result, Set, Struct, Text, U256, U8aFixed, Vec, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
  import type { ITuple } from '@polkadot/types-codec/types';
  import type { Vote } from '@polkadot/types/interfaces/elections';
  import type { AccountId20, Call, H160, H256, Perbill, Percent, Permill } from '@polkadot/types/interfaces/runtime';
  import type { Event } from '@polkadot/types/interfaces/system';

  /** @name FrameSystemAccountInfo (3) */
  export interface FrameSystemAccountInfo extends Struct {
    readonly nonce: u32;
    readonly consumers: u32;
    readonly providers: u32;
    readonly sufficients: u32;
    readonly data: PalletBalancesAccountData;
  }

  /** @name PalletBalancesAccountData (5) */
  export interface PalletBalancesAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly miscFrozen: u128;
    readonly feeFrozen: u128;
  }

  /** @name FrameSupportWeightsPerDispatchClassU64 (7) */
  export interface FrameSupportWeightsPerDispatchClassU64 extends Struct {
    readonly normal: u64;
    readonly operational: u64;
    readonly mandatory: u64;
  }

  /** @name SpRuntimeDigest (12) */
  export interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>;
  }

  /** @name SpRuntimeDigestDigestItem (14) */
  export interface SpRuntimeDigestDigestItem extends Enum {
    readonly isOther: boolean;
    readonly asOther: Bytes;
    readonly isConsensus: boolean;
    readonly asConsensus: ITuple<[U8aFixed, Bytes]>;
    readonly isSeal: boolean;
    readonly asSeal: ITuple<[U8aFixed, Bytes]>;
    readonly isPreRuntime: boolean;
    readonly asPreRuntime: ITuple<[U8aFixed, Bytes]>;
    readonly isRuntimeEnvironmentUpdated: boolean;
    readonly type: 'Other' | 'Consensus' | 'Seal' | 'PreRuntime' | 'RuntimeEnvironmentUpdated';
  }

  /** @name FrameSystemEventRecord (17) */
  export interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase;
    readonly event: Event;
    readonly topics: Vec<H256>;
  }

  /** @name FrameSystemEvent (19) */
  export interface FrameSystemEvent extends Enum {
    readonly isExtrinsicSuccess: boolean;
    readonly asExtrinsicSuccess: FrameSupportWeightsDispatchInfo;
    readonly isExtrinsicFailed: boolean;
    readonly asExtrinsicFailed: ITuple<[SpRuntimeDispatchError, FrameSupportWeightsDispatchInfo]>;
    readonly isCodeUpdated: boolean;
    readonly isNewAccount: boolean;
    readonly asNewAccount: AccountId20;
    readonly isKilledAccount: boolean;
    readonly asKilledAccount: AccountId20;
    readonly isRemarked: boolean;
    readonly asRemarked: ITuple<[AccountId20, H256]>;
    readonly type: 'ExtrinsicSuccess' | 'ExtrinsicFailed' | 'CodeUpdated' | 'NewAccount' | 'KilledAccount' | 'Remarked';
  }

  /** @name FrameSupportWeightsDispatchInfo (20) */
  export interface FrameSupportWeightsDispatchInfo extends Struct {
    readonly weight: u64;
    readonly class: FrameSupportWeightsDispatchClass;
    readonly paysFee: FrameSupportWeightsPays;
  }

  /** @name FrameSupportWeightsDispatchClass (21) */
  export interface FrameSupportWeightsDispatchClass extends Enum {
    readonly isNormal: boolean;
    readonly isOperational: boolean;
    readonly isMandatory: boolean;
    readonly type: 'Normal' | 'Operational' | 'Mandatory';
  }

  /** @name FrameSupportWeightsPays (22) */
  export interface FrameSupportWeightsPays extends Enum {
    readonly isYes: boolean;
    readonly isNo: boolean;
    readonly type: 'Yes' | 'No';
  }

  /** @name SpRuntimeDispatchError (23) */
  export interface SpRuntimeDispatchError extends Enum {
    readonly isOther: boolean;
    readonly isCannotLookup: boolean;
    readonly isBadOrigin: boolean;
    readonly isModule: boolean;
    readonly asModule: {
      readonly index: u8;
      readonly error: u8;
    } & Struct;
    readonly isConsumerRemaining: boolean;
    readonly isNoProviders: boolean;
    readonly isToken: boolean;
    readonly asToken: SpRuntimeTokenError;
    readonly isArithmetic: boolean;
    readonly asArithmetic: SpRuntimeArithmeticError;
    readonly type: 'Other' | 'CannotLookup' | 'BadOrigin' | 'Module' | 'ConsumerRemaining' | 'NoProviders' | 'Token' | 'Arithmetic';
  }

  /** @name SpRuntimeTokenError (24) */
  export interface SpRuntimeTokenError extends Enum {
    readonly isNoFunds: boolean;
    readonly isWouldDie: boolean;
    readonly isBelowMinimum: boolean;
    readonly isCannotCreate: boolean;
    readonly isUnknownAsset: boolean;
    readonly isFrozen: boolean;
    readonly isUnsupported: boolean;
    readonly type: 'NoFunds' | 'WouldDie' | 'BelowMinimum' | 'CannotCreate' | 'UnknownAsset' | 'Frozen' | 'Unsupported';
  }

  /** @name SpRuntimeArithmeticError (25) */
  export interface SpRuntimeArithmeticError extends Enum {
    readonly isUnderflow: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly type: 'Underflow' | 'Overflow' | 'DivisionByZero';
  }

  /** @name PalletUtilityEvent (26) */
  export interface PalletUtilityEvent extends Enum {
    readonly isBatchInterrupted: boolean;
    readonly asBatchInterrupted: {
      readonly index: u32;
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isBatchCompleted: boolean;
    readonly isItemCompleted: boolean;
    readonly isDispatchedAs: boolean;
    readonly asDispatchedAs: Result<Null, SpRuntimeDispatchError>;
    readonly type: 'BatchInterrupted' | 'BatchCompleted' | 'ItemCompleted' | 'DispatchedAs';
  }

  /** @name PalletBalancesEvent (29) */
  export interface PalletBalancesEvent extends Enum {
    readonly isEndowed: boolean;
    readonly asEndowed: {
      readonly account: AccountId20;
      readonly freeBalance: u128;
    } & Struct;
    readonly isDustLost: boolean;
    readonly asDustLost: {
      readonly account: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly from: AccountId20;
      readonly to: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isBalanceSet: boolean;
    readonly asBalanceSet: {
      readonly who: AccountId20;
      readonly free: u128;
      readonly reserved: u128;
    } & Struct;
    readonly isReserved: boolean;
    readonly asReserved: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isUnreserved: boolean;
    readonly asUnreserved: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isReserveRepatriated: boolean;
    readonly asReserveRepatriated: {
      readonly from: AccountId20;
      readonly to: AccountId20;
      readonly amount: u128;
      readonly destinationStatus: FrameSupportTokensMiscBalanceStatus;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Endowed' | 'DustLost' | 'Transfer' | 'BalanceSet' | 'Reserved' | 'Unreserved' | 'ReserveRepatriated' | 'Deposit' | 'Withdraw' | 'Slashed';
  }

  /** @name FrameSupportTokensMiscBalanceStatus (30) */
  export interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean;
    readonly isReserved: boolean;
    readonly type: 'Free' | 'Reserved';
  }

  /** @name PalletSudoEvent (31) */
  export interface PalletSudoEvent extends Enum {
    readonly isSudid: boolean;
    readonly asSudid: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isKeyChanged: boolean;
    readonly asKeyChanged: {
      readonly newSudoer: AccountId20;
    } & Struct;
    readonly isSudoAsDone: boolean;
    readonly asSudoAsDone: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'Sudid' | 'KeyChanged' | 'SudoAsDone';
  }

  /** @name CumulusPalletParachainSystemEvent (32) */
  export interface CumulusPalletParachainSystemEvent extends Enum {
    readonly isValidationFunctionStored: boolean;
    readonly isValidationFunctionApplied: boolean;
    readonly asValidationFunctionApplied: u32;
    readonly isValidationFunctionDiscarded: boolean;
    readonly isUpgradeAuthorized: boolean;
    readonly asUpgradeAuthorized: H256;
    readonly isDownwardMessagesReceived: boolean;
    readonly asDownwardMessagesReceived: u32;
    readonly isDownwardMessagesProcessed: boolean;
    readonly asDownwardMessagesProcessed: ITuple<[u64, H256]>;
    readonly type: 'ValidationFunctionStored' | 'ValidationFunctionApplied' | 'ValidationFunctionDiscarded' | 'UpgradeAuthorized' | 'DownwardMessagesReceived' | 'DownwardMessagesProcessed';
  }

  /** @name PalletEvmEvent (33) */
  export interface PalletEvmEvent extends Enum {
    readonly isLog: boolean;
    readonly asLog: EthereumLog;
    readonly isCreated: boolean;
    readonly asCreated: H160;
    readonly isCreatedFailed: boolean;
    readonly asCreatedFailed: H160;
    readonly isExecuted: boolean;
    readonly asExecuted: H160;
    readonly isExecutedFailed: boolean;
    readonly asExecutedFailed: H160;
    readonly isBalanceDeposit: boolean;
    readonly asBalanceDeposit: ITuple<[AccountId20, H160, U256]>;
    readonly isBalanceWithdraw: boolean;
    readonly asBalanceWithdraw: ITuple<[AccountId20, H160, U256]>;
    readonly type: 'Log' | 'Created' | 'CreatedFailed' | 'Executed' | 'ExecutedFailed' | 'BalanceDeposit' | 'BalanceWithdraw';
  }

  /** @name EthereumLog (34) */
  export interface EthereumLog extends Struct {
    readonly address: H160;
    readonly topics: Vec<H256>;
    readonly data: Bytes;
  }

  /** @name PalletEthereumEvent (39) */
  export interface PalletEthereumEvent extends Enum {
    readonly isExecuted: boolean;
    readonly asExecuted: ITuple<[H160, H160, H256, EvmCoreErrorExitReason]>;
    readonly type: 'Executed';
  }

  /** @name EvmCoreErrorExitReason (40) */
  export interface EvmCoreErrorExitReason extends Enum {
    readonly isSucceed: boolean;
    readonly asSucceed: EvmCoreErrorExitSucceed;
    readonly isError: boolean;
    readonly asError: EvmCoreErrorExitError;
    readonly isRevert: boolean;
    readonly asRevert: EvmCoreErrorExitRevert;
    readonly isFatal: boolean;
    readonly asFatal: EvmCoreErrorExitFatal;
    readonly type: 'Succeed' | 'Error' | 'Revert' | 'Fatal';
  }

  /** @name EvmCoreErrorExitSucceed (41) */
  export interface EvmCoreErrorExitSucceed extends Enum {
    readonly isStopped: boolean;
    readonly isReturned: boolean;
    readonly isSuicided: boolean;
    readonly type: 'Stopped' | 'Returned' | 'Suicided';
  }

  /** @name EvmCoreErrorExitError (42) */
  export interface EvmCoreErrorExitError extends Enum {
    readonly isStackUnderflow: boolean;
    readonly isStackOverflow: boolean;
    readonly isInvalidJump: boolean;
    readonly isInvalidRange: boolean;
    readonly isDesignatedInvalid: boolean;
    readonly isCallTooDeep: boolean;
    readonly isCreateCollision: boolean;
    readonly isCreateContractLimit: boolean;
    readonly isInvalidCode: boolean;
    readonly isOutOfOffset: boolean;
    readonly isOutOfGas: boolean;
    readonly isOutOfFund: boolean;
    readonly isPcUnderflow: boolean;
    readonly isCreateEmpty: boolean;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly type: 'StackUnderflow' | 'StackOverflow' | 'InvalidJump' | 'InvalidRange' | 'DesignatedInvalid' | 'CallTooDeep' | 'CreateCollision' | 'CreateContractLimit' | 'InvalidCode' | 'OutOfOffset' | 'OutOfGas' | 'OutOfFund' | 'PcUnderflow' | 'CreateEmpty' | 'Other';
  }

  /** @name EvmCoreErrorExitRevert (45) */
  export interface EvmCoreErrorExitRevert extends Enum {
    readonly isReverted: boolean;
    readonly type: 'Reverted';
  }

  /** @name EvmCoreErrorExitFatal (46) */
  export interface EvmCoreErrorExitFatal extends Enum {
    readonly isNotSupported: boolean;
    readonly isUnhandledInterrupt: boolean;
    readonly isCallErrorAsFatal: boolean;
    readonly asCallErrorAsFatal: EvmCoreErrorExitError;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly type: 'NotSupported' | 'UnhandledInterrupt' | 'CallErrorAsFatal' | 'Other';
  }

  /** @name ParachainStakingEvent (47) */
  export interface ParachainStakingEvent extends Enum {
    readonly isNewRound: boolean;
    readonly asNewRound: ITuple<[u32, u32, u32, u128]>;
    readonly isJoinedCollatorCandidates: boolean;
    readonly asJoinedCollatorCandidates: ITuple<[AccountId20, u128, u128]>;
    readonly isCollatorChosen: boolean;
    readonly asCollatorChosen: ITuple<[u32, AccountId20, u128]>;
    readonly isCandidateBondLessRequested: boolean;
    readonly asCandidateBondLessRequested: ITuple<[AccountId20, u128, u32]>;
    readonly isCandidateBondedMore: boolean;
    readonly asCandidateBondedMore: ITuple<[AccountId20, u128, u128]>;
    readonly isCandidateBondedLess: boolean;
    readonly asCandidateBondedLess: ITuple<[AccountId20, u128, u128]>;
    readonly isCandidateWentOffline: boolean;
    readonly asCandidateWentOffline: AccountId20;
    readonly isCandidateBackOnline: boolean;
    readonly asCandidateBackOnline: AccountId20;
    readonly isCandidateScheduledExit: boolean;
    readonly asCandidateScheduledExit: ITuple<[u32, AccountId20, u32]>;
    readonly isCancelledCandidateExit: boolean;
    readonly asCancelledCandidateExit: AccountId20;
    readonly isCancelledCandidateBondLess: boolean;
    readonly asCancelledCandidateBondLess: ITuple<[AccountId20, u128, u32]>;
    readonly isCandidateLeft: boolean;
    readonly asCandidateLeft: ITuple<[AccountId20, u128, u128]>;
    readonly isDelegationDecreaseScheduled: boolean;
    readonly asDelegationDecreaseScheduled: ITuple<[AccountId20, AccountId20, u128, u32]>;
    readonly isDelegationIncreased: boolean;
    readonly asDelegationIncreased: ITuple<[AccountId20, AccountId20, u128, bool]>;
    readonly isDelegationDecreased: boolean;
    readonly asDelegationDecreased: ITuple<[AccountId20, AccountId20, u128, bool]>;
    readonly isDelegatorExitScheduled: boolean;
    readonly asDelegatorExitScheduled: ITuple<[u32, AccountId20, u32]>;
    readonly isDelegationRevocationScheduled: boolean;
    readonly asDelegationRevocationScheduled: ITuple<[u32, AccountId20, AccountId20, u32]>;
    readonly isDelegatorLeft: boolean;
    readonly asDelegatorLeft: ITuple<[AccountId20, u128]>;
    readonly isDelegationRevoked: boolean;
    readonly asDelegationRevoked: ITuple<[AccountId20, AccountId20, u128]>;
    readonly isDelegationKicked: boolean;
    readonly asDelegationKicked: ITuple<[AccountId20, AccountId20, u128]>;
    readonly isDelegatorExitCancelled: boolean;
    readonly asDelegatorExitCancelled: AccountId20;
    readonly isCancelledDelegationRequest: boolean;
    readonly asCancelledDelegationRequest: ITuple<[AccountId20, ParachainStakingDelegationRequest]>;
    readonly isDelegation: boolean;
    readonly asDelegation: ITuple<[AccountId20, u128, AccountId20, ParachainStakingDelegatorAdded]>;
    readonly isDelegatorLeftCandidate: boolean;
    readonly asDelegatorLeftCandidate: ITuple<[AccountId20, AccountId20, u128, u128]>;
    readonly isRewarded: boolean;
    readonly asRewarded: ITuple<[AccountId20, u128]>;
    readonly isReservedForParachainBond: boolean;
    readonly asReservedForParachainBond: ITuple<[AccountId20, u128]>;
    readonly isParachainBondAccountSet: boolean;
    readonly asParachainBondAccountSet: ITuple<[AccountId20, AccountId20]>;
    readonly isParachainBondReservePercentSet: boolean;
    readonly asParachainBondReservePercentSet: ITuple<[Percent, Percent]>;
    readonly isInflationSet: boolean;
    readonly asInflationSet: ITuple<[Perbill, Perbill, Perbill, Perbill, Perbill, Perbill]>;
    readonly isStakeExpectationsSet: boolean;
    readonly asStakeExpectationsSet: ITuple<[u128, u128, u128]>;
    readonly isTotalSelectedSet: boolean;
    readonly asTotalSelectedSet: ITuple<[u32, u32]>;
    readonly isCollatorCommissionSet: boolean;
    readonly asCollatorCommissionSet: ITuple<[Perbill, Perbill]>;
    readonly isBlocksPerRoundSet: boolean;
    readonly asBlocksPerRoundSet: ITuple<[u32, u32, u32, u32, Perbill, Perbill, Perbill]>;
    readonly type: 'NewRound' | 'JoinedCollatorCandidates' | 'CollatorChosen' | 'CandidateBondLessRequested' | 'CandidateBondedMore' | 'CandidateBondedLess' | 'CandidateWentOffline' | 'CandidateBackOnline' | 'CandidateScheduledExit' | 'CancelledCandidateExit' | 'CancelledCandidateBondLess' | 'CandidateLeft' | 'DelegationDecreaseScheduled' | 'DelegationIncreased' | 'DelegationDecreased' | 'DelegatorExitScheduled' | 'DelegationRevocationScheduled' | 'DelegatorLeft' | 'DelegationRevoked' | 'DelegationKicked' | 'DelegatorExitCancelled' | 'CancelledDelegationRequest' | 'Delegation' | 'DelegatorLeftCandidate' | 'Rewarded' | 'ReservedForParachainBond' | 'ParachainBondAccountSet' | 'ParachainBondReservePercentSet' | 'InflationSet' | 'StakeExpectationsSet' | 'TotalSelectedSet' | 'CollatorCommissionSet' | 'BlocksPerRoundSet';
  }

  /** @name ParachainStakingDelegationRequest (49) */
  export interface ParachainStakingDelegationRequest extends Struct {
    readonly collator: AccountId20;
    readonly amount: u128;
    readonly whenExecutable: u32;
    readonly action: ParachainStakingDelegationChange;
  }

  /** @name ParachainStakingDelegationChange (50) */
  export interface ParachainStakingDelegationChange extends Enum {
    readonly isRevoke: boolean;
    readonly isDecrease: boolean;
    readonly type: 'Revoke' | 'Decrease';
  }

  /** @name ParachainStakingDelegatorAdded (51) */
  export interface ParachainStakingDelegatorAdded extends Enum {
    readonly isAddedToTop: boolean;
    readonly asAddedToTop: {
      readonly newTotal: u128;
    } & Struct;
    readonly isAddedToBottom: boolean;
    readonly type: 'AddedToTop' | 'AddedToBottom';
  }

  /** @name PalletSchedulerEvent (54) */
  export interface PalletSchedulerEvent extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: ITuple<[u32, u32]>;
    readonly isCanceled: boolean;
    readonly asCanceled: ITuple<[u32, u32]>;
    readonly isDispatched: boolean;
    readonly asDispatched: ITuple<[ITuple<[u32, u32]>, Option<Bytes>, Result<Null, SpRuntimeDispatchError>]>;
    readonly type: 'Scheduled' | 'Canceled' | 'Dispatched';
  }

  /** @name PalletDemocracyEvent (57) */
  export interface PalletDemocracyEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly proposalIndex: u32;
      readonly deposit: u128;
    } & Struct;
    readonly isTabled: boolean;
    readonly asTabled: {
      readonly proposalIndex: u32;
      readonly deposit: u128;
      readonly depositors: Vec<AccountId20>;
    } & Struct;
    readonly isExternalTabled: boolean;
    readonly isStarted: boolean;
    readonly asStarted: {
      readonly refIndex: u32;
      readonly threshold: PalletDemocracyVoteThreshold;
    } & Struct;
    readonly isPassed: boolean;
    readonly asPassed: {
      readonly refIndex: u32;
    } & Struct;
    readonly isNotPassed: boolean;
    readonly asNotPassed: {
      readonly refIndex: u32;
    } & Struct;
    readonly isCancelled: boolean;
    readonly asCancelled: {
      readonly refIndex: u32;
    } & Struct;
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly refIndex: u32;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isDelegated: boolean;
    readonly asDelegated: {
      readonly who: AccountId20;
      readonly target: AccountId20;
    } & Struct;
    readonly isUndelegated: boolean;
    readonly asUndelegated: {
      readonly account: AccountId20;
    } & Struct;
    readonly isVetoed: boolean;
    readonly asVetoed: {
      readonly who: AccountId20;
      readonly proposalHash: H256;
      readonly until: u32;
    } & Struct;
    readonly isPreimageNoted: boolean;
    readonly asPreimageNoted: {
      readonly proposalHash: H256;
      readonly who: AccountId20;
      readonly deposit: u128;
    } & Struct;
    readonly isPreimageUsed: boolean;
    readonly asPreimageUsed: {
      readonly proposalHash: H256;
      readonly provider: AccountId20;
      readonly deposit: u128;
    } & Struct;
    readonly isPreimageInvalid: boolean;
    readonly asPreimageInvalid: {
      readonly proposalHash: H256;
      readonly refIndex: u32;
    } & Struct;
    readonly isPreimageMissing: boolean;
    readonly asPreimageMissing: {
      readonly proposalHash: H256;
      readonly refIndex: u32;
    } & Struct;
    readonly isPreimageReaped: boolean;
    readonly asPreimageReaped: {
      readonly proposalHash: H256;
      readonly provider: AccountId20;
      readonly deposit: u128;
      readonly reaper: AccountId20;
    } & Struct;
    readonly isBlacklisted: boolean;
    readonly asBlacklisted: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isVoted: boolean;
    readonly asVoted: {
      readonly who: AccountId20;
      readonly refIndex: u32;
      readonly vote: PalletDemocracyVoteAccountVote;
    } & Struct;
    readonly isSeconded: boolean;
    readonly asSeconded: {
      readonly who: AccountId20;
      readonly proposalIndex: u32;
    } & Struct;
    readonly type: 'Proposed' | 'Tabled' | 'ExternalTabled' | 'Started' | 'Passed' | 'NotPassed' | 'Cancelled' | 'Executed' | 'Delegated' | 'Undelegated' | 'Vetoed' | 'PreimageNoted' | 'PreimageUsed' | 'PreimageInvalid' | 'PreimageMissing' | 'PreimageReaped' | 'Blacklisted' | 'Voted' | 'Seconded';
  }

  /** @name PalletDemocracyVoteThreshold (59) */
  export interface PalletDemocracyVoteThreshold extends Enum {
    readonly isSuperMajorityApprove: boolean;
    readonly isSuperMajorityAgainst: boolean;
    readonly isSimpleMajority: boolean;
    readonly type: 'SuperMajorityApprove' | 'SuperMajorityAgainst' | 'SimpleMajority';
  }

  /** @name PalletDemocracyVoteAccountVote (60) */
  export interface PalletDemocracyVoteAccountVote extends Enum {
    readonly isStandard: boolean;
    readonly asStandard: {
      readonly vote: Vote;
      readonly balance: u128;
    } & Struct;
    readonly isSplit: boolean;
    readonly asSplit: {
      readonly aye: u128;
      readonly nay: u128;
    } & Struct;
    readonly type: 'Standard' | 'Split';
  }

  /** @name PalletCollectiveEvent (62) */
  export interface PalletCollectiveEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly account: AccountId20;
      readonly proposalIndex: u32;
      readonly proposalHash: H256;
      readonly threshold: u32;
    } & Struct;
    readonly isVoted: boolean;
    readonly asVoted: {
      readonly account: AccountId20;
      readonly proposalHash: H256;
      readonly voted: bool;
      readonly yes: u32;
      readonly no: u32;
    } & Struct;
    readonly isApproved: boolean;
    readonly asApproved: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isDisapproved: boolean;
    readonly asDisapproved: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly proposalHash: H256;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isMemberExecuted: boolean;
    readonly asMemberExecuted: {
      readonly proposalHash: H256;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isClosed: boolean;
    readonly asClosed: {
      readonly proposalHash: H256;
      readonly yes: u32;
      readonly no: u32;
    } & Struct;
    readonly type: 'Proposed' | 'Voted' | 'Approved' | 'Disapproved' | 'Executed' | 'MemberExecuted' | 'Closed';
  }

  /** @name PalletTreasuryEvent (64) */
  export interface PalletTreasuryEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: u32;
    readonly isSpending: boolean;
    readonly asSpending: u128;
    readonly isAwarded: boolean;
    readonly asAwarded: ITuple<[u32, u128, AccountId20]>;
    readonly isRejected: boolean;
    readonly asRejected: ITuple<[u32, u128]>;
    readonly isBurnt: boolean;
    readonly asBurnt: u128;
    readonly isRollover: boolean;
    readonly asRollover: u128;
    readonly isDeposit: boolean;
    readonly asDeposit: u128;
    readonly type: 'Proposed' | 'Spending' | 'Awarded' | 'Rejected' | 'Burnt' | 'Rollover' | 'Deposit';
  }

  /** @name PalletAuthorSlotFilterEvent (65) */
  export interface PalletAuthorSlotFilterEvent extends Enum {
    readonly isEligibleUpdated: boolean;
    readonly asEligibleUpdated: Percent;
    readonly type: 'EligibleUpdated';
  }

  /** @name PalletCrowdloanRewardsEvent (66) */
  export interface PalletCrowdloanRewardsEvent extends Enum {
    readonly isInitialPaymentMade: boolean;
    readonly asInitialPaymentMade: ITuple<[AccountId20, u128]>;
    readonly isNativeIdentityAssociated: boolean;
    readonly asNativeIdentityAssociated: ITuple<[U8aFixed, AccountId20, u128]>;
    readonly isRewardsPaid: boolean;
    readonly asRewardsPaid: ITuple<[AccountId20, u128]>;
    readonly isRewardAddressUpdated: boolean;
    readonly asRewardAddressUpdated: ITuple<[AccountId20, AccountId20]>;
    readonly isInitializedAlreadyInitializedAccount: boolean;
    readonly asInitializedAlreadyInitializedAccount: ITuple<[U8aFixed, Option<AccountId20>, u128]>;
    readonly isInitializedAccountWithNotEnoughContribution: boolean;
    readonly asInitializedAccountWithNotEnoughContribution: ITuple<[U8aFixed, Option<AccountId20>, u128]>;
    readonly type: 'InitialPaymentMade' | 'NativeIdentityAssociated' | 'RewardsPaid' | 'RewardAddressUpdated' | 'InitializedAlreadyInitializedAccount' | 'InitializedAccountWithNotEnoughContribution';
  }

  /** @name PalletAuthorMappingEvent (68) */
  export interface PalletAuthorMappingEvent extends Enum {
    readonly isAuthorRegistered: boolean;
    readonly asAuthorRegistered: ITuple<[NimbusPrimitivesNimbusCryptoPublic, AccountId20]>;
    readonly isAuthorDeRegistered: boolean;
    readonly asAuthorDeRegistered: NimbusPrimitivesNimbusCryptoPublic;
    readonly isAuthorRotated: boolean;
    readonly asAuthorRotated: ITuple<[NimbusPrimitivesNimbusCryptoPublic, AccountId20]>;
    readonly isDefunctAuthorBusted: boolean;
    readonly asDefunctAuthorBusted: ITuple<[NimbusPrimitivesNimbusCryptoPublic, AccountId20]>;
    readonly type: 'AuthorRegistered' | 'AuthorDeRegistered' | 'AuthorRotated' | 'DefunctAuthorBusted';
  }

  /** @name NimbusPrimitivesNimbusCryptoPublic (69) */
  export interface NimbusPrimitivesNimbusCryptoPublic extends SpCoreSr25519Public {}

  /** @name SpCoreSr25519Public (70) */
  export interface SpCoreSr25519Public extends U8aFixed {}

  /** @name PalletProxyEvent (71) */
  export interface PalletProxyEvent extends Enum {
    readonly isProxyExecuted: boolean;
    readonly asProxyExecuted: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isAnonymousCreated: boolean;
    readonly asAnonymousCreated: {
      readonly anonymous: AccountId20;
      readonly who: AccountId20;
      readonly proxyType: MoonbaseRuntimeProxyType;
      readonly disambiguationIndex: u16;
    } & Struct;
    readonly isAnnounced: boolean;
    readonly asAnnounced: {
      readonly real: AccountId20;
      readonly proxy: AccountId20;
      readonly callHash: H256;
    } & Struct;
    readonly isProxyAdded: boolean;
    readonly asProxyAdded: {
      readonly delegator: AccountId20;
      readonly delegatee: AccountId20;
      readonly proxyType: MoonbaseRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly type: 'ProxyExecuted' | 'AnonymousCreated' | 'Announced' | 'ProxyAdded';
  }

  /** @name MoonbaseRuntimeProxyType (72) */
  export interface MoonbaseRuntimeProxyType extends Enum {
    readonly isAny: boolean;
    readonly isNonTransfer: boolean;
    readonly isGovernance: boolean;
    readonly isStaking: boolean;
    readonly isCancelProxy: boolean;
    readonly isBalances: boolean;
    readonly isAuthorMapping: boolean;
    readonly type: 'Any' | 'NonTransfer' | 'Governance' | 'Staking' | 'CancelProxy' | 'Balances' | 'AuthorMapping';
  }

  /** @name PalletMaintenanceModeEvent (74) */
  export interface PalletMaintenanceModeEvent extends Enum {
    readonly isEnteredMaintenanceMode: boolean;
    readonly isNormalOperationResumed: boolean;
    readonly type: 'EnteredMaintenanceMode' | 'NormalOperationResumed';
  }

  /** @name PalletIdentityEvent (75) */
  export interface PalletIdentityEvent extends Enum {
    readonly isIdentitySet: boolean;
    readonly asIdentitySet: {
      readonly who: AccountId20;
    } & Struct;
    readonly isIdentityCleared: boolean;
    readonly asIdentityCleared: {
      readonly who: AccountId20;
      readonly deposit: u128;
    } & Struct;
    readonly isIdentityKilled: boolean;
    readonly asIdentityKilled: {
      readonly who: AccountId20;
      readonly deposit: u128;
    } & Struct;
    readonly isJudgementRequested: boolean;
    readonly asJudgementRequested: {
      readonly who: AccountId20;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isJudgementUnrequested: boolean;
    readonly asJudgementUnrequested: {
      readonly who: AccountId20;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isJudgementGiven: boolean;
    readonly asJudgementGiven: {
      readonly target: AccountId20;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isRegistrarAdded: boolean;
    readonly asRegistrarAdded: {
      readonly registrarIndex: u32;
    } & Struct;
    readonly isSubIdentityAdded: boolean;
    readonly asSubIdentityAdded: {
      readonly sub: AccountId20;
      readonly main: AccountId20;
      readonly deposit: u128;
    } & Struct;
    readonly isSubIdentityRemoved: boolean;
    readonly asSubIdentityRemoved: {
      readonly sub: AccountId20;
      readonly main: AccountId20;
      readonly deposit: u128;
    } & Struct;
    readonly isSubIdentityRevoked: boolean;
    readonly asSubIdentityRevoked: {
      readonly sub: AccountId20;
      readonly main: AccountId20;
      readonly deposit: u128;
    } & Struct;
    readonly type: 'IdentitySet' | 'IdentityCleared' | 'IdentityKilled' | 'JudgementRequested' | 'JudgementUnrequested' | 'JudgementGiven' | 'RegistrarAdded' | 'SubIdentityAdded' | 'SubIdentityRemoved' | 'SubIdentityRevoked';
  }

  /** @name CumulusPalletXcmpQueueEvent (76) */
  export interface CumulusPalletXcmpQueueEvent extends Enum {
    readonly isSuccess: boolean;
    readonly asSuccess: Option<H256>;
    readonly isFail: boolean;
    readonly asFail: ITuple<[Option<H256>, XcmV2TraitsError]>;
    readonly isBadVersion: boolean;
    readonly asBadVersion: Option<H256>;
    readonly isBadFormat: boolean;
    readonly asBadFormat: Option<H256>;
    readonly isUpwardMessageSent: boolean;
    readonly asUpwardMessageSent: Option<H256>;
    readonly isXcmpMessageSent: boolean;
    readonly asXcmpMessageSent: Option<H256>;
    readonly type: 'Success' | 'Fail' | 'BadVersion' | 'BadFormat' | 'UpwardMessageSent' | 'XcmpMessageSent';
  }

  /** @name XcmV2TraitsError (78) */
  export interface XcmV2TraitsError extends Enum {
    readonly isOverflow: boolean;
    readonly isUnimplemented: boolean;
    readonly isUntrustedReserveLocation: boolean;
    readonly isUntrustedTeleportLocation: boolean;
    readonly isMultiLocationFull: boolean;
    readonly isMultiLocationNotInvertible: boolean;
    readonly isBadOrigin: boolean;
    readonly isInvalidLocation: boolean;
    readonly isAssetNotFound: boolean;
    readonly isFailedToTransactAsset: boolean;
    readonly isNotWithdrawable: boolean;
    readonly isLocationCannotHold: boolean;
    readonly isExceedsMaxMessageSize: boolean;
    readonly isDestinationUnsupported: boolean;
    readonly isTransport: boolean;
    readonly isUnroutable: boolean;
    readonly isUnknownClaim: boolean;
    readonly isFailedToDecode: boolean;
    readonly isTooMuchWeightRequired: boolean;
    readonly isNotHoldingFees: boolean;
    readonly isTooExpensive: boolean;
    readonly isTrap: boolean;
    readonly asTrap: u64;
    readonly isUnhandledXcmVersion: boolean;
    readonly isWeightLimitReached: boolean;
    readonly asWeightLimitReached: u64;
    readonly isBarrier: boolean;
    readonly isWeightNotComputable: boolean;
    readonly type: 'Overflow' | 'Unimplemented' | 'UntrustedReserveLocation' | 'UntrustedTeleportLocation' | 'MultiLocationFull' | 'MultiLocationNotInvertible' | 'BadOrigin' | 'InvalidLocation' | 'AssetNotFound' | 'FailedToTransactAsset' | 'NotWithdrawable' | 'LocationCannotHold' | 'ExceedsMaxMessageSize' | 'DestinationUnsupported' | 'Transport' | 'Unroutable' | 'UnknownClaim' | 'FailedToDecode' | 'TooMuchWeightRequired' | 'NotHoldingFees' | 'TooExpensive' | 'Trap' | 'UnhandledXcmVersion' | 'WeightLimitReached' | 'Barrier' | 'WeightNotComputable';
  }

  /** @name CumulusPalletXcmEvent (79) */
  export interface CumulusPalletXcmEvent extends Enum {
    readonly isInvalidFormat: boolean;
    readonly asInvalidFormat: U8aFixed;
    readonly isUnsupportedVersion: boolean;
    readonly asUnsupportedVersion: U8aFixed;
    readonly isExecutedDownward: boolean;
    readonly asExecutedDownward: ITuple<[U8aFixed, XcmV2TraitsOutcome]>;
    readonly type: 'InvalidFormat' | 'UnsupportedVersion' | 'ExecutedDownward';
  }

  /** @name XcmV2TraitsOutcome (81) */
  export interface XcmV2TraitsOutcome extends Enum {
    readonly isComplete: boolean;
    readonly asComplete: u64;
    readonly isIncomplete: boolean;
    readonly asIncomplete: ITuple<[u64, XcmV2TraitsError]>;
    readonly isError: boolean;
    readonly asError: XcmV2TraitsError;
    readonly type: 'Complete' | 'Incomplete' | 'Error';
  }

  /** @name CumulusPalletDmpQueueEvent (82) */
  export interface CumulusPalletDmpQueueEvent extends Enum {
    readonly isInvalidFormat: boolean;
    readonly asInvalidFormat: U8aFixed;
    readonly isUnsupportedVersion: boolean;
    readonly asUnsupportedVersion: U8aFixed;
    readonly isExecutedDownward: boolean;
    readonly asExecutedDownward: ITuple<[U8aFixed, XcmV2TraitsOutcome]>;
    readonly isWeightExhausted: boolean;
    readonly asWeightExhausted: ITuple<[U8aFixed, u64, u64]>;
    readonly isOverweightEnqueued: boolean;
    readonly asOverweightEnqueued: ITuple<[U8aFixed, u64, u64]>;
    readonly isOverweightServiced: boolean;
    readonly asOverweightServiced: ITuple<[u64, u64]>;
    readonly type: 'InvalidFormat' | 'UnsupportedVersion' | 'ExecutedDownward' | 'WeightExhausted' | 'OverweightEnqueued' | 'OverweightServiced';
  }

  /** @name PalletXcmEvent (83) */
  export interface PalletXcmEvent extends Enum {
    readonly isAttempted: boolean;
    readonly asAttempted: XcmV2TraitsOutcome;
    readonly isSent: boolean;
    readonly asSent: ITuple<[XcmV1MultiLocation, XcmV1MultiLocation, XcmV2Xcm]>;
    readonly isUnexpectedResponse: boolean;
    readonly asUnexpectedResponse: ITuple<[XcmV1MultiLocation, u64]>;
    readonly isResponseReady: boolean;
    readonly asResponseReady: ITuple<[u64, XcmV2Response]>;
    readonly isNotified: boolean;
    readonly asNotified: ITuple<[u64, u8, u8]>;
    readonly isNotifyOverweight: boolean;
    readonly asNotifyOverweight: ITuple<[u64, u8, u8, u64, u64]>;
    readonly isNotifyDispatchError: boolean;
    readonly asNotifyDispatchError: ITuple<[u64, u8, u8]>;
    readonly isNotifyDecodeFailed: boolean;
    readonly asNotifyDecodeFailed: ITuple<[u64, u8, u8]>;
    readonly isInvalidResponder: boolean;
    readonly asInvalidResponder: ITuple<[XcmV1MultiLocation, u64, Option<XcmV1MultiLocation>]>;
    readonly isInvalidResponderVersion: boolean;
    readonly asInvalidResponderVersion: ITuple<[XcmV1MultiLocation, u64]>;
    readonly isResponseTaken: boolean;
    readonly asResponseTaken: u64;
    readonly isAssetsTrapped: boolean;
    readonly asAssetsTrapped: ITuple<[H256, XcmV1MultiLocation, XcmVersionedMultiAssets]>;
    readonly isVersionChangeNotified: boolean;
    readonly asVersionChangeNotified: ITuple<[XcmV1MultiLocation, u32]>;
    readonly isSupportedVersionChanged: boolean;
    readonly asSupportedVersionChanged: ITuple<[XcmV1MultiLocation, u32]>;
    readonly isNotifyTargetSendFail: boolean;
    readonly asNotifyTargetSendFail: ITuple<[XcmV1MultiLocation, u64, XcmV2TraitsError]>;
    readonly isNotifyTargetMigrationFail: boolean;
    readonly asNotifyTargetMigrationFail: ITuple<[XcmVersionedMultiLocation, u64]>;
    readonly type: 'Attempted' | 'Sent' | 'UnexpectedResponse' | 'ResponseReady' | 'Notified' | 'NotifyOverweight' | 'NotifyDispatchError' | 'NotifyDecodeFailed' | 'InvalidResponder' | 'InvalidResponderVersion' | 'ResponseTaken' | 'AssetsTrapped' | 'VersionChangeNotified' | 'SupportedVersionChanged' | 'NotifyTargetSendFail' | 'NotifyTargetMigrationFail';
  }

  /** @name XcmV1MultiLocation (84) */
  export interface XcmV1MultiLocation extends Struct {
    readonly parents: u8;
    readonly interior: XcmV1MultilocationJunctions;
  }

  /** @name XcmV1MultilocationJunctions (85) */
  export interface XcmV1MultilocationJunctions extends Enum {
    readonly isHere: boolean;
    readonly isX1: boolean;
    readonly asX1: XcmV1Junction;
    readonly isX2: boolean;
    readonly asX2: ITuple<[XcmV1Junction, XcmV1Junction]>;
    readonly isX3: boolean;
    readonly asX3: ITuple<[XcmV1Junction, XcmV1Junction, XcmV1Junction]>;
    readonly isX4: boolean;
    readonly asX4: ITuple<[XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction]>;
    readonly isX5: boolean;
    readonly asX5: ITuple<[XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction]>;
    readonly isX6: boolean;
    readonly asX6: ITuple<[XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction]>;
    readonly isX7: boolean;
    readonly asX7: ITuple<[XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction]>;
    readonly isX8: boolean;
    readonly asX8: ITuple<[XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction]>;
    readonly type: 'Here' | 'X1' | 'X2' | 'X3' | 'X4' | 'X5' | 'X6' | 'X7' | 'X8';
  }

  /** @name XcmV1Junction (86) */
  export interface XcmV1Junction extends Enum {
    readonly isParachain: boolean;
    readonly asParachain: Compact<u32>;
    readonly isAccountId32: boolean;
    readonly asAccountId32: {
      readonly network: XcmV0JunctionNetworkId;
      readonly id: U8aFixed;
    } & Struct;
    readonly isAccountIndex64: boolean;
    readonly asAccountIndex64: {
      readonly network: XcmV0JunctionNetworkId;
      readonly index: Compact<u64>;
    } & Struct;
    readonly isAccountKey20: boolean;
    readonly asAccountKey20: {
      readonly network: XcmV0JunctionNetworkId;
      readonly key: U8aFixed;
    } & Struct;
    readonly isPalletInstance: boolean;
    readonly asPalletInstance: u8;
    readonly isGeneralIndex: boolean;
    readonly asGeneralIndex: Compact<u128>;
    readonly isGeneralKey: boolean;
    readonly asGeneralKey: Bytes;
    readonly isOnlyChild: boolean;
    readonly isPlurality: boolean;
    readonly asPlurality: {
      readonly id: XcmV0JunctionBodyId;
      readonly part: XcmV0JunctionBodyPart;
    } & Struct;
    readonly type: 'Parachain' | 'AccountId32' | 'AccountIndex64' | 'AccountKey20' | 'PalletInstance' | 'GeneralIndex' | 'GeneralKey' | 'OnlyChild' | 'Plurality';
  }

  /** @name XcmV0JunctionNetworkId (88) */
  export interface XcmV0JunctionNetworkId extends Enum {
    readonly isAny: boolean;
    readonly isNamed: boolean;
    readonly asNamed: Bytes;
    readonly isPolkadot: boolean;
    readonly isKusama: boolean;
    readonly type: 'Any' | 'Named' | 'Polkadot' | 'Kusama';
  }

  /** @name XcmV0JunctionBodyId (91) */
  export interface XcmV0JunctionBodyId extends Enum {
    readonly isUnit: boolean;
    readonly isNamed: boolean;
    readonly asNamed: Bytes;
    readonly isIndex: boolean;
    readonly asIndex: Compact<u32>;
    readonly isExecutive: boolean;
    readonly isTechnical: boolean;
    readonly isLegislative: boolean;
    readonly isJudicial: boolean;
    readonly type: 'Unit' | 'Named' | 'Index' | 'Executive' | 'Technical' | 'Legislative' | 'Judicial';
  }

  /** @name XcmV0JunctionBodyPart (92) */
  export interface XcmV0JunctionBodyPart extends Enum {
    readonly isVoice: boolean;
    readonly isMembers: boolean;
    readonly asMembers: {
      readonly count: Compact<u32>;
    } & Struct;
    readonly isFraction: boolean;
    readonly asFraction: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly isAtLeastProportion: boolean;
    readonly asAtLeastProportion: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly isMoreThanProportion: boolean;
    readonly asMoreThanProportion: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly type: 'Voice' | 'Members' | 'Fraction' | 'AtLeastProportion' | 'MoreThanProportion';
  }

  /** @name XcmV2Xcm (93) */
  export interface XcmV2Xcm extends Vec<XcmV2Instruction> {}

  /** @name XcmV2Instruction (95) */
  export interface XcmV2Instruction extends Enum {
    readonly isWithdrawAsset: boolean;
    readonly asWithdrawAsset: XcmV1MultiassetMultiAssets;
    readonly isReserveAssetDeposited: boolean;
    readonly asReserveAssetDeposited: XcmV1MultiassetMultiAssets;
    readonly isReceiveTeleportedAsset: boolean;
    readonly asReceiveTeleportedAsset: XcmV1MultiassetMultiAssets;
    readonly isQueryResponse: boolean;
    readonly asQueryResponse: {
      readonly queryId: Compact<u64>;
      readonly response: XcmV2Response;
      readonly maxWeight: Compact<u64>;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly beneficiary: XcmV1MultiLocation;
    } & Struct;
    readonly isTransferReserveAsset: boolean;
    readonly asTransferReserveAsset: {
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly dest: XcmV1MultiLocation;
      readonly xcm: XcmV2Xcm;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly originType: XcmV0OriginKind;
      readonly requireWeightAtMost: Compact<u64>;
      readonly call: XcmDoubleEncoded;
    } & Struct;
    readonly isHrmpNewChannelOpenRequest: boolean;
    readonly asHrmpNewChannelOpenRequest: {
      readonly sender: Compact<u32>;
      readonly maxMessageSize: Compact<u32>;
      readonly maxCapacity: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelAccepted: boolean;
    readonly asHrmpChannelAccepted: {
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelClosing: boolean;
    readonly asHrmpChannelClosing: {
      readonly initiator: Compact<u32>;
      readonly sender: Compact<u32>;
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isClearOrigin: boolean;
    readonly isDescendOrigin: boolean;
    readonly asDescendOrigin: XcmV1MultilocationJunctions;
    readonly isReportError: boolean;
    readonly asReportError: {
      readonly queryId: Compact<u64>;
      readonly dest: XcmV1MultiLocation;
      readonly maxResponseWeight: Compact<u64>;
    } & Struct;
    readonly isDepositAsset: boolean;
    readonly asDepositAsset: {
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly maxAssets: Compact<u32>;
      readonly beneficiary: XcmV1MultiLocation;
    } & Struct;
    readonly isDepositReserveAsset: boolean;
    readonly asDepositReserveAsset: {
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly maxAssets: Compact<u32>;
      readonly dest: XcmV1MultiLocation;
      readonly xcm: XcmV2Xcm;
    } & Struct;
    readonly isExchangeAsset: boolean;
    readonly asExchangeAsset: {
      readonly give: XcmV1MultiassetMultiAssetFilter;
      readonly receive: XcmV1MultiassetMultiAssets;
    } & Struct;
    readonly isInitiateReserveWithdraw: boolean;
    readonly asInitiateReserveWithdraw: {
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly reserve: XcmV1MultiLocation;
      readonly xcm: XcmV2Xcm;
    } & Struct;
    readonly isInitiateTeleport: boolean;
    readonly asInitiateTeleport: {
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly dest: XcmV1MultiLocation;
      readonly xcm: XcmV2Xcm;
    } & Struct;
    readonly isQueryHolding: boolean;
    readonly asQueryHolding: {
      readonly queryId: Compact<u64>;
      readonly dest: XcmV1MultiLocation;
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly maxResponseWeight: Compact<u64>;
    } & Struct;
    readonly isBuyExecution: boolean;
    readonly asBuyExecution: {
      readonly fees: XcmV1MultiAsset;
      readonly weightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly isRefundSurplus: boolean;
    readonly isSetErrorHandler: boolean;
    readonly asSetErrorHandler: XcmV2Xcm;
    readonly isSetAppendix: boolean;
    readonly asSetAppendix: XcmV2Xcm;
    readonly isClearError: boolean;
    readonly isClaimAsset: boolean;
    readonly asClaimAsset: {
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly ticket: XcmV1MultiLocation;
    } & Struct;
    readonly isTrap: boolean;
    readonly asTrap: Compact<u64>;
    readonly isSubscribeVersion: boolean;
    readonly asSubscribeVersion: {
      readonly queryId: Compact<u64>;
      readonly maxResponseWeight: Compact<u64>;
    } & Struct;
    readonly isUnsubscribeVersion: boolean;
    readonly type: 'WithdrawAsset' | 'ReserveAssetDeposited' | 'ReceiveTeleportedAsset' | 'QueryResponse' | 'TransferAsset' | 'TransferReserveAsset' | 'Transact' | 'HrmpNewChannelOpenRequest' | 'HrmpChannelAccepted' | 'HrmpChannelClosing' | 'ClearOrigin' | 'DescendOrigin' | 'ReportError' | 'DepositAsset' | 'DepositReserveAsset' | 'ExchangeAsset' | 'InitiateReserveWithdraw' | 'InitiateTeleport' | 'QueryHolding' | 'BuyExecution' | 'RefundSurplus' | 'SetErrorHandler' | 'SetAppendix' | 'ClearError' | 'ClaimAsset' | 'Trap' | 'SubscribeVersion' | 'UnsubscribeVersion';
  }

  /** @name XcmV1MultiassetMultiAssets (96) */
  export interface XcmV1MultiassetMultiAssets extends Vec<XcmV1MultiAsset> {}

  /** @name XcmV1MultiAsset (98) */
  export interface XcmV1MultiAsset extends Struct {
    readonly id: XcmV1MultiassetAssetId;
    readonly fun: XcmV1MultiassetFungibility;
  }

  /** @name XcmV1MultiassetAssetId (99) */
  export interface XcmV1MultiassetAssetId extends Enum {
    readonly isConcrete: boolean;
    readonly asConcrete: XcmV1MultiLocation;
    readonly isAbstract: boolean;
    readonly asAbstract: Bytes;
    readonly type: 'Concrete' | 'Abstract';
  }

  /** @name XcmV1MultiassetFungibility (100) */
  export interface XcmV1MultiassetFungibility extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: Compact<u128>;
    readonly isNonFungible: boolean;
    readonly asNonFungible: XcmV1MultiassetAssetInstance;
    readonly type: 'Fungible' | 'NonFungible';
  }

  /** @name XcmV1MultiassetAssetInstance (101) */
  export interface XcmV1MultiassetAssetInstance extends Enum {
    readonly isUndefined: boolean;
    readonly isIndex: boolean;
    readonly asIndex: Compact<u128>;
    readonly isArray4: boolean;
    readonly asArray4: U8aFixed;
    readonly isArray8: boolean;
    readonly asArray8: U8aFixed;
    readonly isArray16: boolean;
    readonly asArray16: U8aFixed;
    readonly isArray32: boolean;
    readonly asArray32: U8aFixed;
    readonly isBlob: boolean;
    readonly asBlob: Bytes;
    readonly type: 'Undefined' | 'Index' | 'Array4' | 'Array8' | 'Array16' | 'Array32' | 'Blob';
  }

  /** @name XcmV2Response (103) */
  export interface XcmV2Response extends Enum {
    readonly isNull: boolean;
    readonly isAssets: boolean;
    readonly asAssets: XcmV1MultiassetMultiAssets;
    readonly isExecutionResult: boolean;
    readonly asExecutionResult: Option<ITuple<[u32, XcmV2TraitsError]>>;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly type: 'Null' | 'Assets' | 'ExecutionResult' | 'Version';
  }

  /** @name XcmV0OriginKind (106) */
  export interface XcmV0OriginKind extends Enum {
    readonly isNative: boolean;
    readonly isSovereignAccount: boolean;
    readonly isSuperuser: boolean;
    readonly isXcm: boolean;
    readonly type: 'Native' | 'SovereignAccount' | 'Superuser' | 'Xcm';
  }

  /** @name XcmDoubleEncoded (107) */
  export interface XcmDoubleEncoded extends Struct {
    readonly encoded: Bytes;
  }

  /** @name XcmV1MultiassetMultiAssetFilter (108) */
  export interface XcmV1MultiassetMultiAssetFilter extends Enum {
    readonly isDefinite: boolean;
    readonly asDefinite: XcmV1MultiassetMultiAssets;
    readonly isWild: boolean;
    readonly asWild: XcmV1MultiassetWildMultiAsset;
    readonly type: 'Definite' | 'Wild';
  }

  /** @name XcmV1MultiassetWildMultiAsset (109) */
  export interface XcmV1MultiassetWildMultiAsset extends Enum {
    readonly isAll: boolean;
    readonly isAllOf: boolean;
    readonly asAllOf: {
      readonly id: XcmV1MultiassetAssetId;
      readonly fun: XcmV1MultiassetWildFungibility;
    } & Struct;
    readonly type: 'All' | 'AllOf';
  }

  /** @name XcmV1MultiassetWildFungibility (110) */
  export interface XcmV1MultiassetWildFungibility extends Enum {
    readonly isFungible: boolean;
    readonly isNonFungible: boolean;
    readonly type: 'Fungible' | 'NonFungible';
  }

  /** @name XcmV2WeightLimit (111) */
  export interface XcmV2WeightLimit extends Enum {
    readonly isUnlimited: boolean;
    readonly isLimited: boolean;
    readonly asLimited: Compact<u64>;
    readonly type: 'Unlimited' | 'Limited';
  }

  /** @name XcmVersionedMultiAssets (113) */
  export interface XcmVersionedMultiAssets extends Enum {
    readonly isV0: boolean;
    readonly asV0: Vec<XcmV0MultiAsset>;
    readonly isV1: boolean;
    readonly asV1: XcmV1MultiassetMultiAssets;
    readonly type: 'V0' | 'V1';
  }

  /** @name XcmV0MultiAsset (115) */
  export interface XcmV0MultiAsset extends Enum {
    readonly isNone: boolean;
    readonly isAll: boolean;
    readonly isAllFungible: boolean;
    readonly isAllNonFungible: boolean;
    readonly isAllAbstractFungible: boolean;
    readonly asAllAbstractFungible: {
      readonly id: Bytes;
    } & Struct;
    readonly isAllAbstractNonFungible: boolean;
    readonly asAllAbstractNonFungible: {
      readonly class: Bytes;
    } & Struct;
    readonly isAllConcreteFungible: boolean;
    readonly asAllConcreteFungible: {
      readonly id: XcmV0MultiLocation;
    } & Struct;
    readonly isAllConcreteNonFungible: boolean;
    readonly asAllConcreteNonFungible: {
      readonly class: XcmV0MultiLocation;
    } & Struct;
    readonly isAbstractFungible: boolean;
    readonly asAbstractFungible: {
      readonly id: Bytes;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isAbstractNonFungible: boolean;
    readonly asAbstractNonFungible: {
      readonly class: Bytes;
      readonly instance: XcmV1MultiassetAssetInstance;
    } & Struct;
    readonly isConcreteFungible: boolean;
    readonly asConcreteFungible: {
      readonly id: XcmV0MultiLocation;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isConcreteNonFungible: boolean;
    readonly asConcreteNonFungible: {
      readonly class: XcmV0MultiLocation;
      readonly instance: XcmV1MultiassetAssetInstance;
    } & Struct;
    readonly type: 'None' | 'All' | 'AllFungible' | 'AllNonFungible' | 'AllAbstractFungible' | 'AllAbstractNonFungible' | 'AllConcreteFungible' | 'AllConcreteNonFungible' | 'AbstractFungible' | 'AbstractNonFungible' | 'ConcreteFungible' | 'ConcreteNonFungible';
  }

  /** @name XcmV0MultiLocation (116) */
  export interface XcmV0MultiLocation extends Enum {
    readonly isNull: boolean;
    readonly isX1: boolean;
    readonly asX1: XcmV0Junction;
    readonly isX2: boolean;
    readonly asX2: ITuple<[XcmV0Junction, XcmV0Junction]>;
    readonly isX3: boolean;
    readonly asX3: ITuple<[XcmV0Junction, XcmV0Junction, XcmV0Junction]>;
    readonly isX4: boolean;
    readonly asX4: ITuple<[XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction]>;
    readonly isX5: boolean;
    readonly asX5: ITuple<[XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction]>;
    readonly isX6: boolean;
    readonly asX6: ITuple<[XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction]>;
    readonly isX7: boolean;
    readonly asX7: ITuple<[XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction]>;
    readonly isX8: boolean;
    readonly asX8: ITuple<[XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction]>;
    readonly type: 'Null' | 'X1' | 'X2' | 'X3' | 'X4' | 'X5' | 'X6' | 'X7' | 'X8';
  }

  /** @name XcmV0Junction (117) */
  export interface XcmV0Junction extends Enum {
    readonly isParent: boolean;
    readonly isParachain: boolean;
    readonly asParachain: Compact<u32>;
    readonly isAccountId32: boolean;
    readonly asAccountId32: {
      readonly network: XcmV0JunctionNetworkId;
      readonly id: U8aFixed;
    } & Struct;
    readonly isAccountIndex64: boolean;
    readonly asAccountIndex64: {
      readonly network: XcmV0JunctionNetworkId;
      readonly index: Compact<u64>;
    } & Struct;
    readonly isAccountKey20: boolean;
    readonly asAccountKey20: {
      readonly network: XcmV0JunctionNetworkId;
      readonly key: U8aFixed;
    } & Struct;
    readonly isPalletInstance: boolean;
    readonly asPalletInstance: u8;
    readonly isGeneralIndex: boolean;
    readonly asGeneralIndex: Compact<u128>;
    readonly isGeneralKey: boolean;
    readonly asGeneralKey: Bytes;
    readonly isOnlyChild: boolean;
    readonly isPlurality: boolean;
    readonly asPlurality: {
      readonly id: XcmV0JunctionBodyId;
      readonly part: XcmV0JunctionBodyPart;
    } & Struct;
    readonly type: 'Parent' | 'Parachain' | 'AccountId32' | 'AccountIndex64' | 'AccountKey20' | 'PalletInstance' | 'GeneralIndex' | 'GeneralKey' | 'OnlyChild' | 'Plurality';
  }

  /** @name XcmVersionedMultiLocation (118) */
  export interface XcmVersionedMultiLocation extends Enum {
    readonly isV0: boolean;
    readonly asV0: XcmV0MultiLocation;
    readonly isV1: boolean;
    readonly asV1: XcmV1MultiLocation;
    readonly type: 'V0' | 'V1';
  }

  /** @name PalletAssetsEvent (119) */
  export interface PalletAssetsEvent extends Enum {
    readonly isCreated: boolean;
    readonly asCreated: {
      readonly assetId: u128;
      readonly creator: AccountId20;
      readonly owner: AccountId20;
    } & Struct;
    readonly isIssued: boolean;
    readonly asIssued: {
      readonly assetId: u128;
      readonly owner: AccountId20;
      readonly totalSupply: u128;
    } & Struct;
    readonly isTransferred: boolean;
    readonly asTransferred: {
      readonly assetId: u128;
      readonly from: AccountId20;
      readonly to: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isBurned: boolean;
    readonly asBurned: {
      readonly assetId: u128;
      readonly owner: AccountId20;
      readonly balance: u128;
    } & Struct;
    readonly isTeamChanged: boolean;
    readonly asTeamChanged: {
      readonly assetId: u128;
      readonly issuer: AccountId20;
      readonly admin: AccountId20;
      readonly freezer: AccountId20;
    } & Struct;
    readonly isOwnerChanged: boolean;
    readonly asOwnerChanged: {
      readonly assetId: u128;
      readonly owner: AccountId20;
    } & Struct;
    readonly isFrozen: boolean;
    readonly asFrozen: {
      readonly assetId: u128;
      readonly who: AccountId20;
    } & Struct;
    readonly isThawed: boolean;
    readonly asThawed: {
      readonly assetId: u128;
      readonly who: AccountId20;
    } & Struct;
    readonly isAssetFrozen: boolean;
    readonly asAssetFrozen: {
      readonly assetId: u128;
    } & Struct;
    readonly isAssetThawed: boolean;
    readonly asAssetThawed: {
      readonly assetId: u128;
    } & Struct;
    readonly isDestroyed: boolean;
    readonly asDestroyed: {
      readonly assetId: u128;
    } & Struct;
    readonly isForceCreated: boolean;
    readonly asForceCreated: {
      readonly assetId: u128;
      readonly owner: AccountId20;
    } & Struct;
    readonly isMetadataSet: boolean;
    readonly asMetadataSet: {
      readonly assetId: u128;
      readonly name: Bytes;
      readonly symbol: Bytes;
      readonly decimals: u8;
      readonly isFrozen: bool;
    } & Struct;
    readonly isMetadataCleared: boolean;
    readonly asMetadataCleared: {
      readonly assetId: u128;
    } & Struct;
    readonly isApprovedTransfer: boolean;
    readonly asApprovedTransfer: {
      readonly assetId: u128;
      readonly source: AccountId20;
      readonly delegate: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isApprovalCancelled: boolean;
    readonly asApprovalCancelled: {
      readonly assetId: u128;
      readonly owner: AccountId20;
      readonly delegate: AccountId20;
    } & Struct;
    readonly isTransferredApproved: boolean;
    readonly asTransferredApproved: {
      readonly assetId: u128;
      readonly owner: AccountId20;
      readonly delegate: AccountId20;
      readonly destination: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isAssetStatusChanged: boolean;
    readonly asAssetStatusChanged: {
      readonly assetId: u128;
    } & Struct;
    readonly type: 'Created' | 'Issued' | 'Transferred' | 'Burned' | 'TeamChanged' | 'OwnerChanged' | 'Frozen' | 'Thawed' | 'AssetFrozen' | 'AssetThawed' | 'Destroyed' | 'ForceCreated' | 'MetadataSet' | 'MetadataCleared' | 'ApprovedTransfer' | 'ApprovalCancelled' | 'TransferredApproved' | 'AssetStatusChanged';
  }

  /** @name OrmlXtokensModuleEvent (120) */
  export interface OrmlXtokensModuleEvent extends Enum {
    readonly isTransferred: boolean;
    readonly asTransferred: ITuple<[AccountId20, MoonbaseRuntimeCurrencyId, u128, XcmV1MultiLocation]>;
    readonly isTransferredWithFee: boolean;
    readonly asTransferredWithFee: ITuple<[AccountId20, MoonbaseRuntimeCurrencyId, u128, u128, XcmV1MultiLocation]>;
    readonly isTransferredMultiAsset: boolean;
    readonly asTransferredMultiAsset: ITuple<[AccountId20, XcmV1MultiAsset, XcmV1MultiLocation]>;
    readonly isTransferredMultiAssetWithFee: boolean;
    readonly asTransferredMultiAssetWithFee: ITuple<[AccountId20, XcmV1MultiAsset, XcmV1MultiAsset, XcmV1MultiLocation]>;
    readonly type: 'Transferred' | 'TransferredWithFee' | 'TransferredMultiAsset' | 'TransferredMultiAssetWithFee';
  }

  /** @name MoonbaseRuntimeCurrencyId (121) */
  export interface MoonbaseRuntimeCurrencyId extends Enum {
    readonly isSelfReserve: boolean;
    readonly isOtherReserve: boolean;
    readonly asOtherReserve: u128;
    readonly type: 'SelfReserve' | 'OtherReserve';
  }

  /** @name PalletAssetManagerEvent (122) */
  export interface PalletAssetManagerEvent extends Enum {
    readonly isAssetRegistered: boolean;
    readonly asAssetRegistered: ITuple<[u128, MoonbaseRuntimeAssetType, MoonbaseRuntimeAssetRegistrarMetadata]>;
    readonly isUnitsPerSecondChanged: boolean;
    readonly asUnitsPerSecondChanged: ITuple<[MoonbaseRuntimeAssetType, u128]>;
    readonly isAssetTypeChanged: boolean;
    readonly asAssetTypeChanged: ITuple<[u128, MoonbaseRuntimeAssetType]>;
    readonly type: 'AssetRegistered' | 'UnitsPerSecondChanged' | 'AssetTypeChanged';
  }

  /** @name MoonbaseRuntimeAssetType (123) */
  export interface MoonbaseRuntimeAssetType extends Enum {
    readonly isXcm: boolean;
    readonly asXcm: XcmV1MultiLocation;
    readonly type: 'Xcm';
  }

  /** @name MoonbaseRuntimeAssetRegistrarMetadata (124) */
  export interface MoonbaseRuntimeAssetRegistrarMetadata extends Struct {
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly decimals: u8;
    readonly isFrozen: bool;
  }

  /** @name PalletMigrationsEvent (125) */
  export interface PalletMigrationsEvent extends Enum {
    readonly isRuntimeUpgradeStarted: boolean;
    readonly isRuntimeUpgradeCompleted: boolean;
    readonly asRuntimeUpgradeCompleted: u64;
    readonly isMigrationStarted: boolean;
    readonly asMigrationStarted: Bytes;
    readonly isMigrationCompleted: boolean;
    readonly asMigrationCompleted: ITuple<[Bytes, u64]>;
    readonly type: 'RuntimeUpgradeStarted' | 'RuntimeUpgradeCompleted' | 'MigrationStarted' | 'MigrationCompleted';
  }

  /** @name XcmTransactorEvent (126) */
  export interface XcmTransactorEvent extends Enum {
    readonly isTransactedDerivative: boolean;
    readonly asTransactedDerivative: ITuple<[AccountId20, XcmV1MultiLocation, Bytes, u16]>;
    readonly isTransactedSovereign: boolean;
    readonly asTransactedSovereign: ITuple<[AccountId20, XcmV1MultiLocation, Bytes]>;
    readonly isRegisterdDerivative: boolean;
    readonly asRegisterdDerivative: ITuple<[AccountId20, u16]>;
    readonly isTransactFailed: boolean;
    readonly asTransactFailed: XcmV2TraitsError;
    readonly isTransactInfoChanged: boolean;
    readonly asTransactInfoChanged: ITuple<[XcmV1MultiLocation, XcmTransactorRemoteTransactInfoWithMaxWeight]>;
    readonly type: 'TransactedDerivative' | 'TransactedSovereign' | 'RegisterdDerivative' | 'TransactFailed' | 'TransactInfoChanged';
  }

  /** @name XcmTransactorRemoteTransactInfoWithMaxWeight (127) */
  export interface XcmTransactorRemoteTransactInfoWithMaxWeight extends Struct {
    readonly transactExtraWeight: u64;
    readonly feePerSecond: u128;
    readonly maxWeight: u64;
  }

  /** @name PalletBaseFeeEvent (128) */
  export interface PalletBaseFeeEvent extends Enum {
    readonly isNewBaseFeePerGas: boolean;
    readonly asNewBaseFeePerGas: U256;
    readonly isBaseFeeOverflow: boolean;
    readonly isIsActive: boolean;
    readonly asIsActive: bool;
    readonly isNewElasticity: boolean;
    readonly asNewElasticity: Permill;
    readonly type: 'NewBaseFeePerGas' | 'BaseFeeOverflow' | 'IsActive' | 'NewElasticity';
  }

  /** @name FrameSystemPhase (130) */
  export interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean;
    readonly asApplyExtrinsic: u32;
    readonly isFinalization: boolean;
    readonly isInitialization: boolean;
    readonly type: 'ApplyExtrinsic' | 'Finalization' | 'Initialization';
  }

  /** @name FrameSystemLastRuntimeUpgradeInfo (132) */
  export interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>;
    readonly specName: Text;
  }

  /** @name FrameSystemCall (133) */
  export interface FrameSystemCall extends Enum {
    readonly isFillBlock: boolean;
    readonly asFillBlock: {
      readonly ratio: Perbill;
    } & Struct;
    readonly isRemark: boolean;
    readonly asRemark: {
      readonly remark: Bytes;
    } & Struct;
    readonly isSetHeapPages: boolean;
    readonly asSetHeapPages: {
      readonly pages: u64;
    } & Struct;
    readonly isSetCode: boolean;
    readonly asSetCode: {
      readonly code: Bytes;
    } & Struct;
    readonly isSetCodeWithoutChecks: boolean;
    readonly asSetCodeWithoutChecks: {
      readonly code: Bytes;
    } & Struct;
    readonly isSetStorage: boolean;
    readonly asSetStorage: {
      readonly items: Vec<ITuple<[Bytes, Bytes]>>;
    } & Struct;
    readonly isKillStorage: boolean;
    readonly asKillStorage: {
      readonly keys_: Vec<Bytes>;
    } & Struct;
    readonly isKillPrefix: boolean;
    readonly asKillPrefix: {
      readonly prefix: Bytes;
      readonly subkeys: u32;
    } & Struct;
    readonly isRemarkWithEvent: boolean;
    readonly asRemarkWithEvent: {
      readonly remark: Bytes;
    } & Struct;
    readonly type: 'FillBlock' | 'Remark' | 'SetHeapPages' | 'SetCode' | 'SetCodeWithoutChecks' | 'SetStorage' | 'KillStorage' | 'KillPrefix' | 'RemarkWithEvent';
  }

  /** @name FrameSystemLimitsBlockWeights (137) */
  export interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: u64;
    readonly maxBlock: u64;
    readonly perClass: FrameSupportWeightsPerDispatchClassWeightsPerClass;
  }

  /** @name FrameSupportWeightsPerDispatchClassWeightsPerClass (138) */
  export interface FrameSupportWeightsPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass;
    readonly operational: FrameSystemLimitsWeightsPerClass;
    readonly mandatory: FrameSystemLimitsWeightsPerClass;
  }

  /** @name FrameSystemLimitsWeightsPerClass (139) */
  export interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: u64;
    readonly maxExtrinsic: Option<u64>;
    readonly maxTotal: Option<u64>;
    readonly reserved: Option<u64>;
  }

  /** @name FrameSystemLimitsBlockLength (141) */
  export interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportWeightsPerDispatchClassU32;
  }

  /** @name FrameSupportWeightsPerDispatchClassU32 (142) */
  export interface FrameSupportWeightsPerDispatchClassU32 extends Struct {
    readonly normal: u32;
    readonly operational: u32;
    readonly mandatory: u32;
  }

  /** @name FrameSupportWeightsRuntimeDbWeight (143) */
  export interface FrameSupportWeightsRuntimeDbWeight extends Struct {
    readonly read: u64;
    readonly write: u64;
  }

  /** @name SpVersionRuntimeVersion (144) */
  export interface SpVersionRuntimeVersion extends Struct {
    readonly specName: Text;
    readonly implName: Text;
    readonly authoringVersion: u32;
    readonly specVersion: u32;
    readonly implVersion: u32;
    readonly apis: Vec<ITuple<[U8aFixed, u32]>>;
    readonly transactionVersion: u32;
  }

  /** @name FrameSystemError (148) */
  export interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean;
    readonly isSpecVersionNeedsToIncrease: boolean;
    readonly isFailedToExtractRuntimeVersion: boolean;
    readonly isNonDefaultComposite: boolean;
    readonly isNonZeroRefCount: boolean;
    readonly isCallFiltered: boolean;
    readonly type: 'InvalidSpecName' | 'SpecVersionNeedsToIncrease' | 'FailedToExtractRuntimeVersion' | 'NonDefaultComposite' | 'NonZeroRefCount' | 'CallFiltered';
  }

  /** @name PalletUtilityCall (149) */
  export interface PalletUtilityCall extends Enum {
    readonly isBatch: boolean;
    readonly asBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isAsDerivative: boolean;
    readonly asAsDerivative: {
      readonly index: u16;
      readonly call: Call;
    } & Struct;
    readonly isBatchAll: boolean;
    readonly asBatchAll: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isDispatchAs: boolean;
    readonly asDispatchAs: {
      readonly asOrigin: MoonbaseRuntimeOriginCaller;
      readonly call: Call;
    } & Struct;
    readonly type: 'Batch' | 'AsDerivative' | 'BatchAll' | 'DispatchAs';
  }

  /** @name PalletTimestampCall (152) */
  export interface PalletTimestampCall extends Enum {
    readonly isSet: boolean;
    readonly asSet: {
      readonly now: Compact<u64>;
    } & Struct;
    readonly type: 'Set';
  }

  /** @name PalletBalancesCall (153) */
  export interface PalletBalancesCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly dest: AccountId20;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isSetBalance: boolean;
    readonly asSetBalance: {
      readonly who: AccountId20;
      readonly newFree: Compact<u128>;
      readonly newReserved: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly source: AccountId20;
      readonly dest: AccountId20;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferKeepAlive: boolean;
    readonly asTransferKeepAlive: {
      readonly dest: AccountId20;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferAll: boolean;
    readonly asTransferAll: {
      readonly dest: AccountId20;
      readonly keepAlive: bool;
    } & Struct;
    readonly isForceUnreserve: boolean;
    readonly asForceUnreserve: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Transfer' | 'SetBalance' | 'ForceTransfer' | 'TransferKeepAlive' | 'TransferAll' | 'ForceUnreserve';
  }

  /** @name PalletSudoCall (154) */
  export interface PalletSudoCall extends Enum {
    readonly isSudo: boolean;
    readonly asSudo: {
      readonly call: Call;
    } & Struct;
    readonly isSudoUncheckedWeight: boolean;
    readonly asSudoUncheckedWeight: {
      readonly call: Call;
      readonly weight: u64;
    } & Struct;
    readonly isSetKey: boolean;
    readonly asSetKey: {
      readonly new_: AccountId20;
    } & Struct;
    readonly isSudoAs: boolean;
    readonly asSudoAs: {
      readonly who: AccountId20;
      readonly call: Call;
    } & Struct;
    readonly type: 'Sudo' | 'SudoUncheckedWeight' | 'SetKey' | 'SudoAs';
  }

  /** @name CumulusPalletParachainSystemCall (155) */
  export interface CumulusPalletParachainSystemCall extends Enum {
    readonly isSetValidationData: boolean;
    readonly asSetValidationData: {
      readonly data: CumulusPrimitivesParachainInherentParachainInherentData;
    } & Struct;
    readonly isSudoSendUpwardMessage: boolean;
    readonly asSudoSendUpwardMessage: {
      readonly message: Bytes;
    } & Struct;
    readonly isAuthorizeUpgrade: boolean;
    readonly asAuthorizeUpgrade: {
      readonly codeHash: H256;
    } & Struct;
    readonly isEnactAuthorizedUpgrade: boolean;
    readonly asEnactAuthorizedUpgrade: {
      readonly code: Bytes;
    } & Struct;
    readonly type: 'SetValidationData' | 'SudoSendUpwardMessage' | 'AuthorizeUpgrade' | 'EnactAuthorizedUpgrade';
  }

  /** @name CumulusPrimitivesParachainInherentParachainInherentData (156) */
  export interface CumulusPrimitivesParachainInherentParachainInherentData extends Struct {
    readonly validationData: PolkadotPrimitivesV1PersistedValidationData;
    readonly relayChainState: SpTrieStorageProof;
    readonly downwardMessages: Vec<PolkadotCorePrimitivesInboundDownwardMessage>;
    readonly horizontalMessages: BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>;
  }

  /** @name PolkadotPrimitivesV1PersistedValidationData (157) */
  export interface PolkadotPrimitivesV1PersistedValidationData extends Struct {
    readonly parentHead: Bytes;
    readonly relayParentNumber: u32;
    readonly relayParentStorageRoot: H256;
    readonly maxPovSize: u32;
  }

  /** @name SpTrieStorageProof (159) */
  export interface SpTrieStorageProof extends Struct {
    readonly trieNodes: Vec<Bytes>;
  }

  /** @name PolkadotCorePrimitivesInboundDownwardMessage (161) */
  export interface PolkadotCorePrimitivesInboundDownwardMessage extends Struct {
    readonly sentAt: u32;
    readonly msg: Bytes;
  }

  /** @name PolkadotCorePrimitivesInboundHrmpMessage (165) */
  export interface PolkadotCorePrimitivesInboundHrmpMessage extends Struct {
    readonly sentAt: u32;
    readonly data: Bytes;
  }

  /** @name PalletEvmCall (168) */
  export interface PalletEvmCall extends Enum {
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly address: H160;
      readonly value: u128;
    } & Struct;
    readonly isCall: boolean;
    readonly asCall: {
      readonly source: H160;
      readonly target: H160;
      readonly input: Bytes;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly source: H160;
      readonly init: Bytes;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly isCreate2: boolean;
    readonly asCreate2: {
      readonly source: H160;
      readonly init: Bytes;
      readonly salt: H256;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly type: 'Withdraw' | 'Call' | 'Create' | 'Create2';
  }

  /** @name PalletEthereumCall (172) */
  export interface PalletEthereumCall extends Enum {
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly transaction: EthereumTransactionTransactionV2;
    } & Struct;
    readonly type: 'Transact';
  }

  /** @name EthereumTransactionTransactionV2 (173) */
  export interface EthereumTransactionTransactionV2 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumTransactionLegacyTransaction;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumTransactionEip2930Transaction;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumTransactionEip1559Transaction;
    readonly type: 'Legacy' | 'Eip2930' | 'Eip1559';
  }

  /** @name EthereumTransactionLegacyTransaction (174) */
  export interface EthereumTransactionLegacyTransaction extends Struct {
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly signature: EthereumTransactionTransactionSignature;
  }

  /** @name EthereumTransactionTransactionAction (175) */
  export interface EthereumTransactionTransactionAction extends Enum {
    readonly isCall: boolean;
    readonly asCall: H160;
    readonly isCreate: boolean;
    readonly type: 'Call' | 'Create';
  }

  /** @name EthereumTransactionTransactionSignature (176) */
  export interface EthereumTransactionTransactionSignature extends Struct {
    readonly v: u64;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name EthereumTransactionEip2930Transaction (178) */
  export interface EthereumTransactionEip2930Transaction extends Struct {
    readonly chainId: u64;
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Vec<EthereumTransactionAccessListItem>;
    readonly oddYParity: bool;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name EthereumTransactionAccessListItem (180) */
  export interface EthereumTransactionAccessListItem extends Struct {
    readonly address: H160;
    readonly storageKeys: Vec<H256>;
  }

  /** @name EthereumTransactionEip1559Transaction (181) */
  export interface EthereumTransactionEip1559Transaction extends Struct {
    readonly chainId: u64;
    readonly nonce: U256;
    readonly maxPriorityFeePerGas: U256;
    readonly maxFeePerGas: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Vec<EthereumTransactionAccessListItem>;
    readonly oddYParity: bool;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name ParachainStakingCall (182) */
  export interface ParachainStakingCall extends Enum {
    readonly isHotfixRemoveDelegationRequests: boolean;
    readonly asHotfixRemoveDelegationRequests: {
      readonly delegators: Vec<AccountId20>;
    } & Struct;
    readonly isHotfixUpdateCandidatePoolValue: boolean;
    readonly asHotfixUpdateCandidatePoolValue: {
      readonly candidates: Vec<AccountId20>;
    } & Struct;
    readonly isSetStakingExpectations: boolean;
    readonly asSetStakingExpectations: {
      readonly expectations: ParachainStakingInflationRangeU128;
    } & Struct;
    readonly isSetInflation: boolean;
    readonly asSetInflation: {
      readonly schedule: ParachainStakingInflationRangePerbill;
    } & Struct;
    readonly isSetParachainBondAccount: boolean;
    readonly asSetParachainBondAccount: {
      readonly new_: AccountId20;
    } & Struct;
    readonly isSetParachainBondReservePercent: boolean;
    readonly asSetParachainBondReservePercent: {
      readonly new_: Percent;
    } & Struct;
    readonly isSetTotalSelected: boolean;
    readonly asSetTotalSelected: {
      readonly new_: u32;
    } & Struct;
    readonly isSetCollatorCommission: boolean;
    readonly asSetCollatorCommission: {
      readonly new_: Perbill;
    } & Struct;
    readonly isSetBlocksPerRound: boolean;
    readonly asSetBlocksPerRound: {
      readonly new_: u32;
    } & Struct;
    readonly isJoinCandidates: boolean;
    readonly asJoinCandidates: {
      readonly bond: u128;
      readonly candidateCount: u32;
    } & Struct;
    readonly isScheduleLeaveCandidates: boolean;
    readonly asScheduleLeaveCandidates: {
      readonly candidateCount: u32;
    } & Struct;
    readonly isExecuteLeaveCandidates: boolean;
    readonly asExecuteLeaveCandidates: {
      readonly candidate: AccountId20;
      readonly candidateDelegationCount: u32;
    } & Struct;
    readonly isCancelLeaveCandidates: boolean;
    readonly asCancelLeaveCandidates: {
      readonly candidateCount: u32;
    } & Struct;
    readonly isGoOffline: boolean;
    readonly isGoOnline: boolean;
    readonly isCandidateBondMore: boolean;
    readonly asCandidateBondMore: {
      readonly more: u128;
    } & Struct;
    readonly isScheduleCandidateBondLess: boolean;
    readonly asScheduleCandidateBondLess: {
      readonly less: u128;
    } & Struct;
    readonly isExecuteCandidateBondLess: boolean;
    readonly asExecuteCandidateBondLess: {
      readonly candidate: AccountId20;
    } & Struct;
    readonly isCancelCandidateBondLess: boolean;
    readonly isDelegate: boolean;
    readonly asDelegate: {
      readonly candidate: AccountId20;
      readonly amount: u128;
      readonly candidateDelegationCount: u32;
      readonly delegationCount: u32;
    } & Struct;
    readonly isScheduleLeaveDelegators: boolean;
    readonly isExecuteLeaveDelegators: boolean;
    readonly asExecuteLeaveDelegators: {
      readonly delegator: AccountId20;
      readonly delegationCount: u32;
    } & Struct;
    readonly isCancelLeaveDelegators: boolean;
    readonly isScheduleRevokeDelegation: boolean;
    readonly asScheduleRevokeDelegation: {
      readonly collator: AccountId20;
    } & Struct;
    readonly isDelegatorBondMore: boolean;
    readonly asDelegatorBondMore: {
      readonly candidate: AccountId20;
      readonly more: u128;
    } & Struct;
    readonly isScheduleDelegatorBondLess: boolean;
    readonly asScheduleDelegatorBondLess: {
      readonly candidate: AccountId20;
      readonly less: u128;
    } & Struct;
    readonly isExecuteDelegationRequest: boolean;
    readonly asExecuteDelegationRequest: {
      readonly delegator: AccountId20;
      readonly candidate: AccountId20;
    } & Struct;
    readonly isCancelDelegationRequest: boolean;
    readonly asCancelDelegationRequest: {
      readonly candidate: AccountId20;
    } & Struct;
    readonly type: 'HotfixRemoveDelegationRequests' | 'HotfixUpdateCandidatePoolValue' | 'SetStakingExpectations' | 'SetInflation' | 'SetParachainBondAccount' | 'SetParachainBondReservePercent' | 'SetTotalSelected' | 'SetCollatorCommission' | 'SetBlocksPerRound' | 'JoinCandidates' | 'ScheduleLeaveCandidates' | 'ExecuteLeaveCandidates' | 'CancelLeaveCandidates' | 'GoOffline' | 'GoOnline' | 'CandidateBondMore' | 'ScheduleCandidateBondLess' | 'ExecuteCandidateBondLess' | 'CancelCandidateBondLess' | 'Delegate' | 'ScheduleLeaveDelegators' | 'ExecuteLeaveDelegators' | 'CancelLeaveDelegators' | 'ScheduleRevokeDelegation' | 'DelegatorBondMore' | 'ScheduleDelegatorBondLess' | 'ExecuteDelegationRequest' | 'CancelDelegationRequest';
  }

  /** @name ParachainStakingInflationRangeU128 (183) */
  export interface ParachainStakingInflationRangeU128 extends Struct {
    readonly min: u128;
    readonly ideal: u128;
    readonly max: u128;
  }

  /** @name ParachainStakingInflationRangePerbill (184) */
  export interface ParachainStakingInflationRangePerbill extends Struct {
    readonly min: Perbill;
    readonly ideal: Perbill;
    readonly max: Perbill;
  }

  /** @name PalletSchedulerCall (185) */
  export interface PalletSchedulerCall extends Enum {
    readonly isSchedule: boolean;
    readonly asSchedule: {
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancel: boolean;
    readonly asCancel: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isScheduleNamed: boolean;
    readonly asScheduleNamed: {
      readonly id: Bytes;
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancelNamed: boolean;
    readonly asCancelNamed: {
      readonly id: Bytes;
    } & Struct;
    readonly isScheduleAfter: boolean;
    readonly asScheduleAfter: {
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isScheduleNamedAfter: boolean;
    readonly asScheduleNamedAfter: {
      readonly id: Bytes;
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly type: 'Schedule' | 'Cancel' | 'ScheduleNamed' | 'CancelNamed' | 'ScheduleAfter' | 'ScheduleNamedAfter';
  }

  /** @name PalletDemocracyCall (187) */
  export interface PalletDemocracyCall extends Enum {
    readonly isPropose: boolean;
    readonly asPropose: {
      readonly proposalHash: H256;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isSecond: boolean;
    readonly asSecond: {
      readonly proposal: Compact<u32>;
      readonly secondsUpperBound: Compact<u32>;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly refIndex: Compact<u32>;
      readonly vote: PalletDemocracyVoteAccountVote;
    } & Struct;
    readonly isEmergencyCancel: boolean;
    readonly asEmergencyCancel: {
      readonly refIndex: u32;
    } & Struct;
    readonly isExternalPropose: boolean;
    readonly asExternalPropose: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isExternalProposeMajority: boolean;
    readonly asExternalProposeMajority: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isExternalProposeDefault: boolean;
    readonly asExternalProposeDefault: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isFastTrack: boolean;
    readonly asFastTrack: {
      readonly proposalHash: H256;
      readonly votingPeriod: u32;
      readonly delay: u32;
    } & Struct;
    readonly isVetoExternal: boolean;
    readonly asVetoExternal: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isCancelReferendum: boolean;
    readonly asCancelReferendum: {
      readonly refIndex: Compact<u32>;
    } & Struct;
    readonly isCancelQueued: boolean;
    readonly asCancelQueued: {
      readonly which: u32;
    } & Struct;
    readonly isDelegate: boolean;
    readonly asDelegate: {
      readonly to: AccountId20;
      readonly conviction: PalletDemocracyConviction;
      readonly balance: u128;
    } & Struct;
    readonly isUndelegate: boolean;
    readonly isClearPublicProposals: boolean;
    readonly isNotePreimage: boolean;
    readonly asNotePreimage: {
      readonly encodedProposal: Bytes;
    } & Struct;
    readonly isNotePreimageOperational: boolean;
    readonly asNotePreimageOperational: {
      readonly encodedProposal: Bytes;
    } & Struct;
    readonly isNoteImminentPreimage: boolean;
    readonly asNoteImminentPreimage: {
      readonly encodedProposal: Bytes;
    } & Struct;
    readonly isNoteImminentPreimageOperational: boolean;
    readonly asNoteImminentPreimageOperational: {
      readonly encodedProposal: Bytes;
    } & Struct;
    readonly isReapPreimage: boolean;
    readonly asReapPreimage: {
      readonly proposalHash: H256;
      readonly proposalLenUpperBound: Compact<u32>;
    } & Struct;
    readonly isUnlock: boolean;
    readonly asUnlock: {
      readonly target: AccountId20;
    } & Struct;
    readonly isRemoveVote: boolean;
    readonly asRemoveVote: {
      readonly index: u32;
    } & Struct;
    readonly isRemoveOtherVote: boolean;
    readonly asRemoveOtherVote: {
      readonly target: AccountId20;
      readonly index: u32;
    } & Struct;
    readonly isEnactProposal: boolean;
    readonly asEnactProposal: {
      readonly proposalHash: H256;
      readonly index: u32;
    } & Struct;
    readonly isBlacklist: boolean;
    readonly asBlacklist: {
      readonly proposalHash: H256;
      readonly maybeRefIndex: Option<u32>;
    } & Struct;
    readonly isCancelProposal: boolean;
    readonly asCancelProposal: {
      readonly propIndex: Compact<u32>;
    } & Struct;
    readonly type: 'Propose' | 'Second' | 'Vote' | 'EmergencyCancel' | 'ExternalPropose' | 'ExternalProposeMajority' | 'ExternalProposeDefault' | 'FastTrack' | 'VetoExternal' | 'CancelReferendum' | 'CancelQueued' | 'Delegate' | 'Undelegate' | 'ClearPublicProposals' | 'NotePreimage' | 'NotePreimageOperational' | 'NoteImminentPreimage' | 'NoteImminentPreimageOperational' | 'ReapPreimage' | 'Unlock' | 'RemoveVote' | 'RemoveOtherVote' | 'EnactProposal' | 'Blacklist' | 'CancelProposal';
  }

  /** @name PalletDemocracyConviction (188) */
  export interface PalletDemocracyConviction extends Enum {
    readonly isNone: boolean;
    readonly isLocked1x: boolean;
    readonly isLocked2x: boolean;
    readonly isLocked3x: boolean;
    readonly isLocked4x: boolean;
    readonly isLocked5x: boolean;
    readonly isLocked6x: boolean;
    readonly type: 'None' | 'Locked1x' | 'Locked2x' | 'Locked3x' | 'Locked4x' | 'Locked5x' | 'Locked6x';
  }

  /** @name PalletCollectiveCall (190) */
  export interface PalletCollectiveCall extends Enum {
    readonly isSetMembers: boolean;
    readonly asSetMembers: {
      readonly newMembers: Vec<AccountId20>;
      readonly prime: Option<AccountId20>;
      readonly oldCount: u32;
    } & Struct;
    readonly isExecute: boolean;
    readonly asExecute: {
      readonly proposal: Call;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isPropose: boolean;
    readonly asPropose: {
      readonly threshold: Compact<u32>;
      readonly proposal: Call;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly proposal: H256;
      readonly index: Compact<u32>;
      readonly approve: bool;
    } & Struct;
    readonly isClose: boolean;
    readonly asClose: {
      readonly proposalHash: H256;
      readonly index: Compact<u32>;
      readonly proposalWeightBound: Compact<u64>;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isDisapproveProposal: boolean;
    readonly asDisapproveProposal: {
      readonly proposalHash: H256;
    } & Struct;
    readonly type: 'SetMembers' | 'Execute' | 'Propose' | 'Vote' | 'Close' | 'DisapproveProposal';
  }

  /** @name PalletTreasuryCall (192) */
  export interface PalletTreasuryCall extends Enum {
    readonly isProposeSpend: boolean;
    readonly asProposeSpend: {
      readonly value: Compact<u128>;
      readonly beneficiary: AccountId20;
    } & Struct;
    readonly isRejectProposal: boolean;
    readonly asRejectProposal: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly isApproveProposal: boolean;
    readonly asApproveProposal: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly type: 'ProposeSpend' | 'RejectProposal' | 'ApproveProposal';
  }

  /** @name PalletAuthorInherentCall (193) */
  export interface PalletAuthorInherentCall extends Enum {
    readonly isKickOffAuthorshipValidation: boolean;
    readonly type: 'KickOffAuthorshipValidation';
  }

  /** @name PalletAuthorSlotFilterCall (194) */
  export interface PalletAuthorSlotFilterCall extends Enum {
    readonly isSetEligible: boolean;
    readonly asSetEligible: {
      readonly new_: Percent;
    } & Struct;
    readonly type: 'SetEligible';
  }

  /** @name PalletCrowdloanRewardsCall (195) */
  export interface PalletCrowdloanRewardsCall extends Enum {
    readonly isAssociateNativeIdentity: boolean;
    readonly asAssociateNativeIdentity: {
      readonly rewardAccount: AccountId20;
      readonly relayAccount: U8aFixed;
      readonly proof: SpRuntimeMultiSignature;
    } & Struct;
    readonly isChangeAssociationWithRelayKeys: boolean;
    readonly asChangeAssociationWithRelayKeys: {
      readonly rewardAccount: AccountId20;
      readonly previousAccount: AccountId20;
      readonly proofs: Vec<ITuple<[U8aFixed, SpRuntimeMultiSignature]>>;
    } & Struct;
    readonly isClaim: boolean;
    readonly isUpdateRewardAddress: boolean;
    readonly asUpdateRewardAddress: {
      readonly newRewardAccount: AccountId20;
    } & Struct;
    readonly isCompleteInitialization: boolean;
    readonly asCompleteInitialization: {
      readonly leaseEndingBlock: u32;
    } & Struct;
    readonly isInitializeRewardVec: boolean;
    readonly asInitializeRewardVec: {
      readonly rewards: Vec<ITuple<[U8aFixed, Option<AccountId20>, u128]>>;
    } & Struct;
    readonly type: 'AssociateNativeIdentity' | 'ChangeAssociationWithRelayKeys' | 'Claim' | 'UpdateRewardAddress' | 'CompleteInitialization' | 'InitializeRewardVec';
  }

  /** @name SpRuntimeMultiSignature (196) */
  export interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean;
    readonly asEd25519: SpCoreEd25519Signature;
    readonly isSr25519: boolean;
    readonly asSr25519: SpCoreSr25519Signature;
    readonly isEcdsa: boolean;
    readonly asEcdsa: SpCoreEcdsaSignature;
    readonly type: 'Ed25519' | 'Sr25519' | 'Ecdsa';
  }

  /** @name SpCoreEd25519Signature (197) */
  export interface SpCoreEd25519Signature extends U8aFixed {}

  /** @name SpCoreSr25519Signature (199) */
  export interface SpCoreSr25519Signature extends U8aFixed {}

  /** @name SpCoreEcdsaSignature (200) */
  export interface SpCoreEcdsaSignature extends U8aFixed {}

  /** @name PalletAuthorMappingCall (206) */
  export interface PalletAuthorMappingCall extends Enum {
    readonly isAddAssociation: boolean;
    readonly asAddAssociation: {
      readonly authorId: NimbusPrimitivesNimbusCryptoPublic;
    } & Struct;
    readonly isUpdateAssociation: boolean;
    readonly asUpdateAssociation: {
      readonly oldAuthorId: NimbusPrimitivesNimbusCryptoPublic;
      readonly newAuthorId: NimbusPrimitivesNimbusCryptoPublic;
    } & Struct;
    readonly isClearAssociation: boolean;
    readonly asClearAssociation: {
      readonly authorId: NimbusPrimitivesNimbusCryptoPublic;
    } & Struct;
    readonly type: 'AddAssociation' | 'UpdateAssociation' | 'ClearAssociation';
  }

  /** @name PalletProxyCall (207) */
  export interface PalletProxyCall extends Enum {
    readonly isProxy: boolean;
    readonly asProxy: {
      readonly real: AccountId20;
      readonly forceProxyType: Option<MoonbaseRuntimeProxyType>;
      readonly call: Call;
    } & Struct;
    readonly isAddProxy: boolean;
    readonly asAddProxy: {
      readonly delegate: AccountId20;
      readonly proxyType: MoonbaseRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly isRemoveProxy: boolean;
    readonly asRemoveProxy: {
      readonly delegate: AccountId20;
      readonly proxyType: MoonbaseRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly isRemoveProxies: boolean;
    readonly isAnonymous: boolean;
    readonly asAnonymous: {
      readonly proxyType: MoonbaseRuntimeProxyType;
      readonly delay: u32;
      readonly index: u16;
    } & Struct;
    readonly isKillAnonymous: boolean;
    readonly asKillAnonymous: {
      readonly spawner: AccountId20;
      readonly proxyType: MoonbaseRuntimeProxyType;
      readonly index: u16;
      readonly height: Compact<u32>;
      readonly extIndex: Compact<u32>;
    } & Struct;
    readonly isAnnounce: boolean;
    readonly asAnnounce: {
      readonly real: AccountId20;
      readonly callHash: H256;
    } & Struct;
    readonly isRemoveAnnouncement: boolean;
    readonly asRemoveAnnouncement: {
      readonly real: AccountId20;
      readonly callHash: H256;
    } & Struct;
    readonly isRejectAnnouncement: boolean;
    readonly asRejectAnnouncement: {
      readonly delegate: AccountId20;
      readonly callHash: H256;
    } & Struct;
    readonly isProxyAnnounced: boolean;
    readonly asProxyAnnounced: {
      readonly delegate: AccountId20;
      readonly real: AccountId20;
      readonly forceProxyType: Option<MoonbaseRuntimeProxyType>;
      readonly call: Call;
    } & Struct;
    readonly type: 'Proxy' | 'AddProxy' | 'RemoveProxy' | 'RemoveProxies' | 'Anonymous' | 'KillAnonymous' | 'Announce' | 'RemoveAnnouncement' | 'RejectAnnouncement' | 'ProxyAnnounced';
  }

  /** @name PalletMaintenanceModeCall (209) */
  export interface PalletMaintenanceModeCall extends Enum {
    readonly isEnterMaintenanceMode: boolean;
    readonly isResumeNormalOperation: boolean;
    readonly type: 'EnterMaintenanceMode' | 'ResumeNormalOperation';
  }

  /** @name PalletIdentityCall (210) */
  export interface PalletIdentityCall extends Enum {
    readonly isAddRegistrar: boolean;
    readonly asAddRegistrar: {
      readonly account: AccountId20;
    } & Struct;
    readonly isSetIdentity: boolean;
    readonly asSetIdentity: {
      readonly info: PalletIdentityIdentityInfo;
    } & Struct;
    readonly isSetSubs: boolean;
    readonly asSetSubs: {
      readonly subs: Vec<ITuple<[AccountId20, Data]>>;
    } & Struct;
    readonly isClearIdentity: boolean;
    readonly isRequestJudgement: boolean;
    readonly asRequestJudgement: {
      readonly regIndex: Compact<u32>;
      readonly maxFee: Compact<u128>;
    } & Struct;
    readonly isCancelRequest: boolean;
    readonly asCancelRequest: {
      readonly regIndex: u32;
    } & Struct;
    readonly isSetFee: boolean;
    readonly asSetFee: {
      readonly index: Compact<u32>;
      readonly fee: Compact<u128>;
    } & Struct;
    readonly isSetAccountId: boolean;
    readonly asSetAccountId: {
      readonly index: Compact<u32>;
      readonly new_: AccountId20;
    } & Struct;
    readonly isSetFields: boolean;
    readonly asSetFields: {
      readonly index: Compact<u32>;
      readonly fields: PalletIdentityBitFlags;
    } & Struct;
    readonly isProvideJudgement: boolean;
    readonly asProvideJudgement: {
      readonly regIndex: Compact<u32>;
      readonly target: AccountId20;
      readonly judgement: PalletIdentityJudgement;
    } & Struct;
    readonly isKillIdentity: boolean;
    readonly asKillIdentity: {
      readonly target: AccountId20;
    } & Struct;
    readonly isAddSub: boolean;
    readonly asAddSub: {
      readonly sub: AccountId20;
      readonly data: Data;
    } & Struct;
    readonly isRenameSub: boolean;
    readonly asRenameSub: {
      readonly sub: AccountId20;
      readonly data: Data;
    } & Struct;
    readonly isRemoveSub: boolean;
    readonly asRemoveSub: {
      readonly sub: AccountId20;
    } & Struct;
    readonly isQuitSub: boolean;
    readonly type: 'AddRegistrar' | 'SetIdentity' | 'SetSubs' | 'ClearIdentity' | 'RequestJudgement' | 'CancelRequest' | 'SetFee' | 'SetAccountId' | 'SetFields' | 'ProvideJudgement' | 'KillIdentity' | 'AddSub' | 'RenameSub' | 'RemoveSub' | 'QuitSub';
  }

  /** @name PalletIdentityIdentityInfo (211) */
  export interface PalletIdentityIdentityInfo extends Struct {
    readonly additional: Vec<ITuple<[Data, Data]>>;
    readonly display: Data;
    readonly legal: Data;
    readonly web: Data;
    readonly riot: Data;
    readonly email: Data;
    readonly pgpFingerprint: Option<U8aFixed>;
    readonly image: Data;
    readonly twitter: Data;
  }

  /** @name PalletIdentityBitFlags (247) */
  export interface PalletIdentityBitFlags extends Set {
    readonly isDisplay: boolean;
    readonly isLegal: boolean;
    readonly isWeb: boolean;
    readonly isRiot: boolean;
    readonly isEmail: boolean;
    readonly isPgpFingerprint: boolean;
    readonly isImage: boolean;
    readonly isTwitter: boolean;
  }

  /** @name PalletIdentityIdentityField (248) */
  export interface PalletIdentityIdentityField extends Enum {
    readonly isDisplay: boolean;
    readonly isLegal: boolean;
    readonly isWeb: boolean;
    readonly isRiot: boolean;
    readonly isEmail: boolean;
    readonly isPgpFingerprint: boolean;
    readonly isImage: boolean;
    readonly isTwitter: boolean;
    readonly type: 'Display' | 'Legal' | 'Web' | 'Riot' | 'Email' | 'PgpFingerprint' | 'Image' | 'Twitter';
  }

  /** @name PalletIdentityJudgement (249) */
  export interface PalletIdentityJudgement extends Enum {
    readonly isUnknown: boolean;
    readonly isFeePaid: boolean;
    readonly asFeePaid: u128;
    readonly isReasonable: boolean;
    readonly isKnownGood: boolean;
    readonly isOutOfDate: boolean;
    readonly isLowQuality: boolean;
    readonly isErroneous: boolean;
    readonly type: 'Unknown' | 'FeePaid' | 'Reasonable' | 'KnownGood' | 'OutOfDate' | 'LowQuality' | 'Erroneous';
  }

  /** @name CumulusPalletXcmpQueueCall (250) */
  export type CumulusPalletXcmpQueueCall = Null;

  /** @name CumulusPalletDmpQueueCall (251) */
  export interface CumulusPalletDmpQueueCall extends Enum {
    readonly isServiceOverweight: boolean;
    readonly asServiceOverweight: {
      readonly index: u64;
      readonly weightLimit: u64;
    } & Struct;
    readonly type: 'ServiceOverweight';
  }

  /** @name PalletXcmCall (252) */
  export interface PalletXcmCall extends Enum {
    readonly isSend: boolean;
    readonly asSend: {
      readonly dest: XcmVersionedMultiLocation;
      readonly message: XcmVersionedXcm;
    } & Struct;
    readonly isTeleportAssets: boolean;
    readonly asTeleportAssets: {
      readonly dest: XcmVersionedMultiLocation;
      readonly beneficiary: XcmVersionedMultiLocation;
      readonly assets: XcmVersionedMultiAssets;
      readonly feeAssetItem: u32;
    } & Struct;
    readonly isReserveTransferAssets: boolean;
    readonly asReserveTransferAssets: {
      readonly dest: XcmVersionedMultiLocation;
      readonly beneficiary: XcmVersionedMultiLocation;
      readonly assets: XcmVersionedMultiAssets;
      readonly feeAssetItem: u32;
    } & Struct;
    readonly isExecute: boolean;
    readonly asExecute: {
      readonly message: XcmVersionedXcm;
      readonly maxWeight: u64;
    } & Struct;
    readonly isForceXcmVersion: boolean;
    readonly asForceXcmVersion: {
      readonly location: XcmV1MultiLocation;
      readonly xcmVersion: u32;
    } & Struct;
    readonly isForceDefaultXcmVersion: boolean;
    readonly asForceDefaultXcmVersion: {
      readonly maybeXcmVersion: Option<u32>;
    } & Struct;
    readonly isForceSubscribeVersionNotify: boolean;
    readonly asForceSubscribeVersionNotify: {
      readonly location: XcmVersionedMultiLocation;
    } & Struct;
    readonly isForceUnsubscribeVersionNotify: boolean;
    readonly asForceUnsubscribeVersionNotify: {
      readonly location: XcmVersionedMultiLocation;
    } & Struct;
    readonly isLimitedReserveTransferAssets: boolean;
    readonly asLimitedReserveTransferAssets: {
      readonly dest: XcmVersionedMultiLocation;
      readonly beneficiary: XcmVersionedMultiLocation;
      readonly assets: XcmVersionedMultiAssets;
      readonly feeAssetItem: u32;
      readonly weightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly isLimitedTeleportAssets: boolean;
    readonly asLimitedTeleportAssets: {
      readonly dest: XcmVersionedMultiLocation;
      readonly beneficiary: XcmVersionedMultiLocation;
      readonly assets: XcmVersionedMultiAssets;
      readonly feeAssetItem: u32;
      readonly weightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly type: 'Send' | 'TeleportAssets' | 'ReserveTransferAssets' | 'Execute' | 'ForceXcmVersion' | 'ForceDefaultXcmVersion' | 'ForceSubscribeVersionNotify' | 'ForceUnsubscribeVersionNotify' | 'LimitedReserveTransferAssets' | 'LimitedTeleportAssets';
  }

  /** @name XcmVersionedXcm (253) */
  export interface XcmVersionedXcm extends Enum {
    readonly isV0: boolean;
    readonly asV0: XcmV0Xcm;
    readonly isV1: boolean;
    readonly asV1: XcmV1Xcm;
    readonly isV2: boolean;
    readonly asV2: XcmV2Xcm;
    readonly type: 'V0' | 'V1' | 'V2';
  }

  /** @name XcmV0Xcm (254) */
  export interface XcmV0Xcm extends Enum {
    readonly isWithdrawAsset: boolean;
    readonly asWithdrawAsset: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly effects: Vec<XcmV0Order>;
    } & Struct;
    readonly isReserveAssetDeposit: boolean;
    readonly asReserveAssetDeposit: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly effects: Vec<XcmV0Order>;
    } & Struct;
    readonly isTeleportAsset: boolean;
    readonly asTeleportAsset: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly effects: Vec<XcmV0Order>;
    } & Struct;
    readonly isQueryResponse: boolean;
    readonly asQueryResponse: {
      readonly queryId: Compact<u64>;
      readonly response: XcmV0Response;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly dest: XcmV0MultiLocation;
    } & Struct;
    readonly isTransferReserveAsset: boolean;
    readonly asTransferReserveAsset: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly dest: XcmV0MultiLocation;
      readonly effects: Vec<XcmV0Order>;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly originType: XcmV0OriginKind;
      readonly requireWeightAtMost: u64;
      readonly call: XcmDoubleEncoded;
    } & Struct;
    readonly isHrmpNewChannelOpenRequest: boolean;
    readonly asHrmpNewChannelOpenRequest: {
      readonly sender: Compact<u32>;
      readonly maxMessageSize: Compact<u32>;
      readonly maxCapacity: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelAccepted: boolean;
    readonly asHrmpChannelAccepted: {
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelClosing: boolean;
    readonly asHrmpChannelClosing: {
      readonly initiator: Compact<u32>;
      readonly sender: Compact<u32>;
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isRelayedFrom: boolean;
    readonly asRelayedFrom: {
      readonly who: XcmV0MultiLocation;
      readonly message: XcmV0Xcm;
    } & Struct;
    readonly type: 'WithdrawAsset' | 'ReserveAssetDeposit' | 'TeleportAsset' | 'QueryResponse' | 'TransferAsset' | 'TransferReserveAsset' | 'Transact' | 'HrmpNewChannelOpenRequest' | 'HrmpChannelAccepted' | 'HrmpChannelClosing' | 'RelayedFrom';
  }

  /** @name XcmV0Order (256) */
  export interface XcmV0Order extends Enum {
    readonly isNull: boolean;
    readonly isDepositAsset: boolean;
    readonly asDepositAsset: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly dest: XcmV0MultiLocation;
    } & Struct;
    readonly isDepositReserveAsset: boolean;
    readonly asDepositReserveAsset: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly dest: XcmV0MultiLocation;
      readonly effects: Vec<XcmV0Order>;
    } & Struct;
    readonly isExchangeAsset: boolean;
    readonly asExchangeAsset: {
      readonly give: Vec<XcmV0MultiAsset>;
      readonly receive: Vec<XcmV0MultiAsset>;
    } & Struct;
    readonly isInitiateReserveWithdraw: boolean;
    readonly asInitiateReserveWithdraw: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly reserve: XcmV0MultiLocation;
      readonly effects: Vec<XcmV0Order>;
    } & Struct;
    readonly isInitiateTeleport: boolean;
    readonly asInitiateTeleport: {
      readonly assets: Vec<XcmV0MultiAsset>;
      readonly dest: XcmV0MultiLocation;
      readonly effects: Vec<XcmV0Order>;
    } & Struct;
    readonly isQueryHolding: boolean;
    readonly asQueryHolding: {
      readonly queryId: Compact<u64>;
      readonly dest: XcmV0MultiLocation;
      readonly assets: Vec<XcmV0MultiAsset>;
    } & Struct;
    readonly isBuyExecution: boolean;
    readonly asBuyExecution: {
      readonly fees: XcmV0MultiAsset;
      readonly weight: u64;
      readonly debt: u64;
      readonly haltOnError: bool;
      readonly xcm: Vec<XcmV0Xcm>;
    } & Struct;
    readonly type: 'Null' | 'DepositAsset' | 'DepositReserveAsset' | 'ExchangeAsset' | 'InitiateReserveWithdraw' | 'InitiateTeleport' | 'QueryHolding' | 'BuyExecution';
  }

  /** @name XcmV0Response (258) */
  export interface XcmV0Response extends Enum {
    readonly isAssets: boolean;
    readonly asAssets: Vec<XcmV0MultiAsset>;
    readonly type: 'Assets';
  }

  /** @name XcmV1Xcm (259) */
  export interface XcmV1Xcm extends Enum {
    readonly isWithdrawAsset: boolean;
    readonly asWithdrawAsset: {
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly effects: Vec<XcmV1Order>;
    } & Struct;
    readonly isReserveAssetDeposited: boolean;
    readonly asReserveAssetDeposited: {
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly effects: Vec<XcmV1Order>;
    } & Struct;
    readonly isReceiveTeleportedAsset: boolean;
    readonly asReceiveTeleportedAsset: {
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly effects: Vec<XcmV1Order>;
    } & Struct;
    readonly isQueryResponse: boolean;
    readonly asQueryResponse: {
      readonly queryId: Compact<u64>;
      readonly response: XcmV1Response;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly beneficiary: XcmV1MultiLocation;
    } & Struct;
    readonly isTransferReserveAsset: boolean;
    readonly asTransferReserveAsset: {
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly dest: XcmV1MultiLocation;
      readonly effects: Vec<XcmV1Order>;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly originType: XcmV0OriginKind;
      readonly requireWeightAtMost: u64;
      readonly call: XcmDoubleEncoded;
    } & Struct;
    readonly isHrmpNewChannelOpenRequest: boolean;
    readonly asHrmpNewChannelOpenRequest: {
      readonly sender: Compact<u32>;
      readonly maxMessageSize: Compact<u32>;
      readonly maxCapacity: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelAccepted: boolean;
    readonly asHrmpChannelAccepted: {
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelClosing: boolean;
    readonly asHrmpChannelClosing: {
      readonly initiator: Compact<u32>;
      readonly sender: Compact<u32>;
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isRelayedFrom: boolean;
    readonly asRelayedFrom: {
      readonly who: XcmV1MultilocationJunctions;
      readonly message: XcmV1Xcm;
    } & Struct;
    readonly isSubscribeVersion: boolean;
    readonly asSubscribeVersion: {
      readonly queryId: Compact<u64>;
      readonly maxResponseWeight: Compact<u64>;
    } & Struct;
    readonly isUnsubscribeVersion: boolean;
    readonly type: 'WithdrawAsset' | 'ReserveAssetDeposited' | 'ReceiveTeleportedAsset' | 'QueryResponse' | 'TransferAsset' | 'TransferReserveAsset' | 'Transact' | 'HrmpNewChannelOpenRequest' | 'HrmpChannelAccepted' | 'HrmpChannelClosing' | 'RelayedFrom' | 'SubscribeVersion' | 'UnsubscribeVersion';
  }

  /** @name XcmV1Order (261) */
  export interface XcmV1Order extends Enum {
    readonly isNoop: boolean;
    readonly isDepositAsset: boolean;
    readonly asDepositAsset: {
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly maxAssets: u32;
      readonly beneficiary: XcmV1MultiLocation;
    } & Struct;
    readonly isDepositReserveAsset: boolean;
    readonly asDepositReserveAsset: {
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly maxAssets: u32;
      readonly dest: XcmV1MultiLocation;
      readonly effects: Vec<XcmV1Order>;
    } & Struct;
    readonly isExchangeAsset: boolean;
    readonly asExchangeAsset: {
      readonly give: XcmV1MultiassetMultiAssetFilter;
      readonly receive: XcmV1MultiassetMultiAssets;
    } & Struct;
    readonly isInitiateReserveWithdraw: boolean;
    readonly asInitiateReserveWithdraw: {
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly reserve: XcmV1MultiLocation;
      readonly effects: Vec<XcmV1Order>;
    } & Struct;
    readonly isInitiateTeleport: boolean;
    readonly asInitiateTeleport: {
      readonly assets: XcmV1MultiassetMultiAssetFilter;
      readonly dest: XcmV1MultiLocation;
      readonly effects: Vec<XcmV1Order>;
    } & Struct;
    readonly isQueryHolding: boolean;
    readonly asQueryHolding: {
      readonly queryId: Compact<u64>;
      readonly dest: XcmV1MultiLocation;
      readonly assets: XcmV1MultiassetMultiAssetFilter;
    } & Struct;
    readonly isBuyExecution: boolean;
    readonly asBuyExecution: {
      readonly fees: XcmV1MultiAsset;
      readonly weight: u64;
      readonly debt: u64;
      readonly haltOnError: bool;
      readonly instructions: Vec<XcmV1Xcm>;
    } & Struct;
    readonly type: 'Noop' | 'DepositAsset' | 'DepositReserveAsset' | 'ExchangeAsset' | 'InitiateReserveWithdraw' | 'InitiateTeleport' | 'QueryHolding' | 'BuyExecution';
  }

  /** @name XcmV1Response (263) */
  export interface XcmV1Response extends Enum {
    readonly isAssets: boolean;
    readonly asAssets: XcmV1MultiassetMultiAssets;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly type: 'Assets' | 'Version';
  }

  /** @name PalletAssetsCall (277) */
  export interface PalletAssetsCall extends Enum {
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly id: Compact<u128>;
      readonly admin: AccountId20;
      readonly minBalance: u128;
    } & Struct;
    readonly isForceCreate: boolean;
    readonly asForceCreate: {
      readonly id: Compact<u128>;
      readonly owner: AccountId20;
      readonly isSufficient: bool;
      readonly minBalance: Compact<u128>;
    } & Struct;
    readonly isDestroy: boolean;
    readonly asDestroy: {
      readonly id: Compact<u128>;
      readonly witness: PalletAssetsDestroyWitness;
    } & Struct;
    readonly isMint: boolean;
    readonly asMint: {
      readonly id: Compact<u128>;
      readonly beneficiary: AccountId20;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isBurn: boolean;
    readonly asBurn: {
      readonly id: Compact<u128>;
      readonly who: AccountId20;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly id: Compact<u128>;
      readonly target: AccountId20;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isTransferKeepAlive: boolean;
    readonly asTransferKeepAlive: {
      readonly id: Compact<u128>;
      readonly target: AccountId20;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly id: Compact<u128>;
      readonly source: AccountId20;
      readonly dest: AccountId20;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isFreeze: boolean;
    readonly asFreeze: {
      readonly id: Compact<u128>;
      readonly who: AccountId20;
    } & Struct;
    readonly isThaw: boolean;
    readonly asThaw: {
      readonly id: Compact<u128>;
      readonly who: AccountId20;
    } & Struct;
    readonly isFreezeAsset: boolean;
    readonly asFreezeAsset: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isThawAsset: boolean;
    readonly asThawAsset: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isTransferOwnership: boolean;
    readonly asTransferOwnership: {
      readonly id: Compact<u128>;
      readonly owner: AccountId20;
    } & Struct;
    readonly isSetTeam: boolean;
    readonly asSetTeam: {
      readonly id: Compact<u128>;
      readonly issuer: AccountId20;
      readonly admin: AccountId20;
      readonly freezer: AccountId20;
    } & Struct;
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly id: Compact<u128>;
      readonly name: Bytes;
      readonly symbol: Bytes;
      readonly decimals: u8;
    } & Struct;
    readonly isClearMetadata: boolean;
    readonly asClearMetadata: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isForceSetMetadata: boolean;
    readonly asForceSetMetadata: {
      readonly id: Compact<u128>;
      readonly name: Bytes;
      readonly symbol: Bytes;
      readonly decimals: u8;
      readonly isFrozen: bool;
    } & Struct;
    readonly isForceClearMetadata: boolean;
    readonly asForceClearMetadata: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isForceAssetStatus: boolean;
    readonly asForceAssetStatus: {
      readonly id: Compact<u128>;
      readonly owner: AccountId20;
      readonly issuer: AccountId20;
      readonly admin: AccountId20;
      readonly freezer: AccountId20;
      readonly minBalance: Compact<u128>;
      readonly isSufficient: bool;
      readonly isFrozen: bool;
    } & Struct;
    readonly isApproveTransfer: boolean;
    readonly asApproveTransfer: {
      readonly id: Compact<u128>;
      readonly delegate: AccountId20;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isCancelApproval: boolean;
    readonly asCancelApproval: {
      readonly id: Compact<u128>;
      readonly delegate: AccountId20;
    } & Struct;
    readonly isForceCancelApproval: boolean;
    readonly asForceCancelApproval: {
      readonly id: Compact<u128>;
      readonly owner: AccountId20;
      readonly delegate: AccountId20;
    } & Struct;
    readonly isTransferApproved: boolean;
    readonly asTransferApproved: {
      readonly id: Compact<u128>;
      readonly owner: AccountId20;
      readonly destination: AccountId20;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly type: 'Create' | 'ForceCreate' | 'Destroy' | 'Mint' | 'Burn' | 'Transfer' | 'TransferKeepAlive' | 'ForceTransfer' | 'Freeze' | 'Thaw' | 'FreezeAsset' | 'ThawAsset' | 'TransferOwnership' | 'SetTeam' | 'SetMetadata' | 'ClearMetadata' | 'ForceSetMetadata' | 'ForceClearMetadata' | 'ForceAssetStatus' | 'ApproveTransfer' | 'CancelApproval' | 'ForceCancelApproval' | 'TransferApproved';
  }

  /** @name PalletAssetsDestroyWitness (278) */
  export interface PalletAssetsDestroyWitness extends Struct {
    readonly accounts: Compact<u32>;
    readonly sufficients: Compact<u32>;
    readonly approvals: Compact<u32>;
  }

  /** @name OrmlXtokensModuleCall (279) */
  export interface OrmlXtokensModuleCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly currencyId: MoonbaseRuntimeCurrencyId;
      readonly amount: u128;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeight: u64;
    } & Struct;
    readonly isTransferMultiasset: boolean;
    readonly asTransferMultiasset: {
      readonly asset: XcmVersionedMultiAsset;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeight: u64;
    } & Struct;
    readonly isTransferWithFee: boolean;
    readonly asTransferWithFee: {
      readonly currencyId: MoonbaseRuntimeCurrencyId;
      readonly amount: u128;
      readonly fee: u128;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeight: u64;
    } & Struct;
    readonly isTransferMultiassetWithFee: boolean;
    readonly asTransferMultiassetWithFee: {
      readonly asset: XcmVersionedMultiAsset;
      readonly fee: XcmVersionedMultiAsset;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeight: u64;
    } & Struct;
    readonly type: 'Transfer' | 'TransferMultiasset' | 'TransferWithFee' | 'TransferMultiassetWithFee';
  }

  /** @name XcmVersionedMultiAsset (280) */
  export interface XcmVersionedMultiAsset extends Enum {
    readonly isV0: boolean;
    readonly asV0: XcmV0MultiAsset;
    readonly isV1: boolean;
    readonly asV1: XcmV1MultiAsset;
    readonly type: 'V0' | 'V1';
  }

  /** @name PalletAssetManagerCall (281) */
  export interface PalletAssetManagerCall extends Enum {
    readonly isRegisterAsset: boolean;
    readonly asRegisterAsset: {
      readonly asset: MoonbaseRuntimeAssetType;
      readonly metadata: MoonbaseRuntimeAssetRegistrarMetadata;
      readonly minAmount: u128;
      readonly isSufficient: bool;
    } & Struct;
    readonly isSetAssetUnitsPerSecond: boolean;
    readonly asSetAssetUnitsPerSecond: {
      readonly assetType: MoonbaseRuntimeAssetType;
      readonly unitsPerSecond: u128;
    } & Struct;
    readonly isChangeExistingAssetType: boolean;
    readonly asChangeExistingAssetType: {
      readonly assetId: u128;
      readonly newAssetType: MoonbaseRuntimeAssetType;
    } & Struct;
    readonly type: 'RegisterAsset' | 'SetAssetUnitsPerSecond' | 'ChangeExistingAssetType';
  }

  /** @name XcmTransactorCall (282) */
  export interface XcmTransactorCall extends Enum {
    readonly isRegister: boolean;
    readonly asRegister: {
      readonly who: AccountId20;
      readonly index: u16;
    } & Struct;
    readonly isTransactThroughDerivativeMultilocation: boolean;
    readonly asTransactThroughDerivativeMultilocation: {
      readonly dest: MoonbaseRuntimeTransactors;
      readonly index: u16;
      readonly feeLocation: XcmVersionedMultiLocation;
      readonly destWeight: u64;
      readonly innerCall: Bytes;
    } & Struct;
    readonly isTransactThroughDerivative: boolean;
    readonly asTransactThroughDerivative: {
      readonly dest: MoonbaseRuntimeTransactors;
      readonly index: u16;
      readonly currencyId: MoonbaseRuntimeCurrencyId;
      readonly destWeight: u64;
      readonly innerCall: Bytes;
    } & Struct;
    readonly isTransactThroughSovereign: boolean;
    readonly asTransactThroughSovereign: {
      readonly dest: XcmVersionedMultiLocation;
      readonly feePayer: AccountId20;
      readonly feeLocation: XcmVersionedMultiLocation;
      readonly destWeight: u64;
      readonly call: Bytes;
    } & Struct;
    readonly isSetTransactInfo: boolean;
    readonly asSetTransactInfo: {
      readonly location: XcmVersionedMultiLocation;
      readonly transactExtraWeight: u64;
      readonly feePerSecond: u128;
      readonly maxWeight: u64;
    } & Struct;
    readonly type: 'Register' | 'TransactThroughDerivativeMultilocation' | 'TransactThroughDerivative' | 'TransactThroughSovereign' | 'SetTransactInfo';
  }

  /** @name MoonbaseRuntimeTransactors (283) */
  export interface MoonbaseRuntimeTransactors extends Enum {
    readonly isRelay: boolean;
    readonly type: 'Relay';
  }

  /** @name PalletBaseFeeCall (284) */
  export interface PalletBaseFeeCall extends Enum {
    readonly isSetBaseFeePerGas: boolean;
    readonly asSetBaseFeePerGas: {
      readonly fee: U256;
    } & Struct;
    readonly isSetIsActive: boolean;
    readonly asSetIsActive: {
      readonly isActive: bool;
    } & Struct;
    readonly isSetElasticity: boolean;
    readonly asSetElasticity: {
      readonly elasticity: Permill;
    } & Struct;
    readonly type: 'SetBaseFeePerGas' | 'SetIsActive' | 'SetElasticity';
  }

  /** @name MoonbaseRuntimeOriginCaller (285) */
  export interface MoonbaseRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean;
    readonly asSystem: FrameSystemRawOrigin;
    readonly isVoid: boolean;
    readonly isEthereum: boolean;
    readonly asEthereum: PalletEthereumRawOrigin;
    readonly isCouncilCollective: boolean;
    readonly asCouncilCollective: PalletCollectiveRawOrigin;
    readonly isTechCommitteeCollective: boolean;
    readonly asTechCommitteeCollective: PalletCollectiveRawOrigin;
    readonly isCumulusXcm: boolean;
    readonly asCumulusXcm: CumulusPalletXcmOrigin;
    readonly isPolkadotXcm: boolean;
    readonly asPolkadotXcm: PalletXcmOrigin;
    readonly type: 'System' | 'Void' | 'Ethereum' | 'CouncilCollective' | 'TechCommitteeCollective' | 'CumulusXcm' | 'PolkadotXcm';
  }

  /** @name FrameSystemRawOrigin (286) */
  export interface FrameSystemRawOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId20;
    readonly isNone: boolean;
    readonly type: 'Root' | 'Signed' | 'None';
  }

  /** @name PalletEthereumRawOrigin (287) */
  export interface PalletEthereumRawOrigin extends Enum {
    readonly isEthereumTransaction: boolean;
    readonly asEthereumTransaction: H160;
    readonly type: 'EthereumTransaction';
  }

  /** @name PalletCollectiveRawOrigin (288) */
  export interface PalletCollectiveRawOrigin extends Enum {
    readonly isMembers: boolean;
    readonly asMembers: ITuple<[u32, u32]>;
    readonly isMember: boolean;
    readonly asMember: AccountId20;
    readonly isPhantom: boolean;
    readonly type: 'Members' | 'Member' | 'Phantom';
  }

  /** @name CumulusPalletXcmOrigin (290) */
  export interface CumulusPalletXcmOrigin extends Enum {
    readonly isRelay: boolean;
    readonly isSiblingParachain: boolean;
    readonly asSiblingParachain: u32;
    readonly type: 'Relay' | 'SiblingParachain';
  }

  /** @name PalletXcmOrigin (291) */
  export interface PalletXcmOrigin extends Enum {
    readonly isXcm: boolean;
    readonly asXcm: XcmV1MultiLocation;
    readonly isResponse: boolean;
    readonly asResponse: XcmV1MultiLocation;
    readonly type: 'Xcm' | 'Response';
  }

  /** @name SpCoreVoid (292) */
  export type SpCoreVoid = Null;

  /** @name PalletUtilityError (293) */
  export interface PalletUtilityError extends Enum {
    readonly isTooManyCalls: boolean;
    readonly type: 'TooManyCalls';
  }

  /** @name PalletBalancesBalanceLock (295) */
  export interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
    readonly reasons: PalletBalancesReasons;
  }

  /** @name PalletBalancesReasons (296) */
  export interface PalletBalancesReasons extends Enum {
    readonly isFee: boolean;
    readonly isMisc: boolean;
    readonly isAll: boolean;
    readonly type: 'Fee' | 'Misc' | 'All';
  }

  /** @name PalletBalancesReserveData (299) */
  export interface PalletBalancesReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name PalletBalancesReleases (301) */
  export interface PalletBalancesReleases extends Enum {
    readonly isV100: boolean;
    readonly isV200: boolean;
    readonly type: 'V100' | 'V200';
  }

  /** @name PalletBalancesError (302) */
  export interface PalletBalancesError extends Enum {
    readonly isVestingBalance: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isKeepAlive: boolean;
    readonly isExistingVestingSchedule: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly type: 'VestingBalance' | 'LiquidityRestrictions' | 'InsufficientBalance' | 'ExistentialDeposit' | 'KeepAlive' | 'ExistingVestingSchedule' | 'DeadAccount' | 'TooManyReserves';
  }

  /** @name PalletSudoError (303) */
  export interface PalletSudoError extends Enum {
    readonly isRequireSudo: boolean;
    readonly type: 'RequireSudo';
  }

  /** @name PolkadotPrimitivesV1UpgradeRestriction (305) */
  export interface PolkadotPrimitivesV1UpgradeRestriction extends Enum {
    readonly isPresent: boolean;
    readonly type: 'Present';
  }

  /** @name CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot (306) */
  export interface CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot extends Struct {
    readonly dmqMqcHead: H256;
    readonly relayDispatchQueueSize: ITuple<[u32, u32]>;
    readonly ingressChannels: Vec<ITuple<[u32, PolkadotPrimitivesV1AbridgedHrmpChannel]>>;
    readonly egressChannels: Vec<ITuple<[u32, PolkadotPrimitivesV1AbridgedHrmpChannel]>>;
  }

  /** @name PolkadotPrimitivesV1AbridgedHrmpChannel (309) */
  export interface PolkadotPrimitivesV1AbridgedHrmpChannel extends Struct {
    readonly maxCapacity: u32;
    readonly maxTotalSize: u32;
    readonly maxMessageSize: u32;
    readonly msgCount: u32;
    readonly totalSize: u32;
    readonly mqcHead: Option<H256>;
  }

  /** @name PolkadotPrimitivesV1AbridgedHostConfiguration (310) */
  export interface PolkadotPrimitivesV1AbridgedHostConfiguration extends Struct {
    readonly maxCodeSize: u32;
    readonly maxHeadDataSize: u32;
    readonly maxUpwardQueueCount: u32;
    readonly maxUpwardQueueSize: u32;
    readonly maxUpwardMessageSize: u32;
    readonly maxUpwardMessageNumPerCandidate: u32;
    readonly hrmpMaxMessageNumPerCandidate: u32;
    readonly validationUpgradeFrequency: u32;
    readonly validationUpgradeDelay: u32;
  }

  /** @name PolkadotCorePrimitivesOutboundHrmpMessage (316) */
  export interface PolkadotCorePrimitivesOutboundHrmpMessage extends Struct {
    readonly recipient: u32;
    readonly data: Bytes;
  }

  /** @name CumulusPalletParachainSystemError (317) */
  export interface CumulusPalletParachainSystemError extends Enum {
    readonly isOverlappingUpgrades: boolean;
    readonly isProhibitedByPolkadot: boolean;
    readonly isTooBig: boolean;
    readonly isValidationDataNotAvailable: boolean;
    readonly isHostConfigurationNotAvailable: boolean;
    readonly isNotScheduled: boolean;
    readonly isNothingAuthorized: boolean;
    readonly isUnauthorized: boolean;
    readonly type: 'OverlappingUpgrades' | 'ProhibitedByPolkadot' | 'TooBig' | 'ValidationDataNotAvailable' | 'HostConfigurationNotAvailable' | 'NotScheduled' | 'NothingAuthorized' | 'Unauthorized';
  }

  /** @name PalletTransactionPaymentReleases (319) */
  export interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean;
    readonly isV2: boolean;
    readonly type: 'V1Ancient' | 'V2';
  }

  /** @name FrameSupportWeightsWeightToFeeCoefficient (321) */
  export interface FrameSupportWeightsWeightToFeeCoefficient extends Struct {
    readonly coeffInteger: u128;
    readonly coeffFrac: Perbill;
    readonly negative: bool;
    readonly degree: u8;
  }

  /** @name PalletEvmError (323) */
  export interface PalletEvmError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isFeeOverflow: boolean;
    readonly isPaymentOverflow: boolean;
    readonly isWithdrawFailed: boolean;
    readonly isGasPriceTooLow: boolean;
    readonly isInvalidNonce: boolean;
    readonly type: 'BalanceLow' | 'FeeOverflow' | 'PaymentOverflow' | 'WithdrawFailed' | 'GasPriceTooLow' | 'InvalidNonce';
  }

  /** @name FpRpcTransactionStatus (326) */
  export interface FpRpcTransactionStatus extends Struct {
    readonly transactionHash: H256;
    readonly transactionIndex: u32;
    readonly from: H160;
    readonly to: Option<H160>;
    readonly contractAddress: Option<H160>;
    readonly logs: Vec<EthereumLog>;
    readonly logsBloom: EthbloomBloom;
  }

  /** @name EthbloomBloom (329) */
  export interface EthbloomBloom extends U8aFixed {}

  /** @name EthereumReceiptReceiptV3 (331) */
  export interface EthereumReceiptReceiptV3 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumReceiptEip658ReceiptData;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumReceiptEip658ReceiptData;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumReceiptEip658ReceiptData;
    readonly type: 'Legacy' | 'Eip2930' | 'Eip1559';
  }

  /** @name EthereumReceiptEip658ReceiptData (332) */
  export interface EthereumReceiptEip658ReceiptData extends Struct {
    readonly statusCode: u8;
    readonly usedGas: U256;
    readonly logsBloom: EthbloomBloom;
    readonly logs: Vec<EthereumLog>;
  }

  /** @name EthereumBlock (333) */
  export interface EthereumBlock extends Struct {
    readonly header: EthereumHeader;
    readonly transactions: Vec<EthereumTransactionTransactionV2>;
    readonly ommers: Vec<EthereumHeader>;
  }

  /** @name EthereumHeader (334) */
  export interface EthereumHeader extends Struct {
    readonly parentHash: H256;
    readonly ommersHash: H256;
    readonly beneficiary: H160;
    readonly stateRoot: H256;
    readonly transactionsRoot: H256;
    readonly receiptsRoot: H256;
    readonly logsBloom: EthbloomBloom;
    readonly difficulty: U256;
    readonly number: U256;
    readonly gasLimit: U256;
    readonly gasUsed: U256;
    readonly timestamp: u64;
    readonly extraData: Bytes;
    readonly mixHash: H256;
    readonly nonce: EthereumTypesHashH64;
  }

  /** @name EthereumTypesHashH64 (335) */
  export interface EthereumTypesHashH64 extends U8aFixed {}

  /** @name PalletEthereumError (340) */
  export interface PalletEthereumError extends Enum {
    readonly isInvalidSignature: boolean;
    readonly isPreLogExists: boolean;
    readonly type: 'InvalidSignature' | 'PreLogExists';
  }

  /** @name ParachainStakingParachainBondConfig (341) */
  export interface ParachainStakingParachainBondConfig extends Struct {
    readonly account: AccountId20;
    readonly percent: Percent;
  }

  /** @name ParachainStakingRoundInfo (342) */
  export interface ParachainStakingRoundInfo extends Struct {
    readonly current: u32;
    readonly first: u32;
    readonly length: u32;
  }

  /** @name ParachainStakingNominator2 (343) */
  export interface ParachainStakingNominator2 extends Struct {
    readonly delegations: ParachainStakingSetOrderedSetBond;
    readonly revocations: ParachainStakingSetOrderedSetAccountId20;
    readonly total: u128;
    readonly scheduledRevocationsCount: u32;
    readonly scheduledRevocationsTotal: u128;
    readonly status: ParachainStakingDelegatorStatus;
  }

  /** @name ParachainStakingSetOrderedSetBond (344) */
  export interface ParachainStakingSetOrderedSetBond extends Vec<ParachainStakingBond> {}

  /** @name ParachainStakingBond (345) */
  export interface ParachainStakingBond extends Struct {
    readonly owner: AccountId20;
    readonly amount: u128;
  }

  /** @name ParachainStakingSetOrderedSetAccountId20 (347) */
  export interface ParachainStakingSetOrderedSetAccountId20 extends Vec<AccountId20> {}

  /** @name ParachainStakingDelegatorStatus (348) */
  export interface ParachainStakingDelegatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isLeaving: boolean;
    readonly asLeaving: u32;
    readonly type: 'Active' | 'Leaving';
  }

  /** @name ParachainStakingDelegator (349) */
  export interface ParachainStakingDelegator extends Struct {
    readonly id: AccountId20;
    readonly delegations: ParachainStakingSetOrderedSetBond;
    readonly total: u128;
    readonly requests: ParachainStakingPendingDelegationRequests;
    readonly status: ParachainStakingDelegatorStatus;
  }

  /** @name ParachainStakingPendingDelegationRequests (350) */
  export interface ParachainStakingPendingDelegationRequests extends Struct {
    readonly revocationsCount: u32;
    readonly requests: BTreeMap<AccountId20, ParachainStakingDelegationRequest>;
    readonly lessTotal: u128;
  }

  /** @name ParachainStakingCollatorCandidate (354) */
  export interface ParachainStakingCollatorCandidate extends Struct {
    readonly id: AccountId20;
    readonly bond: u128;
    readonly delegators: ParachainStakingSetOrderedSetAccountId20;
    readonly topDelegations: Vec<ParachainStakingBond>;
    readonly bottomDelegations: Vec<ParachainStakingBond>;
    readonly totalCounted: u128;
    readonly totalBacking: u128;
    readonly request: Option<ParachainStakingCandidateBondLessRequest>;
    readonly state: ParachainStakingCollatorStatus;
  }

  /** @name ParachainStakingCandidateBondLessRequest (356) */
  export interface ParachainStakingCandidateBondLessRequest extends Struct {
    readonly amount: u128;
    readonly whenExecutable: u32;
  }

  /** @name ParachainStakingCollatorStatus (357) */
  export interface ParachainStakingCollatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isIdle: boolean;
    readonly isLeaving: boolean;
    readonly asLeaving: u32;
    readonly type: 'Active' | 'Idle' | 'Leaving';
  }

  /** @name ParachainStakingCandidateMetadata (358) */
  export interface ParachainStakingCandidateMetadata extends Struct {
    readonly bond: u128;
    readonly delegationCount: u32;
    readonly totalCounted: u128;
    readonly lowestTopDelegationAmount: u128;
    readonly highestBottomDelegationAmount: u128;
    readonly lowestBottomDelegationAmount: u128;
    readonly topCapacity: ParachainStakingCapacityStatus;
    readonly bottomCapacity: ParachainStakingCapacityStatus;
    readonly request: Option<ParachainStakingCandidateBondLessRequest>;
    readonly status: ParachainStakingCollatorStatus;
  }

  /** @name ParachainStakingCapacityStatus (359) */
  export interface ParachainStakingCapacityStatus extends Enum {
    readonly isFull: boolean;
    readonly isEmpty: boolean;
    readonly isPartial: boolean;
    readonly type: 'Full' | 'Empty' | 'Partial';
  }

  /** @name ParachainStakingDelegations (360) */
  export interface ParachainStakingDelegations extends Struct {
    readonly delegations: Vec<ParachainStakingBond>;
    readonly total: u128;
  }

  /** @name ParachainStakingCollator2 (361) */
  export interface ParachainStakingCollator2 extends Struct {
    readonly id: AccountId20;
    readonly bond: u128;
    readonly nominators: ParachainStakingSetOrderedSetAccountId20;
    readonly topNominators: Vec<ParachainStakingBond>;
    readonly bottomNominators: Vec<ParachainStakingBond>;
    readonly totalCounted: u128;
    readonly totalBacking: u128;
    readonly state: ParachainStakingCollatorStatus;
  }

  /** @name ParachainStakingExitQ (362) */
  export interface ParachainStakingExitQ extends Struct {
    readonly candidates: ParachainStakingSetOrderedSetAccountId20;
    readonly nominatorsLeaving: ParachainStakingSetOrderedSetAccountId20;
    readonly candidateSchedule: Vec<ITuple<[AccountId20, u32]>>;
    readonly nominatorSchedule: Vec<ITuple<[AccountId20, Option<AccountId20>, u32]>>;
  }

  /** @name ParachainStakingCollatorSnapshot (368) */
  export interface ParachainStakingCollatorSnapshot extends Struct {
    readonly bond: u128;
    readonly delegations: Vec<ParachainStakingBond>;
    readonly total: u128;
  }

  /** @name ParachainStakingDelayedPayout (369) */
  export interface ParachainStakingDelayedPayout extends Struct {
    readonly roundIssuance: u128;
    readonly totalStakingReward: u128;
    readonly collatorCommission: Perbill;
  }

  /** @name ParachainStakingInflationInflationInfo (370) */
  export interface ParachainStakingInflationInflationInfo extends Struct {
    readonly expect: ParachainStakingInflationRangeU128;
    readonly annual: ParachainStakingInflationRangePerbill;
    readonly round: ParachainStakingInflationRangePerbill;
  }

  /** @name ParachainStakingError (371) */
  export interface ParachainStakingError extends Enum {
    readonly isDelegatorDNE: boolean;
    readonly isDelegatorDNEinTopNorBottom: boolean;
    readonly isDelegatorDNEInDelegatorSet: boolean;
    readonly isCandidateDNE: boolean;
    readonly isDelegationDNE: boolean;
    readonly isDelegatorExists: boolean;
    readonly isCandidateExists: boolean;
    readonly isCandidateBondBelowMin: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isDelegatorBondBelowMin: boolean;
    readonly isDelegationBelowMin: boolean;
    readonly isAlreadyOffline: boolean;
    readonly isAlreadyActive: boolean;
    readonly isDelegatorAlreadyLeaving: boolean;
    readonly isDelegatorNotLeaving: boolean;
    readonly isDelegatorCannotLeaveYet: boolean;
    readonly isCannotDelegateIfLeaving: boolean;
    readonly isCandidateAlreadyLeaving: boolean;
    readonly isCandidateNotLeaving: boolean;
    readonly isCandidateCannotLeaveYet: boolean;
    readonly isCannotGoOnlineIfLeaving: boolean;
    readonly isExceedMaxDelegationsPerDelegator: boolean;
    readonly isAlreadyDelegatedCandidate: boolean;
    readonly isInvalidSchedule: boolean;
    readonly isCannotSetBelowMin: boolean;
    readonly isRoundLengthMustBeAtLeastTotalSelectedCollators: boolean;
    readonly isNoWritingSameValue: boolean;
    readonly isTooLowCandidateCountWeightHintJoinCandidates: boolean;
    readonly isTooLowCandidateCountWeightHintCancelLeaveCandidates: boolean;
    readonly isTooLowCandidateCountToLeaveCandidates: boolean;
    readonly isTooLowDelegationCountToDelegate: boolean;
    readonly isTooLowCandidateDelegationCountToDelegate: boolean;
    readonly isTooLowCandidateDelegationCountToLeaveCandidates: boolean;
    readonly isTooLowDelegationCountToLeaveDelegators: boolean;
    readonly isPendingCandidateRequestsDNE: boolean;
    readonly isPendingCandidateRequestAlreadyExists: boolean;
    readonly isPendingCandidateRequestNotDueYet: boolean;
    readonly isPendingDelegationRequestDNE: boolean;
    readonly isPendingDelegationRequestAlreadyExists: boolean;
    readonly isPendingDelegationRequestNotDueYet: boolean;
    readonly isCannotDelegateLessThanLowestBottomWhenBottomIsFull: boolean;
    readonly type: 'DelegatorDNE' | 'DelegatorDNEinTopNorBottom' | 'DelegatorDNEInDelegatorSet' | 'CandidateDNE' | 'DelegationDNE' | 'DelegatorExists' | 'CandidateExists' | 'CandidateBondBelowMin' | 'InsufficientBalance' | 'DelegatorBondBelowMin' | 'DelegationBelowMin' | 'AlreadyOffline' | 'AlreadyActive' | 'DelegatorAlreadyLeaving' | 'DelegatorNotLeaving' | 'DelegatorCannotLeaveYet' | 'CannotDelegateIfLeaving' | 'CandidateAlreadyLeaving' | 'CandidateNotLeaving' | 'CandidateCannotLeaveYet' | 'CannotGoOnlineIfLeaving' | 'ExceedMaxDelegationsPerDelegator' | 'AlreadyDelegatedCandidate' | 'InvalidSchedule' | 'CannotSetBelowMin' | 'RoundLengthMustBeAtLeastTotalSelectedCollators' | 'NoWritingSameValue' | 'TooLowCandidateCountWeightHintJoinCandidates' | 'TooLowCandidateCountWeightHintCancelLeaveCandidates' | 'TooLowCandidateCountToLeaveCandidates' | 'TooLowDelegationCountToDelegate' | 'TooLowCandidateDelegationCountToDelegate' | 'TooLowCandidateDelegationCountToLeaveCandidates' | 'TooLowDelegationCountToLeaveDelegators' | 'PendingCandidateRequestsDNE' | 'PendingCandidateRequestAlreadyExists' | 'PendingCandidateRequestNotDueYet' | 'PendingDelegationRequestDNE' | 'PendingDelegationRequestAlreadyExists' | 'PendingDelegationRequestNotDueYet' | 'CannotDelegateLessThanLowestBottomWhenBottomIsFull';
  }

  /** @name PalletSchedulerScheduledV2 (374) */
  export interface PalletSchedulerScheduledV2 extends Struct {
    readonly maybeId: Option<Bytes>;
    readonly priority: u8;
    readonly call: Call;
    readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
    readonly origin: MoonbaseRuntimeOriginCaller;
  }

  /** @name PalletSchedulerReleases (375) */
  export interface PalletSchedulerReleases extends Enum {
    readonly isV1: boolean;
    readonly isV2: boolean;
    readonly type: 'V1' | 'V2';
  }

  /** @name PalletSchedulerError (376) */
  export interface PalletSchedulerError extends Enum {
    readonly isFailedToSchedule: boolean;
    readonly isNotFound: boolean;
    readonly isTargetBlockNumberInPast: boolean;
    readonly isRescheduleNoChange: boolean;
    readonly type: 'FailedToSchedule' | 'NotFound' | 'TargetBlockNumberInPast' | 'RescheduleNoChange';
  }

  /** @name PalletDemocracyPreimageStatus (380) */
  export interface PalletDemocracyPreimageStatus extends Enum {
    readonly isMissing: boolean;
    readonly asMissing: u32;
    readonly isAvailable: boolean;
    readonly asAvailable: {
      readonly data: Bytes;
      readonly provider: AccountId20;
      readonly deposit: u128;
      readonly since: u32;
      readonly expiry: Option<u32>;
    } & Struct;
    readonly type: 'Missing' | 'Available';
  }

  /** @name PalletDemocracyReferendumInfo (381) */
  export interface PalletDemocracyReferendumInfo extends Enum {
    readonly isOngoing: boolean;
    readonly asOngoing: PalletDemocracyReferendumStatus;
    readonly isFinished: boolean;
    readonly asFinished: {
      readonly approved: bool;
      readonly end: u32;
    } & Struct;
    readonly type: 'Ongoing' | 'Finished';
  }

  /** @name PalletDemocracyReferendumStatus (382) */
  export interface PalletDemocracyReferendumStatus extends Struct {
    readonly end: u32;
    readonly proposalHash: H256;
    readonly threshold: PalletDemocracyVoteThreshold;
    readonly delay: u32;
    readonly tally: PalletDemocracyTally;
  }

  /** @name PalletDemocracyTally (383) */
  export interface PalletDemocracyTally extends Struct {
    readonly ayes: u128;
    readonly nays: u128;
    readonly turnout: u128;
  }

  /** @name PalletDemocracyVoteVoting (384) */
  export interface PalletDemocracyVoteVoting extends Enum {
    readonly isDirect: boolean;
    readonly asDirect: {
      readonly votes: Vec<ITuple<[u32, PalletDemocracyVoteAccountVote]>>;
      readonly delegations: PalletDemocracyDelegations;
      readonly prior: PalletDemocracyVotePriorLock;
    } & Struct;
    readonly isDelegating: boolean;
    readonly asDelegating: {
      readonly balance: u128;
      readonly target: AccountId20;
      readonly conviction: PalletDemocracyConviction;
      readonly delegations: PalletDemocracyDelegations;
      readonly prior: PalletDemocracyVotePriorLock;
    } & Struct;
    readonly type: 'Direct' | 'Delegating';
  }

  /** @name PalletDemocracyDelegations (387) */
  export interface PalletDemocracyDelegations extends Struct {
    readonly votes: u128;
    readonly capital: u128;
  }

  /** @name PalletDemocracyVotePriorLock (388) */
  export interface PalletDemocracyVotePriorLock extends ITuple<[u32, u128]> {}

  /** @name PalletDemocracyReleases (391) */
  export interface PalletDemocracyReleases extends Enum {
    readonly isV1: boolean;
    readonly type: 'V1';
  }

  /** @name PalletDemocracyError (392) */
  export interface PalletDemocracyError extends Enum {
    readonly isValueLow: boolean;
    readonly isProposalMissing: boolean;
    readonly isAlreadyCanceled: boolean;
    readonly isDuplicateProposal: boolean;
    readonly isProposalBlacklisted: boolean;
    readonly isNotSimpleMajority: boolean;
    readonly isInvalidHash: boolean;
    readonly isNoProposal: boolean;
    readonly isAlreadyVetoed: boolean;
    readonly isDuplicatePreimage: boolean;
    readonly isNotImminent: boolean;
    readonly isTooEarly: boolean;
    readonly isImminent: boolean;
    readonly isPreimageMissing: boolean;
    readonly isReferendumInvalid: boolean;
    readonly isPreimageInvalid: boolean;
    readonly isNoneWaiting: boolean;
    readonly isNotVoter: boolean;
    readonly isNoPermission: boolean;
    readonly isAlreadyDelegating: boolean;
    readonly isInsufficientFunds: boolean;
    readonly isNotDelegating: boolean;
    readonly isVotesExist: boolean;
    readonly isInstantNotAllowed: boolean;
    readonly isNonsense: boolean;
    readonly isWrongUpperBound: boolean;
    readonly isMaxVotesReached: boolean;
    readonly isTooManyProposals: boolean;
    readonly type: 'ValueLow' | 'ProposalMissing' | 'AlreadyCanceled' | 'DuplicateProposal' | 'ProposalBlacklisted' | 'NotSimpleMajority' | 'InvalidHash' | 'NoProposal' | 'AlreadyVetoed' | 'DuplicatePreimage' | 'NotImminent' | 'TooEarly' | 'Imminent' | 'PreimageMissing' | 'ReferendumInvalid' | 'PreimageInvalid' | 'NoneWaiting' | 'NotVoter' | 'NoPermission' | 'AlreadyDelegating' | 'InsufficientFunds' | 'NotDelegating' | 'VotesExist' | 'InstantNotAllowed' | 'Nonsense' | 'WrongUpperBound' | 'MaxVotesReached' | 'TooManyProposals';
  }

  /** @name PalletCollectiveVotes (394) */
  export interface PalletCollectiveVotes extends Struct {
    readonly index: u32;
    readonly threshold: u32;
    readonly ayes: Vec<AccountId20>;
    readonly nays: Vec<AccountId20>;
    readonly end: u32;
  }

  /** @name PalletCollectiveError (395) */
  export interface PalletCollectiveError extends Enum {
    readonly isNotMember: boolean;
    readonly isDuplicateProposal: boolean;
    readonly isProposalMissing: boolean;
    readonly isWrongIndex: boolean;
    readonly isDuplicateVote: boolean;
    readonly isAlreadyInitialized: boolean;
    readonly isTooEarly: boolean;
    readonly isTooManyProposals: boolean;
    readonly isWrongProposalWeight: boolean;
    readonly isWrongProposalLength: boolean;
    readonly type: 'NotMember' | 'DuplicateProposal' | 'ProposalMissing' | 'WrongIndex' | 'DuplicateVote' | 'AlreadyInitialized' | 'TooEarly' | 'TooManyProposals' | 'WrongProposalWeight' | 'WrongProposalLength';
  }

  /** @name PalletTreasuryProposal (398) */
  export interface PalletTreasuryProposal extends Struct {
    readonly proposer: AccountId20;
    readonly value: u128;
    readonly beneficiary: AccountId20;
    readonly bond: u128;
  }

  /** @name FrameSupportPalletId (401) */
  export interface FrameSupportPalletId extends U8aFixed {}

  /** @name PalletTreasuryError (402) */
  export interface PalletTreasuryError extends Enum {
    readonly isInsufficientProposersBalance: boolean;
    readonly isInvalidIndex: boolean;
    readonly isTooManyApprovals: boolean;
    readonly type: 'InsufficientProposersBalance' | 'InvalidIndex' | 'TooManyApprovals';
  }

  /** @name PalletAuthorInherentError (403) */
  export interface PalletAuthorInherentError extends Enum {
    readonly isAuthorAlreadySet: boolean;
    readonly isNoAccountId: boolean;
    readonly isCannotBeAuthor: boolean;
    readonly type: 'AuthorAlreadySet' | 'NoAccountId' | 'CannotBeAuthor';
  }

  /** @name PalletCrowdloanRewardsRewardInfo (404) */
  export interface PalletCrowdloanRewardsRewardInfo extends Struct {
    readonly totalReward: u128;
    readonly claimedReward: u128;
    readonly contributedRelayAddresses: Vec<U8aFixed>;
  }

  /** @name PalletCrowdloanRewardsError (406) */
  export interface PalletCrowdloanRewardsError extends Enum {
    readonly isAlreadyAssociated: boolean;
    readonly isBatchBeyondFundPot: boolean;
    readonly isFirstClaimAlreadyDone: boolean;
    readonly isRewardNotHighEnough: boolean;
    readonly isInvalidClaimSignature: boolean;
    readonly isInvalidFreeClaimSignature: boolean;
    readonly isNoAssociatedClaim: boolean;
    readonly isRewardsAlreadyClaimed: boolean;
    readonly isRewardVecAlreadyInitialized: boolean;
    readonly isRewardVecNotFullyInitializedYet: boolean;
    readonly isRewardsDoNotMatchFund: boolean;
    readonly isTooManyContributors: boolean;
    readonly isVestingPeriodNonValid: boolean;
    readonly isNonContributedAddressProvided: boolean;
    readonly isInsufficientNumberOfValidProofs: boolean;
    readonly type: 'AlreadyAssociated' | 'BatchBeyondFundPot' | 'FirstClaimAlreadyDone' | 'RewardNotHighEnough' | 'InvalidClaimSignature' | 'InvalidFreeClaimSignature' | 'NoAssociatedClaim' | 'RewardsAlreadyClaimed' | 'RewardVecAlreadyInitialized' | 'RewardVecNotFullyInitializedYet' | 'RewardsDoNotMatchFund' | 'TooManyContributors' | 'VestingPeriodNonValid' | 'NonContributedAddressProvided' | 'InsufficientNumberOfValidProofs';
  }

  /** @name PalletAuthorMappingRegistrationInfo (407) */
  export interface PalletAuthorMappingRegistrationInfo extends Struct {
    readonly account: AccountId20;
    readonly deposit: u128;
  }

  /** @name PalletAuthorMappingError (408) */
  export interface PalletAuthorMappingError extends Enum {
    readonly isAssociationNotFound: boolean;
    readonly isNotYourAssociation: boolean;
    readonly isCannotAffordSecurityDeposit: boolean;
    readonly isAlreadyAssociated: boolean;
    readonly type: 'AssociationNotFound' | 'NotYourAssociation' | 'CannotAffordSecurityDeposit' | 'AlreadyAssociated';
  }

  /** @name PalletProxyProxyDefinition (411) */
  export interface PalletProxyProxyDefinition extends Struct {
    readonly delegate: AccountId20;
    readonly proxyType: MoonbaseRuntimeProxyType;
    readonly delay: u32;
  }

  /** @name PalletProxyAnnouncement (415) */
  export interface PalletProxyAnnouncement extends Struct {
    readonly real: AccountId20;
    readonly callHash: H256;
    readonly height: u32;
  }

  /** @name PalletProxyError (417) */
  export interface PalletProxyError extends Enum {
    readonly isTooMany: boolean;
    readonly isNotFound: boolean;
    readonly isNotProxy: boolean;
    readonly isUnproxyable: boolean;
    readonly isDuplicate: boolean;
    readonly isNoPermission: boolean;
    readonly isUnannounced: boolean;
    readonly isNoSelfProxy: boolean;
    readonly type: 'TooMany' | 'NotFound' | 'NotProxy' | 'Unproxyable' | 'Duplicate' | 'NoPermission' | 'Unannounced' | 'NoSelfProxy';
  }

  /** @name PalletMaintenanceModeError (418) */
  export interface PalletMaintenanceModeError extends Enum {
    readonly isAlreadyInMaintenanceMode: boolean;
    readonly isNotInMaintenanceMode: boolean;
    readonly type: 'AlreadyInMaintenanceMode' | 'NotInMaintenanceMode';
  }

  /** @name PalletIdentityRegistration (419) */
  export interface PalletIdentityRegistration extends Struct {
    readonly judgements: Vec<ITuple<[u32, PalletIdentityJudgement]>>;
    readonly deposit: u128;
    readonly info: PalletIdentityIdentityInfo;
  }

  /** @name PalletIdentityRegistrarInfo (427) */
  export interface PalletIdentityRegistrarInfo extends Struct {
    readonly account: AccountId20;
    readonly fee: u128;
    readonly fields: PalletIdentityBitFlags;
  }

  /** @name PalletIdentityError (429) */
  export interface PalletIdentityError extends Enum {
    readonly isTooManySubAccounts: boolean;
    readonly isNotFound: boolean;
    readonly isNotNamed: boolean;
    readonly isEmptyIndex: boolean;
    readonly isFeeChanged: boolean;
    readonly isNoIdentity: boolean;
    readonly isStickyJudgement: boolean;
    readonly isJudgementGiven: boolean;
    readonly isInvalidJudgement: boolean;
    readonly isInvalidIndex: boolean;
    readonly isInvalidTarget: boolean;
    readonly isTooManyFields: boolean;
    readonly isTooManyRegistrars: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isNotSub: boolean;
    readonly isNotOwned: boolean;
    readonly type: 'TooManySubAccounts' | 'NotFound' | 'NotNamed' | 'EmptyIndex' | 'FeeChanged' | 'NoIdentity' | 'StickyJudgement' | 'JudgementGiven' | 'InvalidJudgement' | 'InvalidIndex' | 'InvalidTarget' | 'TooManyFields' | 'TooManyRegistrars' | 'AlreadyClaimed' | 'NotSub' | 'NotOwned';
  }

  /** @name CumulusPalletXcmpQueueInboundStatus (432) */
  export interface CumulusPalletXcmpQueueInboundStatus extends Enum {
    readonly isOk: boolean;
    readonly isSuspended: boolean;
    readonly type: 'Ok' | 'Suspended';
  }

  /** @name PolkadotParachainPrimitivesXcmpMessageFormat (435) */
  export interface PolkadotParachainPrimitivesXcmpMessageFormat extends Enum {
    readonly isConcatenatedVersionedXcm: boolean;
    readonly isConcatenatedEncodedBlob: boolean;
    readonly isSignals: boolean;
    readonly type: 'ConcatenatedVersionedXcm' | 'ConcatenatedEncodedBlob' | 'Signals';
  }

  /** @name CumulusPalletXcmpQueueOutboundStatus (439) */
  export interface CumulusPalletXcmpQueueOutboundStatus extends Enum {
    readonly isOk: boolean;
    readonly isSuspended: boolean;
    readonly type: 'Ok' | 'Suspended';
  }

  /** @name CumulusPalletXcmpQueueQueueConfigData (441) */
  export interface CumulusPalletXcmpQueueQueueConfigData extends Struct {
    readonly suspendThreshold: u32;
    readonly dropThreshold: u32;
    readonly resumeThreshold: u32;
    readonly thresholdWeight: u64;
    readonly weightRestrictDecay: u64;
  }

  /** @name CumulusPalletXcmpQueueError (442) */
  export interface CumulusPalletXcmpQueueError extends Enum {
    readonly isFailedToSend: boolean;
    readonly isBadXcmOrigin: boolean;
    readonly isBadXcm: boolean;
    readonly type: 'FailedToSend' | 'BadXcmOrigin' | 'BadXcm';
  }

  /** @name CumulusPalletXcmError (443) */
  export type CumulusPalletXcmError = Null;

  /** @name CumulusPalletDmpQueueConfigData (444) */
  export interface CumulusPalletDmpQueueConfigData extends Struct {
    readonly maxIndividual: u64;
  }

  /** @name CumulusPalletDmpQueuePageIndexData (445) */
  export interface CumulusPalletDmpQueuePageIndexData extends Struct {
    readonly beginUsed: u32;
    readonly endUsed: u32;
    readonly overweightCount: u64;
  }

  /** @name CumulusPalletDmpQueueError (448) */
  export interface CumulusPalletDmpQueueError extends Enum {
    readonly isUnknown: boolean;
    readonly isOverLimit: boolean;
    readonly type: 'Unknown' | 'OverLimit';
  }

  /** @name PalletXcmQueryStatus (449) */
  export interface PalletXcmQueryStatus extends Enum {
    readonly isPending: boolean;
    readonly asPending: {
      readonly responder: XcmVersionedMultiLocation;
      readonly maybeNotify: Option<ITuple<[u8, u8]>>;
      readonly timeout: u32;
    } & Struct;
    readonly isVersionNotifier: boolean;
    readonly asVersionNotifier: {
      readonly origin: XcmVersionedMultiLocation;
      readonly isActive: bool;
    } & Struct;
    readonly isReady: boolean;
    readonly asReady: {
      readonly response: XcmVersionedResponse;
      readonly at: u32;
    } & Struct;
    readonly type: 'Pending' | 'VersionNotifier' | 'Ready';
  }

  /** @name XcmVersionedResponse (452) */
  export interface XcmVersionedResponse extends Enum {
    readonly isV0: boolean;
    readonly asV0: XcmV0Response;
    readonly isV1: boolean;
    readonly asV1: XcmV1Response;
    readonly isV2: boolean;
    readonly asV2: XcmV2Response;
    readonly type: 'V0' | 'V1' | 'V2';
  }

  /** @name PalletXcmVersionMigrationStage (458) */
  export interface PalletXcmVersionMigrationStage extends Enum {
    readonly isMigrateSupportedVersion: boolean;
    readonly isMigrateVersionNotifiers: boolean;
    readonly isNotifyCurrentTargets: boolean;
    readonly asNotifyCurrentTargets: Option<Bytes>;
    readonly isMigrateAndNotifyOldTargets: boolean;
    readonly type: 'MigrateSupportedVersion' | 'MigrateVersionNotifiers' | 'NotifyCurrentTargets' | 'MigrateAndNotifyOldTargets';
  }

  /** @name PalletXcmError (459) */
  export interface PalletXcmError extends Enum {
    readonly isUnreachable: boolean;
    readonly isSendFailure: boolean;
    readonly isFiltered: boolean;
    readonly isUnweighableMessage: boolean;
    readonly isDestinationNotInvertible: boolean;
    readonly isEmpty: boolean;
    readonly isCannotReanchor: boolean;
    readonly isTooManyAssets: boolean;
    readonly isInvalidOrigin: boolean;
    readonly isBadVersion: boolean;
    readonly isBadLocation: boolean;
    readonly isNoSubscription: boolean;
    readonly isAlreadySubscribed: boolean;
    readonly type: 'Unreachable' | 'SendFailure' | 'Filtered' | 'UnweighableMessage' | 'DestinationNotInvertible' | 'Empty' | 'CannotReanchor' | 'TooManyAssets' | 'InvalidOrigin' | 'BadVersion' | 'BadLocation' | 'NoSubscription' | 'AlreadySubscribed';
  }

  /** @name PalletAssetsAssetDetails (460) */
  export interface PalletAssetsAssetDetails extends Struct {
    readonly owner: AccountId20;
    readonly issuer: AccountId20;
    readonly admin: AccountId20;
    readonly freezer: AccountId20;
    readonly supply: u128;
    readonly deposit: u128;
    readonly minBalance: u128;
    readonly isSufficient: bool;
    readonly accounts: u32;
    readonly sufficients: u32;
    readonly approvals: u32;
    readonly isFrozen: bool;
  }

  /** @name PalletAssetsAssetBalance (462) */
  export interface PalletAssetsAssetBalance extends Struct {
    readonly balance: u128;
    readonly isFrozen: bool;
    readonly sufficient: bool;
    readonly extra: Null;
  }

  /** @name PalletAssetsApproval (464) */
  export interface PalletAssetsApproval extends Struct {
    readonly amount: u128;
    readonly deposit: u128;
  }

  /** @name PalletAssetsAssetMetadata (465) */
  export interface PalletAssetsAssetMetadata extends Struct {
    readonly deposit: u128;
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly decimals: u8;
    readonly isFrozen: bool;
  }

  /** @name PalletAssetsError (467) */
  export interface PalletAssetsError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isBalanceZero: boolean;
    readonly isNoPermission: boolean;
    readonly isUnknown: boolean;
    readonly isFrozen: boolean;
    readonly isInUse: boolean;
    readonly isBadWitness: boolean;
    readonly isMinBalanceZero: boolean;
    readonly isNoProvider: boolean;
    readonly isBadMetadata: boolean;
    readonly isUnapproved: boolean;
    readonly isWouldDie: boolean;
    readonly type: 'BalanceLow' | 'BalanceZero' | 'NoPermission' | 'Unknown' | 'Frozen' | 'InUse' | 'BadWitness' | 'MinBalanceZero' | 'NoProvider' | 'BadMetadata' | 'Unapproved' | 'WouldDie';
  }

  /** @name OrmlXtokensModuleError (468) */
  export interface OrmlXtokensModuleError extends Enum {
    readonly isAssetHasNoReserve: boolean;
    readonly isNotCrossChainTransfer: boolean;
    readonly isInvalidDest: boolean;
    readonly isNotCrossChainTransferableCurrency: boolean;
    readonly isUnweighableMessage: boolean;
    readonly isXcmExecutionFailed: boolean;
    readonly isCannotReanchor: boolean;
    readonly isInvalidAncestry: boolean;
    readonly isNotFungible: boolean;
    readonly isDestinationNotInvertible: boolean;
    readonly isBadVersion: boolean;
    readonly isDistincAssetAndFeeId: boolean;
    readonly isFeeCannotBeZero: boolean;
    readonly type: 'AssetHasNoReserve' | 'NotCrossChainTransfer' | 'InvalidDest' | 'NotCrossChainTransferableCurrency' | 'UnweighableMessage' | 'XcmExecutionFailed' | 'CannotReanchor' | 'InvalidAncestry' | 'NotFungible' | 'DestinationNotInvertible' | 'BadVersion' | 'DistincAssetAndFeeId' | 'FeeCannotBeZero';
  }

  /** @name PalletAssetManagerError (469) */
  export interface PalletAssetManagerError extends Enum {
    readonly isErrorCreatingAsset: boolean;
    readonly isAssetAlreadyExists: boolean;
    readonly isAssetDoesNotExist: boolean;
    readonly type: 'ErrorCreatingAsset' | 'AssetAlreadyExists' | 'AssetDoesNotExist';
  }

  /** @name XcmTransactorError (470) */
  export interface XcmTransactorError extends Enum {
    readonly isIndexAlreadyClaimed: boolean;
    readonly isUnclaimedIndex: boolean;
    readonly isNotOwner: boolean;
    readonly isUnweighableMessage: boolean;
    readonly isCannotReanchor: boolean;
    readonly isAssetHasNoReserve: boolean;
    readonly isInvalidDest: boolean;
    readonly isNotCrossChainTransfer: boolean;
    readonly isAssetIsNotReserveInDestination: boolean;
    readonly isDestinationNotInvertible: boolean;
    readonly isErrorSending: boolean;
    readonly isDispatchWeightBiggerThanTotalWeight: boolean;
    readonly isWeightOverflow: boolean;
    readonly isAmountOverflow: boolean;
    readonly isTransactorInfoNotSet: boolean;
    readonly isNotCrossChainTransferableCurrency: boolean;
    readonly isXcmExecuteError: boolean;
    readonly isBadVersion: boolean;
    readonly isMaxWeightTransactReached: boolean;
    readonly isUnableToWithdrawAsset: boolean;
    readonly type: 'IndexAlreadyClaimed' | 'UnclaimedIndex' | 'NotOwner' | 'UnweighableMessage' | 'CannotReanchor' | 'AssetHasNoReserve' | 'InvalidDest' | 'NotCrossChainTransfer' | 'AssetIsNotReserveInDestination' | 'DestinationNotInvertible' | 'ErrorSending' | 'DispatchWeightBiggerThanTotalWeight' | 'WeightOverflow' | 'AmountOverflow' | 'TransactorInfoNotSet' | 'NotCrossChainTransferableCurrency' | 'XcmExecuteError' | 'BadVersion' | 'MaxWeightTransactReached' | 'UnableToWithdrawAsset';
  }

  /** @name AccountEthereumSignature (472) */
  export interface AccountEthereumSignature extends SpCoreEcdsaSignature {}

  /** @name FrameSystemExtensionsCheckSpecVersion (474) */
  export type FrameSystemExtensionsCheckSpecVersion = Null;

  /** @name FrameSystemExtensionsCheckTxVersion (475) */
  export type FrameSystemExtensionsCheckTxVersion = Null;

  /** @name FrameSystemExtensionsCheckGenesis (476) */
  export type FrameSystemExtensionsCheckGenesis = Null;

  /** @name FrameSystemExtensionsCheckNonce (479) */
  export interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /** @name FrameSystemExtensionsCheckWeight (480) */
  export type FrameSystemExtensionsCheckWeight = Null;

  /** @name PalletTransactionPaymentChargeTransactionPayment (481) */
  export interface PalletTransactionPaymentChargeTransactionPayment extends Compact<u128> {}

  /** @name MoonbaseRuntimeRuntime (483) */
  export type MoonbaseRuntimeRuntime = Null;

} // declare module
