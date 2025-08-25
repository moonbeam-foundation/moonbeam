// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/types/lookup";

import type { Data } from "@polkadot/types";
import type {
  BTreeMap,
  BTreeSet,
  Bytes,
  Compact,
  Enum,
  Null,
  Option,
  Result,
  Struct,
  Text,
  U256,
  U8aFixed,
  Vec,
  bool,
  i64,
  u128,
  u16,
  u32,
  u64,
  u8
} from "@polkadot/types-codec";
import type { ITuple } from "@polkadot/types-codec/types";
import type { Vote } from "@polkadot/types/interfaces/elections";
import type {
  AccountId20,
  Call,
  H160,
  H256,
  Perbill,
  Percent
} from "@polkadot/types/interfaces/runtime";
import type { Event } from "@polkadot/types/interfaces/system";

declare module "@polkadot/types/lookup" {
  /** @name FrameSystemAccountInfo (3) */
  interface FrameSystemAccountInfo extends Struct {
    readonly nonce: u32;
    readonly consumers: u32;
    readonly providers: u32;
    readonly sufficients: u32;
    readonly data: PalletBalancesAccountData;
  }

  /** @name PalletBalancesAccountData (5) */
  interface PalletBalancesAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly frozen: u128;
    readonly flags: u128;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeight (9) */
  interface FrameSupportDispatchPerDispatchClassWeight extends Struct {
    readonly normal: SpWeightsWeightV2Weight;
    readonly operational: SpWeightsWeightV2Weight;
    readonly mandatory: SpWeightsWeightV2Weight;
  }

  /** @name SpWeightsWeightV2Weight (10) */
  interface SpWeightsWeightV2Weight extends Struct {
    readonly refTime: Compact<u64>;
    readonly proofSize: Compact<u64>;
  }

  /** @name SpRuntimeDigest (16) */
  interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>;
  }

  /** @name SpRuntimeDigestDigestItem (18) */
  interface SpRuntimeDigestDigestItem extends Enum {
    readonly isOther: boolean;
    readonly asOther: Bytes;
    readonly isConsensus: boolean;
    readonly asConsensus: ITuple<[U8aFixed, Bytes]>;
    readonly isSeal: boolean;
    readonly asSeal: ITuple<[U8aFixed, Bytes]>;
    readonly isPreRuntime: boolean;
    readonly asPreRuntime: ITuple<[U8aFixed, Bytes]>;
    readonly isRuntimeEnvironmentUpdated: boolean;
    readonly type: "Other" | "Consensus" | "Seal" | "PreRuntime" | "RuntimeEnvironmentUpdated";
  }

  /** @name FrameSystemEventRecord (21) */
  interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase;
    readonly event: Event;
    readonly topics: Vec<H256>;
  }

  /** @name FrameSystemEvent (23) */
  interface FrameSystemEvent extends Enum {
    readonly isExtrinsicSuccess: boolean;
    readonly asExtrinsicSuccess: {
      readonly dispatchInfo: FrameSystemDispatchEventInfo;
    } & Struct;
    readonly isExtrinsicFailed: boolean;
    readonly asExtrinsicFailed: {
      readonly dispatchError: SpRuntimeDispatchError;
      readonly dispatchInfo: FrameSystemDispatchEventInfo;
    } & Struct;
    readonly isCodeUpdated: boolean;
    readonly isNewAccount: boolean;
    readonly asNewAccount: {
      readonly account: AccountId20;
    } & Struct;
    readonly isKilledAccount: boolean;
    readonly asKilledAccount: {
      readonly account: AccountId20;
    } & Struct;
    readonly isRemarked: boolean;
    readonly asRemarked: {
      readonly sender: AccountId20;
      readonly hash_: H256;
    } & Struct;
    readonly isUpgradeAuthorized: boolean;
    readonly asUpgradeAuthorized: {
      readonly codeHash: H256;
      readonly checkVersion: bool;
    } & Struct;
    readonly type:
      | "ExtrinsicSuccess"
      | "ExtrinsicFailed"
      | "CodeUpdated"
      | "NewAccount"
      | "KilledAccount"
      | "Remarked"
      | "UpgradeAuthorized";
  }

  /** @name FrameSystemDispatchEventInfo (24) */
  interface FrameSystemDispatchEventInfo extends Struct {
    readonly weight: SpWeightsWeightV2Weight;
    readonly class: FrameSupportDispatchDispatchClass;
    readonly paysFee: FrameSupportDispatchPays;
  }

  /** @name FrameSupportDispatchDispatchClass (25) */
  interface FrameSupportDispatchDispatchClass extends Enum {
    readonly isNormal: boolean;
    readonly isOperational: boolean;
    readonly isMandatory: boolean;
    readonly type: "Normal" | "Operational" | "Mandatory";
  }

  /** @name FrameSupportDispatchPays (26) */
  interface FrameSupportDispatchPays extends Enum {
    readonly isYes: boolean;
    readonly isNo: boolean;
    readonly type: "Yes" | "No";
  }

  /** @name SpRuntimeDispatchError (27) */
  interface SpRuntimeDispatchError extends Enum {
    readonly isOther: boolean;
    readonly isCannotLookup: boolean;
    readonly isBadOrigin: boolean;
    readonly isModule: boolean;
    readonly asModule: SpRuntimeModuleError;
    readonly isConsumerRemaining: boolean;
    readonly isNoProviders: boolean;
    readonly isTooManyConsumers: boolean;
    readonly isToken: boolean;
    readonly asToken: SpRuntimeTokenError;
    readonly isArithmetic: boolean;
    readonly asArithmetic: SpArithmeticArithmeticError;
    readonly isTransactional: boolean;
    readonly asTransactional: SpRuntimeTransactionalError;
    readonly isExhausted: boolean;
    readonly isCorruption: boolean;
    readonly isUnavailable: boolean;
    readonly isRootNotAllowed: boolean;
    readonly isTrie: boolean;
    readonly asTrie: SpRuntimeProvingTrieTrieError;
    readonly type:
      | "Other"
      | "CannotLookup"
      | "BadOrigin"
      | "Module"
      | "ConsumerRemaining"
      | "NoProviders"
      | "TooManyConsumers"
      | "Token"
      | "Arithmetic"
      | "Transactional"
      | "Exhausted"
      | "Corruption"
      | "Unavailable"
      | "RootNotAllowed"
      | "Trie";
  }

  /** @name SpRuntimeModuleError (28) */
  interface SpRuntimeModuleError extends Struct {
    readonly index: u8;
    readonly error: U8aFixed;
  }

  /** @name SpRuntimeTokenError (29) */
  interface SpRuntimeTokenError extends Enum {
    readonly isFundsUnavailable: boolean;
    readonly isOnlyProvider: boolean;
    readonly isBelowMinimum: boolean;
    readonly isCannotCreate: boolean;
    readonly isUnknownAsset: boolean;
    readonly isFrozen: boolean;
    readonly isUnsupported: boolean;
    readonly isCannotCreateHold: boolean;
    readonly isNotExpendable: boolean;
    readonly isBlocked: boolean;
    readonly type:
      | "FundsUnavailable"
      | "OnlyProvider"
      | "BelowMinimum"
      | "CannotCreate"
      | "UnknownAsset"
      | "Frozen"
      | "Unsupported"
      | "CannotCreateHold"
      | "NotExpendable"
      | "Blocked";
  }

  /** @name SpArithmeticArithmeticError (30) */
  interface SpArithmeticArithmeticError extends Enum {
    readonly isUnderflow: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly type: "Underflow" | "Overflow" | "DivisionByZero";
  }

  /** @name SpRuntimeTransactionalError (31) */
  interface SpRuntimeTransactionalError extends Enum {
    readonly isLimitReached: boolean;
    readonly isNoLayer: boolean;
    readonly type: "LimitReached" | "NoLayer";
  }

  /** @name SpRuntimeProvingTrieTrieError (32) */
  interface SpRuntimeProvingTrieTrieError extends Enum {
    readonly isInvalidStateRoot: boolean;
    readonly isIncompleteDatabase: boolean;
    readonly isValueAtIncompleteKey: boolean;
    readonly isDecoderError: boolean;
    readonly isInvalidHash: boolean;
    readonly isDuplicateKey: boolean;
    readonly isExtraneousNode: boolean;
    readonly isExtraneousValue: boolean;
    readonly isExtraneousHashReference: boolean;
    readonly isInvalidChildReference: boolean;
    readonly isValueMismatch: boolean;
    readonly isIncompleteProof: boolean;
    readonly isRootMismatch: boolean;
    readonly isDecodeError: boolean;
    readonly type:
      | "InvalidStateRoot"
      | "IncompleteDatabase"
      | "ValueAtIncompleteKey"
      | "DecoderError"
      | "InvalidHash"
      | "DuplicateKey"
      | "ExtraneousNode"
      | "ExtraneousValue"
      | "ExtraneousHashReference"
      | "InvalidChildReference"
      | "ValueMismatch"
      | "IncompleteProof"
      | "RootMismatch"
      | "DecodeError";
  }

  /** @name PalletUtilityEvent (33) */
  interface PalletUtilityEvent extends Enum {
    readonly isBatchInterrupted: boolean;
    readonly asBatchInterrupted: {
      readonly index: u32;
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isBatchCompleted: boolean;
    readonly isBatchCompletedWithErrors: boolean;
    readonly isItemCompleted: boolean;
    readonly isItemFailed: boolean;
    readonly asItemFailed: {
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isDispatchedAs: boolean;
    readonly asDispatchedAs: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type:
      | "BatchInterrupted"
      | "BatchCompleted"
      | "BatchCompletedWithErrors"
      | "ItemCompleted"
      | "ItemFailed"
      | "DispatchedAs";
  }

  /** @name PalletBalancesEvent (36) */
  interface PalletBalancesEvent extends Enum {
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
    readonly isMinted: boolean;
    readonly asMinted: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isBurned: boolean;
    readonly asBurned: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isSuspended: boolean;
    readonly asSuspended: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isRestored: boolean;
    readonly asRestored: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isUpgraded: boolean;
    readonly asUpgraded: {
      readonly who: AccountId20;
    } & Struct;
    readonly isIssued: boolean;
    readonly asIssued: {
      readonly amount: u128;
    } & Struct;
    readonly isRescinded: boolean;
    readonly asRescinded: {
      readonly amount: u128;
    } & Struct;
    readonly isLocked: boolean;
    readonly asLocked: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isUnlocked: boolean;
    readonly asUnlocked: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isFrozen: boolean;
    readonly asFrozen: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isThawed: boolean;
    readonly asThawed: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isTotalIssuanceForced: boolean;
    readonly asTotalIssuanceForced: {
      readonly old: u128;
      readonly new_: u128;
    } & Struct;
    readonly type:
      | "Endowed"
      | "DustLost"
      | "Transfer"
      | "BalanceSet"
      | "Reserved"
      | "Unreserved"
      | "ReserveRepatriated"
      | "Deposit"
      | "Withdraw"
      | "Slashed"
      | "Minted"
      | "Burned"
      | "Suspended"
      | "Restored"
      | "Upgraded"
      | "Issued"
      | "Rescinded"
      | "Locked"
      | "Unlocked"
      | "Frozen"
      | "Thawed"
      | "TotalIssuanceForced";
  }

  /** @name FrameSupportTokensMiscBalanceStatus (37) */
  interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean;
    readonly isReserved: boolean;
    readonly type: "Free" | "Reserved";
  }

  /** @name PalletSudoEvent (38) */
  interface PalletSudoEvent extends Enum {
    readonly isSudid: boolean;
    readonly asSudid: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isKeyChanged: boolean;
    readonly asKeyChanged: {
      readonly old: Option<AccountId20>;
      readonly new_: AccountId20;
    } & Struct;
    readonly isKeyRemoved: boolean;
    readonly isSudoAsDone: boolean;
    readonly asSudoAsDone: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: "Sudid" | "KeyChanged" | "KeyRemoved" | "SudoAsDone";
  }

  /** @name CumulusPalletParachainSystemEvent (40) */
  interface CumulusPalletParachainSystemEvent extends Enum {
    readonly isValidationFunctionStored: boolean;
    readonly isValidationFunctionApplied: boolean;
    readonly asValidationFunctionApplied: {
      readonly relayChainBlockNum: u32;
    } & Struct;
    readonly isValidationFunctionDiscarded: boolean;
    readonly isDownwardMessagesReceived: boolean;
    readonly asDownwardMessagesReceived: {
      readonly count: u32;
    } & Struct;
    readonly isDownwardMessagesProcessed: boolean;
    readonly asDownwardMessagesProcessed: {
      readonly weightUsed: SpWeightsWeightV2Weight;
      readonly dmqHead: H256;
    } & Struct;
    readonly isUpwardMessageSent: boolean;
    readonly asUpwardMessageSent: {
      readonly messageHash: Option<U8aFixed>;
    } & Struct;
    readonly type:
      | "ValidationFunctionStored"
      | "ValidationFunctionApplied"
      | "ValidationFunctionDiscarded"
      | "DownwardMessagesReceived"
      | "DownwardMessagesProcessed"
      | "UpwardMessageSent";
  }

  /** @name PalletTransactionPaymentEvent (42) */
  interface PalletTransactionPaymentEvent extends Enum {
    readonly isTransactionFeePaid: boolean;
    readonly asTransactionFeePaid: {
      readonly who: AccountId20;
      readonly actualFee: u128;
      readonly tip: u128;
    } & Struct;
    readonly type: "TransactionFeePaid";
  }

  /** @name PalletEvmEvent (43) */
  interface PalletEvmEvent extends Enum {
    readonly isLog: boolean;
    readonly asLog: {
      readonly log: EthereumLog;
    } & Struct;
    readonly isCreated: boolean;
    readonly asCreated: {
      readonly address: H160;
    } & Struct;
    readonly isCreatedFailed: boolean;
    readonly asCreatedFailed: {
      readonly address: H160;
    } & Struct;
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly address: H160;
    } & Struct;
    readonly isExecutedFailed: boolean;
    readonly asExecutedFailed: {
      readonly address: H160;
    } & Struct;
    readonly type: "Log" | "Created" | "CreatedFailed" | "Executed" | "ExecutedFailed";
  }

  /** @name EthereumLog (44) */
  interface EthereumLog extends Struct {
    readonly address: H160;
    readonly topics: Vec<H256>;
    readonly data: Bytes;
  }

  /** @name PalletEthereumEvent (47) */
  interface PalletEthereumEvent extends Enum {
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly from: H160;
      readonly to: H160;
      readonly transactionHash: H256;
      readonly exitReason: EvmCoreErrorExitReason;
      readonly extraData: Bytes;
    } & Struct;
    readonly type: "Executed";
  }

  /** @name EvmCoreErrorExitReason (48) */
  interface EvmCoreErrorExitReason extends Enum {
    readonly isSucceed: boolean;
    readonly asSucceed: EvmCoreErrorExitSucceed;
    readonly isError: boolean;
    readonly asError: EvmCoreErrorExitError;
    readonly isRevert: boolean;
    readonly asRevert: EvmCoreErrorExitRevert;
    readonly isFatal: boolean;
    readonly asFatal: EvmCoreErrorExitFatal;
    readonly type: "Succeed" | "Error" | "Revert" | "Fatal";
  }

  /** @name EvmCoreErrorExitSucceed (49) */
  interface EvmCoreErrorExitSucceed extends Enum {
    readonly isStopped: boolean;
    readonly isReturned: boolean;
    readonly isSuicided: boolean;
    readonly type: "Stopped" | "Returned" | "Suicided";
  }

  /** @name EvmCoreErrorExitError (50) */
  interface EvmCoreErrorExitError extends Enum {
    readonly isStackUnderflow: boolean;
    readonly isStackOverflow: boolean;
    readonly isInvalidJump: boolean;
    readonly isInvalidRange: boolean;
    readonly isDesignatedInvalid: boolean;
    readonly isCallTooDeep: boolean;
    readonly isCreateCollision: boolean;
    readonly isCreateContractLimit: boolean;
    readonly isOutOfOffset: boolean;
    readonly isOutOfGas: boolean;
    readonly isOutOfFund: boolean;
    readonly isPcUnderflow: boolean;
    readonly isCreateEmpty: boolean;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly isMaxNonce: boolean;
    readonly isInvalidCode: boolean;
    readonly asInvalidCode: u8;
    readonly type:
      | "StackUnderflow"
      | "StackOverflow"
      | "InvalidJump"
      | "InvalidRange"
      | "DesignatedInvalid"
      | "CallTooDeep"
      | "CreateCollision"
      | "CreateContractLimit"
      | "OutOfOffset"
      | "OutOfGas"
      | "OutOfFund"
      | "PcUnderflow"
      | "CreateEmpty"
      | "Other"
      | "MaxNonce"
      | "InvalidCode";
  }

  /** @name EvmCoreErrorExitRevert (54) */
  interface EvmCoreErrorExitRevert extends Enum {
    readonly isReverted: boolean;
    readonly type: "Reverted";
  }

  /** @name EvmCoreErrorExitFatal (55) */
  interface EvmCoreErrorExitFatal extends Enum {
    readonly isNotSupported: boolean;
    readonly isUnhandledInterrupt: boolean;
    readonly isCallErrorAsFatal: boolean;
    readonly asCallErrorAsFatal: EvmCoreErrorExitError;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly type: "NotSupported" | "UnhandledInterrupt" | "CallErrorAsFatal" | "Other";
  }

  /** @name PalletParachainStakingEvent (56) */
  interface PalletParachainStakingEvent extends Enum {
    readonly isNewRound: boolean;
    readonly asNewRound: {
      readonly startingBlock: u32;
      readonly round: u32;
      readonly selectedCollatorsNumber: u32;
      readonly totalBalance: u128;
    } & Struct;
    readonly isJoinedCollatorCandidates: boolean;
    readonly asJoinedCollatorCandidates: {
      readonly account: AccountId20;
      readonly amountLocked: u128;
      readonly newTotalAmtLocked: u128;
    } & Struct;
    readonly isCollatorChosen: boolean;
    readonly asCollatorChosen: {
      readonly round: u32;
      readonly collatorAccount: AccountId20;
      readonly totalExposedAmount: u128;
    } & Struct;
    readonly isCandidateBondLessRequested: boolean;
    readonly asCandidateBondLessRequested: {
      readonly candidate: AccountId20;
      readonly amountToDecrease: u128;
      readonly executeRound: u32;
    } & Struct;
    readonly isCandidateBondedMore: boolean;
    readonly asCandidateBondedMore: {
      readonly candidate: AccountId20;
      readonly amount: u128;
      readonly newTotalBond: u128;
    } & Struct;
    readonly isCandidateBondedLess: boolean;
    readonly asCandidateBondedLess: {
      readonly candidate: AccountId20;
      readonly amount: u128;
      readonly newBond: u128;
    } & Struct;
    readonly isCandidateWentOffline: boolean;
    readonly asCandidateWentOffline: {
      readonly candidate: AccountId20;
    } & Struct;
    readonly isCandidateBackOnline: boolean;
    readonly asCandidateBackOnline: {
      readonly candidate: AccountId20;
    } & Struct;
    readonly isCandidateScheduledExit: boolean;
    readonly asCandidateScheduledExit: {
      readonly exitAllowedRound: u32;
      readonly candidate: AccountId20;
      readonly scheduledExit: u32;
    } & Struct;
    readonly isCancelledCandidateExit: boolean;
    readonly asCancelledCandidateExit: {
      readonly candidate: AccountId20;
    } & Struct;
    readonly isCancelledCandidateBondLess: boolean;
    readonly asCancelledCandidateBondLess: {
      readonly candidate: AccountId20;
      readonly amount: u128;
      readonly executeRound: u32;
    } & Struct;
    readonly isCandidateLeft: boolean;
    readonly asCandidateLeft: {
      readonly exCandidate: AccountId20;
      readonly unlockedAmount: u128;
      readonly newTotalAmtLocked: u128;
    } & Struct;
    readonly isDelegationDecreaseScheduled: boolean;
    readonly asDelegationDecreaseScheduled: {
      readonly delegator: AccountId20;
      readonly candidate: AccountId20;
      readonly amountToDecrease: u128;
      readonly executeRound: u32;
    } & Struct;
    readonly isDelegationIncreased: boolean;
    readonly asDelegationIncreased: {
      readonly delegator: AccountId20;
      readonly candidate: AccountId20;
      readonly amount: u128;
      readonly inTop: bool;
    } & Struct;
    readonly isDelegationDecreased: boolean;
    readonly asDelegationDecreased: {
      readonly delegator: AccountId20;
      readonly candidate: AccountId20;
      readonly amount: u128;
      readonly inTop: bool;
    } & Struct;
    readonly isDelegatorExitScheduled: boolean;
    readonly asDelegatorExitScheduled: {
      readonly round: u32;
      readonly delegator: AccountId20;
      readonly scheduledExit: u32;
    } & Struct;
    readonly isDelegationRevocationScheduled: boolean;
    readonly asDelegationRevocationScheduled: {
      readonly round: u32;
      readonly delegator: AccountId20;
      readonly candidate: AccountId20;
      readonly scheduledExit: u32;
    } & Struct;
    readonly isDelegatorLeft: boolean;
    readonly asDelegatorLeft: {
      readonly delegator: AccountId20;
      readonly unstakedAmount: u128;
    } & Struct;
    readonly isDelegationRevoked: boolean;
    readonly asDelegationRevoked: {
      readonly delegator: AccountId20;
      readonly candidate: AccountId20;
      readonly unstakedAmount: u128;
    } & Struct;
    readonly isDelegationKicked: boolean;
    readonly asDelegationKicked: {
      readonly delegator: AccountId20;
      readonly candidate: AccountId20;
      readonly unstakedAmount: u128;
    } & Struct;
    readonly isDelegatorExitCancelled: boolean;
    readonly asDelegatorExitCancelled: {
      readonly delegator: AccountId20;
    } & Struct;
    readonly isCancelledDelegationRequest: boolean;
    readonly asCancelledDelegationRequest: {
      readonly delegator: AccountId20;
      readonly cancelledRequest: PalletParachainStakingDelegationRequestsCancelledScheduledRequest;
      readonly collator: AccountId20;
    } & Struct;
    readonly isDelegation: boolean;
    readonly asDelegation: {
      readonly delegator: AccountId20;
      readonly lockedAmount: u128;
      readonly candidate: AccountId20;
      readonly delegatorPosition: PalletParachainStakingDelegatorAdded;
      readonly autoCompound: Percent;
    } & Struct;
    readonly isDelegatorLeftCandidate: boolean;
    readonly asDelegatorLeftCandidate: {
      readonly delegator: AccountId20;
      readonly candidate: AccountId20;
      readonly unstakedAmount: u128;
      readonly totalCandidateStaked: u128;
    } & Struct;
    readonly isRewarded: boolean;
    readonly asRewarded: {
      readonly account: AccountId20;
      readonly rewards: u128;
    } & Struct;
    readonly isInflationDistributed: boolean;
    readonly asInflationDistributed: {
      readonly index: u32;
      readonly account: AccountId20;
      readonly value: u128;
    } & Struct;
    readonly isInflationDistributionConfigUpdated: boolean;
    readonly asInflationDistributionConfigUpdated: {
      readonly old: PalletParachainStakingInflationDistributionConfig;
      readonly new_: PalletParachainStakingInflationDistributionConfig;
    } & Struct;
    readonly isInflationSet: boolean;
    readonly asInflationSet: {
      readonly annualMin: Perbill;
      readonly annualIdeal: Perbill;
      readonly annualMax: Perbill;
      readonly roundMin: Perbill;
      readonly roundIdeal: Perbill;
      readonly roundMax: Perbill;
    } & Struct;
    readonly isStakeExpectationsSet: boolean;
    readonly asStakeExpectationsSet: {
      readonly expectMin: u128;
      readonly expectIdeal: u128;
      readonly expectMax: u128;
    } & Struct;
    readonly isTotalSelectedSet: boolean;
    readonly asTotalSelectedSet: {
      readonly old: u32;
      readonly new_: u32;
    } & Struct;
    readonly isCollatorCommissionSet: boolean;
    readonly asCollatorCommissionSet: {
      readonly old: Perbill;
      readonly new_: Perbill;
    } & Struct;
    readonly isBlocksPerRoundSet: boolean;
    readonly asBlocksPerRoundSet: {
      readonly currentRound: u32;
      readonly firstBlock: u32;
      readonly old: u32;
      readonly new_: u32;
      readonly newPerRoundInflationMin: Perbill;
      readonly newPerRoundInflationIdeal: Perbill;
      readonly newPerRoundInflationMax: Perbill;
    } & Struct;
    readonly isAutoCompoundSet: boolean;
    readonly asAutoCompoundSet: {
      readonly candidate: AccountId20;
      readonly delegator: AccountId20;
      readonly value: Percent;
    } & Struct;
    readonly isCompounded: boolean;
    readonly asCompounded: {
      readonly candidate: AccountId20;
      readonly delegator: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly type:
      | "NewRound"
      | "JoinedCollatorCandidates"
      | "CollatorChosen"
      | "CandidateBondLessRequested"
      | "CandidateBondedMore"
      | "CandidateBondedLess"
      | "CandidateWentOffline"
      | "CandidateBackOnline"
      | "CandidateScheduledExit"
      | "CancelledCandidateExit"
      | "CancelledCandidateBondLess"
      | "CandidateLeft"
      | "DelegationDecreaseScheduled"
      | "DelegationIncreased"
      | "DelegationDecreased"
      | "DelegatorExitScheduled"
      | "DelegationRevocationScheduled"
      | "DelegatorLeft"
      | "DelegationRevoked"
      | "DelegationKicked"
      | "DelegatorExitCancelled"
      | "CancelledDelegationRequest"
      | "Delegation"
      | "DelegatorLeftCandidate"
      | "Rewarded"
      | "InflationDistributed"
      | "InflationDistributionConfigUpdated"
      | "InflationSet"
      | "StakeExpectationsSet"
      | "TotalSelectedSet"
      | "CollatorCommissionSet"
      | "BlocksPerRoundSet"
      | "AutoCompoundSet"
      | "Compounded";
  }

  /** @name PalletParachainStakingDelegationRequestsCancelledScheduledRequest (57) */
  interface PalletParachainStakingDelegationRequestsCancelledScheduledRequest extends Struct {
    readonly whenExecutable: u32;
    readonly action: PalletParachainStakingDelegationRequestsDelegationAction;
  }

  /** @name PalletParachainStakingDelegationRequestsDelegationAction (58) */
  interface PalletParachainStakingDelegationRequestsDelegationAction extends Enum {
    readonly isRevoke: boolean;
    readonly asRevoke: u128;
    readonly isDecrease: boolean;
    readonly asDecrease: u128;
    readonly type: "Revoke" | "Decrease";
  }

  /** @name PalletParachainStakingDelegatorAdded (59) */
  interface PalletParachainStakingDelegatorAdded extends Enum {
    readonly isAddedToTop: boolean;
    readonly asAddedToTop: {
      readonly newTotal: u128;
    } & Struct;
    readonly isAddedToBottom: boolean;
    readonly type: "AddedToTop" | "AddedToBottom";
  }

  /** @name PalletParachainStakingInflationDistributionConfig (61) */
  interface PalletParachainStakingInflationDistributionConfig
    extends Vec<PalletParachainStakingInflationDistributionAccount> {}

  /** @name PalletParachainStakingInflationDistributionAccount (63) */
  interface PalletParachainStakingInflationDistributionAccount extends Struct {
    readonly account: AccountId20;
    readonly percent: Percent;
  }

  /** @name PalletSchedulerEvent (65) */
  interface PalletSchedulerEvent extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isCanceled: boolean;
    readonly asCanceled: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isDispatched: boolean;
    readonly asDispatched: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isRetrySet: boolean;
    readonly asRetrySet: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
      readonly period: u32;
      readonly retries: u8;
    } & Struct;
    readonly isRetryCancelled: boolean;
    readonly asRetryCancelled: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isCallUnavailable: boolean;
    readonly asCallUnavailable: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPeriodicFailed: boolean;
    readonly asPeriodicFailed: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isRetryFailed: boolean;
    readonly asRetryFailed: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPermanentlyOverweight: boolean;
    readonly asPermanentlyOverweight: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly type:
      | "Scheduled"
      | "Canceled"
      | "Dispatched"
      | "RetrySet"
      | "RetryCancelled"
      | "CallUnavailable"
      | "PeriodicFailed"
      | "RetryFailed"
      | "PermanentlyOverweight";
  }

  /** @name PalletTreasuryEvent (67) */
  interface PalletTreasuryEvent extends Enum {
    readonly isSpending: boolean;
    readonly asSpending: {
      readonly budgetRemaining: u128;
    } & Struct;
    readonly isAwarded: boolean;
    readonly asAwarded: {
      readonly proposalIndex: u32;
      readonly award: u128;
      readonly account: AccountId20;
    } & Struct;
    readonly isBurnt: boolean;
    readonly asBurnt: {
      readonly burntFunds: u128;
    } & Struct;
    readonly isRollover: boolean;
    readonly asRollover: {
      readonly rolloverBalance: u128;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly value: u128;
    } & Struct;
    readonly isSpendApproved: boolean;
    readonly asSpendApproved: {
      readonly proposalIndex: u32;
      readonly amount: u128;
      readonly beneficiary: AccountId20;
    } & Struct;
    readonly isUpdatedInactive: boolean;
    readonly asUpdatedInactive: {
      readonly reactivated: u128;
      readonly deactivated: u128;
    } & Struct;
    readonly isAssetSpendApproved: boolean;
    readonly asAssetSpendApproved: {
      readonly index: u32;
      readonly assetKind: FrameSupportTokensFungibleUnionOfNativeOrWithId;
      readonly amount: u128;
      readonly beneficiary: AccountId20;
      readonly validFrom: u32;
      readonly expireAt: u32;
    } & Struct;
    readonly isAssetSpendVoided: boolean;
    readonly asAssetSpendVoided: {
      readonly index: u32;
    } & Struct;
    readonly isPaid: boolean;
    readonly asPaid: {
      readonly index: u32;
      readonly paymentId: Null;
    } & Struct;
    readonly isPaymentFailed: boolean;
    readonly asPaymentFailed: {
      readonly index: u32;
      readonly paymentId: Null;
    } & Struct;
    readonly isSpendProcessed: boolean;
    readonly asSpendProcessed: {
      readonly index: u32;
    } & Struct;
    readonly type:
      | "Spending"
      | "Awarded"
      | "Burnt"
      | "Rollover"
      | "Deposit"
      | "SpendApproved"
      | "UpdatedInactive"
      | "AssetSpendApproved"
      | "AssetSpendVoided"
      | "Paid"
      | "PaymentFailed"
      | "SpendProcessed";
  }

  /** @name FrameSupportTokensFungibleUnionOfNativeOrWithId (68) */
  interface FrameSupportTokensFungibleUnionOfNativeOrWithId extends Enum {
    readonly isNative: boolean;
    readonly isWithId: boolean;
    readonly asWithId: u128;
    readonly type: "Native" | "WithId";
  }

  /** @name PalletAuthorSlotFilterEvent (69) */
  interface PalletAuthorSlotFilterEvent extends Enum {
    readonly isEligibleUpdated: boolean;
    readonly asEligibleUpdated: u32;
    readonly type: "EligibleUpdated";
  }

  /** @name PalletCrowdloanRewardsEvent (71) */
  interface PalletCrowdloanRewardsEvent extends Enum {
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
    readonly asInitializedAccountWithNotEnoughContribution: ITuple<
      [U8aFixed, Option<AccountId20>, u128]
    >;
    readonly type:
      | "InitialPaymentMade"
      | "NativeIdentityAssociated"
      | "RewardsPaid"
      | "RewardAddressUpdated"
      | "InitializedAlreadyInitializedAccount"
      | "InitializedAccountWithNotEnoughContribution";
  }

  /** @name PalletAuthorMappingEvent (72) */
  interface PalletAuthorMappingEvent extends Enum {
    readonly isKeysRegistered: boolean;
    readonly asKeysRegistered: {
      readonly nimbusId: NimbusPrimitivesNimbusCryptoPublic;
      readonly accountId: AccountId20;
      readonly keys_: SessionKeysPrimitivesVrfVrfCryptoPublic;
    } & Struct;
    readonly isKeysRemoved: boolean;
    readonly asKeysRemoved: {
      readonly nimbusId: NimbusPrimitivesNimbusCryptoPublic;
      readonly accountId: AccountId20;
      readonly keys_: SessionKeysPrimitivesVrfVrfCryptoPublic;
    } & Struct;
    readonly isKeysRotated: boolean;
    readonly asKeysRotated: {
      readonly newNimbusId: NimbusPrimitivesNimbusCryptoPublic;
      readonly accountId: AccountId20;
      readonly newKeys: SessionKeysPrimitivesVrfVrfCryptoPublic;
    } & Struct;
    readonly type: "KeysRegistered" | "KeysRemoved" | "KeysRotated";
  }

  /** @name NimbusPrimitivesNimbusCryptoPublic (73) */
  interface NimbusPrimitivesNimbusCryptoPublic extends U8aFixed {}

  /** @name SessionKeysPrimitivesVrfVrfCryptoPublic (74) */
  interface SessionKeysPrimitivesVrfVrfCryptoPublic extends U8aFixed {}

  /** @name PalletProxyEvent (75) */
  interface PalletProxyEvent extends Enum {
    readonly isProxyExecuted: boolean;
    readonly asProxyExecuted: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isPureCreated: boolean;
    readonly asPureCreated: {
      readonly pure: AccountId20;
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
    readonly isProxyRemoved: boolean;
    readonly asProxyRemoved: {
      readonly delegator: AccountId20;
      readonly delegatee: AccountId20;
      readonly proxyType: MoonbaseRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly type: "ProxyExecuted" | "PureCreated" | "Announced" | "ProxyAdded" | "ProxyRemoved";
  }

  /** @name MoonbaseRuntimeProxyType (76) */
  interface MoonbaseRuntimeProxyType extends Enum {
    readonly isAny: boolean;
    readonly isNonTransfer: boolean;
    readonly isGovernance: boolean;
    readonly isStaking: boolean;
    readonly isCancelProxy: boolean;
    readonly isBalances: boolean;
    readonly isAuthorMapping: boolean;
    readonly isIdentityJudgement: boolean;
    readonly type:
      | "Any"
      | "NonTransfer"
      | "Governance"
      | "Staking"
      | "CancelProxy"
      | "Balances"
      | "AuthorMapping"
      | "IdentityJudgement";
  }

  /** @name PalletMaintenanceModeEvent (78) */
  interface PalletMaintenanceModeEvent extends Enum {
    readonly isEnteredMaintenanceMode: boolean;
    readonly isNormalOperationResumed: boolean;
    readonly isFailedToSuspendIdleXcmExecution: boolean;
    readonly asFailedToSuspendIdleXcmExecution: {
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isFailedToResumeIdleXcmExecution: boolean;
    readonly asFailedToResumeIdleXcmExecution: {
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly type:
      | "EnteredMaintenanceMode"
      | "NormalOperationResumed"
      | "FailedToSuspendIdleXcmExecution"
      | "FailedToResumeIdleXcmExecution";
  }

  /** @name PalletIdentityEvent (79) */
  interface PalletIdentityEvent extends Enum {
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
    readonly isSubIdentitiesSet: boolean;
    readonly asSubIdentitiesSet: {
      readonly main: AccountId20;
      readonly numberOfSubs: u32;
      readonly newDeposit: u128;
    } & Struct;
    readonly isSubIdentityRenamed: boolean;
    readonly asSubIdentityRenamed: {
      readonly sub: AccountId20;
      readonly main: AccountId20;
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
    readonly isAuthorityAdded: boolean;
    readonly asAuthorityAdded: {
      readonly authority: AccountId20;
    } & Struct;
    readonly isAuthorityRemoved: boolean;
    readonly asAuthorityRemoved: {
      readonly authority: AccountId20;
    } & Struct;
    readonly isUsernameSet: boolean;
    readonly asUsernameSet: {
      readonly who: AccountId20;
      readonly username: Bytes;
    } & Struct;
    readonly isUsernameQueued: boolean;
    readonly asUsernameQueued: {
      readonly who: AccountId20;
      readonly username: Bytes;
      readonly expiration: u32;
    } & Struct;
    readonly isPreapprovalExpired: boolean;
    readonly asPreapprovalExpired: {
      readonly whose: AccountId20;
    } & Struct;
    readonly isPrimaryUsernameSet: boolean;
    readonly asPrimaryUsernameSet: {
      readonly who: AccountId20;
      readonly username: Bytes;
    } & Struct;
    readonly isDanglingUsernameRemoved: boolean;
    readonly asDanglingUsernameRemoved: {
      readonly who: AccountId20;
      readonly username: Bytes;
    } & Struct;
    readonly isUsernameUnbound: boolean;
    readonly asUsernameUnbound: {
      readonly username: Bytes;
    } & Struct;
    readonly isUsernameRemoved: boolean;
    readonly asUsernameRemoved: {
      readonly username: Bytes;
    } & Struct;
    readonly isUsernameKilled: boolean;
    readonly asUsernameKilled: {
      readonly username: Bytes;
    } & Struct;
    readonly type:
      | "IdentitySet"
      | "IdentityCleared"
      | "IdentityKilled"
      | "JudgementRequested"
      | "JudgementUnrequested"
      | "JudgementGiven"
      | "RegistrarAdded"
      | "SubIdentityAdded"
      | "SubIdentitiesSet"
      | "SubIdentityRenamed"
      | "SubIdentityRemoved"
      | "SubIdentityRevoked"
      | "AuthorityAdded"
      | "AuthorityRemoved"
      | "UsernameSet"
      | "UsernameQueued"
      | "PreapprovalExpired"
      | "PrimaryUsernameSet"
      | "DanglingUsernameRemoved"
      | "UsernameUnbound"
      | "UsernameRemoved"
      | "UsernameKilled";
  }

  /** @name CumulusPalletXcmpQueueEvent (81) */
  interface CumulusPalletXcmpQueueEvent extends Enum {
    readonly isXcmpMessageSent: boolean;
    readonly asXcmpMessageSent: {
      readonly messageHash: U8aFixed;
    } & Struct;
    readonly type: "XcmpMessageSent";
  }

  /** @name CumulusPalletXcmEvent (82) */
  interface CumulusPalletXcmEvent extends Enum {
    readonly isInvalidFormat: boolean;
    readonly asInvalidFormat: U8aFixed;
    readonly isUnsupportedVersion: boolean;
    readonly asUnsupportedVersion: U8aFixed;
    readonly isExecutedDownward: boolean;
    readonly asExecutedDownward: ITuple<[U8aFixed, StagingXcmV5TraitsOutcome]>;
    readonly type: "InvalidFormat" | "UnsupportedVersion" | "ExecutedDownward";
  }

  /** @name StagingXcmV5TraitsOutcome (83) */
  interface StagingXcmV5TraitsOutcome extends Enum {
    readonly isComplete: boolean;
    readonly asComplete: {
      readonly used: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isIncomplete: boolean;
    readonly asIncomplete: {
      readonly used: SpWeightsWeightV2Weight;
      readonly error: XcmV5TraitsError;
    } & Struct;
    readonly isError: boolean;
    readonly asError: {
      readonly error: XcmV5TraitsError;
    } & Struct;
    readonly type: "Complete" | "Incomplete" | "Error";
  }

  /** @name XcmV5TraitsError (84) */
  interface XcmV5TraitsError extends Enum {
    readonly isOverflow: boolean;
    readonly isUnimplemented: boolean;
    readonly isUntrustedReserveLocation: boolean;
    readonly isUntrustedTeleportLocation: boolean;
    readonly isLocationFull: boolean;
    readonly isLocationNotInvertible: boolean;
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
    readonly isMaxWeightInvalid: boolean;
    readonly isNotHoldingFees: boolean;
    readonly isTooExpensive: boolean;
    readonly isTrap: boolean;
    readonly asTrap: u64;
    readonly isExpectationFalse: boolean;
    readonly isPalletNotFound: boolean;
    readonly isNameMismatch: boolean;
    readonly isVersionIncompatible: boolean;
    readonly isHoldingWouldOverflow: boolean;
    readonly isExportError: boolean;
    readonly isReanchorFailed: boolean;
    readonly isNoDeal: boolean;
    readonly isFeesNotMet: boolean;
    readonly isLockError: boolean;
    readonly isNoPermission: boolean;
    readonly isUnanchored: boolean;
    readonly isNotDepositable: boolean;
    readonly isTooManyAssets: boolean;
    readonly isUnhandledXcmVersion: boolean;
    readonly isWeightLimitReached: boolean;
    readonly asWeightLimitReached: SpWeightsWeightV2Weight;
    readonly isBarrier: boolean;
    readonly isWeightNotComputable: boolean;
    readonly isExceedsStackLimit: boolean;
    readonly type:
      | "Overflow"
      | "Unimplemented"
      | "UntrustedReserveLocation"
      | "UntrustedTeleportLocation"
      | "LocationFull"
      | "LocationNotInvertible"
      | "BadOrigin"
      | "InvalidLocation"
      | "AssetNotFound"
      | "FailedToTransactAsset"
      | "NotWithdrawable"
      | "LocationCannotHold"
      | "ExceedsMaxMessageSize"
      | "DestinationUnsupported"
      | "Transport"
      | "Unroutable"
      | "UnknownClaim"
      | "FailedToDecode"
      | "MaxWeightInvalid"
      | "NotHoldingFees"
      | "TooExpensive"
      | "Trap"
      | "ExpectationFalse"
      | "PalletNotFound"
      | "NameMismatch"
      | "VersionIncompatible"
      | "HoldingWouldOverflow"
      | "ExportError"
      | "ReanchorFailed"
      | "NoDeal"
      | "FeesNotMet"
      | "LockError"
      | "NoPermission"
      | "Unanchored"
      | "NotDepositable"
      | "TooManyAssets"
      | "UnhandledXcmVersion"
      | "WeightLimitReached"
      | "Barrier"
      | "WeightNotComputable"
      | "ExceedsStackLimit";
  }

  /** @name PalletXcmEvent (85) */
  interface PalletXcmEvent extends Enum {
    readonly isAttempted: boolean;
    readonly asAttempted: {
      readonly outcome: StagingXcmV5TraitsOutcome;
    } & Struct;
    readonly isSent: boolean;
    readonly asSent: {
      readonly origin: StagingXcmV5Location;
      readonly destination: StagingXcmV5Location;
      readonly message: StagingXcmV5Xcm;
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isUnexpectedResponse: boolean;
    readonly asUnexpectedResponse: {
      readonly origin: StagingXcmV5Location;
      readonly queryId: u64;
    } & Struct;
    readonly isResponseReady: boolean;
    readonly asResponseReady: {
      readonly queryId: u64;
      readonly response: StagingXcmV5Response;
    } & Struct;
    readonly isNotified: boolean;
    readonly asNotified: {
      readonly queryId: u64;
      readonly palletIndex: u8;
      readonly callIndex: u8;
    } & Struct;
    readonly isNotifyOverweight: boolean;
    readonly asNotifyOverweight: {
      readonly queryId: u64;
      readonly palletIndex: u8;
      readonly callIndex: u8;
      readonly actualWeight: SpWeightsWeightV2Weight;
      readonly maxBudgetedWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isNotifyDispatchError: boolean;
    readonly asNotifyDispatchError: {
      readonly queryId: u64;
      readonly palletIndex: u8;
      readonly callIndex: u8;
    } & Struct;
    readonly isNotifyDecodeFailed: boolean;
    readonly asNotifyDecodeFailed: {
      readonly queryId: u64;
      readonly palletIndex: u8;
      readonly callIndex: u8;
    } & Struct;
    readonly isInvalidResponder: boolean;
    readonly asInvalidResponder: {
      readonly origin: StagingXcmV5Location;
      readonly queryId: u64;
      readonly expectedLocation: Option<StagingXcmV5Location>;
    } & Struct;
    readonly isInvalidResponderVersion: boolean;
    readonly asInvalidResponderVersion: {
      readonly origin: StagingXcmV5Location;
      readonly queryId: u64;
    } & Struct;
    readonly isResponseTaken: boolean;
    readonly asResponseTaken: {
      readonly queryId: u64;
    } & Struct;
    readonly isAssetsTrapped: boolean;
    readonly asAssetsTrapped: {
      readonly hash_: H256;
      readonly origin: StagingXcmV5Location;
      readonly assets: XcmVersionedAssets;
    } & Struct;
    readonly isVersionChangeNotified: boolean;
    readonly asVersionChangeNotified: {
      readonly destination: StagingXcmV5Location;
      readonly result: u32;
      readonly cost: StagingXcmV5AssetAssets;
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isSupportedVersionChanged: boolean;
    readonly asSupportedVersionChanged: {
      readonly location: StagingXcmV5Location;
      readonly version: u32;
    } & Struct;
    readonly isNotifyTargetSendFail: boolean;
    readonly asNotifyTargetSendFail: {
      readonly location: StagingXcmV5Location;
      readonly queryId: u64;
      readonly error: XcmV5TraitsError;
    } & Struct;
    readonly isNotifyTargetMigrationFail: boolean;
    readonly asNotifyTargetMigrationFail: {
      readonly location: XcmVersionedLocation;
      readonly queryId: u64;
    } & Struct;
    readonly isInvalidQuerierVersion: boolean;
    readonly asInvalidQuerierVersion: {
      readonly origin: StagingXcmV5Location;
      readonly queryId: u64;
    } & Struct;
    readonly isInvalidQuerier: boolean;
    readonly asInvalidQuerier: {
      readonly origin: StagingXcmV5Location;
      readonly queryId: u64;
      readonly expectedQuerier: StagingXcmV5Location;
      readonly maybeActualQuerier: Option<StagingXcmV5Location>;
    } & Struct;
    readonly isVersionNotifyStarted: boolean;
    readonly asVersionNotifyStarted: {
      readonly destination: StagingXcmV5Location;
      readonly cost: StagingXcmV5AssetAssets;
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isVersionNotifyRequested: boolean;
    readonly asVersionNotifyRequested: {
      readonly destination: StagingXcmV5Location;
      readonly cost: StagingXcmV5AssetAssets;
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isVersionNotifyUnrequested: boolean;
    readonly asVersionNotifyUnrequested: {
      readonly destination: StagingXcmV5Location;
      readonly cost: StagingXcmV5AssetAssets;
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isFeesPaid: boolean;
    readonly asFeesPaid: {
      readonly paying: StagingXcmV5Location;
      readonly fees: StagingXcmV5AssetAssets;
    } & Struct;
    readonly isAssetsClaimed: boolean;
    readonly asAssetsClaimed: {
      readonly hash_: H256;
      readonly origin: StagingXcmV5Location;
      readonly assets: XcmVersionedAssets;
    } & Struct;
    readonly isVersionMigrationFinished: boolean;
    readonly asVersionMigrationFinished: {
      readonly version: u32;
    } & Struct;
    readonly type:
      | "Attempted"
      | "Sent"
      | "UnexpectedResponse"
      | "ResponseReady"
      | "Notified"
      | "NotifyOverweight"
      | "NotifyDispatchError"
      | "NotifyDecodeFailed"
      | "InvalidResponder"
      | "InvalidResponderVersion"
      | "ResponseTaken"
      | "AssetsTrapped"
      | "VersionChangeNotified"
      | "SupportedVersionChanged"
      | "NotifyTargetSendFail"
      | "NotifyTargetMigrationFail"
      | "InvalidQuerierVersion"
      | "InvalidQuerier"
      | "VersionNotifyStarted"
      | "VersionNotifyRequested"
      | "VersionNotifyUnrequested"
      | "FeesPaid"
      | "AssetsClaimed"
      | "VersionMigrationFinished";
  }

  /** @name StagingXcmV5Location (86) */
  interface StagingXcmV5Location extends Struct {
    readonly parents: u8;
    readonly interior: StagingXcmV5Junctions;
  }

  /** @name StagingXcmV5Junctions (87) */
  interface StagingXcmV5Junctions extends Enum {
    readonly isHere: boolean;
    readonly isX1: boolean;
    readonly asX1: Vec<StagingXcmV5Junction>;
    readonly isX2: boolean;
    readonly asX2: Vec<StagingXcmV5Junction>;
    readonly isX3: boolean;
    readonly asX3: Vec<StagingXcmV5Junction>;
    readonly isX4: boolean;
    readonly asX4: Vec<StagingXcmV5Junction>;
    readonly isX5: boolean;
    readonly asX5: Vec<StagingXcmV5Junction>;
    readonly isX6: boolean;
    readonly asX6: Vec<StagingXcmV5Junction>;
    readonly isX7: boolean;
    readonly asX7: Vec<StagingXcmV5Junction>;
    readonly isX8: boolean;
    readonly asX8: Vec<StagingXcmV5Junction>;
    readonly type: "Here" | "X1" | "X2" | "X3" | "X4" | "X5" | "X6" | "X7" | "X8";
  }

  /** @name StagingXcmV5Junction (89) */
  interface StagingXcmV5Junction extends Enum {
    readonly isParachain: boolean;
    readonly asParachain: Compact<u32>;
    readonly isAccountId32: boolean;
    readonly asAccountId32: {
      readonly network: Option<StagingXcmV5JunctionNetworkId>;
      readonly id: U8aFixed;
    } & Struct;
    readonly isAccountIndex64: boolean;
    readonly asAccountIndex64: {
      readonly network: Option<StagingXcmV5JunctionNetworkId>;
      readonly index: Compact<u64>;
    } & Struct;
    readonly isAccountKey20: boolean;
    readonly asAccountKey20: {
      readonly network: Option<StagingXcmV5JunctionNetworkId>;
      readonly key: U8aFixed;
    } & Struct;
    readonly isPalletInstance: boolean;
    readonly asPalletInstance: u8;
    readonly isGeneralIndex: boolean;
    readonly asGeneralIndex: Compact<u128>;
    readonly isGeneralKey: boolean;
    readonly asGeneralKey: {
      readonly length: u8;
      readonly data: U8aFixed;
    } & Struct;
    readonly isOnlyChild: boolean;
    readonly isPlurality: boolean;
    readonly asPlurality: {
      readonly id: XcmV3JunctionBodyId;
      readonly part: XcmV3JunctionBodyPart;
    } & Struct;
    readonly isGlobalConsensus: boolean;
    readonly asGlobalConsensus: StagingXcmV5JunctionNetworkId;
    readonly type:
      | "Parachain"
      | "AccountId32"
      | "AccountIndex64"
      | "AccountKey20"
      | "PalletInstance"
      | "GeneralIndex"
      | "GeneralKey"
      | "OnlyChild"
      | "Plurality"
      | "GlobalConsensus";
  }

  /** @name StagingXcmV5JunctionNetworkId (92) */
  interface StagingXcmV5JunctionNetworkId extends Enum {
    readonly isByGenesis: boolean;
    readonly asByGenesis: U8aFixed;
    readonly isByFork: boolean;
    readonly asByFork: {
      readonly blockNumber: u64;
      readonly blockHash: U8aFixed;
    } & Struct;
    readonly isPolkadot: boolean;
    readonly isKusama: boolean;
    readonly isEthereum: boolean;
    readonly asEthereum: {
      readonly chainId: Compact<u64>;
    } & Struct;
    readonly isBitcoinCore: boolean;
    readonly isBitcoinCash: boolean;
    readonly isPolkadotBulletin: boolean;
    readonly type:
      | "ByGenesis"
      | "ByFork"
      | "Polkadot"
      | "Kusama"
      | "Ethereum"
      | "BitcoinCore"
      | "BitcoinCash"
      | "PolkadotBulletin";
  }

  /** @name XcmV3JunctionBodyId (94) */
  interface XcmV3JunctionBodyId extends Enum {
    readonly isUnit: boolean;
    readonly isMoniker: boolean;
    readonly asMoniker: U8aFixed;
    readonly isIndex: boolean;
    readonly asIndex: Compact<u32>;
    readonly isExecutive: boolean;
    readonly isTechnical: boolean;
    readonly isLegislative: boolean;
    readonly isJudicial: boolean;
    readonly isDefense: boolean;
    readonly isAdministration: boolean;
    readonly isTreasury: boolean;
    readonly type:
      | "Unit"
      | "Moniker"
      | "Index"
      | "Executive"
      | "Technical"
      | "Legislative"
      | "Judicial"
      | "Defense"
      | "Administration"
      | "Treasury";
  }

  /** @name XcmV3JunctionBodyPart (95) */
  interface XcmV3JunctionBodyPart extends Enum {
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
    readonly type: "Voice" | "Members" | "Fraction" | "AtLeastProportion" | "MoreThanProportion";
  }

  /** @name StagingXcmV5Xcm (103) */
  interface StagingXcmV5Xcm extends Vec<StagingXcmV5Instruction> {}

  /** @name StagingXcmV5Instruction (105) */
  interface StagingXcmV5Instruction extends Enum {
    readonly isWithdrawAsset: boolean;
    readonly asWithdrawAsset: StagingXcmV5AssetAssets;
    readonly isReserveAssetDeposited: boolean;
    readonly asReserveAssetDeposited: StagingXcmV5AssetAssets;
    readonly isReceiveTeleportedAsset: boolean;
    readonly asReceiveTeleportedAsset: StagingXcmV5AssetAssets;
    readonly isQueryResponse: boolean;
    readonly asQueryResponse: {
      readonly queryId: Compact<u64>;
      readonly response: StagingXcmV5Response;
      readonly maxWeight: SpWeightsWeightV2Weight;
      readonly querier: Option<StagingXcmV5Location>;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assets: StagingXcmV5AssetAssets;
      readonly beneficiary: StagingXcmV5Location;
    } & Struct;
    readonly isTransferReserveAsset: boolean;
    readonly asTransferReserveAsset: {
      readonly assets: StagingXcmV5AssetAssets;
      readonly dest: StagingXcmV5Location;
      readonly xcm: StagingXcmV5Xcm;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly originKind: XcmV3OriginKind;
      readonly fallbackMaxWeight: Option<SpWeightsWeightV2Weight>;
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
    readonly asDescendOrigin: StagingXcmV5Junctions;
    readonly isReportError: boolean;
    readonly asReportError: StagingXcmV5QueryResponseInfo;
    readonly isDepositAsset: boolean;
    readonly asDepositAsset: {
      readonly assets: StagingXcmV5AssetAssetFilter;
      readonly beneficiary: StagingXcmV5Location;
    } & Struct;
    readonly isDepositReserveAsset: boolean;
    readonly asDepositReserveAsset: {
      readonly assets: StagingXcmV5AssetAssetFilter;
      readonly dest: StagingXcmV5Location;
      readonly xcm: StagingXcmV5Xcm;
    } & Struct;
    readonly isExchangeAsset: boolean;
    readonly asExchangeAsset: {
      readonly give: StagingXcmV5AssetAssetFilter;
      readonly want: StagingXcmV5AssetAssets;
      readonly maximal: bool;
    } & Struct;
    readonly isInitiateReserveWithdraw: boolean;
    readonly asInitiateReserveWithdraw: {
      readonly assets: StagingXcmV5AssetAssetFilter;
      readonly reserve: StagingXcmV5Location;
      readonly xcm: StagingXcmV5Xcm;
    } & Struct;
    readonly isInitiateTeleport: boolean;
    readonly asInitiateTeleport: {
      readonly assets: StagingXcmV5AssetAssetFilter;
      readonly dest: StagingXcmV5Location;
      readonly xcm: StagingXcmV5Xcm;
    } & Struct;
    readonly isReportHolding: boolean;
    readonly asReportHolding: {
      readonly responseInfo: StagingXcmV5QueryResponseInfo;
      readonly assets: StagingXcmV5AssetAssetFilter;
    } & Struct;
    readonly isBuyExecution: boolean;
    readonly asBuyExecution: {
      readonly fees: StagingXcmV5Asset;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isRefundSurplus: boolean;
    readonly isSetErrorHandler: boolean;
    readonly asSetErrorHandler: StagingXcmV5Xcm;
    readonly isSetAppendix: boolean;
    readonly asSetAppendix: StagingXcmV5Xcm;
    readonly isClearError: boolean;
    readonly isClaimAsset: boolean;
    readonly asClaimAsset: {
      readonly assets: StagingXcmV5AssetAssets;
      readonly ticket: StagingXcmV5Location;
    } & Struct;
    readonly isTrap: boolean;
    readonly asTrap: Compact<u64>;
    readonly isSubscribeVersion: boolean;
    readonly asSubscribeVersion: {
      readonly queryId: Compact<u64>;
      readonly maxResponseWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isUnsubscribeVersion: boolean;
    readonly isBurnAsset: boolean;
    readonly asBurnAsset: StagingXcmV5AssetAssets;
    readonly isExpectAsset: boolean;
    readonly asExpectAsset: StagingXcmV5AssetAssets;
    readonly isExpectOrigin: boolean;
    readonly asExpectOrigin: Option<StagingXcmV5Location>;
    readonly isExpectError: boolean;
    readonly asExpectError: Option<ITuple<[u32, XcmV5TraitsError]>>;
    readonly isExpectTransactStatus: boolean;
    readonly asExpectTransactStatus: XcmV3MaybeErrorCode;
    readonly isQueryPallet: boolean;
    readonly asQueryPallet: {
      readonly moduleName: Bytes;
      readonly responseInfo: StagingXcmV5QueryResponseInfo;
    } & Struct;
    readonly isExpectPallet: boolean;
    readonly asExpectPallet: {
      readonly index: Compact<u32>;
      readonly name: Bytes;
      readonly moduleName: Bytes;
      readonly crateMajor: Compact<u32>;
      readonly minCrateMinor: Compact<u32>;
    } & Struct;
    readonly isReportTransactStatus: boolean;
    readonly asReportTransactStatus: StagingXcmV5QueryResponseInfo;
    readonly isClearTransactStatus: boolean;
    readonly isUniversalOrigin: boolean;
    readonly asUniversalOrigin: StagingXcmV5Junction;
    readonly isExportMessage: boolean;
    readonly asExportMessage: {
      readonly network: StagingXcmV5JunctionNetworkId;
      readonly destination: StagingXcmV5Junctions;
      readonly xcm: StagingXcmV5Xcm;
    } & Struct;
    readonly isLockAsset: boolean;
    readonly asLockAsset: {
      readonly asset: StagingXcmV5Asset;
      readonly unlocker: StagingXcmV5Location;
    } & Struct;
    readonly isUnlockAsset: boolean;
    readonly asUnlockAsset: {
      readonly asset: StagingXcmV5Asset;
      readonly target: StagingXcmV5Location;
    } & Struct;
    readonly isNoteUnlockable: boolean;
    readonly asNoteUnlockable: {
      readonly asset: StagingXcmV5Asset;
      readonly owner: StagingXcmV5Location;
    } & Struct;
    readonly isRequestUnlock: boolean;
    readonly asRequestUnlock: {
      readonly asset: StagingXcmV5Asset;
      readonly locker: StagingXcmV5Location;
    } & Struct;
    readonly isSetFeesMode: boolean;
    readonly asSetFeesMode: {
      readonly jitWithdraw: bool;
    } & Struct;
    readonly isSetTopic: boolean;
    readonly asSetTopic: U8aFixed;
    readonly isClearTopic: boolean;
    readonly isAliasOrigin: boolean;
    readonly asAliasOrigin: StagingXcmV5Location;
    readonly isUnpaidExecution: boolean;
    readonly asUnpaidExecution: {
      readonly weightLimit: XcmV3WeightLimit;
      readonly checkOrigin: Option<StagingXcmV5Location>;
    } & Struct;
    readonly isPayFees: boolean;
    readonly asPayFees: {
      readonly asset: StagingXcmV5Asset;
    } & Struct;
    readonly isInitiateTransfer: boolean;
    readonly asInitiateTransfer: {
      readonly destination: StagingXcmV5Location;
      readonly remoteFees: Option<StagingXcmV5AssetAssetTransferFilter>;
      readonly preserveOrigin: bool;
      readonly assets: Vec<StagingXcmV5AssetAssetTransferFilter>;
      readonly remoteXcm: StagingXcmV5Xcm;
    } & Struct;
    readonly isExecuteWithOrigin: boolean;
    readonly asExecuteWithOrigin: {
      readonly descendantOrigin: Option<StagingXcmV5Junctions>;
      readonly xcm: StagingXcmV5Xcm;
    } & Struct;
    readonly isSetHints: boolean;
    readonly asSetHints: {
      readonly hints: Vec<StagingXcmV5Hint>;
    } & Struct;
    readonly type:
      | "WithdrawAsset"
      | "ReserveAssetDeposited"
      | "ReceiveTeleportedAsset"
      | "QueryResponse"
      | "TransferAsset"
      | "TransferReserveAsset"
      | "Transact"
      | "HrmpNewChannelOpenRequest"
      | "HrmpChannelAccepted"
      | "HrmpChannelClosing"
      | "ClearOrigin"
      | "DescendOrigin"
      | "ReportError"
      | "DepositAsset"
      | "DepositReserveAsset"
      | "ExchangeAsset"
      | "InitiateReserveWithdraw"
      | "InitiateTeleport"
      | "ReportHolding"
      | "BuyExecution"
      | "RefundSurplus"
      | "SetErrorHandler"
      | "SetAppendix"
      | "ClearError"
      | "ClaimAsset"
      | "Trap"
      | "SubscribeVersion"
      | "UnsubscribeVersion"
      | "BurnAsset"
      | "ExpectAsset"
      | "ExpectOrigin"
      | "ExpectError"
      | "ExpectTransactStatus"
      | "QueryPallet"
      | "ExpectPallet"
      | "ReportTransactStatus"
      | "ClearTransactStatus"
      | "UniversalOrigin"
      | "ExportMessage"
      | "LockAsset"
      | "UnlockAsset"
      | "NoteUnlockable"
      | "RequestUnlock"
      | "SetFeesMode"
      | "SetTopic"
      | "ClearTopic"
      | "AliasOrigin"
      | "UnpaidExecution"
      | "PayFees"
      | "InitiateTransfer"
      | "ExecuteWithOrigin"
      | "SetHints";
  }

  /** @name StagingXcmV5AssetAssets (106) */
  interface StagingXcmV5AssetAssets extends Vec<StagingXcmV5Asset> {}

  /** @name StagingXcmV5Asset (108) */
  interface StagingXcmV5Asset extends Struct {
    readonly id: StagingXcmV5AssetAssetId;
    readonly fun: StagingXcmV5AssetFungibility;
  }

  /** @name StagingXcmV5AssetAssetId (109) */
  interface StagingXcmV5AssetAssetId extends StagingXcmV5Location {}

  /** @name StagingXcmV5AssetFungibility (110) */
  interface StagingXcmV5AssetFungibility extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: Compact<u128>;
    readonly isNonFungible: boolean;
    readonly asNonFungible: StagingXcmV5AssetAssetInstance;
    readonly type: "Fungible" | "NonFungible";
  }

  /** @name StagingXcmV5AssetAssetInstance (111) */
  interface StagingXcmV5AssetAssetInstance extends Enum {
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
    readonly type: "Undefined" | "Index" | "Array4" | "Array8" | "Array16" | "Array32";
  }

  /** @name StagingXcmV5Response (114) */
  interface StagingXcmV5Response extends Enum {
    readonly isNull: boolean;
    readonly isAssets: boolean;
    readonly asAssets: StagingXcmV5AssetAssets;
    readonly isExecutionResult: boolean;
    readonly asExecutionResult: Option<ITuple<[u32, XcmV5TraitsError]>>;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly isPalletsInfo: boolean;
    readonly asPalletsInfo: Vec<StagingXcmV5PalletInfo>;
    readonly isDispatchResult: boolean;
    readonly asDispatchResult: XcmV3MaybeErrorCode;
    readonly type:
      | "Null"
      | "Assets"
      | "ExecutionResult"
      | "Version"
      | "PalletsInfo"
      | "DispatchResult";
  }

  /** @name StagingXcmV5PalletInfo (118) */
  interface StagingXcmV5PalletInfo extends Struct {
    readonly index: Compact<u32>;
    readonly name: Bytes;
    readonly moduleName: Bytes;
    readonly major: Compact<u32>;
    readonly minor: Compact<u32>;
    readonly patch: Compact<u32>;
  }

  /** @name XcmV3MaybeErrorCode (121) */
  interface XcmV3MaybeErrorCode extends Enum {
    readonly isSuccess: boolean;
    readonly isError: boolean;
    readonly asError: Bytes;
    readonly isTruncatedError: boolean;
    readonly asTruncatedError: Bytes;
    readonly type: "Success" | "Error" | "TruncatedError";
  }

  /** @name XcmV3OriginKind (124) */
  interface XcmV3OriginKind extends Enum {
    readonly isNative: boolean;
    readonly isSovereignAccount: boolean;
    readonly isSuperuser: boolean;
    readonly isXcm: boolean;
    readonly type: "Native" | "SovereignAccount" | "Superuser" | "Xcm";
  }

  /** @name XcmDoubleEncoded (126) */
  interface XcmDoubleEncoded extends Struct {
    readonly encoded: Bytes;
  }

  /** @name StagingXcmV5QueryResponseInfo (127) */
  interface StagingXcmV5QueryResponseInfo extends Struct {
    readonly destination: StagingXcmV5Location;
    readonly queryId: Compact<u64>;
    readonly maxWeight: SpWeightsWeightV2Weight;
  }

  /** @name StagingXcmV5AssetAssetFilter (128) */
  interface StagingXcmV5AssetAssetFilter extends Enum {
    readonly isDefinite: boolean;
    readonly asDefinite: StagingXcmV5AssetAssets;
    readonly isWild: boolean;
    readonly asWild: StagingXcmV5AssetWildAsset;
    readonly type: "Definite" | "Wild";
  }

  /** @name StagingXcmV5AssetWildAsset (129) */
  interface StagingXcmV5AssetWildAsset extends Enum {
    readonly isAll: boolean;
    readonly isAllOf: boolean;
    readonly asAllOf: {
      readonly id: StagingXcmV5AssetAssetId;
      readonly fun: StagingXcmV5AssetWildFungibility;
    } & Struct;
    readonly isAllCounted: boolean;
    readonly asAllCounted: Compact<u32>;
    readonly isAllOfCounted: boolean;
    readonly asAllOfCounted: {
      readonly id: StagingXcmV5AssetAssetId;
      readonly fun: StagingXcmV5AssetWildFungibility;
      readonly count: Compact<u32>;
    } & Struct;
    readonly type: "All" | "AllOf" | "AllCounted" | "AllOfCounted";
  }

  /** @name StagingXcmV5AssetWildFungibility (130) */
  interface StagingXcmV5AssetWildFungibility extends Enum {
    readonly isFungible: boolean;
    readonly isNonFungible: boolean;
    readonly type: "Fungible" | "NonFungible";
  }

  /** @name XcmV3WeightLimit (131) */
  interface XcmV3WeightLimit extends Enum {
    readonly isUnlimited: boolean;
    readonly isLimited: boolean;
    readonly asLimited: SpWeightsWeightV2Weight;
    readonly type: "Unlimited" | "Limited";
  }

  /** @name StagingXcmV5AssetAssetTransferFilter (133) */
  interface StagingXcmV5AssetAssetTransferFilter extends Enum {
    readonly isTeleport: boolean;
    readonly asTeleport: StagingXcmV5AssetAssetFilter;
    readonly isReserveDeposit: boolean;
    readonly asReserveDeposit: StagingXcmV5AssetAssetFilter;
    readonly isReserveWithdraw: boolean;
    readonly asReserveWithdraw: StagingXcmV5AssetAssetFilter;
    readonly type: "Teleport" | "ReserveDeposit" | "ReserveWithdraw";
  }

  /** @name StagingXcmV5Hint (138) */
  interface StagingXcmV5Hint extends Enum {
    readonly isAssetClaimer: boolean;
    readonly asAssetClaimer: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly type: "AssetClaimer";
  }

  /** @name XcmVersionedAssets (140) */
  interface XcmVersionedAssets extends Enum {
    readonly isV3: boolean;
    readonly asV3: XcmV3MultiassetMultiAssets;
    readonly isV4: boolean;
    readonly asV4: StagingXcmV4AssetAssets;
    readonly isV5: boolean;
    readonly asV5: StagingXcmV5AssetAssets;
    readonly type: "V3" | "V4" | "V5";
  }

  /** @name XcmV3MultiassetMultiAssets (141) */
  interface XcmV3MultiassetMultiAssets extends Vec<XcmV3MultiAsset> {}

  /** @name XcmV3MultiAsset (143) */
  interface XcmV3MultiAsset extends Struct {
    readonly id: XcmV3MultiassetAssetId;
    readonly fun: XcmV3MultiassetFungibility;
  }

  /** @name XcmV3MultiassetAssetId (144) */
  interface XcmV3MultiassetAssetId extends Enum {
    readonly isConcrete: boolean;
    readonly asConcrete: StagingXcmV3MultiLocation;
    readonly isAbstract: boolean;
    readonly asAbstract: U8aFixed;
    readonly type: "Concrete" | "Abstract";
  }

  /** @name StagingXcmV3MultiLocation (145) */
  interface StagingXcmV3MultiLocation extends Struct {
    readonly parents: u8;
    readonly interior: XcmV3Junctions;
  }

  /** @name XcmV3Junctions (146) */
  interface XcmV3Junctions extends Enum {
    readonly isHere: boolean;
    readonly isX1: boolean;
    readonly asX1: XcmV3Junction;
    readonly isX2: boolean;
    readonly asX2: ITuple<[XcmV3Junction, XcmV3Junction]>;
    readonly isX3: boolean;
    readonly asX3: ITuple<[XcmV3Junction, XcmV3Junction, XcmV3Junction]>;
    readonly isX4: boolean;
    readonly asX4: ITuple<[XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction]>;
    readonly isX5: boolean;
    readonly asX5: ITuple<
      [XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction]
    >;
    readonly isX6: boolean;
    readonly asX6: ITuple<
      [XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction]
    >;
    readonly isX7: boolean;
    readonly asX7: ITuple<
      [
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction
      ]
    >;
    readonly isX8: boolean;
    readonly asX8: ITuple<
      [
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction,
        XcmV3Junction
      ]
    >;
    readonly type: "Here" | "X1" | "X2" | "X3" | "X4" | "X5" | "X6" | "X7" | "X8";
  }

  /** @name XcmV3Junction (147) */
  interface XcmV3Junction extends Enum {
    readonly isParachain: boolean;
    readonly asParachain: Compact<u32>;
    readonly isAccountId32: boolean;
    readonly asAccountId32: {
      readonly network: Option<XcmV3JunctionNetworkId>;
      readonly id: U8aFixed;
    } & Struct;
    readonly isAccountIndex64: boolean;
    readonly asAccountIndex64: {
      readonly network: Option<XcmV3JunctionNetworkId>;
      readonly index: Compact<u64>;
    } & Struct;
    readonly isAccountKey20: boolean;
    readonly asAccountKey20: {
      readonly network: Option<XcmV3JunctionNetworkId>;
      readonly key: U8aFixed;
    } & Struct;
    readonly isPalletInstance: boolean;
    readonly asPalletInstance: u8;
    readonly isGeneralIndex: boolean;
    readonly asGeneralIndex: Compact<u128>;
    readonly isGeneralKey: boolean;
    readonly asGeneralKey: {
      readonly length: u8;
      readonly data: U8aFixed;
    } & Struct;
    readonly isOnlyChild: boolean;
    readonly isPlurality: boolean;
    readonly asPlurality: {
      readonly id: XcmV3JunctionBodyId;
      readonly part: XcmV3JunctionBodyPart;
    } & Struct;
    readonly isGlobalConsensus: boolean;
    readonly asGlobalConsensus: XcmV3JunctionNetworkId;
    readonly type:
      | "Parachain"
      | "AccountId32"
      | "AccountIndex64"
      | "AccountKey20"
      | "PalletInstance"
      | "GeneralIndex"
      | "GeneralKey"
      | "OnlyChild"
      | "Plurality"
      | "GlobalConsensus";
  }

  /** @name XcmV3JunctionNetworkId (149) */
  interface XcmV3JunctionNetworkId extends Enum {
    readonly isByGenesis: boolean;
    readonly asByGenesis: U8aFixed;
    readonly isByFork: boolean;
    readonly asByFork: {
      readonly blockNumber: u64;
      readonly blockHash: U8aFixed;
    } & Struct;
    readonly isPolkadot: boolean;
    readonly isKusama: boolean;
    readonly isWestend: boolean;
    readonly isRococo: boolean;
    readonly isWococo: boolean;
    readonly isEthereum: boolean;
    readonly asEthereum: {
      readonly chainId: Compact<u64>;
    } & Struct;
    readonly isBitcoinCore: boolean;
    readonly isBitcoinCash: boolean;
    readonly isPolkadotBulletin: boolean;
    readonly type:
      | "ByGenesis"
      | "ByFork"
      | "Polkadot"
      | "Kusama"
      | "Westend"
      | "Rococo"
      | "Wococo"
      | "Ethereum"
      | "BitcoinCore"
      | "BitcoinCash"
      | "PolkadotBulletin";
  }

  /** @name XcmV3MultiassetFungibility (150) */
  interface XcmV3MultiassetFungibility extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: Compact<u128>;
    readonly isNonFungible: boolean;
    readonly asNonFungible: XcmV3MultiassetAssetInstance;
    readonly type: "Fungible" | "NonFungible";
  }

  /** @name XcmV3MultiassetAssetInstance (151) */
  interface XcmV3MultiassetAssetInstance extends Enum {
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
    readonly type: "Undefined" | "Index" | "Array4" | "Array8" | "Array16" | "Array32";
  }

  /** @name StagingXcmV4AssetAssets (152) */
  interface StagingXcmV4AssetAssets extends Vec<StagingXcmV4Asset> {}

  /** @name StagingXcmV4Asset (154) */
  interface StagingXcmV4Asset extends Struct {
    readonly id: StagingXcmV4AssetAssetId;
    readonly fun: StagingXcmV4AssetFungibility;
  }

  /** @name StagingXcmV4AssetAssetId (155) */
  interface StagingXcmV4AssetAssetId extends StagingXcmV4Location {}

  /** @name StagingXcmV4Location (156) */
  interface StagingXcmV4Location extends Struct {
    readonly parents: u8;
    readonly interior: StagingXcmV4Junctions;
  }

  /** @name StagingXcmV4Junctions (157) */
  interface StagingXcmV4Junctions extends Enum {
    readonly isHere: boolean;
    readonly isX1: boolean;
    readonly asX1: StagingXcmV4Junction;
    readonly isX2: boolean;
    readonly asX2: StagingXcmV4Junction;
    readonly isX3: boolean;
    readonly asX3: StagingXcmV4Junction;
    readonly isX4: boolean;
    readonly asX4: StagingXcmV4Junction;
    readonly isX5: boolean;
    readonly asX5: StagingXcmV4Junction;
    readonly isX6: boolean;
    readonly asX6: StagingXcmV4Junction;
    readonly isX7: boolean;
    readonly asX7: StagingXcmV4Junction;
    readonly isX8: boolean;
    readonly asX8: StagingXcmV4Junction;
    readonly type: "Here" | "X1" | "X2" | "X3" | "X4" | "X5" | "X6" | "X7" | "X8";
  }

  /** @name StagingXcmV4Junction (159) */
  interface StagingXcmV4Junction extends Enum {
    readonly isParachain: boolean;
    readonly asParachain: Compact<u32>;
    readonly isAccountId32: boolean;
    readonly asAccountId32: {
      readonly network: Option<StagingXcmV4JunctionNetworkId>;
      readonly id: U8aFixed;
    } & Struct;
    readonly isAccountIndex64: boolean;
    readonly asAccountIndex64: {
      readonly network: Option<StagingXcmV4JunctionNetworkId>;
      readonly index: Compact<u64>;
    } & Struct;
    readonly isAccountKey20: boolean;
    readonly asAccountKey20: {
      readonly network: Option<StagingXcmV4JunctionNetworkId>;
      readonly key: U8aFixed;
    } & Struct;
    readonly isPalletInstance: boolean;
    readonly asPalletInstance: u8;
    readonly isGeneralIndex: boolean;
    readonly asGeneralIndex: Compact<u128>;
    readonly isGeneralKey: boolean;
    readonly asGeneralKey: {
      readonly length: u8;
      readonly data: U8aFixed;
    } & Struct;
    readonly isOnlyChild: boolean;
    readonly isPlurality: boolean;
    readonly asPlurality: {
      readonly id: XcmV3JunctionBodyId;
      readonly part: XcmV3JunctionBodyPart;
    } & Struct;
    readonly isGlobalConsensus: boolean;
    readonly asGlobalConsensus: StagingXcmV4JunctionNetworkId;
    readonly type:
      | "Parachain"
      | "AccountId32"
      | "AccountIndex64"
      | "AccountKey20"
      | "PalletInstance"
      | "GeneralIndex"
      | "GeneralKey"
      | "OnlyChild"
      | "Plurality"
      | "GlobalConsensus";
  }

  /** @name StagingXcmV4JunctionNetworkId (161) */
  interface StagingXcmV4JunctionNetworkId extends Enum {
    readonly isByGenesis: boolean;
    readonly asByGenesis: U8aFixed;
    readonly isByFork: boolean;
    readonly asByFork: {
      readonly blockNumber: u64;
      readonly blockHash: U8aFixed;
    } & Struct;
    readonly isPolkadot: boolean;
    readonly isKusama: boolean;
    readonly isWestend: boolean;
    readonly isRococo: boolean;
    readonly isWococo: boolean;
    readonly isEthereum: boolean;
    readonly asEthereum: {
      readonly chainId: Compact<u64>;
    } & Struct;
    readonly isBitcoinCore: boolean;
    readonly isBitcoinCash: boolean;
    readonly isPolkadotBulletin: boolean;
    readonly type:
      | "ByGenesis"
      | "ByFork"
      | "Polkadot"
      | "Kusama"
      | "Westend"
      | "Rococo"
      | "Wococo"
      | "Ethereum"
      | "BitcoinCore"
      | "BitcoinCash"
      | "PolkadotBulletin";
  }

  /** @name StagingXcmV4AssetFungibility (169) */
  interface StagingXcmV4AssetFungibility extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: Compact<u128>;
    readonly isNonFungible: boolean;
    readonly asNonFungible: StagingXcmV4AssetAssetInstance;
    readonly type: "Fungible" | "NonFungible";
  }

  /** @name StagingXcmV4AssetAssetInstance (170) */
  interface StagingXcmV4AssetAssetInstance extends Enum {
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
    readonly type: "Undefined" | "Index" | "Array4" | "Array8" | "Array16" | "Array32";
  }

  /** @name XcmVersionedLocation (171) */
  interface XcmVersionedLocation extends Enum {
    readonly isV3: boolean;
    readonly asV3: StagingXcmV3MultiLocation;
    readonly isV4: boolean;
    readonly asV4: StagingXcmV4Location;
    readonly isV5: boolean;
    readonly asV5: StagingXcmV5Location;
    readonly type: "V3" | "V4" | "V5";
  }

  /** @name PalletAssetsEvent (172) */
  interface PalletAssetsEvent extends Enum {
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
      readonly amount: u128;
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
    readonly isAccountsDestroyed: boolean;
    readonly asAccountsDestroyed: {
      readonly assetId: u128;
      readonly accountsDestroyed: u32;
      readonly accountsRemaining: u32;
    } & Struct;
    readonly isApprovalsDestroyed: boolean;
    readonly asApprovalsDestroyed: {
      readonly assetId: u128;
      readonly approvalsDestroyed: u32;
      readonly approvalsRemaining: u32;
    } & Struct;
    readonly isDestructionStarted: boolean;
    readonly asDestructionStarted: {
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
    readonly isAssetMinBalanceChanged: boolean;
    readonly asAssetMinBalanceChanged: {
      readonly assetId: u128;
      readonly newMinBalance: u128;
    } & Struct;
    readonly isTouched: boolean;
    readonly asTouched: {
      readonly assetId: u128;
      readonly who: AccountId20;
      readonly depositor: AccountId20;
    } & Struct;
    readonly isBlocked: boolean;
    readonly asBlocked: {
      readonly assetId: u128;
      readonly who: AccountId20;
    } & Struct;
    readonly isDeposited: boolean;
    readonly asDeposited: {
      readonly assetId: u128;
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isWithdrawn: boolean;
    readonly asWithdrawn: {
      readonly assetId: u128;
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly type:
      | "Created"
      | "Issued"
      | "Transferred"
      | "Burned"
      | "TeamChanged"
      | "OwnerChanged"
      | "Frozen"
      | "Thawed"
      | "AssetFrozen"
      | "AssetThawed"
      | "AccountsDestroyed"
      | "ApprovalsDestroyed"
      | "DestructionStarted"
      | "Destroyed"
      | "ForceCreated"
      | "MetadataSet"
      | "MetadataCleared"
      | "ApprovedTransfer"
      | "ApprovalCancelled"
      | "TransferredApproved"
      | "AssetStatusChanged"
      | "AssetMinBalanceChanged"
      | "Touched"
      | "Blocked"
      | "Deposited"
      | "Withdrawn";
  }

  /** @name PalletAssetManagerEvent (173) */
  interface PalletAssetManagerEvent extends Enum {
    readonly isForeignAssetRegistered: boolean;
    readonly asForeignAssetRegistered: {
      readonly assetId: u128;
      readonly asset: MoonbaseRuntimeXcmConfigAssetType;
      readonly metadata: MoonbaseRuntimeAssetConfigAssetRegistrarMetadata;
    } & Struct;
    readonly isUnitsPerSecondChanged: boolean;
    readonly isForeignAssetXcmLocationChanged: boolean;
    readonly asForeignAssetXcmLocationChanged: {
      readonly assetId: u128;
      readonly newAssetType: MoonbaseRuntimeXcmConfigAssetType;
    } & Struct;
    readonly isForeignAssetRemoved: boolean;
    readonly asForeignAssetRemoved: {
      readonly assetId: u128;
      readonly assetType: MoonbaseRuntimeXcmConfigAssetType;
    } & Struct;
    readonly isSupportedAssetRemoved: boolean;
    readonly asSupportedAssetRemoved: {
      readonly assetType: MoonbaseRuntimeXcmConfigAssetType;
    } & Struct;
    readonly isForeignAssetDestroyed: boolean;
    readonly asForeignAssetDestroyed: {
      readonly assetId: u128;
      readonly assetType: MoonbaseRuntimeXcmConfigAssetType;
    } & Struct;
    readonly isLocalAssetDestroyed: boolean;
    readonly asLocalAssetDestroyed: {
      readonly assetId: u128;
    } & Struct;
    readonly type:
      | "ForeignAssetRegistered"
      | "UnitsPerSecondChanged"
      | "ForeignAssetXcmLocationChanged"
      | "ForeignAssetRemoved"
      | "SupportedAssetRemoved"
      | "ForeignAssetDestroyed"
      | "LocalAssetDestroyed";
  }

  /** @name MoonbaseRuntimeXcmConfigAssetType (174) */
  interface MoonbaseRuntimeXcmConfigAssetType extends Enum {
    readonly isXcm: boolean;
    readonly asXcm: StagingXcmV3MultiLocation;
    readonly type: "Xcm";
  }

  /** @name MoonbaseRuntimeAssetConfigAssetRegistrarMetadata (175) */
  interface MoonbaseRuntimeAssetConfigAssetRegistrarMetadata extends Struct {
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly decimals: u8;
    readonly isFrozen: bool;
  }

  /** @name PalletXcmTransactorEvent (176) */
  interface PalletXcmTransactorEvent extends Enum {
    readonly isTransactedDerivative: boolean;
    readonly asTransactedDerivative: {
      readonly accountId: AccountId20;
      readonly dest: StagingXcmV5Location;
      readonly call: Bytes;
      readonly index: u16;
    } & Struct;
    readonly isTransactedSovereign: boolean;
    readonly asTransactedSovereign: {
      readonly feePayer: Option<AccountId20>;
      readonly dest: StagingXcmV5Location;
      readonly call: Bytes;
    } & Struct;
    readonly isTransactedSigned: boolean;
    readonly asTransactedSigned: {
      readonly feePayer: AccountId20;
      readonly dest: StagingXcmV5Location;
      readonly call: Bytes;
    } & Struct;
    readonly isRegisteredDerivative: boolean;
    readonly asRegisteredDerivative: {
      readonly accountId: AccountId20;
      readonly index: u16;
    } & Struct;
    readonly isDeRegisteredDerivative: boolean;
    readonly asDeRegisteredDerivative: {
      readonly index: u16;
    } & Struct;
    readonly isTransactFailed: boolean;
    readonly asTransactFailed: {
      readonly error: XcmV5TraitsError;
    } & Struct;
    readonly isTransactInfoChanged: boolean;
    readonly asTransactInfoChanged: {
      readonly location: StagingXcmV5Location;
      readonly remoteInfo: PalletXcmTransactorRemoteTransactInfoWithMaxWeight;
    } & Struct;
    readonly isTransactInfoRemoved: boolean;
    readonly asTransactInfoRemoved: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly isDestFeePerSecondChanged: boolean;
    readonly asDestFeePerSecondChanged: {
      readonly location: StagingXcmV5Location;
      readonly feePerSecond: u128;
    } & Struct;
    readonly isDestFeePerSecondRemoved: boolean;
    readonly asDestFeePerSecondRemoved: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly isHrmpManagementSent: boolean;
    readonly asHrmpManagementSent: {
      readonly action: PalletXcmTransactorHrmpOperation;
    } & Struct;
    readonly type:
      | "TransactedDerivative"
      | "TransactedSovereign"
      | "TransactedSigned"
      | "RegisteredDerivative"
      | "DeRegisteredDerivative"
      | "TransactFailed"
      | "TransactInfoChanged"
      | "TransactInfoRemoved"
      | "DestFeePerSecondChanged"
      | "DestFeePerSecondRemoved"
      | "HrmpManagementSent";
  }

  /** @name PalletXcmTransactorRemoteTransactInfoWithMaxWeight (177) */
  interface PalletXcmTransactorRemoteTransactInfoWithMaxWeight extends Struct {
    readonly transactExtraWeight: SpWeightsWeightV2Weight;
    readonly maxWeight: SpWeightsWeightV2Weight;
    readonly transactExtraWeightSigned: Option<SpWeightsWeightV2Weight>;
  }

  /** @name PalletXcmTransactorHrmpOperation (178) */
  interface PalletXcmTransactorHrmpOperation extends Enum {
    readonly isInitOpen: boolean;
    readonly asInitOpen: PalletXcmTransactorHrmpInitParams;
    readonly isAccept: boolean;
    readonly asAccept: {
      readonly paraId: u32;
    } & Struct;
    readonly isClose: boolean;
    readonly asClose: PolkadotParachainPrimitivesPrimitivesHrmpChannelId;
    readonly isCancel: boolean;
    readonly asCancel: {
      readonly channelId: PolkadotParachainPrimitivesPrimitivesHrmpChannelId;
      readonly openRequests: u32;
    } & Struct;
    readonly type: "InitOpen" | "Accept" | "Close" | "Cancel";
  }

  /** @name PalletXcmTransactorHrmpInitParams (179) */
  interface PalletXcmTransactorHrmpInitParams extends Struct {
    readonly paraId: u32;
    readonly proposedMaxCapacity: u32;
    readonly proposedMaxMessageSize: u32;
  }

  /** @name PolkadotParachainPrimitivesPrimitivesHrmpChannelId (181) */
  interface PolkadotParachainPrimitivesPrimitivesHrmpChannelId extends Struct {
    readonly sender: u32;
    readonly recipient: u32;
  }

  /** @name PalletMoonbeamOrbitersEvent (182) */
  interface PalletMoonbeamOrbitersEvent extends Enum {
    readonly isOrbiterJoinCollatorPool: boolean;
    readonly asOrbiterJoinCollatorPool: {
      readonly collator: AccountId20;
      readonly orbiter: AccountId20;
    } & Struct;
    readonly isOrbiterLeaveCollatorPool: boolean;
    readonly asOrbiterLeaveCollatorPool: {
      readonly collator: AccountId20;
      readonly orbiter: AccountId20;
    } & Struct;
    readonly isOrbiterRewarded: boolean;
    readonly asOrbiterRewarded: {
      readonly account: AccountId20;
      readonly rewards: u128;
    } & Struct;
    readonly isOrbiterRotation: boolean;
    readonly asOrbiterRotation: {
      readonly collator: AccountId20;
      readonly oldOrbiter: Option<AccountId20>;
      readonly newOrbiter: Option<AccountId20>;
    } & Struct;
    readonly isOrbiterRegistered: boolean;
    readonly asOrbiterRegistered: {
      readonly account: AccountId20;
      readonly deposit: u128;
    } & Struct;
    readonly isOrbiterUnregistered: boolean;
    readonly asOrbiterUnregistered: {
      readonly account: AccountId20;
    } & Struct;
    readonly type:
      | "OrbiterJoinCollatorPool"
      | "OrbiterLeaveCollatorPool"
      | "OrbiterRewarded"
      | "OrbiterRotation"
      | "OrbiterRegistered"
      | "OrbiterUnregistered";
  }

  /** @name PalletEthereumXcmEvent (183) */
  interface PalletEthereumXcmEvent extends Enum {
    readonly isExecutedFromXcm: boolean;
    readonly asExecutedFromXcm: {
      readonly xcmMsgHash: H256;
      readonly ethTxHash: H256;
    } & Struct;
    readonly type: "ExecutedFromXcm";
  }

  /** @name PalletRandomnessEvent (184) */
  interface PalletRandomnessEvent extends Enum {
    readonly isRandomnessRequestedBabeEpoch: boolean;
    readonly asRandomnessRequestedBabeEpoch: {
      readonly id: u64;
      readonly refundAddress: H160;
      readonly contractAddress: H160;
      readonly fee: u128;
      readonly gasLimit: u64;
      readonly numWords: u8;
      readonly salt: H256;
      readonly earliestEpoch: u64;
    } & Struct;
    readonly isRandomnessRequestedLocal: boolean;
    readonly asRandomnessRequestedLocal: {
      readonly id: u64;
      readonly refundAddress: H160;
      readonly contractAddress: H160;
      readonly fee: u128;
      readonly gasLimit: u64;
      readonly numWords: u8;
      readonly salt: H256;
      readonly earliestBlock: u32;
    } & Struct;
    readonly isRequestFulfilled: boolean;
    readonly asRequestFulfilled: {
      readonly id: u64;
    } & Struct;
    readonly isRequestFeeIncreased: boolean;
    readonly asRequestFeeIncreased: {
      readonly id: u64;
      readonly newFee: u128;
    } & Struct;
    readonly isRequestExpirationExecuted: boolean;
    readonly asRequestExpirationExecuted: {
      readonly id: u64;
    } & Struct;
    readonly type:
      | "RandomnessRequestedBabeEpoch"
      | "RandomnessRequestedLocal"
      | "RequestFulfilled"
      | "RequestFeeIncreased"
      | "RequestExpirationExecuted";
  }

  /** @name PalletCollectiveEvent (185) */
  interface PalletCollectiveEvent extends Enum {
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
    readonly isKilled: boolean;
    readonly asKilled: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isProposalCostBurned: boolean;
    readonly asProposalCostBurned: {
      readonly proposalHash: H256;
      readonly who: AccountId20;
    } & Struct;
    readonly isProposalCostReleased: boolean;
    readonly asProposalCostReleased: {
      readonly proposalHash: H256;
      readonly who: AccountId20;
    } & Struct;
    readonly type:
      | "Proposed"
      | "Voted"
      | "Approved"
      | "Disapproved"
      | "Executed"
      | "MemberExecuted"
      | "Closed"
      | "Killed"
      | "ProposalCostBurned"
      | "ProposalCostReleased";
  }

  /** @name PalletConvictionVotingEvent (186) */
  interface PalletConvictionVotingEvent extends Enum {
    readonly isDelegated: boolean;
    readonly asDelegated: ITuple<[AccountId20, AccountId20]>;
    readonly isUndelegated: boolean;
    readonly asUndelegated: AccountId20;
    readonly isVoted: boolean;
    readonly asVoted: {
      readonly who: AccountId20;
      readonly vote: PalletConvictionVotingVoteAccountVote;
    } & Struct;
    readonly isVoteRemoved: boolean;
    readonly asVoteRemoved: {
      readonly who: AccountId20;
      readonly vote: PalletConvictionVotingVoteAccountVote;
    } & Struct;
    readonly type: "Delegated" | "Undelegated" | "Voted" | "VoteRemoved";
  }

  /** @name PalletConvictionVotingVoteAccountVote (187) */
  interface PalletConvictionVotingVoteAccountVote extends Enum {
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
    readonly isSplitAbstain: boolean;
    readonly asSplitAbstain: {
      readonly aye: u128;
      readonly nay: u128;
      readonly abstain: u128;
    } & Struct;
    readonly type: "Standard" | "Split" | "SplitAbstain";
  }

  /** @name PalletReferendaEvent (189) */
  interface PalletReferendaEvent extends Enum {
    readonly isSubmitted: boolean;
    readonly asSubmitted: {
      readonly index: u32;
      readonly track: u16;
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isDecisionDepositPlaced: boolean;
    readonly asDecisionDepositPlaced: {
      readonly index: u32;
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isDecisionDepositRefunded: boolean;
    readonly asDecisionDepositRefunded: {
      readonly index: u32;
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isDepositSlashed: boolean;
    readonly asDepositSlashed: {
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isDecisionStarted: boolean;
    readonly asDecisionStarted: {
      readonly index: u32;
      readonly track: u16;
      readonly proposal: FrameSupportPreimagesBounded;
      readonly tally: PalletConvictionVotingTally;
    } & Struct;
    readonly isConfirmStarted: boolean;
    readonly asConfirmStarted: {
      readonly index: u32;
    } & Struct;
    readonly isConfirmAborted: boolean;
    readonly asConfirmAborted: {
      readonly index: u32;
    } & Struct;
    readonly isConfirmed: boolean;
    readonly asConfirmed: {
      readonly index: u32;
      readonly tally: PalletConvictionVotingTally;
    } & Struct;
    readonly isApproved: boolean;
    readonly asApproved: {
      readonly index: u32;
    } & Struct;
    readonly isRejected: boolean;
    readonly asRejected: {
      readonly index: u32;
      readonly tally: PalletConvictionVotingTally;
    } & Struct;
    readonly isTimedOut: boolean;
    readonly asTimedOut: {
      readonly index: u32;
      readonly tally: PalletConvictionVotingTally;
    } & Struct;
    readonly isCancelled: boolean;
    readonly asCancelled: {
      readonly index: u32;
      readonly tally: PalletConvictionVotingTally;
    } & Struct;
    readonly isKilled: boolean;
    readonly asKilled: {
      readonly index: u32;
      readonly tally: PalletConvictionVotingTally;
    } & Struct;
    readonly isSubmissionDepositRefunded: boolean;
    readonly asSubmissionDepositRefunded: {
      readonly index: u32;
      readonly who: AccountId20;
      readonly amount: u128;
    } & Struct;
    readonly isMetadataSet: boolean;
    readonly asMetadataSet: {
      readonly index: u32;
      readonly hash_: H256;
    } & Struct;
    readonly isMetadataCleared: boolean;
    readonly asMetadataCleared: {
      readonly index: u32;
      readonly hash_: H256;
    } & Struct;
    readonly type:
      | "Submitted"
      | "DecisionDepositPlaced"
      | "DecisionDepositRefunded"
      | "DepositSlashed"
      | "DecisionStarted"
      | "ConfirmStarted"
      | "ConfirmAborted"
      | "Confirmed"
      | "Approved"
      | "Rejected"
      | "TimedOut"
      | "Cancelled"
      | "Killed"
      | "SubmissionDepositRefunded"
      | "MetadataSet"
      | "MetadataCleared";
  }

  /** @name FrameSupportPreimagesBounded (190) */
  interface FrameSupportPreimagesBounded extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: {
      readonly hash_: H256;
    } & Struct;
    readonly isInline: boolean;
    readonly asInline: Bytes;
    readonly isLookup: boolean;
    readonly asLookup: {
      readonly hash_: H256;
      readonly len: u32;
    } & Struct;
    readonly type: "Legacy" | "Inline" | "Lookup";
  }

  /** @name FrameSystemCall (192) */
  interface FrameSystemCall extends Enum {
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
    readonly isAuthorizeUpgrade: boolean;
    readonly asAuthorizeUpgrade: {
      readonly codeHash: H256;
    } & Struct;
    readonly isAuthorizeUpgradeWithoutChecks: boolean;
    readonly asAuthorizeUpgradeWithoutChecks: {
      readonly codeHash: H256;
    } & Struct;
    readonly isApplyAuthorizedUpgrade: boolean;
    readonly asApplyAuthorizedUpgrade: {
      readonly code: Bytes;
    } & Struct;
    readonly type:
      | "Remark"
      | "SetHeapPages"
      | "SetCode"
      | "SetCodeWithoutChecks"
      | "SetStorage"
      | "KillStorage"
      | "KillPrefix"
      | "RemarkWithEvent"
      | "AuthorizeUpgrade"
      | "AuthorizeUpgradeWithoutChecks"
      | "ApplyAuthorizedUpgrade";
  }

  /** @name PalletUtilityCall (196) */
  interface PalletUtilityCall extends Enum {
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
    readonly isForceBatch: boolean;
    readonly asForceBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isWithWeight: boolean;
    readonly asWithWeight: {
      readonly call: Call;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type:
      | "Batch"
      | "AsDerivative"
      | "BatchAll"
      | "DispatchAs"
      | "ForceBatch"
      | "WithWeight";
  }

  /** @name MoonbaseRuntimeOriginCaller (198) */
  interface MoonbaseRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean;
    readonly asSystem: FrameSupportDispatchRawOrigin;
    readonly isEthereum: boolean;
    readonly asEthereum: PalletEthereumRawOrigin;
    readonly isCumulusXcm: boolean;
    readonly asCumulusXcm: CumulusPalletXcmOrigin;
    readonly isPolkadotXcm: boolean;
    readonly asPolkadotXcm: PalletXcmOrigin;
    readonly isEthereumXcm: boolean;
    readonly asEthereumXcm: PalletEthereumXcmRawOrigin;
    readonly isTreasuryCouncilCollective: boolean;
    readonly asTreasuryCouncilCollective: PalletCollectiveRawOrigin;
    readonly isOrigins: boolean;
    readonly asOrigins: MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin;
    readonly isOpenTechCommitteeCollective: boolean;
    readonly asOpenTechCommitteeCollective: PalletCollectiveRawOrigin;
    readonly type:
      | "System"
      | "Ethereum"
      | "CumulusXcm"
      | "PolkadotXcm"
      | "EthereumXcm"
      | "TreasuryCouncilCollective"
      | "Origins"
      | "OpenTechCommitteeCollective";
  }

  /** @name FrameSupportDispatchRawOrigin (199) */
  interface FrameSupportDispatchRawOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId20;
    readonly isNone: boolean;
    readonly type: "Root" | "Signed" | "None";
  }

  /** @name PalletEthereumRawOrigin (200) */
  interface PalletEthereumRawOrigin extends Enum {
    readonly isEthereumTransaction: boolean;
    readonly asEthereumTransaction: H160;
    readonly type: "EthereumTransaction";
  }

  /** @name CumulusPalletXcmOrigin (201) */
  interface CumulusPalletXcmOrigin extends Enum {
    readonly isRelay: boolean;
    readonly isSiblingParachain: boolean;
    readonly asSiblingParachain: u32;
    readonly type: "Relay" | "SiblingParachain";
  }

  /** @name PalletXcmOrigin (202) */
  interface PalletXcmOrigin extends Enum {
    readonly isXcm: boolean;
    readonly asXcm: StagingXcmV5Location;
    readonly isResponse: boolean;
    readonly asResponse: StagingXcmV5Location;
    readonly type: "Xcm" | "Response";
  }

  /** @name PalletEthereumXcmRawOrigin (203) */
  interface PalletEthereumXcmRawOrigin extends Enum {
    readonly isXcmEthereumTransaction: boolean;
    readonly asXcmEthereumTransaction: H160;
    readonly type: "XcmEthereumTransaction";
  }

  /** @name PalletCollectiveRawOrigin (204) */
  interface PalletCollectiveRawOrigin extends Enum {
    readonly isMembers: boolean;
    readonly asMembers: ITuple<[u32, u32]>;
    readonly isMember: boolean;
    readonly asMember: AccountId20;
    readonly isPhantom: boolean;
    readonly type: "Members" | "Member" | "Phantom";
  }

  /** @name MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin (205) */
  interface MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin extends Enum {
    readonly isWhitelistedCaller: boolean;
    readonly isGeneralAdmin: boolean;
    readonly isReferendumCanceller: boolean;
    readonly isReferendumKiller: boolean;
    readonly isFastGeneralAdmin: boolean;
    readonly type:
      | "WhitelistedCaller"
      | "GeneralAdmin"
      | "ReferendumCanceller"
      | "ReferendumKiller"
      | "FastGeneralAdmin";
  }

  /** @name PalletTimestampCall (207) */
  interface PalletTimestampCall extends Enum {
    readonly isSet: boolean;
    readonly asSet: {
      readonly now: Compact<u64>;
    } & Struct;
    readonly type: "Set";
  }

  /** @name PalletBalancesCall (208) */
  interface PalletBalancesCall extends Enum {
    readonly isTransferAllowDeath: boolean;
    readonly asTransferAllowDeath: {
      readonly dest: AccountId20;
      readonly value: Compact<u128>;
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
    readonly isUpgradeAccounts: boolean;
    readonly asUpgradeAccounts: {
      readonly who: Vec<AccountId20>;
    } & Struct;
    readonly isForceSetBalance: boolean;
    readonly asForceSetBalance: {
      readonly who: AccountId20;
      readonly newFree: Compact<u128>;
    } & Struct;
    readonly isForceAdjustTotalIssuance: boolean;
    readonly asForceAdjustTotalIssuance: {
      readonly direction: PalletBalancesAdjustmentDirection;
      readonly delta: Compact<u128>;
    } & Struct;
    readonly isBurn: boolean;
    readonly asBurn: {
      readonly value: Compact<u128>;
      readonly keepAlive: bool;
    } & Struct;
    readonly type:
      | "TransferAllowDeath"
      | "ForceTransfer"
      | "TransferKeepAlive"
      | "TransferAll"
      | "ForceUnreserve"
      | "UpgradeAccounts"
      | "ForceSetBalance"
      | "ForceAdjustTotalIssuance"
      | "Burn";
  }

  /** @name PalletBalancesAdjustmentDirection (210) */
  interface PalletBalancesAdjustmentDirection extends Enum {
    readonly isIncrease: boolean;
    readonly isDecrease: boolean;
    readonly type: "Increase" | "Decrease";
  }

  /** @name PalletSudoCall (211) */
  interface PalletSudoCall extends Enum {
    readonly isSudo: boolean;
    readonly asSudo: {
      readonly call: Call;
    } & Struct;
    readonly isSudoUncheckedWeight: boolean;
    readonly asSudoUncheckedWeight: {
      readonly call: Call;
      readonly weight: SpWeightsWeightV2Weight;
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
    readonly isRemoveKey: boolean;
    readonly type: "Sudo" | "SudoUncheckedWeight" | "SetKey" | "SudoAs" | "RemoveKey";
  }

  /** @name CumulusPalletParachainSystemCall (212) */
  interface CumulusPalletParachainSystemCall extends Enum {
    readonly isSetValidationData: boolean;
    readonly asSetValidationData: {
      readonly data: CumulusPrimitivesParachainInherentParachainInherentData;
    } & Struct;
    readonly isSudoSendUpwardMessage: boolean;
    readonly asSudoSendUpwardMessage: {
      readonly message: Bytes;
    } & Struct;
    readonly type: "SetValidationData" | "SudoSendUpwardMessage";
  }

  /** @name CumulusPrimitivesParachainInherentParachainInherentData (213) */
  interface CumulusPrimitivesParachainInherentParachainInherentData extends Struct {
    readonly validationData: PolkadotPrimitivesV8PersistedValidationData;
    readonly relayChainState: SpTrieStorageProof;
    readonly downwardMessages: Vec<PolkadotCorePrimitivesInboundDownwardMessage>;
    readonly horizontalMessages: BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>;
  }

  /** @name PolkadotPrimitivesV8PersistedValidationData (214) */
  interface PolkadotPrimitivesV8PersistedValidationData extends Struct {
    readonly parentHead: Bytes;
    readonly relayParentNumber: u32;
    readonly relayParentStorageRoot: H256;
    readonly maxPovSize: u32;
  }

  /** @name SpTrieStorageProof (216) */
  interface SpTrieStorageProof extends Struct {
    readonly trieNodes: BTreeSet<Bytes>;
  }

  /** @name PolkadotCorePrimitivesInboundDownwardMessage (219) */
  interface PolkadotCorePrimitivesInboundDownwardMessage extends Struct {
    readonly sentAt: u32;
    readonly msg: Bytes;
  }

  /** @name PolkadotCorePrimitivesInboundHrmpMessage (222) */
  interface PolkadotCorePrimitivesInboundHrmpMessage extends Struct {
    readonly sentAt: u32;
    readonly data: Bytes;
  }

  /** @name PalletEvmCall (225) */
  interface PalletEvmCall extends Enum {
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
    readonly type: "Withdraw" | "Call" | "Create" | "Create2";
  }

  /** @name PalletEthereumCall (231) */
  interface PalletEthereumCall extends Enum {
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly transaction: EthereumTransactionTransactionV2;
    } & Struct;
    readonly type: "Transact";
  }

  /** @name EthereumTransactionTransactionV2 (232) */
  interface EthereumTransactionTransactionV2 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumTransactionLegacyLegacyTransaction;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumTransactionEip2930Eip2930Transaction;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumTransactionEip1559Eip1559Transaction;
    readonly type: "Legacy" | "Eip2930" | "Eip1559";
  }

  /** @name EthereumTransactionLegacyLegacyTransaction (233) */
  interface EthereumTransactionLegacyLegacyTransaction extends Struct {
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionLegacyTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly signature: EthereumTransactionLegacyTransactionSignature;
  }

  /** @name EthereumTransactionLegacyTransactionAction (234) */
  interface EthereumTransactionLegacyTransactionAction extends Enum {
    readonly isCall: boolean;
    readonly asCall: H160;
    readonly isCreate: boolean;
    readonly type: "Call" | "Create";
  }

  /** @name EthereumTransactionLegacyTransactionSignature (235) */
  interface EthereumTransactionLegacyTransactionSignature extends Struct {
    readonly v: u64;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name EthereumTransactionEip2930Eip2930Transaction (237) */
  interface EthereumTransactionEip2930Eip2930Transaction extends Struct {
    readonly chainId: u64;
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionLegacyTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Vec<EthereumTransactionEip2930AccessListItem>;
    readonly oddYParity: bool;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name EthereumTransactionEip2930AccessListItem (239) */
  interface EthereumTransactionEip2930AccessListItem extends Struct {
    readonly address: H160;
    readonly storageKeys: Vec<H256>;
  }

  /** @name EthereumTransactionEip1559Eip1559Transaction (240) */
  interface EthereumTransactionEip1559Eip1559Transaction extends Struct {
    readonly chainId: u64;
    readonly nonce: U256;
    readonly maxPriorityFeePerGas: U256;
    readonly maxFeePerGas: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionLegacyTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Vec<EthereumTransactionEip2930AccessListItem>;
    readonly oddYParity: bool;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name PalletParachainStakingCall (241) */
  interface PalletParachainStakingCall extends Enum {
    readonly isSetStakingExpectations: boolean;
    readonly asSetStakingExpectations: {
      readonly expectations: {
        readonly min: u128;
        readonly ideal: u128;
        readonly max: u128;
      } & Struct;
    } & Struct;
    readonly isSetInflation: boolean;
    readonly asSetInflation: {
      readonly schedule: {
        readonly min: Perbill;
        readonly ideal: Perbill;
        readonly max: Perbill;
      } & Struct;
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
    readonly isDelegateWithAutoCompound: boolean;
    readonly asDelegateWithAutoCompound: {
      readonly candidate: AccountId20;
      readonly amount: u128;
      readonly autoCompound: Percent;
      readonly candidateDelegationCount: u32;
      readonly candidateAutoCompoundingDelegationCount: u32;
      readonly delegationCount: u32;
    } & Struct;
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
    readonly isSetAutoCompound: boolean;
    readonly asSetAutoCompound: {
      readonly candidate: AccountId20;
      readonly value: Percent;
      readonly candidateAutoCompoundingDelegationCountHint: u32;
      readonly delegationCountHint: u32;
    } & Struct;
    readonly isNotifyInactiveCollator: boolean;
    readonly asNotifyInactiveCollator: {
      readonly collator: AccountId20;
    } & Struct;
    readonly isEnableMarkingOffline: boolean;
    readonly asEnableMarkingOffline: {
      readonly value: bool;
    } & Struct;
    readonly isForceJoinCandidates: boolean;
    readonly asForceJoinCandidates: {
      readonly account: AccountId20;
      readonly bond: u128;
      readonly candidateCount: u32;
    } & Struct;
    readonly isSetInflationDistributionConfig: boolean;
    readonly asSetInflationDistributionConfig: {
      readonly new_: PalletParachainStakingInflationDistributionConfig;
    } & Struct;
    readonly type:
      | "SetStakingExpectations"
      | "SetInflation"
      | "SetTotalSelected"
      | "SetCollatorCommission"
      | "SetBlocksPerRound"
      | "JoinCandidates"
      | "ScheduleLeaveCandidates"
      | "ExecuteLeaveCandidates"
      | "CancelLeaveCandidates"
      | "GoOffline"
      | "GoOnline"
      | "CandidateBondMore"
      | "ScheduleCandidateBondLess"
      | "ExecuteCandidateBondLess"
      | "CancelCandidateBondLess"
      | "DelegateWithAutoCompound"
      | "ScheduleRevokeDelegation"
      | "DelegatorBondMore"
      | "ScheduleDelegatorBondLess"
      | "ExecuteDelegationRequest"
      | "CancelDelegationRequest"
      | "SetAutoCompound"
      | "NotifyInactiveCollator"
      | "EnableMarkingOffline"
      | "ForceJoinCandidates"
      | "SetInflationDistributionConfig";
  }

  /** @name PalletSchedulerCall (244) */
  interface PalletSchedulerCall extends Enum {
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
      readonly id: U8aFixed;
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancelNamed: boolean;
    readonly asCancelNamed: {
      readonly id: U8aFixed;
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
      readonly id: U8aFixed;
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isSetRetry: boolean;
    readonly asSetRetry: {
      readonly task: ITuple<[u32, u32]>;
      readonly retries: u8;
      readonly period: u32;
    } & Struct;
    readonly isSetRetryNamed: boolean;
    readonly asSetRetryNamed: {
      readonly id: U8aFixed;
      readonly retries: u8;
      readonly period: u32;
    } & Struct;
    readonly isCancelRetry: boolean;
    readonly asCancelRetry: {
      readonly task: ITuple<[u32, u32]>;
    } & Struct;
    readonly isCancelRetryNamed: boolean;
    readonly asCancelRetryNamed: {
      readonly id: U8aFixed;
    } & Struct;
    readonly type:
      | "Schedule"
      | "Cancel"
      | "ScheduleNamed"
      | "CancelNamed"
      | "ScheduleAfter"
      | "ScheduleNamedAfter"
      | "SetRetry"
      | "SetRetryNamed"
      | "CancelRetry"
      | "CancelRetryNamed";
  }

  /** @name PalletTreasuryCall (246) */
  interface PalletTreasuryCall extends Enum {
    readonly isSpendLocal: boolean;
    readonly asSpendLocal: {
      readonly amount: Compact<u128>;
      readonly beneficiary: AccountId20;
    } & Struct;
    readonly isRemoveApproval: boolean;
    readonly asRemoveApproval: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly isSpend: boolean;
    readonly asSpend: {
      readonly assetKind: FrameSupportTokensFungibleUnionOfNativeOrWithId;
      readonly amount: Compact<u128>;
      readonly beneficiary: AccountId20;
      readonly validFrom: Option<u32>;
    } & Struct;
    readonly isPayout: boolean;
    readonly asPayout: {
      readonly index: u32;
    } & Struct;
    readonly isCheckStatus: boolean;
    readonly asCheckStatus: {
      readonly index: u32;
    } & Struct;
    readonly isVoidSpend: boolean;
    readonly asVoidSpend: {
      readonly index: u32;
    } & Struct;
    readonly type:
      | "SpendLocal"
      | "RemoveApproval"
      | "Spend"
      | "Payout"
      | "CheckStatus"
      | "VoidSpend";
  }

  /** @name PalletAuthorInherentCall (248) */
  interface PalletAuthorInherentCall extends Enum {
    readonly isKickOffAuthorshipValidation: boolean;
    readonly type: "KickOffAuthorshipValidation";
  }

  /** @name PalletAuthorSlotFilterCall (249) */
  interface PalletAuthorSlotFilterCall extends Enum {
    readonly isSetEligible: boolean;
    readonly asSetEligible: {
      readonly new_: u32;
    } & Struct;
    readonly type: "SetEligible";
  }

  /** @name PalletCrowdloanRewardsCall (250) */
  interface PalletCrowdloanRewardsCall extends Enum {
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
    readonly type:
      | "AssociateNativeIdentity"
      | "ChangeAssociationWithRelayKeys"
      | "Claim"
      | "UpdateRewardAddress"
      | "CompleteInitialization"
      | "InitializeRewardVec";
  }

  /** @name SpRuntimeMultiSignature (251) */
  interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean;
    readonly asEd25519: U8aFixed;
    readonly isSr25519: boolean;
    readonly asSr25519: U8aFixed;
    readonly isEcdsa: boolean;
    readonly asEcdsa: U8aFixed;
    readonly type: "Ed25519" | "Sr25519" | "Ecdsa";
  }

  /** @name PalletAuthorMappingCall (258) */
  interface PalletAuthorMappingCall extends Enum {
    readonly isAddAssociation: boolean;
    readonly asAddAssociation: {
      readonly nimbusId: NimbusPrimitivesNimbusCryptoPublic;
    } & Struct;
    readonly isUpdateAssociation: boolean;
    readonly asUpdateAssociation: {
      readonly oldNimbusId: NimbusPrimitivesNimbusCryptoPublic;
      readonly newNimbusId: NimbusPrimitivesNimbusCryptoPublic;
    } & Struct;
    readonly isClearAssociation: boolean;
    readonly asClearAssociation: {
      readonly nimbusId: NimbusPrimitivesNimbusCryptoPublic;
    } & Struct;
    readonly isRemoveKeys: boolean;
    readonly isSetKeys: boolean;
    readonly asSetKeys: {
      readonly keys_: Bytes;
    } & Struct;
    readonly type:
      | "AddAssociation"
      | "UpdateAssociation"
      | "ClearAssociation"
      | "RemoveKeys"
      | "SetKeys";
  }

  /** @name PalletProxyCall (259) */
  interface PalletProxyCall extends Enum {
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
    readonly isCreatePure: boolean;
    readonly asCreatePure: {
      readonly proxyType: MoonbaseRuntimeProxyType;
      readonly delay: u32;
      readonly index: u16;
    } & Struct;
    readonly isKillPure: boolean;
    readonly asKillPure: {
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
    readonly type:
      | "Proxy"
      | "AddProxy"
      | "RemoveProxy"
      | "RemoveProxies"
      | "CreatePure"
      | "KillPure"
      | "Announce"
      | "RemoveAnnouncement"
      | "RejectAnnouncement"
      | "ProxyAnnounced";
  }

  /** @name PalletMaintenanceModeCall (261) */
  interface PalletMaintenanceModeCall extends Enum {
    readonly isEnterMaintenanceMode: boolean;
    readonly isResumeNormalOperation: boolean;
    readonly type: "EnterMaintenanceMode" | "ResumeNormalOperation";
  }

  /** @name PalletIdentityCall (262) */
  interface PalletIdentityCall extends Enum {
    readonly isAddRegistrar: boolean;
    readonly asAddRegistrar: {
      readonly account: AccountId20;
    } & Struct;
    readonly isSetIdentity: boolean;
    readonly asSetIdentity: {
      readonly info: PalletIdentityLegacyIdentityInfo;
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
      readonly fields: u64;
    } & Struct;
    readonly isProvideJudgement: boolean;
    readonly asProvideJudgement: {
      readonly regIndex: Compact<u32>;
      readonly target: AccountId20;
      readonly judgement: PalletIdentityJudgement;
      readonly identity: H256;
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
    readonly isAddUsernameAuthority: boolean;
    readonly asAddUsernameAuthority: {
      readonly authority: AccountId20;
      readonly suffix: Bytes;
      readonly allocation: u32;
    } & Struct;
    readonly isRemoveUsernameAuthority: boolean;
    readonly asRemoveUsernameAuthority: {
      readonly suffix: Bytes;
      readonly authority: AccountId20;
    } & Struct;
    readonly isSetUsernameFor: boolean;
    readonly asSetUsernameFor: {
      readonly who: AccountId20;
      readonly username: Bytes;
      readonly signature: Option<AccountEthereumSignature>;
      readonly useAllocation: bool;
    } & Struct;
    readonly isAcceptUsername: boolean;
    readonly asAcceptUsername: {
      readonly username: Bytes;
    } & Struct;
    readonly isRemoveExpiredApproval: boolean;
    readonly asRemoveExpiredApproval: {
      readonly username: Bytes;
    } & Struct;
    readonly isSetPrimaryUsername: boolean;
    readonly asSetPrimaryUsername: {
      readonly username: Bytes;
    } & Struct;
    readonly isUnbindUsername: boolean;
    readonly asUnbindUsername: {
      readonly username: Bytes;
    } & Struct;
    readonly isRemoveUsername: boolean;
    readonly asRemoveUsername: {
      readonly username: Bytes;
    } & Struct;
    readonly isKillUsername: boolean;
    readonly asKillUsername: {
      readonly username: Bytes;
    } & Struct;
    readonly type:
      | "AddRegistrar"
      | "SetIdentity"
      | "SetSubs"
      | "ClearIdentity"
      | "RequestJudgement"
      | "CancelRequest"
      | "SetFee"
      | "SetAccountId"
      | "SetFields"
      | "ProvideJudgement"
      | "KillIdentity"
      | "AddSub"
      | "RenameSub"
      | "RemoveSub"
      | "QuitSub"
      | "AddUsernameAuthority"
      | "RemoveUsernameAuthority"
      | "SetUsernameFor"
      | "AcceptUsername"
      | "RemoveExpiredApproval"
      | "SetPrimaryUsername"
      | "UnbindUsername"
      | "RemoveUsername"
      | "KillUsername";
  }

  /** @name PalletIdentityLegacyIdentityInfo (263) */
  interface PalletIdentityLegacyIdentityInfo extends Struct {
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

  /** @name PalletIdentityJudgement (299) */
  interface PalletIdentityJudgement extends Enum {
    readonly isUnknown: boolean;
    readonly isFeePaid: boolean;
    readonly asFeePaid: u128;
    readonly isReasonable: boolean;
    readonly isKnownGood: boolean;
    readonly isOutOfDate: boolean;
    readonly isLowQuality: boolean;
    readonly isErroneous: boolean;
    readonly type:
      | "Unknown"
      | "FeePaid"
      | "Reasonable"
      | "KnownGood"
      | "OutOfDate"
      | "LowQuality"
      | "Erroneous";
  }

  /** @name AccountEthereumSignature (301) */
  interface AccountEthereumSignature extends U8aFixed {}

  /** @name CumulusPalletXcmpQueueCall (302) */
  interface CumulusPalletXcmpQueueCall extends Enum {
    readonly isSuspendXcmExecution: boolean;
    readonly isResumeXcmExecution: boolean;
    readonly isUpdateSuspendThreshold: boolean;
    readonly asUpdateSuspendThreshold: {
      readonly new_: u32;
    } & Struct;
    readonly isUpdateDropThreshold: boolean;
    readonly asUpdateDropThreshold: {
      readonly new_: u32;
    } & Struct;
    readonly isUpdateResumeThreshold: boolean;
    readonly asUpdateResumeThreshold: {
      readonly new_: u32;
    } & Struct;
    readonly type:
      | "SuspendXcmExecution"
      | "ResumeXcmExecution"
      | "UpdateSuspendThreshold"
      | "UpdateDropThreshold"
      | "UpdateResumeThreshold";
  }

  /** @name PalletXcmCall (303) */
  interface PalletXcmCall extends Enum {
    readonly isSend: boolean;
    readonly asSend: {
      readonly dest: XcmVersionedLocation;
      readonly message: XcmVersionedXcm;
    } & Struct;
    readonly isTeleportAssets: boolean;
    readonly asTeleportAssets: {
      readonly dest: XcmVersionedLocation;
      readonly beneficiary: XcmVersionedLocation;
      readonly assets: XcmVersionedAssets;
      readonly feeAssetItem: u32;
    } & Struct;
    readonly isReserveTransferAssets: boolean;
    readonly asReserveTransferAssets: {
      readonly dest: XcmVersionedLocation;
      readonly beneficiary: XcmVersionedLocation;
      readonly assets: XcmVersionedAssets;
      readonly feeAssetItem: u32;
    } & Struct;
    readonly isExecute: boolean;
    readonly asExecute: {
      readonly message: XcmVersionedXcm;
      readonly maxWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isForceXcmVersion: boolean;
    readonly asForceXcmVersion: {
      readonly location: StagingXcmV5Location;
      readonly version: u32;
    } & Struct;
    readonly isForceDefaultXcmVersion: boolean;
    readonly asForceDefaultXcmVersion: {
      readonly maybeXcmVersion: Option<u32>;
    } & Struct;
    readonly isForceSubscribeVersionNotify: boolean;
    readonly asForceSubscribeVersionNotify: {
      readonly location: XcmVersionedLocation;
    } & Struct;
    readonly isForceUnsubscribeVersionNotify: boolean;
    readonly asForceUnsubscribeVersionNotify: {
      readonly location: XcmVersionedLocation;
    } & Struct;
    readonly isLimitedReserveTransferAssets: boolean;
    readonly asLimitedReserveTransferAssets: {
      readonly dest: XcmVersionedLocation;
      readonly beneficiary: XcmVersionedLocation;
      readonly assets: XcmVersionedAssets;
      readonly feeAssetItem: u32;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isLimitedTeleportAssets: boolean;
    readonly asLimitedTeleportAssets: {
      readonly dest: XcmVersionedLocation;
      readonly beneficiary: XcmVersionedLocation;
      readonly assets: XcmVersionedAssets;
      readonly feeAssetItem: u32;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isForceSuspension: boolean;
    readonly asForceSuspension: {
      readonly suspended: bool;
    } & Struct;
    readonly isTransferAssets: boolean;
    readonly asTransferAssets: {
      readonly dest: XcmVersionedLocation;
      readonly beneficiary: XcmVersionedLocation;
      readonly assets: XcmVersionedAssets;
      readonly feeAssetItem: u32;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isClaimAssets: boolean;
    readonly asClaimAssets: {
      readonly assets: XcmVersionedAssets;
      readonly beneficiary: XcmVersionedLocation;
    } & Struct;
    readonly isTransferAssetsUsingTypeAndThen: boolean;
    readonly asTransferAssetsUsingTypeAndThen: {
      readonly dest: XcmVersionedLocation;
      readonly assets: XcmVersionedAssets;
      readonly assetsTransferType: StagingXcmExecutorAssetTransferTransferType;
      readonly remoteFeesId: XcmVersionedAssetId;
      readonly feesTransferType: StagingXcmExecutorAssetTransferTransferType;
      readonly customXcmOnDest: XcmVersionedXcm;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly type:
      | "Send"
      | "TeleportAssets"
      | "ReserveTransferAssets"
      | "Execute"
      | "ForceXcmVersion"
      | "ForceDefaultXcmVersion"
      | "ForceSubscribeVersionNotify"
      | "ForceUnsubscribeVersionNotify"
      | "LimitedReserveTransferAssets"
      | "LimitedTeleportAssets"
      | "ForceSuspension"
      | "TransferAssets"
      | "ClaimAssets"
      | "TransferAssetsUsingTypeAndThen";
  }

  /** @name XcmVersionedXcm (304) */
  interface XcmVersionedXcm extends Enum {
    readonly isV3: boolean;
    readonly asV3: XcmV3Xcm;
    readonly isV4: boolean;
    readonly asV4: StagingXcmV4Xcm;
    readonly isV5: boolean;
    readonly asV5: StagingXcmV5Xcm;
    readonly type: "V3" | "V4" | "V5";
  }

  /** @name XcmV3Xcm (305) */
  interface XcmV3Xcm extends Vec<XcmV3Instruction> {}

  /** @name XcmV3Instruction (307) */
  interface XcmV3Instruction extends Enum {
    readonly isWithdrawAsset: boolean;
    readonly asWithdrawAsset: XcmV3MultiassetMultiAssets;
    readonly isReserveAssetDeposited: boolean;
    readonly asReserveAssetDeposited: XcmV3MultiassetMultiAssets;
    readonly isReceiveTeleportedAsset: boolean;
    readonly asReceiveTeleportedAsset: XcmV3MultiassetMultiAssets;
    readonly isQueryResponse: boolean;
    readonly asQueryResponse: {
      readonly queryId: Compact<u64>;
      readonly response: XcmV3Response;
      readonly maxWeight: SpWeightsWeightV2Weight;
      readonly querier: Option<StagingXcmV3MultiLocation>;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assets: XcmV3MultiassetMultiAssets;
      readonly beneficiary: StagingXcmV3MultiLocation;
    } & Struct;
    readonly isTransferReserveAsset: boolean;
    readonly asTransferReserveAsset: {
      readonly assets: XcmV3MultiassetMultiAssets;
      readonly dest: StagingXcmV3MultiLocation;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly originKind: XcmV3OriginKind;
      readonly requireWeightAtMost: SpWeightsWeightV2Weight;
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
    readonly asDescendOrigin: XcmV3Junctions;
    readonly isReportError: boolean;
    readonly asReportError: XcmV3QueryResponseInfo;
    readonly isDepositAsset: boolean;
    readonly asDepositAsset: {
      readonly assets: XcmV3MultiassetMultiAssetFilter;
      readonly beneficiary: StagingXcmV3MultiLocation;
    } & Struct;
    readonly isDepositReserveAsset: boolean;
    readonly asDepositReserveAsset: {
      readonly assets: XcmV3MultiassetMultiAssetFilter;
      readonly dest: StagingXcmV3MultiLocation;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isExchangeAsset: boolean;
    readonly asExchangeAsset: {
      readonly give: XcmV3MultiassetMultiAssetFilter;
      readonly want: XcmV3MultiassetMultiAssets;
      readonly maximal: bool;
    } & Struct;
    readonly isInitiateReserveWithdraw: boolean;
    readonly asInitiateReserveWithdraw: {
      readonly assets: XcmV3MultiassetMultiAssetFilter;
      readonly reserve: StagingXcmV3MultiLocation;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isInitiateTeleport: boolean;
    readonly asInitiateTeleport: {
      readonly assets: XcmV3MultiassetMultiAssetFilter;
      readonly dest: StagingXcmV3MultiLocation;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isReportHolding: boolean;
    readonly asReportHolding: {
      readonly responseInfo: XcmV3QueryResponseInfo;
      readonly assets: XcmV3MultiassetMultiAssetFilter;
    } & Struct;
    readonly isBuyExecution: boolean;
    readonly asBuyExecution: {
      readonly fees: XcmV3MultiAsset;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isRefundSurplus: boolean;
    readonly isSetErrorHandler: boolean;
    readonly asSetErrorHandler: XcmV3Xcm;
    readonly isSetAppendix: boolean;
    readonly asSetAppendix: XcmV3Xcm;
    readonly isClearError: boolean;
    readonly isClaimAsset: boolean;
    readonly asClaimAsset: {
      readonly assets: XcmV3MultiassetMultiAssets;
      readonly ticket: StagingXcmV3MultiLocation;
    } & Struct;
    readonly isTrap: boolean;
    readonly asTrap: Compact<u64>;
    readonly isSubscribeVersion: boolean;
    readonly asSubscribeVersion: {
      readonly queryId: Compact<u64>;
      readonly maxResponseWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isUnsubscribeVersion: boolean;
    readonly isBurnAsset: boolean;
    readonly asBurnAsset: XcmV3MultiassetMultiAssets;
    readonly isExpectAsset: boolean;
    readonly asExpectAsset: XcmV3MultiassetMultiAssets;
    readonly isExpectOrigin: boolean;
    readonly asExpectOrigin: Option<StagingXcmV3MultiLocation>;
    readonly isExpectError: boolean;
    readonly asExpectError: Option<ITuple<[u32, XcmV3TraitsError]>>;
    readonly isExpectTransactStatus: boolean;
    readonly asExpectTransactStatus: XcmV3MaybeErrorCode;
    readonly isQueryPallet: boolean;
    readonly asQueryPallet: {
      readonly moduleName: Bytes;
      readonly responseInfo: XcmV3QueryResponseInfo;
    } & Struct;
    readonly isExpectPallet: boolean;
    readonly asExpectPallet: {
      readonly index: Compact<u32>;
      readonly name: Bytes;
      readonly moduleName: Bytes;
      readonly crateMajor: Compact<u32>;
      readonly minCrateMinor: Compact<u32>;
    } & Struct;
    readonly isReportTransactStatus: boolean;
    readonly asReportTransactStatus: XcmV3QueryResponseInfo;
    readonly isClearTransactStatus: boolean;
    readonly isUniversalOrigin: boolean;
    readonly asUniversalOrigin: XcmV3Junction;
    readonly isExportMessage: boolean;
    readonly asExportMessage: {
      readonly network: XcmV3JunctionNetworkId;
      readonly destination: XcmV3Junctions;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isLockAsset: boolean;
    readonly asLockAsset: {
      readonly asset: XcmV3MultiAsset;
      readonly unlocker: StagingXcmV3MultiLocation;
    } & Struct;
    readonly isUnlockAsset: boolean;
    readonly asUnlockAsset: {
      readonly asset: XcmV3MultiAsset;
      readonly target: StagingXcmV3MultiLocation;
    } & Struct;
    readonly isNoteUnlockable: boolean;
    readonly asNoteUnlockable: {
      readonly asset: XcmV3MultiAsset;
      readonly owner: StagingXcmV3MultiLocation;
    } & Struct;
    readonly isRequestUnlock: boolean;
    readonly asRequestUnlock: {
      readonly asset: XcmV3MultiAsset;
      readonly locker: StagingXcmV3MultiLocation;
    } & Struct;
    readonly isSetFeesMode: boolean;
    readonly asSetFeesMode: {
      readonly jitWithdraw: bool;
    } & Struct;
    readonly isSetTopic: boolean;
    readonly asSetTopic: U8aFixed;
    readonly isClearTopic: boolean;
    readonly isAliasOrigin: boolean;
    readonly asAliasOrigin: StagingXcmV3MultiLocation;
    readonly isUnpaidExecution: boolean;
    readonly asUnpaidExecution: {
      readonly weightLimit: XcmV3WeightLimit;
      readonly checkOrigin: Option<StagingXcmV3MultiLocation>;
    } & Struct;
    readonly type:
      | "WithdrawAsset"
      | "ReserveAssetDeposited"
      | "ReceiveTeleportedAsset"
      | "QueryResponse"
      | "TransferAsset"
      | "TransferReserveAsset"
      | "Transact"
      | "HrmpNewChannelOpenRequest"
      | "HrmpChannelAccepted"
      | "HrmpChannelClosing"
      | "ClearOrigin"
      | "DescendOrigin"
      | "ReportError"
      | "DepositAsset"
      | "DepositReserveAsset"
      | "ExchangeAsset"
      | "InitiateReserveWithdraw"
      | "InitiateTeleport"
      | "ReportHolding"
      | "BuyExecution"
      | "RefundSurplus"
      | "SetErrorHandler"
      | "SetAppendix"
      | "ClearError"
      | "ClaimAsset"
      | "Trap"
      | "SubscribeVersion"
      | "UnsubscribeVersion"
      | "BurnAsset"
      | "ExpectAsset"
      | "ExpectOrigin"
      | "ExpectError"
      | "ExpectTransactStatus"
      | "QueryPallet"
      | "ExpectPallet"
      | "ReportTransactStatus"
      | "ClearTransactStatus"
      | "UniversalOrigin"
      | "ExportMessage"
      | "LockAsset"
      | "UnlockAsset"
      | "NoteUnlockable"
      | "RequestUnlock"
      | "SetFeesMode"
      | "SetTopic"
      | "ClearTopic"
      | "AliasOrigin"
      | "UnpaidExecution";
  }

  /** @name XcmV3Response (308) */
  interface XcmV3Response extends Enum {
    readonly isNull: boolean;
    readonly isAssets: boolean;
    readonly asAssets: XcmV3MultiassetMultiAssets;
    readonly isExecutionResult: boolean;
    readonly asExecutionResult: Option<ITuple<[u32, XcmV3TraitsError]>>;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly isPalletsInfo: boolean;
    readonly asPalletsInfo: Vec<XcmV3PalletInfo>;
    readonly isDispatchResult: boolean;
    readonly asDispatchResult: XcmV3MaybeErrorCode;
    readonly type:
      | "Null"
      | "Assets"
      | "ExecutionResult"
      | "Version"
      | "PalletsInfo"
      | "DispatchResult";
  }

  /** @name XcmV3TraitsError (311) */
  interface XcmV3TraitsError extends Enum {
    readonly isOverflow: boolean;
    readonly isUnimplemented: boolean;
    readonly isUntrustedReserveLocation: boolean;
    readonly isUntrustedTeleportLocation: boolean;
    readonly isLocationFull: boolean;
    readonly isLocationNotInvertible: boolean;
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
    readonly isMaxWeightInvalid: boolean;
    readonly isNotHoldingFees: boolean;
    readonly isTooExpensive: boolean;
    readonly isTrap: boolean;
    readonly asTrap: u64;
    readonly isExpectationFalse: boolean;
    readonly isPalletNotFound: boolean;
    readonly isNameMismatch: boolean;
    readonly isVersionIncompatible: boolean;
    readonly isHoldingWouldOverflow: boolean;
    readonly isExportError: boolean;
    readonly isReanchorFailed: boolean;
    readonly isNoDeal: boolean;
    readonly isFeesNotMet: boolean;
    readonly isLockError: boolean;
    readonly isNoPermission: boolean;
    readonly isUnanchored: boolean;
    readonly isNotDepositable: boolean;
    readonly isUnhandledXcmVersion: boolean;
    readonly isWeightLimitReached: boolean;
    readonly asWeightLimitReached: SpWeightsWeightV2Weight;
    readonly isBarrier: boolean;
    readonly isWeightNotComputable: boolean;
    readonly isExceedsStackLimit: boolean;
    readonly type:
      | "Overflow"
      | "Unimplemented"
      | "UntrustedReserveLocation"
      | "UntrustedTeleportLocation"
      | "LocationFull"
      | "LocationNotInvertible"
      | "BadOrigin"
      | "InvalidLocation"
      | "AssetNotFound"
      | "FailedToTransactAsset"
      | "NotWithdrawable"
      | "LocationCannotHold"
      | "ExceedsMaxMessageSize"
      | "DestinationUnsupported"
      | "Transport"
      | "Unroutable"
      | "UnknownClaim"
      | "FailedToDecode"
      | "MaxWeightInvalid"
      | "NotHoldingFees"
      | "TooExpensive"
      | "Trap"
      | "ExpectationFalse"
      | "PalletNotFound"
      | "NameMismatch"
      | "VersionIncompatible"
      | "HoldingWouldOverflow"
      | "ExportError"
      | "ReanchorFailed"
      | "NoDeal"
      | "FeesNotMet"
      | "LockError"
      | "NoPermission"
      | "Unanchored"
      | "NotDepositable"
      | "UnhandledXcmVersion"
      | "WeightLimitReached"
      | "Barrier"
      | "WeightNotComputable"
      | "ExceedsStackLimit";
  }

  /** @name XcmV3PalletInfo (313) */
  interface XcmV3PalletInfo extends Struct {
    readonly index: Compact<u32>;
    readonly name: Bytes;
    readonly moduleName: Bytes;
    readonly major: Compact<u32>;
    readonly minor: Compact<u32>;
    readonly patch: Compact<u32>;
  }

  /** @name XcmV3QueryResponseInfo (317) */
  interface XcmV3QueryResponseInfo extends Struct {
    readonly destination: StagingXcmV3MultiLocation;
    readonly queryId: Compact<u64>;
    readonly maxWeight: SpWeightsWeightV2Weight;
  }

  /** @name XcmV3MultiassetMultiAssetFilter (318) */
  interface XcmV3MultiassetMultiAssetFilter extends Enum {
    readonly isDefinite: boolean;
    readonly asDefinite: XcmV3MultiassetMultiAssets;
    readonly isWild: boolean;
    readonly asWild: XcmV3MultiassetWildMultiAsset;
    readonly type: "Definite" | "Wild";
  }

  /** @name XcmV3MultiassetWildMultiAsset (319) */
  interface XcmV3MultiassetWildMultiAsset extends Enum {
    readonly isAll: boolean;
    readonly isAllOf: boolean;
    readonly asAllOf: {
      readonly id: XcmV3MultiassetAssetId;
      readonly fun: XcmV3MultiassetWildFungibility;
    } & Struct;
    readonly isAllCounted: boolean;
    readonly asAllCounted: Compact<u32>;
    readonly isAllOfCounted: boolean;
    readonly asAllOfCounted: {
      readonly id: XcmV3MultiassetAssetId;
      readonly fun: XcmV3MultiassetWildFungibility;
      readonly count: Compact<u32>;
    } & Struct;
    readonly type: "All" | "AllOf" | "AllCounted" | "AllOfCounted";
  }

  /** @name XcmV3MultiassetWildFungibility (320) */
  interface XcmV3MultiassetWildFungibility extends Enum {
    readonly isFungible: boolean;
    readonly isNonFungible: boolean;
    readonly type: "Fungible" | "NonFungible";
  }

  /** @name StagingXcmV4Xcm (321) */
  interface StagingXcmV4Xcm extends Vec<StagingXcmV4Instruction> {}

  /** @name StagingXcmV4Instruction (323) */
  interface StagingXcmV4Instruction extends Enum {
    readonly isWithdrawAsset: boolean;
    readonly asWithdrawAsset: StagingXcmV4AssetAssets;
    readonly isReserveAssetDeposited: boolean;
    readonly asReserveAssetDeposited: StagingXcmV4AssetAssets;
    readonly isReceiveTeleportedAsset: boolean;
    readonly asReceiveTeleportedAsset: StagingXcmV4AssetAssets;
    readonly isQueryResponse: boolean;
    readonly asQueryResponse: {
      readonly queryId: Compact<u64>;
      readonly response: StagingXcmV4Response;
      readonly maxWeight: SpWeightsWeightV2Weight;
      readonly querier: Option<StagingXcmV4Location>;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assets: StagingXcmV4AssetAssets;
      readonly beneficiary: StagingXcmV4Location;
    } & Struct;
    readonly isTransferReserveAsset: boolean;
    readonly asTransferReserveAsset: {
      readonly assets: StagingXcmV4AssetAssets;
      readonly dest: StagingXcmV4Location;
      readonly xcm: StagingXcmV4Xcm;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly originKind: XcmV3OriginKind;
      readonly requireWeightAtMost: SpWeightsWeightV2Weight;
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
    readonly asDescendOrigin: StagingXcmV4Junctions;
    readonly isReportError: boolean;
    readonly asReportError: StagingXcmV4QueryResponseInfo;
    readonly isDepositAsset: boolean;
    readonly asDepositAsset: {
      readonly assets: StagingXcmV4AssetAssetFilter;
      readonly beneficiary: StagingXcmV4Location;
    } & Struct;
    readonly isDepositReserveAsset: boolean;
    readonly asDepositReserveAsset: {
      readonly assets: StagingXcmV4AssetAssetFilter;
      readonly dest: StagingXcmV4Location;
      readonly xcm: StagingXcmV4Xcm;
    } & Struct;
    readonly isExchangeAsset: boolean;
    readonly asExchangeAsset: {
      readonly give: StagingXcmV4AssetAssetFilter;
      readonly want: StagingXcmV4AssetAssets;
      readonly maximal: bool;
    } & Struct;
    readonly isInitiateReserveWithdraw: boolean;
    readonly asInitiateReserveWithdraw: {
      readonly assets: StagingXcmV4AssetAssetFilter;
      readonly reserve: StagingXcmV4Location;
      readonly xcm: StagingXcmV4Xcm;
    } & Struct;
    readonly isInitiateTeleport: boolean;
    readonly asInitiateTeleport: {
      readonly assets: StagingXcmV4AssetAssetFilter;
      readonly dest: StagingXcmV4Location;
      readonly xcm: StagingXcmV4Xcm;
    } & Struct;
    readonly isReportHolding: boolean;
    readonly asReportHolding: {
      readonly responseInfo: StagingXcmV4QueryResponseInfo;
      readonly assets: StagingXcmV4AssetAssetFilter;
    } & Struct;
    readonly isBuyExecution: boolean;
    readonly asBuyExecution: {
      readonly fees: StagingXcmV4Asset;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isRefundSurplus: boolean;
    readonly isSetErrorHandler: boolean;
    readonly asSetErrorHandler: StagingXcmV4Xcm;
    readonly isSetAppendix: boolean;
    readonly asSetAppendix: StagingXcmV4Xcm;
    readonly isClearError: boolean;
    readonly isClaimAsset: boolean;
    readonly asClaimAsset: {
      readonly assets: StagingXcmV4AssetAssets;
      readonly ticket: StagingXcmV4Location;
    } & Struct;
    readonly isTrap: boolean;
    readonly asTrap: Compact<u64>;
    readonly isSubscribeVersion: boolean;
    readonly asSubscribeVersion: {
      readonly queryId: Compact<u64>;
      readonly maxResponseWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isUnsubscribeVersion: boolean;
    readonly isBurnAsset: boolean;
    readonly asBurnAsset: StagingXcmV4AssetAssets;
    readonly isExpectAsset: boolean;
    readonly asExpectAsset: StagingXcmV4AssetAssets;
    readonly isExpectOrigin: boolean;
    readonly asExpectOrigin: Option<StagingXcmV4Location>;
    readonly isExpectError: boolean;
    readonly asExpectError: Option<ITuple<[u32, XcmV3TraitsError]>>;
    readonly isExpectTransactStatus: boolean;
    readonly asExpectTransactStatus: XcmV3MaybeErrorCode;
    readonly isQueryPallet: boolean;
    readonly asQueryPallet: {
      readonly moduleName: Bytes;
      readonly responseInfo: StagingXcmV4QueryResponseInfo;
    } & Struct;
    readonly isExpectPallet: boolean;
    readonly asExpectPallet: {
      readonly index: Compact<u32>;
      readonly name: Bytes;
      readonly moduleName: Bytes;
      readonly crateMajor: Compact<u32>;
      readonly minCrateMinor: Compact<u32>;
    } & Struct;
    readonly isReportTransactStatus: boolean;
    readonly asReportTransactStatus: StagingXcmV4QueryResponseInfo;
    readonly isClearTransactStatus: boolean;
    readonly isUniversalOrigin: boolean;
    readonly asUniversalOrigin: StagingXcmV4Junction;
    readonly isExportMessage: boolean;
    readonly asExportMessage: {
      readonly network: StagingXcmV4JunctionNetworkId;
      readonly destination: StagingXcmV4Junctions;
      readonly xcm: StagingXcmV4Xcm;
    } & Struct;
    readonly isLockAsset: boolean;
    readonly asLockAsset: {
      readonly asset: StagingXcmV4Asset;
      readonly unlocker: StagingXcmV4Location;
    } & Struct;
    readonly isUnlockAsset: boolean;
    readonly asUnlockAsset: {
      readonly asset: StagingXcmV4Asset;
      readonly target: StagingXcmV4Location;
    } & Struct;
    readonly isNoteUnlockable: boolean;
    readonly asNoteUnlockable: {
      readonly asset: StagingXcmV4Asset;
      readonly owner: StagingXcmV4Location;
    } & Struct;
    readonly isRequestUnlock: boolean;
    readonly asRequestUnlock: {
      readonly asset: StagingXcmV4Asset;
      readonly locker: StagingXcmV4Location;
    } & Struct;
    readonly isSetFeesMode: boolean;
    readonly asSetFeesMode: {
      readonly jitWithdraw: bool;
    } & Struct;
    readonly isSetTopic: boolean;
    readonly asSetTopic: U8aFixed;
    readonly isClearTopic: boolean;
    readonly isAliasOrigin: boolean;
    readonly asAliasOrigin: StagingXcmV4Location;
    readonly isUnpaidExecution: boolean;
    readonly asUnpaidExecution: {
      readonly weightLimit: XcmV3WeightLimit;
      readonly checkOrigin: Option<StagingXcmV4Location>;
    } & Struct;
    readonly type:
      | "WithdrawAsset"
      | "ReserveAssetDeposited"
      | "ReceiveTeleportedAsset"
      | "QueryResponse"
      | "TransferAsset"
      | "TransferReserveAsset"
      | "Transact"
      | "HrmpNewChannelOpenRequest"
      | "HrmpChannelAccepted"
      | "HrmpChannelClosing"
      | "ClearOrigin"
      | "DescendOrigin"
      | "ReportError"
      | "DepositAsset"
      | "DepositReserveAsset"
      | "ExchangeAsset"
      | "InitiateReserveWithdraw"
      | "InitiateTeleport"
      | "ReportHolding"
      | "BuyExecution"
      | "RefundSurplus"
      | "SetErrorHandler"
      | "SetAppendix"
      | "ClearError"
      | "ClaimAsset"
      | "Trap"
      | "SubscribeVersion"
      | "UnsubscribeVersion"
      | "BurnAsset"
      | "ExpectAsset"
      | "ExpectOrigin"
      | "ExpectError"
      | "ExpectTransactStatus"
      | "QueryPallet"
      | "ExpectPallet"
      | "ReportTransactStatus"
      | "ClearTransactStatus"
      | "UniversalOrigin"
      | "ExportMessage"
      | "LockAsset"
      | "UnlockAsset"
      | "NoteUnlockable"
      | "RequestUnlock"
      | "SetFeesMode"
      | "SetTopic"
      | "ClearTopic"
      | "AliasOrigin"
      | "UnpaidExecution";
  }

  /** @name StagingXcmV4Response (324) */
  interface StagingXcmV4Response extends Enum {
    readonly isNull: boolean;
    readonly isAssets: boolean;
    readonly asAssets: StagingXcmV4AssetAssets;
    readonly isExecutionResult: boolean;
    readonly asExecutionResult: Option<ITuple<[u32, XcmV3TraitsError]>>;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly isPalletsInfo: boolean;
    readonly asPalletsInfo: Vec<StagingXcmV4PalletInfo>;
    readonly isDispatchResult: boolean;
    readonly asDispatchResult: XcmV3MaybeErrorCode;
    readonly type:
      | "Null"
      | "Assets"
      | "ExecutionResult"
      | "Version"
      | "PalletsInfo"
      | "DispatchResult";
  }

  /** @name StagingXcmV4PalletInfo (326) */
  interface StagingXcmV4PalletInfo extends Struct {
    readonly index: Compact<u32>;
    readonly name: Bytes;
    readonly moduleName: Bytes;
    readonly major: Compact<u32>;
    readonly minor: Compact<u32>;
    readonly patch: Compact<u32>;
  }

  /** @name StagingXcmV4QueryResponseInfo (330) */
  interface StagingXcmV4QueryResponseInfo extends Struct {
    readonly destination: StagingXcmV4Location;
    readonly queryId: Compact<u64>;
    readonly maxWeight: SpWeightsWeightV2Weight;
  }

  /** @name StagingXcmV4AssetAssetFilter (331) */
  interface StagingXcmV4AssetAssetFilter extends Enum {
    readonly isDefinite: boolean;
    readonly asDefinite: StagingXcmV4AssetAssets;
    readonly isWild: boolean;
    readonly asWild: StagingXcmV4AssetWildAsset;
    readonly type: "Definite" | "Wild";
  }

  /** @name StagingXcmV4AssetWildAsset (332) */
  interface StagingXcmV4AssetWildAsset extends Enum {
    readonly isAll: boolean;
    readonly isAllOf: boolean;
    readonly asAllOf: {
      readonly id: StagingXcmV4AssetAssetId;
      readonly fun: StagingXcmV4AssetWildFungibility;
    } & Struct;
    readonly isAllCounted: boolean;
    readonly asAllCounted: Compact<u32>;
    readonly isAllOfCounted: boolean;
    readonly asAllOfCounted: {
      readonly id: StagingXcmV4AssetAssetId;
      readonly fun: StagingXcmV4AssetWildFungibility;
      readonly count: Compact<u32>;
    } & Struct;
    readonly type: "All" | "AllOf" | "AllCounted" | "AllOfCounted";
  }

  /** @name StagingXcmV4AssetWildFungibility (333) */
  interface StagingXcmV4AssetWildFungibility extends Enum {
    readonly isFungible: boolean;
    readonly isNonFungible: boolean;
    readonly type: "Fungible" | "NonFungible";
  }

  /** @name StagingXcmExecutorAssetTransferTransferType (345) */
  interface StagingXcmExecutorAssetTransferTransferType extends Enum {
    readonly isTeleport: boolean;
    readonly isLocalReserve: boolean;
    readonly isDestinationReserve: boolean;
    readonly isRemoteReserve: boolean;
    readonly asRemoteReserve: XcmVersionedLocation;
    readonly type: "Teleport" | "LocalReserve" | "DestinationReserve" | "RemoteReserve";
  }

  /** @name XcmVersionedAssetId (346) */
  interface XcmVersionedAssetId extends Enum {
    readonly isV3: boolean;
    readonly asV3: XcmV3MultiassetAssetId;
    readonly isV4: boolean;
    readonly asV4: StagingXcmV4AssetAssetId;
    readonly isV5: boolean;
    readonly asV5: StagingXcmV5AssetAssetId;
    readonly type: "V3" | "V4" | "V5";
  }

  /** @name PalletAssetsCall (347) */
  interface PalletAssetsCall extends Enum {
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
    readonly isStartDestroy: boolean;
    readonly asStartDestroy: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isDestroyAccounts: boolean;
    readonly asDestroyAccounts: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isDestroyApprovals: boolean;
    readonly asDestroyApprovals: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isFinishDestroy: boolean;
    readonly asFinishDestroy: {
      readonly id: Compact<u128>;
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
    readonly isTouch: boolean;
    readonly asTouch: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isRefund: boolean;
    readonly asRefund: {
      readonly id: Compact<u128>;
      readonly allowBurn: bool;
    } & Struct;
    readonly isSetMinBalance: boolean;
    readonly asSetMinBalance: {
      readonly id: Compact<u128>;
      readonly minBalance: u128;
    } & Struct;
    readonly isTouchOther: boolean;
    readonly asTouchOther: {
      readonly id: Compact<u128>;
      readonly who: AccountId20;
    } & Struct;
    readonly isRefundOther: boolean;
    readonly asRefundOther: {
      readonly id: Compact<u128>;
      readonly who: AccountId20;
    } & Struct;
    readonly isBlock: boolean;
    readonly asBlock: {
      readonly id: Compact<u128>;
      readonly who: AccountId20;
    } & Struct;
    readonly isTransferAll: boolean;
    readonly asTransferAll: {
      readonly id: Compact<u128>;
      readonly dest: AccountId20;
      readonly keepAlive: bool;
    } & Struct;
    readonly type:
      | "Create"
      | "ForceCreate"
      | "StartDestroy"
      | "DestroyAccounts"
      | "DestroyApprovals"
      | "FinishDestroy"
      | "Mint"
      | "Burn"
      | "Transfer"
      | "TransferKeepAlive"
      | "ForceTransfer"
      | "Freeze"
      | "Thaw"
      | "FreezeAsset"
      | "ThawAsset"
      | "TransferOwnership"
      | "SetTeam"
      | "SetMetadata"
      | "ClearMetadata"
      | "ForceSetMetadata"
      | "ForceClearMetadata"
      | "ForceAssetStatus"
      | "ApproveTransfer"
      | "CancelApproval"
      | "ForceCancelApproval"
      | "TransferApproved"
      | "Touch"
      | "Refund"
      | "SetMinBalance"
      | "TouchOther"
      | "RefundOther"
      | "Block"
      | "TransferAll";
  }

  /** @name PalletAssetManagerCall (348) */
  interface PalletAssetManagerCall extends Enum {
    readonly isRegisterForeignAsset: boolean;
    readonly asRegisterForeignAsset: {
      readonly asset: MoonbaseRuntimeXcmConfigAssetType;
      readonly metadata: MoonbaseRuntimeAssetConfigAssetRegistrarMetadata;
      readonly minAmount: u128;
      readonly isSufficient: bool;
    } & Struct;
    readonly isChangeExistingAssetType: boolean;
    readonly asChangeExistingAssetType: {
      readonly assetId: u128;
      readonly newAssetType: MoonbaseRuntimeXcmConfigAssetType;
      readonly numAssetsWeightHint: u32;
    } & Struct;
    readonly isRemoveExistingAssetType: boolean;
    readonly asRemoveExistingAssetType: {
      readonly assetId: u128;
      readonly numAssetsWeightHint: u32;
    } & Struct;
    readonly isDestroyForeignAsset: boolean;
    readonly asDestroyForeignAsset: {
      readonly assetId: u128;
      readonly numAssetsWeightHint: u32;
    } & Struct;
    readonly type:
      | "RegisterForeignAsset"
      | "ChangeExistingAssetType"
      | "RemoveExistingAssetType"
      | "DestroyForeignAsset";
  }

  /** @name PalletXcmTransactorCall (349) */
  interface PalletXcmTransactorCall extends Enum {
    readonly isRegister: boolean;
    readonly asRegister: {
      readonly who: AccountId20;
      readonly index: u16;
    } & Struct;
    readonly isDeregister: boolean;
    readonly asDeregister: {
      readonly index: u16;
    } & Struct;
    readonly isTransactThroughDerivative: boolean;
    readonly asTransactThroughDerivative: {
      readonly dest: MoonbaseRuntimeXcmConfigTransactors;
      readonly index: u16;
      readonly fee: PalletXcmTransactorCurrencyPayment;
      readonly innerCall: Bytes;
      readonly weightInfo: PalletXcmTransactorTransactWeights;
      readonly refund: bool;
    } & Struct;
    readonly isTransactThroughSovereign: boolean;
    readonly asTransactThroughSovereign: {
      readonly dest: XcmVersionedLocation;
      readonly feePayer: Option<AccountId20>;
      readonly fee: PalletXcmTransactorCurrencyPayment;
      readonly call: Bytes;
      readonly originKind: XcmV3OriginKind;
      readonly weightInfo: PalletXcmTransactorTransactWeights;
      readonly refund: bool;
    } & Struct;
    readonly isSetTransactInfo: boolean;
    readonly asSetTransactInfo: {
      readonly location: XcmVersionedLocation;
      readonly transactExtraWeight: SpWeightsWeightV2Weight;
      readonly maxWeight: SpWeightsWeightV2Weight;
      readonly transactExtraWeightSigned: Option<SpWeightsWeightV2Weight>;
    } & Struct;
    readonly isRemoveTransactInfo: boolean;
    readonly asRemoveTransactInfo: {
      readonly location: XcmVersionedLocation;
    } & Struct;
    readonly isTransactThroughSigned: boolean;
    readonly asTransactThroughSigned: {
      readonly dest: XcmVersionedLocation;
      readonly fee: PalletXcmTransactorCurrencyPayment;
      readonly call: Bytes;
      readonly weightInfo: PalletXcmTransactorTransactWeights;
      readonly refund: bool;
    } & Struct;
    readonly isSetFeePerSecond: boolean;
    readonly asSetFeePerSecond: {
      readonly assetLocation: XcmVersionedLocation;
      readonly feePerSecond: u128;
    } & Struct;
    readonly isRemoveFeePerSecond: boolean;
    readonly asRemoveFeePerSecond: {
      readonly assetLocation: XcmVersionedLocation;
    } & Struct;
    readonly isHrmpManage: boolean;
    readonly asHrmpManage: {
      readonly action: PalletXcmTransactorHrmpOperation;
      readonly fee: PalletXcmTransactorCurrencyPayment;
      readonly weightInfo: PalletXcmTransactorTransactWeights;
    } & Struct;
    readonly type:
      | "Register"
      | "Deregister"
      | "TransactThroughDerivative"
      | "TransactThroughSovereign"
      | "SetTransactInfo"
      | "RemoveTransactInfo"
      | "TransactThroughSigned"
      | "SetFeePerSecond"
      | "RemoveFeePerSecond"
      | "HrmpManage";
  }

  /** @name MoonbaseRuntimeXcmConfigTransactors (350) */
  interface MoonbaseRuntimeXcmConfigTransactors extends Enum {
    readonly isRelay: boolean;
    readonly type: "Relay";
  }

  /** @name PalletXcmTransactorCurrencyPayment (351) */
  interface PalletXcmTransactorCurrencyPayment extends Struct {
    readonly currency: PalletXcmTransactorCurrency;
    readonly feeAmount: Option<u128>;
  }

  /** @name MoonbaseRuntimeXcmConfigCurrencyId (352) */
  interface MoonbaseRuntimeXcmConfigCurrencyId extends Enum {
    readonly isSelfReserve: boolean;
    readonly isForeignAsset: boolean;
    readonly asForeignAsset: u128;
    readonly isErc20: boolean;
    readonly asErc20: {
      readonly contractAddress: H160;
    } & Struct;
    readonly type: "SelfReserve" | "ForeignAsset" | "Erc20";
  }

  /** @name PalletXcmTransactorCurrency (353) */
  interface PalletXcmTransactorCurrency extends Enum {
    readonly isAsCurrencyId: boolean;
    readonly asAsCurrencyId: MoonbaseRuntimeXcmConfigCurrencyId;
    readonly isAsMultiLocation: boolean;
    readonly asAsMultiLocation: XcmVersionedLocation;
    readonly type: "AsCurrencyId" | "AsMultiLocation";
  }

  /** @name PalletXcmTransactorTransactWeights (355) */
  interface PalletXcmTransactorTransactWeights extends Struct {
    readonly transactRequiredWeightAtMost: SpWeightsWeightV2Weight;
    readonly overallWeight: Option<XcmV3WeightLimit>;
  }

  /** @name PalletMoonbeamOrbitersCall (357) */
  interface PalletMoonbeamOrbitersCall extends Enum {
    readonly isCollatorAddOrbiter: boolean;
    readonly asCollatorAddOrbiter: {
      readonly orbiter: AccountId20;
    } & Struct;
    readonly isCollatorRemoveOrbiter: boolean;
    readonly asCollatorRemoveOrbiter: {
      readonly orbiter: AccountId20;
    } & Struct;
    readonly isOrbiterLeaveCollatorPool: boolean;
    readonly asOrbiterLeaveCollatorPool: {
      readonly collator: AccountId20;
    } & Struct;
    readonly isOrbiterRegister: boolean;
    readonly isOrbiterUnregister: boolean;
    readonly asOrbiterUnregister: {
      readonly collatorsPoolCount: u32;
    } & Struct;
    readonly isAddCollator: boolean;
    readonly asAddCollator: {
      readonly collator: AccountId20;
    } & Struct;
    readonly isRemoveCollator: boolean;
    readonly asRemoveCollator: {
      readonly collator: AccountId20;
    } & Struct;
    readonly type:
      | "CollatorAddOrbiter"
      | "CollatorRemoveOrbiter"
      | "OrbiterLeaveCollatorPool"
      | "OrbiterRegister"
      | "OrbiterUnregister"
      | "AddCollator"
      | "RemoveCollator";
  }

  /** @name PalletEthereumXcmCall (358) */
  interface PalletEthereumXcmCall extends Enum {
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly xcmTransaction: XcmPrimitivesEthereumXcmEthereumXcmTransaction;
    } & Struct;
    readonly isTransactThroughProxy: boolean;
    readonly asTransactThroughProxy: {
      readonly transactAs: H160;
      readonly xcmTransaction: XcmPrimitivesEthereumXcmEthereumXcmTransaction;
    } & Struct;
    readonly isSuspendEthereumXcmExecution: boolean;
    readonly isResumeEthereumXcmExecution: boolean;
    readonly isForceTransactAs: boolean;
    readonly asForceTransactAs: {
      readonly transactAs: H160;
      readonly xcmTransaction: XcmPrimitivesEthereumXcmEthereumXcmTransaction;
      readonly forceCreateAddress: Option<H160>;
    } & Struct;
    readonly type:
      | "Transact"
      | "TransactThroughProxy"
      | "SuspendEthereumXcmExecution"
      | "ResumeEthereumXcmExecution"
      | "ForceTransactAs";
  }

  /** @name XcmPrimitivesEthereumXcmEthereumXcmTransaction (359) */
  interface XcmPrimitivesEthereumXcmEthereumXcmTransaction extends Enum {
    readonly isV1: boolean;
    readonly asV1: XcmPrimitivesEthereumXcmEthereumXcmTransactionV1;
    readonly isV2: boolean;
    readonly asV2: XcmPrimitivesEthereumXcmEthereumXcmTransactionV2;
    readonly type: "V1" | "V2";
  }

  /** @name XcmPrimitivesEthereumXcmEthereumXcmTransactionV1 (360) */
  interface XcmPrimitivesEthereumXcmEthereumXcmTransactionV1 extends Struct {
    readonly gasLimit: U256;
    readonly feePayment: XcmPrimitivesEthereumXcmEthereumXcmFee;
    readonly action: EthereumTransactionLegacyTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Option<Vec<ITuple<[H160, Vec<H256>]>>>;
  }

  /** @name XcmPrimitivesEthereumXcmEthereumXcmFee (361) */
  interface XcmPrimitivesEthereumXcmEthereumXcmFee extends Enum {
    readonly isManual: boolean;
    readonly asManual: XcmPrimitivesEthereumXcmManualEthereumXcmFee;
    readonly isAuto: boolean;
    readonly type: "Manual" | "Auto";
  }

  /** @name XcmPrimitivesEthereumXcmManualEthereumXcmFee (362) */
  interface XcmPrimitivesEthereumXcmManualEthereumXcmFee extends Struct {
    readonly gasPrice: Option<U256>;
    readonly maxFeePerGas: Option<U256>;
  }

  /** @name XcmPrimitivesEthereumXcmEthereumXcmTransactionV2 (365) */
  interface XcmPrimitivesEthereumXcmEthereumXcmTransactionV2 extends Struct {
    readonly gasLimit: U256;
    readonly action: EthereumTransactionLegacyTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Option<Vec<ITuple<[H160, Vec<H256>]>>>;
  }

  /** @name PalletRandomnessCall (367) */
  interface PalletRandomnessCall extends Enum {
    readonly isSetBabeRandomnessResults: boolean;
    readonly type: "SetBabeRandomnessResults";
  }

  /** @name PalletCollectiveCall (368) */
  interface PalletCollectiveCall extends Enum {
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
    readonly isDisapproveProposal: boolean;
    readonly asDisapproveProposal: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isClose: boolean;
    readonly asClose: {
      readonly proposalHash: H256;
      readonly index: Compact<u32>;
      readonly proposalWeightBound: SpWeightsWeightV2Weight;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isKill: boolean;
    readonly asKill: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isReleaseProposalCost: boolean;
    readonly asReleaseProposalCost: {
      readonly proposalHash: H256;
    } & Struct;
    readonly type:
      | "SetMembers"
      | "Execute"
      | "Propose"
      | "Vote"
      | "DisapproveProposal"
      | "Close"
      | "Kill"
      | "ReleaseProposalCost";
  }

  /** @name PalletConvictionVotingCall (369) */
  interface PalletConvictionVotingCall extends Enum {
    readonly isVote: boolean;
    readonly asVote: {
      readonly pollIndex: Compact<u32>;
      readonly vote: PalletConvictionVotingVoteAccountVote;
    } & Struct;
    readonly isDelegate: boolean;
    readonly asDelegate: {
      readonly class: u16;
      readonly to: AccountId20;
      readonly conviction: PalletConvictionVotingConviction;
      readonly balance: u128;
    } & Struct;
    readonly isUndelegate: boolean;
    readonly asUndelegate: {
      readonly class: u16;
    } & Struct;
    readonly isUnlock: boolean;
    readonly asUnlock: {
      readonly class: u16;
      readonly target: AccountId20;
    } & Struct;
    readonly isRemoveVote: boolean;
    readonly asRemoveVote: {
      readonly class: Option<u16>;
      readonly index: u32;
    } & Struct;
    readonly isRemoveOtherVote: boolean;
    readonly asRemoveOtherVote: {
      readonly target: AccountId20;
      readonly class: u16;
      readonly index: u32;
    } & Struct;
    readonly type: "Vote" | "Delegate" | "Undelegate" | "Unlock" | "RemoveVote" | "RemoveOtherVote";
  }

  /** @name PalletConvictionVotingConviction (370) */
  interface PalletConvictionVotingConviction extends Enum {
    readonly isNone: boolean;
    readonly isLocked1x: boolean;
    readonly isLocked2x: boolean;
    readonly isLocked3x: boolean;
    readonly isLocked4x: boolean;
    readonly isLocked5x: boolean;
    readonly isLocked6x: boolean;
    readonly type:
      | "None"
      | "Locked1x"
      | "Locked2x"
      | "Locked3x"
      | "Locked4x"
      | "Locked5x"
      | "Locked6x";
  }

  /** @name PalletReferendaCall (372) */
  interface PalletReferendaCall extends Enum {
    readonly isSubmit: boolean;
    readonly asSubmit: {
      readonly proposalOrigin: MoonbaseRuntimeOriginCaller;
      readonly proposal: FrameSupportPreimagesBounded;
      readonly enactmentMoment: FrameSupportScheduleDispatchTime;
    } & Struct;
    readonly isPlaceDecisionDeposit: boolean;
    readonly asPlaceDecisionDeposit: {
      readonly index: u32;
    } & Struct;
    readonly isRefundDecisionDeposit: boolean;
    readonly asRefundDecisionDeposit: {
      readonly index: u32;
    } & Struct;
    readonly isCancel: boolean;
    readonly asCancel: {
      readonly index: u32;
    } & Struct;
    readonly isKill: boolean;
    readonly asKill: {
      readonly index: u32;
    } & Struct;
    readonly isNudgeReferendum: boolean;
    readonly asNudgeReferendum: {
      readonly index: u32;
    } & Struct;
    readonly isOneFewerDeciding: boolean;
    readonly asOneFewerDeciding: {
      readonly track: u16;
    } & Struct;
    readonly isRefundSubmissionDeposit: boolean;
    readonly asRefundSubmissionDeposit: {
      readonly index: u32;
    } & Struct;
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly index: u32;
      readonly maybeHash: Option<H256>;
    } & Struct;
    readonly type:
      | "Submit"
      | "PlaceDecisionDeposit"
      | "RefundDecisionDeposit"
      | "Cancel"
      | "Kill"
      | "NudgeReferendum"
      | "OneFewerDeciding"
      | "RefundSubmissionDeposit"
      | "SetMetadata";
  }

  /** @name FrameSupportScheduleDispatchTime (373) */
  interface FrameSupportScheduleDispatchTime extends Enum {
    readonly isAt: boolean;
    readonly asAt: u32;
    readonly isAfter: boolean;
    readonly asAfter: u32;
    readonly type: "At" | "After";
  }

  /** @name PalletPreimageCall (375) */
  interface PalletPreimageCall extends Enum {
    readonly isNotePreimage: boolean;
    readonly asNotePreimage: {
      readonly bytes: Bytes;
    } & Struct;
    readonly isUnnotePreimage: boolean;
    readonly asUnnotePreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isRequestPreimage: boolean;
    readonly asRequestPreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isUnrequestPreimage: boolean;
    readonly asUnrequestPreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isEnsureUpdated: boolean;
    readonly asEnsureUpdated: {
      readonly hashes: Vec<H256>;
    } & Struct;
    readonly type:
      | "NotePreimage"
      | "UnnotePreimage"
      | "RequestPreimage"
      | "UnrequestPreimage"
      | "EnsureUpdated";
  }

  /** @name PalletWhitelistCall (376) */
  interface PalletWhitelistCall extends Enum {
    readonly isWhitelistCall: boolean;
    readonly asWhitelistCall: {
      readonly callHash: H256;
    } & Struct;
    readonly isRemoveWhitelistedCall: boolean;
    readonly asRemoveWhitelistedCall: {
      readonly callHash: H256;
    } & Struct;
    readonly isDispatchWhitelistedCall: boolean;
    readonly asDispatchWhitelistedCall: {
      readonly callHash: H256;
      readonly callEncodedLen: u32;
      readonly callWeightWitness: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isDispatchWhitelistedCallWithPreimage: boolean;
    readonly asDispatchWhitelistedCallWithPreimage: {
      readonly call: Call;
    } & Struct;
    readonly type:
      | "WhitelistCall"
      | "RemoveWhitelistedCall"
      | "DispatchWhitelistedCall"
      | "DispatchWhitelistedCallWithPreimage";
  }

  /** @name PalletRootTestingCall (378) */
  interface PalletRootTestingCall extends Enum {
    readonly isFillBlock: boolean;
    readonly asFillBlock: {
      readonly ratio: Perbill;
    } & Struct;
    readonly isTriggerDefensive: boolean;
    readonly type: "FillBlock" | "TriggerDefensive";
  }

  /** @name PalletMultisigCall (379) */
  interface PalletMultisigCall extends Enum {
    readonly isAsMultiThreshold1: boolean;
    readonly asAsMultiThreshold1: {
      readonly otherSignatories: Vec<AccountId20>;
      readonly call: Call;
    } & Struct;
    readonly isAsMulti: boolean;
    readonly asAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId20>;
      readonly maybeTimepoint: Option<PalletMultisigTimepoint>;
      readonly call: Call;
      readonly maxWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isApproveAsMulti: boolean;
    readonly asApproveAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId20>;
      readonly maybeTimepoint: Option<PalletMultisigTimepoint>;
      readonly callHash: U8aFixed;
      readonly maxWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isCancelAsMulti: boolean;
    readonly asCancelAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId20>;
      readonly timepoint: PalletMultisigTimepoint;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly type: "AsMultiThreshold1" | "AsMulti" | "ApproveAsMulti" | "CancelAsMulti";
  }

  /** @name PalletMultisigTimepoint (381) */
  interface PalletMultisigTimepoint extends Struct {
    readonly height: u32;
    readonly index: u32;
  }

  /** @name PalletMoonbeamLazyMigrationsCall (382) */
  interface PalletMoonbeamLazyMigrationsCall extends Enum {
    readonly isCreateContractMetadata: boolean;
    readonly asCreateContractMetadata: {
      readonly address: H160;
    } & Struct;
    readonly type: "CreateContractMetadata";
  }

  /** @name PalletMessageQueueCall (383) */
  interface PalletMessageQueueCall extends Enum {
    readonly isReapPage: boolean;
    readonly asReapPage: {
      readonly messageOrigin: CumulusPrimitivesCoreAggregateMessageOrigin;
      readonly pageIndex: u32;
    } & Struct;
    readonly isExecuteOverweight: boolean;
    readonly asExecuteOverweight: {
      readonly messageOrigin: CumulusPrimitivesCoreAggregateMessageOrigin;
      readonly page: u32;
      readonly index: u32;
      readonly weightLimit: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type: "ReapPage" | "ExecuteOverweight";
  }

  /** @name CumulusPrimitivesCoreAggregateMessageOrigin (384) */
  interface CumulusPrimitivesCoreAggregateMessageOrigin extends Enum {
    readonly isHere: boolean;
    readonly isParent: boolean;
    readonly isSibling: boolean;
    readonly asSibling: u32;
    readonly type: "Here" | "Parent" | "Sibling";
  }

  /** @name PalletEmergencyParaXcmCall (385) */
  interface PalletEmergencyParaXcmCall extends Enum {
    readonly isPausedToNormal: boolean;
    readonly isFastAuthorizeUpgrade: boolean;
    readonly asFastAuthorizeUpgrade: {
      readonly codeHash: H256;
    } & Struct;
    readonly type: "PausedToNormal" | "FastAuthorizeUpgrade";
  }

  /** @name PalletMoonbeamForeignAssetsCall (386) */
  interface PalletMoonbeamForeignAssetsCall extends Enum {
    readonly isCreateForeignAsset: boolean;
    readonly asCreateForeignAsset: {
      readonly assetId: u128;
      readonly assetXcmLocation: StagingXcmV5Location;
      readonly decimals: u8;
      readonly symbol: Bytes;
      readonly name: Bytes;
    } & Struct;
    readonly isChangeXcmLocation: boolean;
    readonly asChangeXcmLocation: {
      readonly assetId: u128;
      readonly newXcmLocation: StagingXcmV5Location;
    } & Struct;
    readonly isFreezeForeignAsset: boolean;
    readonly asFreezeForeignAsset: {
      readonly assetId: u128;
      readonly allowXcmDeposit: bool;
    } & Struct;
    readonly isUnfreezeForeignAsset: boolean;
    readonly asUnfreezeForeignAsset: {
      readonly assetId: u128;
    } & Struct;
    readonly type:
      | "CreateForeignAsset"
      | "ChangeXcmLocation"
      | "FreezeForeignAsset"
      | "UnfreezeForeignAsset";
  }

  /** @name PalletParametersCall (388) */
  interface PalletParametersCall extends Enum {
    readonly isSetParameter: boolean;
    readonly asSetParameter: {
      readonly keyValue: MoonbaseRuntimeRuntimeParamsRuntimeParameters;
    } & Struct;
    readonly type: "SetParameter";
  }

  /** @name MoonbaseRuntimeRuntimeParamsRuntimeParameters (389) */
  interface MoonbaseRuntimeRuntimeParamsRuntimeParameters extends Enum {
    readonly isRuntimeConfig: boolean;
    readonly asRuntimeConfig: MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParameters;
    readonly isPalletRandomness: boolean;
    readonly asPalletRandomness: MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParameters;
    readonly isXcmConfig: boolean;
    readonly asXcmConfig: MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParameters;
    readonly type: "RuntimeConfig" | "PalletRandomness" | "XcmConfig";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParameters (390) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParameters extends Enum {
    readonly isFeesTreasuryProportion: boolean;
    readonly asFeesTreasuryProportion: ITuple<
      [
        MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigFeesTreasuryProportion,
        Option<Perbill>
      ]
    >;
    readonly type: "FeesTreasuryProportion";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigFeesTreasuryProportion (391) */
  type MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigFeesTreasuryProportion = Null;

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParameters (393) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParameters extends Enum {
    readonly isDeposit: boolean;
    readonly asDeposit: ITuple<
      [MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessDeposit, Option<u128>]
    >;
    readonly type: "Deposit";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessDeposit (394) */
  type MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessDeposit = Null;

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParameters (397) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParameters extends Enum {
    readonly isForeignAssetCreationDeposit: boolean;
    readonly asForeignAssetCreationDeposit: ITuple<
      [MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigForeignAssetCreationDeposit, Option<u128>]
    >;
    readonly type: "ForeignAssetCreationDeposit";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigForeignAssetCreationDeposit (398) */
  type MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigForeignAssetCreationDeposit = Null;

  /** @name PalletXcmWeightTraderCall (399) */
  interface PalletXcmWeightTraderCall extends Enum {
    readonly isAddAsset: boolean;
    readonly asAddAsset: {
      readonly location: StagingXcmV5Location;
      readonly relativePrice: u128;
    } & Struct;
    readonly isEditAsset: boolean;
    readonly asEditAsset: {
      readonly location: StagingXcmV5Location;
      readonly relativePrice: u128;
    } & Struct;
    readonly isPauseAssetSupport: boolean;
    readonly asPauseAssetSupport: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly isResumeAssetSupport: boolean;
    readonly asResumeAssetSupport: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly isRemoveAsset: boolean;
    readonly asRemoveAsset: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly type:
      | "AddAsset"
      | "EditAsset"
      | "PauseAssetSupport"
      | "ResumeAssetSupport"
      | "RemoveAsset";
  }

  /** @name PalletMigrationsCall (400) */
  interface PalletMigrationsCall extends Enum {
    readonly isForceSetCursor: boolean;
    readonly asForceSetCursor: {
      readonly cursor: Option<PalletMigrationsMigrationCursor>;
    } & Struct;
    readonly isForceSetActiveCursor: boolean;
    readonly asForceSetActiveCursor: {
      readonly index: u32;
      readonly innerCursor: Option<Bytes>;
      readonly startedAt: Option<u32>;
    } & Struct;
    readonly isForceOnboardMbms: boolean;
    readonly isClearHistoric: boolean;
    readonly asClearHistoric: {
      readonly selector: PalletMigrationsHistoricCleanupSelector;
    } & Struct;
    readonly type: "ForceSetCursor" | "ForceSetActiveCursor" | "ForceOnboardMbms" | "ClearHistoric";
  }

  /** @name PalletMigrationsMigrationCursor (402) */
  interface PalletMigrationsMigrationCursor extends Enum {
    readonly isActive: boolean;
    readonly asActive: PalletMigrationsActiveCursor;
    readonly isStuck: boolean;
    readonly type: "Active" | "Stuck";
  }

  /** @name PalletMigrationsActiveCursor (403) */
  interface PalletMigrationsActiveCursor extends Struct {
    readonly index: u32;
    readonly innerCursor: Option<Bytes>;
    readonly startedAt: u32;
  }

  /** @name PalletMigrationsHistoricCleanupSelector (405) */
  interface PalletMigrationsHistoricCleanupSelector extends Enum {
    readonly isSpecific: boolean;
    readonly asSpecific: Vec<Bytes>;
    readonly isWildcard: boolean;
    readonly asWildcard: {
      readonly limit: Option<u32>;
      readonly previousCursor: Option<Bytes>;
    } & Struct;
    readonly type: "Specific" | "Wildcard";
  }

  /** @name SpRuntimeBlakeTwo256 (408) */
  type SpRuntimeBlakeTwo256 = Null;

  /** @name PalletConvictionVotingTally (410) */
  interface PalletConvictionVotingTally extends Struct {
    readonly ayes: u128;
    readonly nays: u128;
    readonly support: u128;
  }

  /** @name PalletPreimageEvent (411) */
  interface PalletPreimageEvent extends Enum {
    readonly isNoted: boolean;
    readonly asNoted: {
      readonly hash_: H256;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly hash_: H256;
    } & Struct;
    readonly isCleared: boolean;
    readonly asCleared: {
      readonly hash_: H256;
    } & Struct;
    readonly type: "Noted" | "Requested" | "Cleared";
  }

  /** @name PalletWhitelistEvent (412) */
  interface PalletWhitelistEvent extends Enum {
    readonly isCallWhitelisted: boolean;
    readonly asCallWhitelisted: {
      readonly callHash: H256;
    } & Struct;
    readonly isWhitelistedCallRemoved: boolean;
    readonly asWhitelistedCallRemoved: {
      readonly callHash: H256;
    } & Struct;
    readonly isWhitelistedCallDispatched: boolean;
    readonly asWhitelistedCallDispatched: {
      readonly callHash: H256;
      readonly result: Result<
        FrameSupportDispatchPostDispatchInfo,
        SpRuntimeDispatchErrorWithPostInfo
      >;
    } & Struct;
    readonly type: "CallWhitelisted" | "WhitelistedCallRemoved" | "WhitelistedCallDispatched";
  }

  /** @name FrameSupportDispatchPostDispatchInfo (414) */
  interface FrameSupportDispatchPostDispatchInfo extends Struct {
    readonly actualWeight: Option<SpWeightsWeightV2Weight>;
    readonly paysFee: FrameSupportDispatchPays;
  }

  /** @name SpRuntimeDispatchErrorWithPostInfo (415) */
  interface SpRuntimeDispatchErrorWithPostInfo extends Struct {
    readonly postInfo: FrameSupportDispatchPostDispatchInfo;
    readonly error: SpRuntimeDispatchError;
  }

  /** @name PalletRootTestingEvent (417) */
  interface PalletRootTestingEvent extends Enum {
    readonly isDefensiveTestCall: boolean;
    readonly type: "DefensiveTestCall";
  }

  /** @name PalletMultisigEvent (418) */
  interface PalletMultisigEvent extends Enum {
    readonly isNewMultisig: boolean;
    readonly asNewMultisig: {
      readonly approving: AccountId20;
      readonly multisig: AccountId20;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly isMultisigApproval: boolean;
    readonly asMultisigApproval: {
      readonly approving: AccountId20;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId20;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly isMultisigExecuted: boolean;
    readonly asMultisigExecuted: {
      readonly approving: AccountId20;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId20;
      readonly callHash: U8aFixed;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isMultisigCancelled: boolean;
    readonly asMultisigCancelled: {
      readonly cancelling: AccountId20;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId20;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly type: "NewMultisig" | "MultisigApproval" | "MultisigExecuted" | "MultisigCancelled";
  }

  /** @name PalletMessageQueueEvent (419) */
  interface PalletMessageQueueEvent extends Enum {
    readonly isProcessingFailed: boolean;
    readonly asProcessingFailed: {
      readonly id: H256;
      readonly origin: CumulusPrimitivesCoreAggregateMessageOrigin;
      readonly error: FrameSupportMessagesProcessMessageError;
    } & Struct;
    readonly isProcessed: boolean;
    readonly asProcessed: {
      readonly id: H256;
      readonly origin: CumulusPrimitivesCoreAggregateMessageOrigin;
      readonly weightUsed: SpWeightsWeightV2Weight;
      readonly success: bool;
    } & Struct;
    readonly isOverweightEnqueued: boolean;
    readonly asOverweightEnqueued: {
      readonly id: U8aFixed;
      readonly origin: CumulusPrimitivesCoreAggregateMessageOrigin;
      readonly pageIndex: u32;
      readonly messageIndex: u32;
    } & Struct;
    readonly isPageReaped: boolean;
    readonly asPageReaped: {
      readonly origin: CumulusPrimitivesCoreAggregateMessageOrigin;
      readonly index: u32;
    } & Struct;
    readonly type: "ProcessingFailed" | "Processed" | "OverweightEnqueued" | "PageReaped";
  }

  /** @name FrameSupportMessagesProcessMessageError (420) */
  interface FrameSupportMessagesProcessMessageError extends Enum {
    readonly isBadFormat: boolean;
    readonly isCorrupt: boolean;
    readonly isUnsupported: boolean;
    readonly isOverweight: boolean;
    readonly asOverweight: SpWeightsWeightV2Weight;
    readonly isYield: boolean;
    readonly isStackLimitReached: boolean;
    readonly type:
      | "BadFormat"
      | "Corrupt"
      | "Unsupported"
      | "Overweight"
      | "Yield"
      | "StackLimitReached";
  }

  /** @name PalletEmergencyParaXcmEvent (421) */
  interface PalletEmergencyParaXcmEvent extends Enum {
    readonly isEnteredPausedXcmMode: boolean;
    readonly isNormalXcmOperationResumed: boolean;
    readonly type: "EnteredPausedXcmMode" | "NormalXcmOperationResumed";
  }

  /** @name PalletMoonbeamForeignAssetsEvent (422) */
  interface PalletMoonbeamForeignAssetsEvent extends Enum {
    readonly isForeignAssetCreated: boolean;
    readonly asForeignAssetCreated: {
      readonly contractAddress: H160;
      readonly assetId: u128;
      readonly xcmLocation: StagingXcmV5Location;
      readonly deposit: Option<u128>;
    } & Struct;
    readonly isForeignAssetXcmLocationChanged: boolean;
    readonly asForeignAssetXcmLocationChanged: {
      readonly assetId: u128;
      readonly previousXcmLocation: StagingXcmV5Location;
      readonly newXcmLocation: StagingXcmV5Location;
    } & Struct;
    readonly isForeignAssetFrozen: boolean;
    readonly asForeignAssetFrozen: {
      readonly assetId: u128;
      readonly xcmLocation: StagingXcmV5Location;
    } & Struct;
    readonly isForeignAssetUnfrozen: boolean;
    readonly asForeignAssetUnfrozen: {
      readonly assetId: u128;
      readonly xcmLocation: StagingXcmV5Location;
    } & Struct;
    readonly isTokensLocked: boolean;
    readonly asTokensLocked: ITuple<[AccountId20, u128, U256]>;
    readonly type:
      | "ForeignAssetCreated"
      | "ForeignAssetXcmLocationChanged"
      | "ForeignAssetFrozen"
      | "ForeignAssetUnfrozen"
      | "TokensLocked";
  }

  /** @name PalletParametersEvent (423) */
  interface PalletParametersEvent extends Enum {
    readonly isUpdated: boolean;
    readonly asUpdated: {
      readonly key: MoonbaseRuntimeRuntimeParamsRuntimeParametersKey;
      readonly oldValue: Option<MoonbaseRuntimeRuntimeParamsRuntimeParametersValue>;
      readonly newValue: Option<MoonbaseRuntimeRuntimeParamsRuntimeParametersValue>;
    } & Struct;
    readonly type: "Updated";
  }

  /** @name MoonbaseRuntimeRuntimeParamsRuntimeParametersKey (424) */
  interface MoonbaseRuntimeRuntimeParamsRuntimeParametersKey extends Enum {
    readonly isRuntimeConfig: boolean;
    readonly asRuntimeConfig: MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersKey;
    readonly isPalletRandomness: boolean;
    readonly asPalletRandomness: MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersKey;
    readonly isXcmConfig: boolean;
    readonly asXcmConfig: MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParametersKey;
    readonly type: "RuntimeConfig" | "PalletRandomness" | "XcmConfig";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersKey (425) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersKey extends Enum {
    readonly isFeesTreasuryProportion: boolean;
    readonly type: "FeesTreasuryProportion";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersKey (426) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersKey extends Enum {
    readonly isDeposit: boolean;
    readonly type: "Deposit";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParametersKey (427) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParametersKey extends Enum {
    readonly isForeignAssetCreationDeposit: boolean;
    readonly type: "ForeignAssetCreationDeposit";
  }

  /** @name MoonbaseRuntimeRuntimeParamsRuntimeParametersValue (429) */
  interface MoonbaseRuntimeRuntimeParamsRuntimeParametersValue extends Enum {
    readonly isRuntimeConfig: boolean;
    readonly asRuntimeConfig: MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersValue;
    readonly isPalletRandomness: boolean;
    readonly asPalletRandomness: MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersValue;
    readonly isXcmConfig: boolean;
    readonly asXcmConfig: MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParametersValue;
    readonly type: "RuntimeConfig" | "PalletRandomness" | "XcmConfig";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersValue (430) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersValue extends Enum {
    readonly isFeesTreasuryProportion: boolean;
    readonly asFeesTreasuryProportion: Perbill;
    readonly type: "FeesTreasuryProportion";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersValue (431) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersValue extends Enum {
    readonly isDeposit: boolean;
    readonly asDeposit: u128;
    readonly type: "Deposit";
  }

  /** @name MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParametersValue (432) */
  interface MoonbaseRuntimeRuntimeParamsDynamicParamsXcmConfigParametersValue extends Enum {
    readonly isForeignAssetCreationDeposit: boolean;
    readonly asForeignAssetCreationDeposit: u128;
    readonly type: "ForeignAssetCreationDeposit";
  }

  /** @name PalletXcmWeightTraderEvent (433) */
  interface PalletXcmWeightTraderEvent extends Enum {
    readonly isSupportedAssetAdded: boolean;
    readonly asSupportedAssetAdded: {
      readonly location: StagingXcmV5Location;
      readonly relativePrice: u128;
    } & Struct;
    readonly isSupportedAssetEdited: boolean;
    readonly asSupportedAssetEdited: {
      readonly location: StagingXcmV5Location;
      readonly relativePrice: u128;
    } & Struct;
    readonly isPauseAssetSupport: boolean;
    readonly asPauseAssetSupport: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly isResumeAssetSupport: boolean;
    readonly asResumeAssetSupport: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly isSupportedAssetRemoved: boolean;
    readonly asSupportedAssetRemoved: {
      readonly location: StagingXcmV5Location;
    } & Struct;
    readonly type:
      | "SupportedAssetAdded"
      | "SupportedAssetEdited"
      | "PauseAssetSupport"
      | "ResumeAssetSupport"
      | "SupportedAssetRemoved";
  }

  /** @name PalletMigrationsEvent (434) */
  interface PalletMigrationsEvent extends Enum {
    readonly isUpgradeStarted: boolean;
    readonly asUpgradeStarted: {
      readonly migrations: u32;
    } & Struct;
    readonly isUpgradeCompleted: boolean;
    readonly isUpgradeFailed: boolean;
    readonly isMigrationSkipped: boolean;
    readonly asMigrationSkipped: {
      readonly index: u32;
    } & Struct;
    readonly isMigrationAdvanced: boolean;
    readonly asMigrationAdvanced: {
      readonly index: u32;
      readonly took: u32;
    } & Struct;
    readonly isMigrationCompleted: boolean;
    readonly asMigrationCompleted: {
      readonly index: u32;
      readonly took: u32;
    } & Struct;
    readonly isMigrationFailed: boolean;
    readonly asMigrationFailed: {
      readonly index: u32;
      readonly took: u32;
    } & Struct;
    readonly isHistoricCleared: boolean;
    readonly asHistoricCleared: {
      readonly nextCursor: Option<Bytes>;
    } & Struct;
    readonly type:
      | "UpgradeStarted"
      | "UpgradeCompleted"
      | "UpgradeFailed"
      | "MigrationSkipped"
      | "MigrationAdvanced"
      | "MigrationCompleted"
      | "MigrationFailed"
      | "HistoricCleared";
  }

  /** @name FrameSystemPhase (435) */
  interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean;
    readonly asApplyExtrinsic: u32;
    readonly isFinalization: boolean;
    readonly isInitialization: boolean;
    readonly type: "ApplyExtrinsic" | "Finalization" | "Initialization";
  }

  /** @name FrameSystemLastRuntimeUpgradeInfo (437) */
  interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>;
    readonly specName: Text;
  }

  /** @name FrameSystemCodeUpgradeAuthorization (438) */
  interface FrameSystemCodeUpgradeAuthorization extends Struct {
    readonly codeHash: H256;
    readonly checkVersion: bool;
  }

  /** @name FrameSystemLimitsBlockWeights (439) */
  interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: SpWeightsWeightV2Weight;
    readonly maxBlock: SpWeightsWeightV2Weight;
    readonly perClass: FrameSupportDispatchPerDispatchClassWeightsPerClass;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeightsPerClass (440) */
  interface FrameSupportDispatchPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass;
    readonly operational: FrameSystemLimitsWeightsPerClass;
    readonly mandatory: FrameSystemLimitsWeightsPerClass;
  }

  /** @name FrameSystemLimitsWeightsPerClass (441) */
  interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: SpWeightsWeightV2Weight;
    readonly maxExtrinsic: Option<SpWeightsWeightV2Weight>;
    readonly maxTotal: Option<SpWeightsWeightV2Weight>;
    readonly reserved: Option<SpWeightsWeightV2Weight>;
  }

  /** @name FrameSystemLimitsBlockLength (442) */
  interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportDispatchPerDispatchClassU32;
  }

  /** @name FrameSupportDispatchPerDispatchClassU32 (443) */
  interface FrameSupportDispatchPerDispatchClassU32 extends Struct {
    readonly normal: u32;
    readonly operational: u32;
    readonly mandatory: u32;
  }

  /** @name SpWeightsRuntimeDbWeight (444) */
  interface SpWeightsRuntimeDbWeight extends Struct {
    readonly read: u64;
    readonly write: u64;
  }

  /** @name SpVersionRuntimeVersion (445) */
  interface SpVersionRuntimeVersion extends Struct {
    readonly specName: Text;
    readonly implName: Text;
    readonly authoringVersion: u32;
    readonly specVersion: u32;
    readonly implVersion: u32;
    readonly apis: Vec<ITuple<[U8aFixed, u32]>>;
    readonly transactionVersion: u32;
    readonly systemVersion: u8;
  }

  /** @name FrameSystemError (449) */
  interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean;
    readonly isSpecVersionNeedsToIncrease: boolean;
    readonly isFailedToExtractRuntimeVersion: boolean;
    readonly isNonDefaultComposite: boolean;
    readonly isNonZeroRefCount: boolean;
    readonly isCallFiltered: boolean;
    readonly isMultiBlockMigrationsOngoing: boolean;
    readonly isNothingAuthorized: boolean;
    readonly isUnauthorized: boolean;
    readonly type:
      | "InvalidSpecName"
      | "SpecVersionNeedsToIncrease"
      | "FailedToExtractRuntimeVersion"
      | "NonDefaultComposite"
      | "NonZeroRefCount"
      | "CallFiltered"
      | "MultiBlockMigrationsOngoing"
      | "NothingAuthorized"
      | "Unauthorized";
  }

  /** @name PalletUtilityError (450) */
  interface PalletUtilityError extends Enum {
    readonly isTooManyCalls: boolean;
    readonly type: "TooManyCalls";
  }

  /** @name PalletBalancesBalanceLock (452) */
  interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
    readonly reasons: PalletBalancesReasons;
  }

  /** @name PalletBalancesReasons (453) */
  interface PalletBalancesReasons extends Enum {
    readonly isFee: boolean;
    readonly isMisc: boolean;
    readonly isAll: boolean;
    readonly type: "Fee" | "Misc" | "All";
  }

  /** @name PalletBalancesReserveData (456) */
  interface PalletBalancesReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name MoonbaseRuntimeRuntimeHoldReason (460) */
  interface MoonbaseRuntimeRuntimeHoldReason extends Enum {
    readonly isPreimage: boolean;
    readonly asPreimage: PalletPreimageHoldReason;
    readonly isBridgeXcmOver: boolean;
    readonly asBridgeXcmOver: PalletXcmBridgeHubHoldReason;
    readonly type: "Preimage" | "BridgeXcmOver";
  }

  /** @name PalletPreimageHoldReason (461) */
  interface PalletPreimageHoldReason extends Enum {
    readonly isPreimage: boolean;
    readonly type: "Preimage";
  }

  /** @name PalletXcmBridgeHubHoldReason (462) */
  interface PalletXcmBridgeHubHoldReason extends Enum {
    readonly isBridgeDeposit: boolean;
    readonly type: "BridgeDeposit";
  }

  /** @name FrameSupportTokensMiscIdAmount (465) */
  interface FrameSupportTokensMiscIdAmount extends Struct {
    readonly id: Null;
    readonly amount: u128;
  }

  /** @name PalletBalancesError (467) */
  interface PalletBalancesError extends Enum {
    readonly isVestingBalance: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isExpendability: boolean;
    readonly isExistingVestingSchedule: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly isTooManyHolds: boolean;
    readonly isTooManyFreezes: boolean;
    readonly isIssuanceDeactivated: boolean;
    readonly isDeltaZero: boolean;
    readonly type:
      | "VestingBalance"
      | "LiquidityRestrictions"
      | "InsufficientBalance"
      | "ExistentialDeposit"
      | "Expendability"
      | "ExistingVestingSchedule"
      | "DeadAccount"
      | "TooManyReserves"
      | "TooManyHolds"
      | "TooManyFreezes"
      | "IssuanceDeactivated"
      | "DeltaZero";
  }

  /** @name PalletSudoError (468) */
  interface PalletSudoError extends Enum {
    readonly isRequireSudo: boolean;
    readonly type: "RequireSudo";
  }

  /** @name CumulusPalletParachainSystemUnincludedSegmentAncestor (470) */
  interface CumulusPalletParachainSystemUnincludedSegmentAncestor extends Struct {
    readonly usedBandwidth: CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth;
    readonly paraHeadHash: Option<H256>;
    readonly consumedGoAheadSignal: Option<PolkadotPrimitivesV8UpgradeGoAhead>;
  }

  /** @name CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth (471) */
  interface CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth extends Struct {
    readonly umpMsgCount: u32;
    readonly umpTotalBytes: u32;
    readonly hrmpOutgoing: BTreeMap<
      u32,
      CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate
    >;
  }

  /** @name CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate (473) */
  interface CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate extends Struct {
    readonly msgCount: u32;
    readonly totalBytes: u32;
  }

  /** @name PolkadotPrimitivesV8UpgradeGoAhead (477) */
  interface PolkadotPrimitivesV8UpgradeGoAhead extends Enum {
    readonly isAbort: boolean;
    readonly isGoAhead: boolean;
    readonly type: "Abort" | "GoAhead";
  }

  /** @name CumulusPalletParachainSystemUnincludedSegmentSegmentTracker (478) */
  interface CumulusPalletParachainSystemUnincludedSegmentSegmentTracker extends Struct {
    readonly usedBandwidth: CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth;
    readonly hrmpWatermark: Option<u32>;
    readonly consumedGoAheadSignal: Option<PolkadotPrimitivesV8UpgradeGoAhead>;
  }

  /** @name PolkadotPrimitivesV8UpgradeRestriction (480) */
  interface PolkadotPrimitivesV8UpgradeRestriction extends Enum {
    readonly isPresent: boolean;
    readonly type: "Present";
  }

  /** @name CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot (481) */
  interface CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot extends Struct {
    readonly dmqMqcHead: H256;
    readonly relayDispatchQueueRemainingCapacity: CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity;
    readonly ingressChannels: Vec<ITuple<[u32, PolkadotPrimitivesV8AbridgedHrmpChannel]>>;
    readonly egressChannels: Vec<ITuple<[u32, PolkadotPrimitivesV8AbridgedHrmpChannel]>>;
  }

  /** @name CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity (482) */
  interface CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity
    extends Struct {
    readonly remainingCount: u32;
    readonly remainingSize: u32;
  }

  /** @name PolkadotPrimitivesV8AbridgedHrmpChannel (485) */
  interface PolkadotPrimitivesV8AbridgedHrmpChannel extends Struct {
    readonly maxCapacity: u32;
    readonly maxTotalSize: u32;
    readonly maxMessageSize: u32;
    readonly msgCount: u32;
    readonly totalSize: u32;
    readonly mqcHead: Option<H256>;
  }

  /** @name PolkadotPrimitivesV8AbridgedHostConfiguration (486) */
  interface PolkadotPrimitivesV8AbridgedHostConfiguration extends Struct {
    readonly maxCodeSize: u32;
    readonly maxHeadDataSize: u32;
    readonly maxUpwardQueueCount: u32;
    readonly maxUpwardQueueSize: u32;
    readonly maxUpwardMessageSize: u32;
    readonly maxUpwardMessageNumPerCandidate: u32;
    readonly hrmpMaxMessageNumPerCandidate: u32;
    readonly validationUpgradeCooldown: u32;
    readonly validationUpgradeDelay: u32;
    readonly asyncBackingParams: PolkadotPrimitivesV8AsyncBackingAsyncBackingParams;
  }

  /** @name PolkadotPrimitivesV8AsyncBackingAsyncBackingParams (487) */
  interface PolkadotPrimitivesV8AsyncBackingAsyncBackingParams extends Struct {
    readonly maxCandidateDepth: u32;
    readonly allowedAncestryLen: u32;
  }

  /** @name PolkadotCorePrimitivesOutboundHrmpMessage (493) */
  interface PolkadotCorePrimitivesOutboundHrmpMessage extends Struct {
    readonly recipient: u32;
    readonly data: Bytes;
  }

  /** @name CumulusPalletParachainSystemError (495) */
  interface CumulusPalletParachainSystemError extends Enum {
    readonly isOverlappingUpgrades: boolean;
    readonly isProhibitedByPolkadot: boolean;
    readonly isTooBig: boolean;
    readonly isValidationDataNotAvailable: boolean;
    readonly isHostConfigurationNotAvailable: boolean;
    readonly isNotScheduled: boolean;
    readonly isNothingAuthorized: boolean;
    readonly isUnauthorized: boolean;
    readonly type:
      | "OverlappingUpgrades"
      | "ProhibitedByPolkadot"
      | "TooBig"
      | "ValidationDataNotAvailable"
      | "HostConfigurationNotAvailable"
      | "NotScheduled"
      | "NothingAuthorized"
      | "Unauthorized";
  }

  /** @name PalletTransactionPaymentReleases (496) */
  interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean;
    readonly isV2: boolean;
    readonly type: "V1Ancient" | "V2";
  }

  /** @name PalletEvmCodeMetadata (497) */
  interface PalletEvmCodeMetadata extends Struct {
    readonly size_: u64;
    readonly hash_: H256;
  }

  /** @name PalletEvmError (499) */
  interface PalletEvmError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isFeeOverflow: boolean;
    readonly isPaymentOverflow: boolean;
    readonly isWithdrawFailed: boolean;
    readonly isGasPriceTooLow: boolean;
    readonly isInvalidNonce: boolean;
    readonly isGasLimitTooLow: boolean;
    readonly isGasLimitTooHigh: boolean;
    readonly isInvalidChainId: boolean;
    readonly isInvalidSignature: boolean;
    readonly isReentrancy: boolean;
    readonly isTransactionMustComeFromEOA: boolean;
    readonly isUndefined: boolean;
    readonly type:
      | "BalanceLow"
      | "FeeOverflow"
      | "PaymentOverflow"
      | "WithdrawFailed"
      | "GasPriceTooLow"
      | "InvalidNonce"
      | "GasLimitTooLow"
      | "GasLimitTooHigh"
      | "InvalidChainId"
      | "InvalidSignature"
      | "Reentrancy"
      | "TransactionMustComeFromEOA"
      | "Undefined";
  }

  /** @name FpRpcTransactionStatus (501) */
  interface FpRpcTransactionStatus extends Struct {
    readonly transactionHash: H256;
    readonly transactionIndex: u32;
    readonly from: H160;
    readonly to: Option<H160>;
    readonly contractAddress: Option<H160>;
    readonly logs: Vec<EthereumLog>;
    readonly logsBloom: EthbloomBloom;
  }

  /** @name EthbloomBloom (503) */
  interface EthbloomBloom extends U8aFixed {}

  /** @name EthereumReceiptReceiptV3 (505) */
  interface EthereumReceiptReceiptV3 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumReceiptEip658ReceiptData;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumReceiptEip658ReceiptData;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumReceiptEip658ReceiptData;
    readonly type: "Legacy" | "Eip2930" | "Eip1559";
  }

  /** @name EthereumReceiptEip658ReceiptData (506) */
  interface EthereumReceiptEip658ReceiptData extends Struct {
    readonly statusCode: u8;
    readonly usedGas: U256;
    readonly logsBloom: EthbloomBloom;
    readonly logs: Vec<EthereumLog>;
  }

  /** @name EthereumBlock (507) */
  interface EthereumBlock extends Struct {
    readonly header: EthereumHeader;
    readonly transactions: Vec<EthereumTransactionTransactionV2>;
    readonly ommers: Vec<EthereumHeader>;
  }

  /** @name EthereumHeader (508) */
  interface EthereumHeader extends Struct {
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

  /** @name EthereumTypesHashH64 (509) */
  interface EthereumTypesHashH64 extends U8aFixed {}

  /** @name PalletEthereumError (514) */
  interface PalletEthereumError extends Enum {
    readonly isInvalidSignature: boolean;
    readonly isPreLogExists: boolean;
    readonly type: "InvalidSignature" | "PreLogExists";
  }

  /** @name PalletParachainStakingRoundInfo (515) */
  interface PalletParachainStakingRoundInfo extends Struct {
    readonly current: u32;
    readonly first: u32;
    readonly length: u32;
    readonly firstSlot: u64;
  }

  /** @name PalletParachainStakingDelegator (516) */
  interface PalletParachainStakingDelegator extends Struct {
    readonly id: AccountId20;
    readonly delegations: PalletParachainStakingSetOrderedSet;
    readonly total: u128;
    readonly lessTotal: u128;
    readonly status: PalletParachainStakingDelegatorStatus;
  }

  /** @name PalletParachainStakingSetOrderedSet (517) */
  interface PalletParachainStakingSetOrderedSet extends Vec<PalletParachainStakingBond> {}

  /** @name PalletParachainStakingBond (518) */
  interface PalletParachainStakingBond extends Struct {
    readonly owner: AccountId20;
    readonly amount: u128;
  }

  /** @name PalletParachainStakingDelegatorStatus (520) */
  interface PalletParachainStakingDelegatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isLeaving: boolean;
    readonly asLeaving: u32;
    readonly type: "Active" | "Leaving";
  }

  /** @name PalletParachainStakingCandidateMetadata (521) */
  interface PalletParachainStakingCandidateMetadata extends Struct {
    readonly bond: u128;
    readonly delegationCount: u32;
    readonly totalCounted: u128;
    readonly lowestTopDelegationAmount: u128;
    readonly highestBottomDelegationAmount: u128;
    readonly lowestBottomDelegationAmount: u128;
    readonly topCapacity: PalletParachainStakingCapacityStatus;
    readonly bottomCapacity: PalletParachainStakingCapacityStatus;
    readonly request: Option<PalletParachainStakingCandidateBondLessRequest>;
    readonly status: PalletParachainStakingCollatorStatus;
  }

  /** @name PalletParachainStakingCapacityStatus (522) */
  interface PalletParachainStakingCapacityStatus extends Enum {
    readonly isFull: boolean;
    readonly isEmpty: boolean;
    readonly isPartial: boolean;
    readonly type: "Full" | "Empty" | "Partial";
  }

  /** @name PalletParachainStakingCandidateBondLessRequest (524) */
  interface PalletParachainStakingCandidateBondLessRequest extends Struct {
    readonly amount: u128;
    readonly whenExecutable: u32;
  }

  /** @name PalletParachainStakingCollatorStatus (525) */
  interface PalletParachainStakingCollatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isIdle: boolean;
    readonly isLeaving: boolean;
    readonly asLeaving: u32;
    readonly type: "Active" | "Idle" | "Leaving";
  }

  /** @name PalletParachainStakingDelegationRequestsScheduledRequest (527) */
  interface PalletParachainStakingDelegationRequestsScheduledRequest extends Struct {
    readonly delegator: AccountId20;
    readonly whenExecutable: u32;
    readonly action: PalletParachainStakingDelegationRequestsDelegationAction;
  }

  /** @name PalletParachainStakingAutoCompoundAutoCompoundConfig (530) */
  interface PalletParachainStakingAutoCompoundAutoCompoundConfig extends Struct {
    readonly delegator: AccountId20;
    readonly value: Percent;
  }

  /** @name PalletParachainStakingDelegations (532) */
  interface PalletParachainStakingDelegations extends Struct {
    readonly delegations: Vec<PalletParachainStakingBond>;
    readonly total: u128;
  }

  /** @name PalletParachainStakingSetBoundedOrderedSet (534) */
  interface PalletParachainStakingSetBoundedOrderedSet extends Vec<PalletParachainStakingBond> {}

  /** @name PalletParachainStakingCollatorSnapshot (537) */
  interface PalletParachainStakingCollatorSnapshot extends Struct {
    readonly bond: u128;
    readonly delegations: Vec<PalletParachainStakingBondWithAutoCompound>;
    readonly total: u128;
  }

  /** @name PalletParachainStakingBondWithAutoCompound (539) */
  interface PalletParachainStakingBondWithAutoCompound extends Struct {
    readonly owner: AccountId20;
    readonly amount: u128;
    readonly autoCompound: Percent;
  }

  /** @name PalletParachainStakingDelayedPayout (540) */
  interface PalletParachainStakingDelayedPayout extends Struct {
    readonly roundIssuance: u128;
    readonly totalStakingReward: u128;
    readonly collatorCommission: Perbill;
  }

  /** @name PalletParachainStakingInflationInflationInfo (541) */
  interface PalletParachainStakingInflationInflationInfo extends Struct {
    readonly expect: {
      readonly min: u128;
      readonly ideal: u128;
      readonly max: u128;
    } & Struct;
    readonly annual: {
      readonly min: Perbill;
      readonly ideal: Perbill;
      readonly max: Perbill;
    } & Struct;
    readonly round: {
      readonly min: Perbill;
      readonly ideal: Perbill;
      readonly max: Perbill;
    } & Struct;
  }

  /** @name PalletParachainStakingError (542) */
  interface PalletParachainStakingError extends Enum {
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
    readonly isRoundLengthMustBeGreaterThanTotalSelectedCollators: boolean;
    readonly isNoWritingSameValue: boolean;
    readonly isTotalInflationDistributionPercentExceeds100: boolean;
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
    readonly isCannotDelegateLessThanOrEqualToLowestBottomWhenFull: boolean;
    readonly isPendingDelegationRevoke: boolean;
    readonly isTooLowDelegationCountToAutoCompound: boolean;
    readonly isTooLowCandidateAutoCompoundingDelegationCountToAutoCompound: boolean;
    readonly isTooLowCandidateAutoCompoundingDelegationCountToDelegate: boolean;
    readonly isTooLowCollatorCountToNotifyAsInactive: boolean;
    readonly isCannotBeNotifiedAsInactive: boolean;
    readonly isTooLowCandidateAutoCompoundingDelegationCountToLeaveCandidates: boolean;
    readonly isTooLowCandidateCountWeightHint: boolean;
    readonly isTooLowCandidateCountWeightHintGoOffline: boolean;
    readonly isCandidateLimitReached: boolean;
    readonly isCannotSetAboveMaxCandidates: boolean;
    readonly isMarkingOfflineNotEnabled: boolean;
    readonly isCurrentRoundTooLow: boolean;
    readonly type:
      | "DelegatorDNE"
      | "DelegatorDNEinTopNorBottom"
      | "DelegatorDNEInDelegatorSet"
      | "CandidateDNE"
      | "DelegationDNE"
      | "DelegatorExists"
      | "CandidateExists"
      | "CandidateBondBelowMin"
      | "InsufficientBalance"
      | "DelegatorBondBelowMin"
      | "DelegationBelowMin"
      | "AlreadyOffline"
      | "AlreadyActive"
      | "DelegatorAlreadyLeaving"
      | "DelegatorNotLeaving"
      | "DelegatorCannotLeaveYet"
      | "CannotDelegateIfLeaving"
      | "CandidateAlreadyLeaving"
      | "CandidateNotLeaving"
      | "CandidateCannotLeaveYet"
      | "CannotGoOnlineIfLeaving"
      | "ExceedMaxDelegationsPerDelegator"
      | "AlreadyDelegatedCandidate"
      | "InvalidSchedule"
      | "CannotSetBelowMin"
      | "RoundLengthMustBeGreaterThanTotalSelectedCollators"
      | "NoWritingSameValue"
      | "TotalInflationDistributionPercentExceeds100"
      | "TooLowCandidateCountWeightHintJoinCandidates"
      | "TooLowCandidateCountWeightHintCancelLeaveCandidates"
      | "TooLowCandidateCountToLeaveCandidates"
      | "TooLowDelegationCountToDelegate"
      | "TooLowCandidateDelegationCountToDelegate"
      | "TooLowCandidateDelegationCountToLeaveCandidates"
      | "TooLowDelegationCountToLeaveDelegators"
      | "PendingCandidateRequestsDNE"
      | "PendingCandidateRequestAlreadyExists"
      | "PendingCandidateRequestNotDueYet"
      | "PendingDelegationRequestDNE"
      | "PendingDelegationRequestAlreadyExists"
      | "PendingDelegationRequestNotDueYet"
      | "CannotDelegateLessThanOrEqualToLowestBottomWhenFull"
      | "PendingDelegationRevoke"
      | "TooLowDelegationCountToAutoCompound"
      | "TooLowCandidateAutoCompoundingDelegationCountToAutoCompound"
      | "TooLowCandidateAutoCompoundingDelegationCountToDelegate"
      | "TooLowCollatorCountToNotifyAsInactive"
      | "CannotBeNotifiedAsInactive"
      | "TooLowCandidateAutoCompoundingDelegationCountToLeaveCandidates"
      | "TooLowCandidateCountWeightHint"
      | "TooLowCandidateCountWeightHintGoOffline"
      | "CandidateLimitReached"
      | "CannotSetAboveMaxCandidates"
      | "MarkingOfflineNotEnabled"
      | "CurrentRoundTooLow";
  }

  /** @name PalletSchedulerScheduled (545) */
  interface PalletSchedulerScheduled extends Struct {
    readonly maybeId: Option<U8aFixed>;
    readonly priority: u8;
    readonly call: FrameSupportPreimagesBounded;
    readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
    readonly origin: MoonbaseRuntimeOriginCaller;
  }

  /** @name PalletSchedulerRetryConfig (547) */
  interface PalletSchedulerRetryConfig extends Struct {
    readonly totalRetries: u8;
    readonly remaining: u8;
    readonly period: u32;
  }

  /** @name PalletSchedulerError (548) */
  interface PalletSchedulerError extends Enum {
    readonly isFailedToSchedule: boolean;
    readonly isNotFound: boolean;
    readonly isTargetBlockNumberInPast: boolean;
    readonly isRescheduleNoChange: boolean;
    readonly isNamed: boolean;
    readonly type:
      | "FailedToSchedule"
      | "NotFound"
      | "TargetBlockNumberInPast"
      | "RescheduleNoChange"
      | "Named";
  }

  /** @name PalletTreasuryProposal (549) */
  interface PalletTreasuryProposal extends Struct {
    readonly proposer: AccountId20;
    readonly value: u128;
    readonly beneficiary: AccountId20;
    readonly bond: u128;
  }

  /** @name PalletTreasurySpendStatus (552) */
  interface PalletTreasurySpendStatus extends Struct {
    readonly assetKind: FrameSupportTokensFungibleUnionOfNativeOrWithId;
    readonly amount: u128;
    readonly beneficiary: AccountId20;
    readonly validFrom: u32;
    readonly expireAt: u32;
    readonly status: PalletTreasuryPaymentState;
  }

  /** @name PalletTreasuryPaymentState (553) */
  interface PalletTreasuryPaymentState extends Enum {
    readonly isPending: boolean;
    readonly isAttempted: boolean;
    readonly asAttempted: {
      readonly id: Null;
    } & Struct;
    readonly isFailed: boolean;
    readonly type: "Pending" | "Attempted" | "Failed";
  }

  /** @name FrameSupportPalletId (555) */
  interface FrameSupportPalletId extends U8aFixed {}

  /** @name PalletTreasuryError (556) */
  interface PalletTreasuryError extends Enum {
    readonly isInvalidIndex: boolean;
    readonly isTooManyApprovals: boolean;
    readonly isInsufficientPermission: boolean;
    readonly isProposalNotApproved: boolean;
    readonly isFailedToConvertBalance: boolean;
    readonly isSpendExpired: boolean;
    readonly isEarlyPayout: boolean;
    readonly isAlreadyAttempted: boolean;
    readonly isPayoutError: boolean;
    readonly isNotAttempted: boolean;
    readonly isInconclusive: boolean;
    readonly type:
      | "InvalidIndex"
      | "TooManyApprovals"
      | "InsufficientPermission"
      | "ProposalNotApproved"
      | "FailedToConvertBalance"
      | "SpendExpired"
      | "EarlyPayout"
      | "AlreadyAttempted"
      | "PayoutError"
      | "NotAttempted"
      | "Inconclusive";
  }

  /** @name PalletAuthorInherentError (557) */
  interface PalletAuthorInherentError extends Enum {
    readonly isAuthorAlreadySet: boolean;
    readonly isNoAccountId: boolean;
    readonly isCannotBeAuthor: boolean;
    readonly type: "AuthorAlreadySet" | "NoAccountId" | "CannotBeAuthor";
  }

  /** @name PalletCrowdloanRewardsRewardInfo (558) */
  interface PalletCrowdloanRewardsRewardInfo extends Struct {
    readonly totalReward: u128;
    readonly claimedReward: u128;
    readonly contributedRelayAddresses: Vec<U8aFixed>;
  }

  /** @name PalletCrowdloanRewardsError (560) */
  interface PalletCrowdloanRewardsError extends Enum {
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
    readonly type:
      | "AlreadyAssociated"
      | "BatchBeyondFundPot"
      | "FirstClaimAlreadyDone"
      | "RewardNotHighEnough"
      | "InvalidClaimSignature"
      | "InvalidFreeClaimSignature"
      | "NoAssociatedClaim"
      | "RewardsAlreadyClaimed"
      | "RewardVecAlreadyInitialized"
      | "RewardVecNotFullyInitializedYet"
      | "RewardsDoNotMatchFund"
      | "TooManyContributors"
      | "VestingPeriodNonValid"
      | "NonContributedAddressProvided"
      | "InsufficientNumberOfValidProofs";
  }

  /** @name PalletAuthorMappingRegistrationInfo (561) */
  interface PalletAuthorMappingRegistrationInfo extends Struct {
    readonly account: AccountId20;
    readonly deposit: u128;
    readonly keys_: SessionKeysPrimitivesVrfVrfCryptoPublic;
  }

  /** @name PalletAuthorMappingError (562) */
  interface PalletAuthorMappingError extends Enum {
    readonly isAssociationNotFound: boolean;
    readonly isNotYourAssociation: boolean;
    readonly isCannotAffordSecurityDeposit: boolean;
    readonly isAlreadyAssociated: boolean;
    readonly isOldAuthorIdNotFound: boolean;
    readonly isWrongKeySize: boolean;
    readonly isDecodeNimbusFailed: boolean;
    readonly isDecodeKeysFailed: boolean;
    readonly type:
      | "AssociationNotFound"
      | "NotYourAssociation"
      | "CannotAffordSecurityDeposit"
      | "AlreadyAssociated"
      | "OldAuthorIdNotFound"
      | "WrongKeySize"
      | "DecodeNimbusFailed"
      | "DecodeKeysFailed";
  }

  /** @name PalletProxyProxyDefinition (565) */
  interface PalletProxyProxyDefinition extends Struct {
    readonly delegate: AccountId20;
    readonly proxyType: MoonbaseRuntimeProxyType;
    readonly delay: u32;
  }

  /** @name PalletProxyAnnouncement (569) */
  interface PalletProxyAnnouncement extends Struct {
    readonly real: AccountId20;
    readonly callHash: H256;
    readonly height: u32;
  }

  /** @name PalletProxyError (571) */
  interface PalletProxyError extends Enum {
    readonly isTooMany: boolean;
    readonly isNotFound: boolean;
    readonly isNotProxy: boolean;
    readonly isUnproxyable: boolean;
    readonly isDuplicate: boolean;
    readonly isNoPermission: boolean;
    readonly isUnannounced: boolean;
    readonly isNoSelfProxy: boolean;
    readonly type:
      | "TooMany"
      | "NotFound"
      | "NotProxy"
      | "Unproxyable"
      | "Duplicate"
      | "NoPermission"
      | "Unannounced"
      | "NoSelfProxy";
  }

  /** @name PalletMaintenanceModeError (572) */
  interface PalletMaintenanceModeError extends Enum {
    readonly isAlreadyInMaintenanceMode: boolean;
    readonly isNotInMaintenanceMode: boolean;
    readonly type: "AlreadyInMaintenanceMode" | "NotInMaintenanceMode";
  }

  /** @name PalletIdentityRegistration (573) */
  interface PalletIdentityRegistration extends Struct {
    readonly judgements: Vec<ITuple<[u32, PalletIdentityJudgement]>>;
    readonly deposit: u128;
    readonly info: PalletIdentityLegacyIdentityInfo;
  }

  /** @name PalletIdentityRegistrarInfo (581) */
  interface PalletIdentityRegistrarInfo extends Struct {
    readonly account: AccountId20;
    readonly fee: u128;
    readonly fields: u64;
  }

  /** @name PalletIdentityAuthorityProperties (584) */
  interface PalletIdentityAuthorityProperties extends Struct {
    readonly accountId: AccountId20;
    readonly allocation: u32;
  }

  /** @name PalletIdentityUsernameInformation (585) */
  interface PalletIdentityUsernameInformation extends Struct {
    readonly owner: AccountId20;
    readonly provider: PalletIdentityProvider;
  }

  /** @name PalletIdentityProvider (586) */
  interface PalletIdentityProvider extends Enum {
    readonly isAllocation: boolean;
    readonly isAuthorityDeposit: boolean;
    readonly asAuthorityDeposit: u128;
    readonly isSystem: boolean;
    readonly type: "Allocation" | "AuthorityDeposit" | "System";
  }

  /** @name PalletIdentityError (588) */
  interface PalletIdentityError extends Enum {
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
    readonly isTooManyRegistrars: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isNotSub: boolean;
    readonly isNotOwned: boolean;
    readonly isJudgementForDifferentIdentity: boolean;
    readonly isJudgementPaymentFailed: boolean;
    readonly isInvalidSuffix: boolean;
    readonly isNotUsernameAuthority: boolean;
    readonly isNoAllocation: boolean;
    readonly isInvalidSignature: boolean;
    readonly isRequiresSignature: boolean;
    readonly isInvalidUsername: boolean;
    readonly isUsernameTaken: boolean;
    readonly isNoUsername: boolean;
    readonly isNotExpired: boolean;
    readonly isTooEarly: boolean;
    readonly isNotUnbinding: boolean;
    readonly isAlreadyUnbinding: boolean;
    readonly isInsufficientPrivileges: boolean;
    readonly type:
      | "TooManySubAccounts"
      | "NotFound"
      | "NotNamed"
      | "EmptyIndex"
      | "FeeChanged"
      | "NoIdentity"
      | "StickyJudgement"
      | "JudgementGiven"
      | "InvalidJudgement"
      | "InvalidIndex"
      | "InvalidTarget"
      | "TooManyRegistrars"
      | "AlreadyClaimed"
      | "NotSub"
      | "NotOwned"
      | "JudgementForDifferentIdentity"
      | "JudgementPaymentFailed"
      | "InvalidSuffix"
      | "NotUsernameAuthority"
      | "NoAllocation"
      | "InvalidSignature"
      | "RequiresSignature"
      | "InvalidUsername"
      | "UsernameTaken"
      | "NoUsername"
      | "NotExpired"
      | "TooEarly"
      | "NotUnbinding"
      | "AlreadyUnbinding"
      | "InsufficientPrivileges";
  }

  /** @name CumulusPalletXcmpQueueOutboundChannelDetails (593) */
  interface CumulusPalletXcmpQueueOutboundChannelDetails extends Struct {
    readonly recipient: u32;
    readonly state: CumulusPalletXcmpQueueOutboundState;
    readonly signalsExist: bool;
    readonly firstIndex: u16;
    readonly lastIndex: u16;
  }

  /** @name CumulusPalletXcmpQueueOutboundState (594) */
  interface CumulusPalletXcmpQueueOutboundState extends Enum {
    readonly isOk: boolean;
    readonly isSuspended: boolean;
    readonly type: "Ok" | "Suspended";
  }

  /** @name CumulusPalletXcmpQueueQueueConfigData (598) */
  interface CumulusPalletXcmpQueueQueueConfigData extends Struct {
    readonly suspendThreshold: u32;
    readonly dropThreshold: u32;
    readonly resumeThreshold: u32;
  }

  /** @name CumulusPalletXcmpQueueError (599) */
  interface CumulusPalletXcmpQueueError extends Enum {
    readonly isBadQueueConfig: boolean;
    readonly isAlreadySuspended: boolean;
    readonly isAlreadyResumed: boolean;
    readonly isTooManyActiveOutboundChannels: boolean;
    readonly isTooBig: boolean;
    readonly type:
      | "BadQueueConfig"
      | "AlreadySuspended"
      | "AlreadyResumed"
      | "TooManyActiveOutboundChannels"
      | "TooBig";
  }

  /** @name PalletXcmQueryStatus (600) */
  interface PalletXcmQueryStatus extends Enum {
    readonly isPending: boolean;
    readonly asPending: {
      readonly responder: XcmVersionedLocation;
      readonly maybeMatchQuerier: Option<XcmVersionedLocation>;
      readonly maybeNotify: Option<ITuple<[u8, u8]>>;
      readonly timeout: u32;
    } & Struct;
    readonly isVersionNotifier: boolean;
    readonly asVersionNotifier: {
      readonly origin: XcmVersionedLocation;
      readonly isActive: bool;
    } & Struct;
    readonly isReady: boolean;
    readonly asReady: {
      readonly response: XcmVersionedResponse;
      readonly at: u32;
    } & Struct;
    readonly type: "Pending" | "VersionNotifier" | "Ready";
  }

  /** @name XcmVersionedResponse (604) */
  interface XcmVersionedResponse extends Enum {
    readonly isV3: boolean;
    readonly asV3: XcmV3Response;
    readonly isV4: boolean;
    readonly asV4: StagingXcmV4Response;
    readonly isV5: boolean;
    readonly asV5: StagingXcmV5Response;
    readonly type: "V3" | "V4" | "V5";
  }

  /** @name PalletXcmVersionMigrationStage (610) */
  interface PalletXcmVersionMigrationStage extends Enum {
    readonly isMigrateSupportedVersion: boolean;
    readonly isMigrateVersionNotifiers: boolean;
    readonly isNotifyCurrentTargets: boolean;
    readonly asNotifyCurrentTargets: Option<Bytes>;
    readonly isMigrateAndNotifyOldTargets: boolean;
    readonly type:
      | "MigrateSupportedVersion"
      | "MigrateVersionNotifiers"
      | "NotifyCurrentTargets"
      | "MigrateAndNotifyOldTargets";
  }

  /** @name PalletXcmRemoteLockedFungibleRecord (612) */
  interface PalletXcmRemoteLockedFungibleRecord extends Struct {
    readonly amount: u128;
    readonly owner: XcmVersionedLocation;
    readonly locker: XcmVersionedLocation;
    readonly consumers: Vec<ITuple<[Null, u128]>>;
  }

  /** @name PalletXcmError (619) */
  interface PalletXcmError extends Enum {
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
    readonly isCannotCheckOutTeleport: boolean;
    readonly isLowBalance: boolean;
    readonly isTooManyLocks: boolean;
    readonly isAccountNotSovereign: boolean;
    readonly isFeesNotMet: boolean;
    readonly isLockNotFound: boolean;
    readonly isInUse: boolean;
    readonly isInvalidAssetUnknownReserve: boolean;
    readonly isInvalidAssetUnsupportedReserve: boolean;
    readonly isTooManyReserves: boolean;
    readonly isLocalExecutionIncomplete: boolean;
    readonly type:
      | "Unreachable"
      | "SendFailure"
      | "Filtered"
      | "UnweighableMessage"
      | "DestinationNotInvertible"
      | "Empty"
      | "CannotReanchor"
      | "TooManyAssets"
      | "InvalidOrigin"
      | "BadVersion"
      | "BadLocation"
      | "NoSubscription"
      | "AlreadySubscribed"
      | "CannotCheckOutTeleport"
      | "LowBalance"
      | "TooManyLocks"
      | "AccountNotSovereign"
      | "FeesNotMet"
      | "LockNotFound"
      | "InUse"
      | "InvalidAssetUnknownReserve"
      | "InvalidAssetUnsupportedReserve"
      | "TooManyReserves"
      | "LocalExecutionIncomplete";
  }

  /** @name PalletAssetsAssetDetails (620) */
  interface PalletAssetsAssetDetails extends Struct {
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
    readonly status: PalletAssetsAssetStatus;
  }

  /** @name PalletAssetsAssetStatus (621) */
  interface PalletAssetsAssetStatus extends Enum {
    readonly isLive: boolean;
    readonly isFrozen: boolean;
    readonly isDestroying: boolean;
    readonly type: "Live" | "Frozen" | "Destroying";
  }

  /** @name PalletAssetsAssetAccount (623) */
  interface PalletAssetsAssetAccount extends Struct {
    readonly balance: u128;
    readonly status: PalletAssetsAccountStatus;
    readonly reason: PalletAssetsExistenceReason;
    readonly extra: Null;
  }

  /** @name PalletAssetsAccountStatus (624) */
  interface PalletAssetsAccountStatus extends Enum {
    readonly isLiquid: boolean;
    readonly isFrozen: boolean;
    readonly isBlocked: boolean;
    readonly type: "Liquid" | "Frozen" | "Blocked";
  }

  /** @name PalletAssetsExistenceReason (625) */
  interface PalletAssetsExistenceReason extends Enum {
    readonly isConsumer: boolean;
    readonly isSufficient: boolean;
    readonly isDepositHeld: boolean;
    readonly asDepositHeld: u128;
    readonly isDepositRefunded: boolean;
    readonly isDepositFrom: boolean;
    readonly asDepositFrom: ITuple<[AccountId20, u128]>;
    readonly type: "Consumer" | "Sufficient" | "DepositHeld" | "DepositRefunded" | "DepositFrom";
  }

  /** @name PalletAssetsApproval (627) */
  interface PalletAssetsApproval extends Struct {
    readonly amount: u128;
    readonly deposit: u128;
  }

  /** @name PalletAssetsAssetMetadata (628) */
  interface PalletAssetsAssetMetadata extends Struct {
    readonly deposit: u128;
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly decimals: u8;
    readonly isFrozen: bool;
  }

  /** @name PalletAssetsError (630) */
  interface PalletAssetsError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isNoAccount: boolean;
    readonly isNoPermission: boolean;
    readonly isUnknown: boolean;
    readonly isFrozen: boolean;
    readonly isInUse: boolean;
    readonly isBadWitness: boolean;
    readonly isMinBalanceZero: boolean;
    readonly isUnavailableConsumer: boolean;
    readonly isBadMetadata: boolean;
    readonly isUnapproved: boolean;
    readonly isWouldDie: boolean;
    readonly isAlreadyExists: boolean;
    readonly isNoDeposit: boolean;
    readonly isWouldBurn: boolean;
    readonly isLiveAsset: boolean;
    readonly isAssetNotLive: boolean;
    readonly isIncorrectStatus: boolean;
    readonly isNotFrozen: boolean;
    readonly isCallbackFailed: boolean;
    readonly isBadAssetId: boolean;
    readonly type:
      | "BalanceLow"
      | "NoAccount"
      | "NoPermission"
      | "Unknown"
      | "Frozen"
      | "InUse"
      | "BadWitness"
      | "MinBalanceZero"
      | "UnavailableConsumer"
      | "BadMetadata"
      | "Unapproved"
      | "WouldDie"
      | "AlreadyExists"
      | "NoDeposit"
      | "WouldBurn"
      | "LiveAsset"
      | "AssetNotLive"
      | "IncorrectStatus"
      | "NotFrozen"
      | "CallbackFailed"
      | "BadAssetId";
  }

  /** @name PalletAssetManagerError (631) */
  interface PalletAssetManagerError extends Enum {
    readonly isErrorCreatingAsset: boolean;
    readonly isAssetAlreadyExists: boolean;
    readonly isAssetDoesNotExist: boolean;
    readonly isTooLowNumAssetsWeightHint: boolean;
    readonly isLocalAssetLimitReached: boolean;
    readonly isErrorDestroyingAsset: boolean;
    readonly isNotSufficientDeposit: boolean;
    readonly isNonExistentLocalAsset: boolean;
    readonly type:
      | "ErrorCreatingAsset"
      | "AssetAlreadyExists"
      | "AssetDoesNotExist"
      | "TooLowNumAssetsWeightHint"
      | "LocalAssetLimitReached"
      | "ErrorDestroyingAsset"
      | "NotSufficientDeposit"
      | "NonExistentLocalAsset";
  }

  /** @name PalletXcmTransactorRelayIndicesRelayChainIndices (632) */
  interface PalletXcmTransactorRelayIndicesRelayChainIndices extends Struct {
    readonly staking: u8;
    readonly utility: u8;
    readonly hrmp: u8;
    readonly bond: u8;
    readonly bondExtra: u8;
    readonly unbond: u8;
    readonly withdrawUnbonded: u8;
    readonly validate: u8;
    readonly nominate: u8;
    readonly chill: u8;
    readonly setPayee: u8;
    readonly setController: u8;
    readonly rebond: u8;
    readonly asDerivative: u8;
    readonly initOpenChannel: u8;
    readonly acceptOpenChannel: u8;
    readonly closeChannel: u8;
    readonly cancelOpenRequest: u8;
  }

  /** @name PalletXcmTransactorError (633) */
  interface PalletXcmTransactorError extends Enum {
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
    readonly isErrorDelivering: boolean;
    readonly isDispatchWeightBiggerThanTotalWeight: boolean;
    readonly isWeightOverflow: boolean;
    readonly isAmountOverflow: boolean;
    readonly isTransactorInfoNotSet: boolean;
    readonly isNotCrossChainTransferableCurrency: boolean;
    readonly isXcmExecuteError: boolean;
    readonly isBadVersion: boolean;
    readonly isMaxWeightTransactReached: boolean;
    readonly isUnableToWithdrawAsset: boolean;
    readonly isFeePerSecondNotSet: boolean;
    readonly isSignedTransactNotAllowedForDestination: boolean;
    readonly isFailedMultiLocationToJunction: boolean;
    readonly isHrmpHandlerNotImplemented: boolean;
    readonly isTooMuchFeeUsed: boolean;
    readonly isErrorValidating: boolean;
    readonly isRefundNotSupportedWithTransactInfo: boolean;
    readonly type:
      | "IndexAlreadyClaimed"
      | "UnclaimedIndex"
      | "NotOwner"
      | "UnweighableMessage"
      | "CannotReanchor"
      | "AssetHasNoReserve"
      | "InvalidDest"
      | "NotCrossChainTransfer"
      | "AssetIsNotReserveInDestination"
      | "DestinationNotInvertible"
      | "ErrorDelivering"
      | "DispatchWeightBiggerThanTotalWeight"
      | "WeightOverflow"
      | "AmountOverflow"
      | "TransactorInfoNotSet"
      | "NotCrossChainTransferableCurrency"
      | "XcmExecuteError"
      | "BadVersion"
      | "MaxWeightTransactReached"
      | "UnableToWithdrawAsset"
      | "FeePerSecondNotSet"
      | "SignedTransactNotAllowedForDestination"
      | "FailedMultiLocationToJunction"
      | "HrmpHandlerNotImplemented"
      | "TooMuchFeeUsed"
      | "ErrorValidating"
      | "RefundNotSupportedWithTransactInfo";
  }

  /** @name PalletMoonbeamOrbitersCollatorPoolInfo (634) */
  interface PalletMoonbeamOrbitersCollatorPoolInfo extends Struct {
    readonly orbiters: Vec<AccountId20>;
    readonly maybeCurrentOrbiter: Option<PalletMoonbeamOrbitersCurrentOrbiter>;
    readonly nextOrbiter: u32;
  }

  /** @name PalletMoonbeamOrbitersCurrentOrbiter (636) */
  interface PalletMoonbeamOrbitersCurrentOrbiter extends Struct {
    readonly accountId: AccountId20;
    readonly removed: bool;
  }

  /** @name PalletMoonbeamOrbitersError (637) */
  interface PalletMoonbeamOrbitersError extends Enum {
    readonly isCollatorAlreadyAdded: boolean;
    readonly isCollatorNotFound: boolean;
    readonly isCollatorPoolTooLarge: boolean;
    readonly isCollatorsPoolCountTooLow: boolean;
    readonly isMinOrbiterDepositNotSet: boolean;
    readonly isOrbiterAlreadyInPool: boolean;
    readonly isOrbiterDepositNotFound: boolean;
    readonly isOrbiterNotFound: boolean;
    readonly isOrbiterStillInAPool: boolean;
    readonly type:
      | "CollatorAlreadyAdded"
      | "CollatorNotFound"
      | "CollatorPoolTooLarge"
      | "CollatorsPoolCountTooLow"
      | "MinOrbiterDepositNotSet"
      | "OrbiterAlreadyInPool"
      | "OrbiterDepositNotFound"
      | "OrbiterNotFound"
      | "OrbiterStillInAPool";
  }

  /** @name PalletEthereumXcmError (638) */
  interface PalletEthereumXcmError extends Enum {
    readonly isEthereumXcmExecutionSuspended: boolean;
    readonly type: "EthereumXcmExecutionSuspended";
  }

  /** @name PalletRandomnessRequestState (639) */
  interface PalletRandomnessRequestState extends Struct {
    readonly request: PalletRandomnessRequest;
    readonly deposit: u128;
  }

  /** @name PalletRandomnessRequest (640) */
  interface PalletRandomnessRequest extends Struct {
    readonly refundAddress: H160;
    readonly contractAddress: H160;
    readonly fee: u128;
    readonly gasLimit: u64;
    readonly numWords: u8;
    readonly salt: H256;
    readonly info: PalletRandomnessRequestInfo;
  }

  /** @name PalletRandomnessRequestInfo (641) */
  interface PalletRandomnessRequestInfo extends Enum {
    readonly isBabeEpoch: boolean;
    readonly asBabeEpoch: ITuple<[u64, u64]>;
    readonly isLocal: boolean;
    readonly asLocal: ITuple<[u32, u32]>;
    readonly type: "BabeEpoch" | "Local";
  }

  /** @name PalletRandomnessRequestType (642) */
  interface PalletRandomnessRequestType extends Enum {
    readonly isBabeEpoch: boolean;
    readonly asBabeEpoch: u64;
    readonly isLocal: boolean;
    readonly asLocal: u32;
    readonly type: "BabeEpoch" | "Local";
  }

  /** @name PalletRandomnessRandomnessResult (643) */
  interface PalletRandomnessRandomnessResult extends Struct {
    readonly randomness: Option<H256>;
    readonly requestCount: u64;
  }

  /** @name PalletRandomnessError (644) */
  interface PalletRandomnessError extends Enum {
    readonly isRequestCounterOverflowed: boolean;
    readonly isRequestFeeOverflowed: boolean;
    readonly isMustRequestAtLeastOneWord: boolean;
    readonly isCannotRequestMoreWordsThanMax: boolean;
    readonly isCannotRequestRandomnessAfterMaxDelay: boolean;
    readonly isCannotRequestRandomnessBeforeMinDelay: boolean;
    readonly isRequestDNE: boolean;
    readonly isRequestCannotYetBeFulfilled: boolean;
    readonly isOnlyRequesterCanIncreaseFee: boolean;
    readonly isRequestHasNotExpired: boolean;
    readonly isRandomnessResultDNE: boolean;
    readonly isRandomnessResultNotFilled: boolean;
    readonly type:
      | "RequestCounterOverflowed"
      | "RequestFeeOverflowed"
      | "MustRequestAtLeastOneWord"
      | "CannotRequestMoreWordsThanMax"
      | "CannotRequestRandomnessAfterMaxDelay"
      | "CannotRequestRandomnessBeforeMinDelay"
      | "RequestDNE"
      | "RequestCannotYetBeFulfilled"
      | "OnlyRequesterCanIncreaseFee"
      | "RequestHasNotExpired"
      | "RandomnessResultDNE"
      | "RandomnessResultNotFilled";
  }

  /** @name PalletCollectiveVotes (647) */
  interface PalletCollectiveVotes extends Struct {
    readonly index: u32;
    readonly threshold: u32;
    readonly ayes: Vec<AccountId20>;
    readonly nays: Vec<AccountId20>;
    readonly end: u32;
  }

  /** @name PalletCollectiveError (648) */
  interface PalletCollectiveError extends Enum {
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
    readonly isPrimeAccountNotMember: boolean;
    readonly isProposalActive: boolean;
    readonly type:
      | "NotMember"
      | "DuplicateProposal"
      | "ProposalMissing"
      | "WrongIndex"
      | "DuplicateVote"
      | "AlreadyInitialized"
      | "TooEarly"
      | "TooManyProposals"
      | "WrongProposalWeight"
      | "WrongProposalLength"
      | "PrimeAccountNotMember"
      | "ProposalActive";
  }

  /** @name PalletConvictionVotingVoteVoting (650) */
  interface PalletConvictionVotingVoteVoting extends Enum {
    readonly isCasting: boolean;
    readonly asCasting: PalletConvictionVotingVoteCasting;
    readonly isDelegating: boolean;
    readonly asDelegating: PalletConvictionVotingVoteDelegating;
    readonly type: "Casting" | "Delegating";
  }

  /** @name PalletConvictionVotingVoteCasting (651) */
  interface PalletConvictionVotingVoteCasting extends Struct {
    readonly votes: Vec<ITuple<[u32, PalletConvictionVotingVoteAccountVote]>>;
    readonly delegations: PalletConvictionVotingDelegations;
    readonly prior: PalletConvictionVotingVotePriorLock;
  }

  /** @name PalletConvictionVotingDelegations (655) */
  interface PalletConvictionVotingDelegations extends Struct {
    readonly votes: u128;
    readonly capital: u128;
  }

  /** @name PalletConvictionVotingVotePriorLock (656) */
  interface PalletConvictionVotingVotePriorLock extends ITuple<[u32, u128]> {}

  /** @name PalletConvictionVotingVoteDelegating (657) */
  interface PalletConvictionVotingVoteDelegating extends Struct {
    readonly balance: u128;
    readonly target: AccountId20;
    readonly conviction: PalletConvictionVotingConviction;
    readonly delegations: PalletConvictionVotingDelegations;
    readonly prior: PalletConvictionVotingVotePriorLock;
  }

  /** @name PalletConvictionVotingError (661) */
  interface PalletConvictionVotingError extends Enum {
    readonly isNotOngoing: boolean;
    readonly isNotVoter: boolean;
    readonly isNoPermission: boolean;
    readonly isNoPermissionYet: boolean;
    readonly isAlreadyDelegating: boolean;
    readonly isAlreadyVoting: boolean;
    readonly isInsufficientFunds: boolean;
    readonly isNotDelegating: boolean;
    readonly isNonsense: boolean;
    readonly isMaxVotesReached: boolean;
    readonly isClassNeeded: boolean;
    readonly isBadClass: boolean;
    readonly type:
      | "NotOngoing"
      | "NotVoter"
      | "NoPermission"
      | "NoPermissionYet"
      | "AlreadyDelegating"
      | "AlreadyVoting"
      | "InsufficientFunds"
      | "NotDelegating"
      | "Nonsense"
      | "MaxVotesReached"
      | "ClassNeeded"
      | "BadClass";
  }

  /** @name PalletReferendaReferendumInfo (662) */
  interface PalletReferendaReferendumInfo extends Enum {
    readonly isOngoing: boolean;
    readonly asOngoing: PalletReferendaReferendumStatus;
    readonly isApproved: boolean;
    readonly asApproved: ITuple<
      [u32, Option<PalletReferendaDeposit>, Option<PalletReferendaDeposit>]
    >;
    readonly isRejected: boolean;
    readonly asRejected: ITuple<
      [u32, Option<PalletReferendaDeposit>, Option<PalletReferendaDeposit>]
    >;
    readonly isCancelled: boolean;
    readonly asCancelled: ITuple<
      [u32, Option<PalletReferendaDeposit>, Option<PalletReferendaDeposit>]
    >;
    readonly isTimedOut: boolean;
    readonly asTimedOut: ITuple<
      [u32, Option<PalletReferendaDeposit>, Option<PalletReferendaDeposit>]
    >;
    readonly isKilled: boolean;
    readonly asKilled: u32;
    readonly type: "Ongoing" | "Approved" | "Rejected" | "Cancelled" | "TimedOut" | "Killed";
  }

  /** @name PalletReferendaReferendumStatus (663) */
  interface PalletReferendaReferendumStatus extends Struct {
    readonly track: u16;
    readonly origin: MoonbaseRuntimeOriginCaller;
    readonly proposal: FrameSupportPreimagesBounded;
    readonly enactment: FrameSupportScheduleDispatchTime;
    readonly submitted: u32;
    readonly submissionDeposit: PalletReferendaDeposit;
    readonly decisionDeposit: Option<PalletReferendaDeposit>;
    readonly deciding: Option<PalletReferendaDecidingStatus>;
    readonly tally: PalletConvictionVotingTally;
    readonly inQueue: bool;
    readonly alarm: Option<ITuple<[u32, ITuple<[u32, u32]>]>>;
  }

  /** @name PalletReferendaDeposit (664) */
  interface PalletReferendaDeposit extends Struct {
    readonly who: AccountId20;
    readonly amount: u128;
  }

  /** @name PalletReferendaDecidingStatus (667) */
  interface PalletReferendaDecidingStatus extends Struct {
    readonly since: u32;
    readonly confirming: Option<u32>;
  }

  /** @name PalletReferendaTrackInfo (675) */
  interface PalletReferendaTrackInfo extends Struct {
    readonly name: Text;
    readonly maxDeciding: u32;
    readonly decisionDeposit: u128;
    readonly preparePeriod: u32;
    readonly decisionPeriod: u32;
    readonly confirmPeriod: u32;
    readonly minEnactmentPeriod: u32;
    readonly minApproval: PalletReferendaCurve;
    readonly minSupport: PalletReferendaCurve;
  }

  /** @name PalletReferendaCurve (676) */
  interface PalletReferendaCurve extends Enum {
    readonly isLinearDecreasing: boolean;
    readonly asLinearDecreasing: {
      readonly length: Perbill;
      readonly floor: Perbill;
      readonly ceil: Perbill;
    } & Struct;
    readonly isSteppedDecreasing: boolean;
    readonly asSteppedDecreasing: {
      readonly begin: Perbill;
      readonly end: Perbill;
      readonly step: Perbill;
      readonly period: Perbill;
    } & Struct;
    readonly isReciprocal: boolean;
    readonly asReciprocal: {
      readonly factor: i64;
      readonly xOffset: i64;
      readonly yOffset: i64;
    } & Struct;
    readonly type: "LinearDecreasing" | "SteppedDecreasing" | "Reciprocal";
  }

  /** @name PalletReferendaError (679) */
  interface PalletReferendaError extends Enum {
    readonly isNotOngoing: boolean;
    readonly isHasDeposit: boolean;
    readonly isBadTrack: boolean;
    readonly isFull: boolean;
    readonly isQueueEmpty: boolean;
    readonly isBadReferendum: boolean;
    readonly isNothingToDo: boolean;
    readonly isNoTrack: boolean;
    readonly isUnfinished: boolean;
    readonly isNoPermission: boolean;
    readonly isNoDeposit: boolean;
    readonly isBadStatus: boolean;
    readonly isPreimageNotExist: boolean;
    readonly isPreimageStoredWithDifferentLength: boolean;
    readonly type:
      | "NotOngoing"
      | "HasDeposit"
      | "BadTrack"
      | "Full"
      | "QueueEmpty"
      | "BadReferendum"
      | "NothingToDo"
      | "NoTrack"
      | "Unfinished"
      | "NoPermission"
      | "NoDeposit"
      | "BadStatus"
      | "PreimageNotExist"
      | "PreimageStoredWithDifferentLength";
  }

  /** @name PalletPreimageOldRequestStatus (680) */
  interface PalletPreimageOldRequestStatus extends Enum {
    readonly isUnrequested: boolean;
    readonly asUnrequested: {
      readonly deposit: ITuple<[AccountId20, u128]>;
      readonly len: u32;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly deposit: Option<ITuple<[AccountId20, u128]>>;
      readonly count: u32;
      readonly len: Option<u32>;
    } & Struct;
    readonly type: "Unrequested" | "Requested";
  }

  /** @name PalletPreimageRequestStatus (683) */
  interface PalletPreimageRequestStatus extends Enum {
    readonly isUnrequested: boolean;
    readonly asUnrequested: {
      readonly ticket: ITuple<[AccountId20, u128]>;
      readonly len: u32;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly maybeTicket: Option<ITuple<[AccountId20, u128]>>;
      readonly count: u32;
      readonly maybeLen: Option<u32>;
    } & Struct;
    readonly type: "Unrequested" | "Requested";
  }

  /** @name PalletPreimageError (689) */
  interface PalletPreimageError extends Enum {
    readonly isTooBig: boolean;
    readonly isAlreadyNoted: boolean;
    readonly isNotAuthorized: boolean;
    readonly isNotNoted: boolean;
    readonly isRequested: boolean;
    readonly isNotRequested: boolean;
    readonly isTooMany: boolean;
    readonly isTooFew: boolean;
    readonly type:
      | "TooBig"
      | "AlreadyNoted"
      | "NotAuthorized"
      | "NotNoted"
      | "Requested"
      | "NotRequested"
      | "TooMany"
      | "TooFew";
  }

  /** @name PalletWhitelistError (690) */
  interface PalletWhitelistError extends Enum {
    readonly isUnavailablePreImage: boolean;
    readonly isUndecodableCall: boolean;
    readonly isInvalidCallWeightWitness: boolean;
    readonly isCallIsNotWhitelisted: boolean;
    readonly isCallAlreadyWhitelisted: boolean;
    readonly type:
      | "UnavailablePreImage"
      | "UndecodableCall"
      | "InvalidCallWeightWitness"
      | "CallIsNotWhitelisted"
      | "CallAlreadyWhitelisted";
  }

  /** @name PalletMultisigMultisig (694) */
  interface PalletMultisigMultisig extends Struct {
    readonly when: PalletMultisigTimepoint;
    readonly deposit: u128;
    readonly depositor: AccountId20;
    readonly approvals: Vec<AccountId20>;
  }

  /** @name PalletMultisigError (696) */
  interface PalletMultisigError extends Enum {
    readonly isMinimumThreshold: boolean;
    readonly isAlreadyApproved: boolean;
    readonly isNoApprovalsNeeded: boolean;
    readonly isTooFewSignatories: boolean;
    readonly isTooManySignatories: boolean;
    readonly isSignatoriesOutOfOrder: boolean;
    readonly isSenderInSignatories: boolean;
    readonly isNotFound: boolean;
    readonly isNotOwner: boolean;
    readonly isNoTimepoint: boolean;
    readonly isWrongTimepoint: boolean;
    readonly isUnexpectedTimepoint: boolean;
    readonly isMaxWeightTooLow: boolean;
    readonly isAlreadyStored: boolean;
    readonly type:
      | "MinimumThreshold"
      | "AlreadyApproved"
      | "NoApprovalsNeeded"
      | "TooFewSignatories"
      | "TooManySignatories"
      | "SignatoriesOutOfOrder"
      | "SenderInSignatories"
      | "NotFound"
      | "NotOwner"
      | "NoTimepoint"
      | "WrongTimepoint"
      | "UnexpectedTimepoint"
      | "MaxWeightTooLow"
      | "AlreadyStored";
  }

  /** @name PalletMoonbeamLazyMigrationsError (699) */
  interface PalletMoonbeamLazyMigrationsError extends Enum {
    readonly isContractMetadataAlreadySet: boolean;
    readonly isContractNotExist: boolean;
    readonly type: "ContractMetadataAlreadySet" | "ContractNotExist";
  }

  /** @name PalletMessageQueueBookState (701) */
  interface PalletMessageQueueBookState extends Struct {
    readonly begin: u32;
    readonly end: u32;
    readonly count: u32;
    readonly readyNeighbours: Option<PalletMessageQueueNeighbours>;
    readonly messageCount: u64;
    readonly size_: u64;
  }

  /** @name PalletMessageQueueNeighbours (703) */
  interface PalletMessageQueueNeighbours extends Struct {
    readonly prev: CumulusPrimitivesCoreAggregateMessageOrigin;
    readonly next: CumulusPrimitivesCoreAggregateMessageOrigin;
  }

  /** @name PalletMessageQueuePage (705) */
  interface PalletMessageQueuePage extends Struct {
    readonly remaining: u32;
    readonly remainingSize: u32;
    readonly firstIndex: u32;
    readonly first: u32;
    readonly last: u32;
    readonly heap: Bytes;
  }

  /** @name PalletMessageQueueError (707) */
  interface PalletMessageQueueError extends Enum {
    readonly isNotReapable: boolean;
    readonly isNoPage: boolean;
    readonly isNoMessage: boolean;
    readonly isAlreadyProcessed: boolean;
    readonly isQueued: boolean;
    readonly isInsufficientWeight: boolean;
    readonly isTemporarilyUnprocessable: boolean;
    readonly isQueuePaused: boolean;
    readonly isRecursiveDisallowed: boolean;
    readonly type:
      | "NotReapable"
      | "NoPage"
      | "NoMessage"
      | "AlreadyProcessed"
      | "Queued"
      | "InsufficientWeight"
      | "TemporarilyUnprocessable"
      | "QueuePaused"
      | "RecursiveDisallowed";
  }

  /** @name PalletEmergencyParaXcmXcmMode (708) */
  interface PalletEmergencyParaXcmXcmMode extends Enum {
    readonly isNormal: boolean;
    readonly isPaused: boolean;
    readonly type: "Normal" | "Paused";
  }

  /** @name PalletEmergencyParaXcmError (709) */
  interface PalletEmergencyParaXcmError extends Enum {
    readonly isNotInPausedMode: boolean;
    readonly type: "NotInPausedMode";
  }

  /** @name PalletMoonbeamForeignAssetsAssetStatus (711) */
  interface PalletMoonbeamForeignAssetsAssetStatus extends Enum {
    readonly isActive: boolean;
    readonly isFrozenXcmDepositAllowed: boolean;
    readonly isFrozenXcmDepositForbidden: boolean;
    readonly type: "Active" | "FrozenXcmDepositAllowed" | "FrozenXcmDepositForbidden";
  }

  /** @name PalletMoonbeamForeignAssetsAssetDepositDetails (712) */
  interface PalletMoonbeamForeignAssetsAssetDepositDetails extends Struct {
    readonly depositAccount: AccountId20;
    readonly deposit: u128;
  }

  /** @name MoonbaseRuntimeRuntime (713) */
  type MoonbaseRuntimeRuntime = Null;

  /** @name PalletMoonbeamForeignAssetsError (714) */
  interface PalletMoonbeamForeignAssetsError extends Enum {
    readonly isAssetAlreadyExists: boolean;
    readonly isAssetAlreadyFrozen: boolean;
    readonly isAssetDoesNotExist: boolean;
    readonly isAssetIdFiltered: boolean;
    readonly isAssetNotFrozen: boolean;
    readonly isCorruptedStorageOrphanLocation: boolean;
    readonly isErc20ContractCreationFail: boolean;
    readonly isEvmCallPauseFail: boolean;
    readonly isEvmCallUnpauseFail: boolean;
    readonly isEvmCallMintIntoFail: boolean;
    readonly isEvmCallTransferFail: boolean;
    readonly isEvmInternalError: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isCannotConvertLocationToAccount: boolean;
    readonly isLocationOutsideOfOrigin: boolean;
    readonly isAssetNotInSiblingPara: boolean;
    readonly isInvalidSymbol: boolean;
    readonly isInvalidTokenName: boolean;
    readonly isLocationAlreadyExists: boolean;
    readonly isTooManyForeignAssets: boolean;
    readonly type:
      | "AssetAlreadyExists"
      | "AssetAlreadyFrozen"
      | "AssetDoesNotExist"
      | "AssetIdFiltered"
      | "AssetNotFrozen"
      | "CorruptedStorageOrphanLocation"
      | "Erc20ContractCreationFail"
      | "EvmCallPauseFail"
      | "EvmCallUnpauseFail"
      | "EvmCallMintIntoFail"
      | "EvmCallTransferFail"
      | "EvmInternalError"
      | "InsufficientBalance"
      | "CannotConvertLocationToAccount"
      | "LocationOutsideOfOrigin"
      | "AssetNotInSiblingPara"
      | "InvalidSymbol"
      | "InvalidTokenName"
      | "LocationAlreadyExists"
      | "TooManyForeignAssets";
  }

  /** @name PalletXcmWeightTraderError (716) */
  interface PalletXcmWeightTraderError extends Enum {
    readonly isAssetAlreadyAdded: boolean;
    readonly isAssetAlreadyPaused: boolean;
    readonly isAssetNotFound: boolean;
    readonly isAssetNotPaused: boolean;
    readonly isXcmLocationFiltered: boolean;
    readonly isPriceCannotBeZero: boolean;
    readonly isPriceOverflow: boolean;
    readonly type:
      | "AssetAlreadyAdded"
      | "AssetAlreadyPaused"
      | "AssetNotFound"
      | "AssetNotPaused"
      | "XcmLocationFiltered"
      | "PriceCannotBeZero"
      | "PriceOverflow";
  }

  /** @name PalletMigrationsError (717) */
  interface PalletMigrationsError extends Enum {
    readonly isOngoing: boolean;
    readonly type: "Ongoing";
  }

  /** @name FrameSystemExtensionsCheckNonZeroSender (720) */
  type FrameSystemExtensionsCheckNonZeroSender = Null;

  /** @name FrameSystemExtensionsCheckSpecVersion (721) */
  type FrameSystemExtensionsCheckSpecVersion = Null;

  /** @name FrameSystemExtensionsCheckTxVersion (722) */
  type FrameSystemExtensionsCheckTxVersion = Null;

  /** @name FrameSystemExtensionsCheckGenesis (723) */
  type FrameSystemExtensionsCheckGenesis = Null;

  /** @name FrameSystemExtensionsCheckNonce (726) */
  interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /** @name FrameSystemExtensionsCheckWeight (727) */
  type FrameSystemExtensionsCheckWeight = Null;

  /** @name PalletTransactionPaymentChargeTransactionPayment (728) */
  interface PalletTransactionPaymentChargeTransactionPayment extends Compact<u128> {}

  /** @name FrameMetadataHashExtensionCheckMetadataHash (729) */
  interface FrameMetadataHashExtensionCheckMetadataHash extends Struct {
    readonly mode: FrameMetadataHashExtensionMode;
  }

  /** @name FrameMetadataHashExtensionMode (730) */
  interface FrameMetadataHashExtensionMode extends Enum {
    readonly isDisabled: boolean;
    readonly isEnabled: boolean;
    readonly type: "Disabled" | "Enabled";
  }

  /** @name CumulusPrimitivesStorageWeightReclaimStorageWeightReclaim (731) */
  type CumulusPrimitivesStorageWeightReclaimStorageWeightReclaim = Null;
} // declare module
