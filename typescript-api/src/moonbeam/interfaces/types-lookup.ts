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
  Set,
  Struct,
  Text,
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
import type { Vote } from "@polkadot/types/interfaces/elections";
import type {
  AccountId20,
  Call,
  H160,
  H256,
  Perbill,
  Percent,
} from "@polkadot/types/interfaces/runtime";
import type { Event } from "@polkadot/types/interfaces/system";

declare module "@polkadot/types/lookup" {
  /**
   * @name FrameSystemAccountInfo (3)
   */
  interface FrameSystemAccountInfo extends Struct {
    readonly nonce: u32;
    readonly consumers: u32;
    readonly providers: u32;
    readonly sufficients: u32;
    readonly data: PalletBalancesAccountData;
  }

  /**
   * @name PalletBalancesAccountData (5)
   */
  interface PalletBalancesAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly miscFrozen: u128;
    readonly feeFrozen: u128;
  }

  /**
   * @name FrameSupportDispatchPerDispatchClassWeight (7)
   */
  interface FrameSupportDispatchPerDispatchClassWeight extends Struct {
    readonly normal: SpWeightsWeightV2Weight;
    readonly operational: SpWeightsWeightV2Weight;
    readonly mandatory: SpWeightsWeightV2Weight;
  }

  /**
   * @name SpWeightsWeightV2Weight (8)
   */
  interface SpWeightsWeightV2Weight extends Struct {
    readonly refTime: Compact<u64>;
    readonly proofSize: Compact<u64>;
  }

  /**
   * @name SpRuntimeDigest (14)
   */
  interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>;
  }

  /**
   * @name SpRuntimeDigestDigestItem (16)
   */
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

  /**
   * @name FrameSystemEventRecord (19)
   */
  interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase;
    readonly event: Event;
    readonly topics: Vec<H256>;
  }

  /**
   * @name FrameSystemEvent (21)
   */
  interface FrameSystemEvent extends Enum {
    readonly isExtrinsicSuccess: boolean;
    readonly asExtrinsicSuccess: {
      readonly dispatchInfo: FrameSupportDispatchDispatchInfo;
    } & Struct;
    readonly isExtrinsicFailed: boolean;
    readonly asExtrinsicFailed: {
      readonly dispatchError: SpRuntimeDispatchError;
      readonly dispatchInfo: FrameSupportDispatchDispatchInfo;
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
    readonly type:
      | "ExtrinsicSuccess"
      | "ExtrinsicFailed"
      | "CodeUpdated"
      | "NewAccount"
      | "KilledAccount"
      | "Remarked";
  }

  /**
   * @name FrameSupportDispatchDispatchInfo (22)
   */
  interface FrameSupportDispatchDispatchInfo extends Struct {
    readonly weight: SpWeightsWeightV2Weight;
    readonly class: FrameSupportDispatchDispatchClass;
    readonly paysFee: FrameSupportDispatchPays;
  }

  /**
   * @name FrameSupportDispatchDispatchClass (23)
   */
  interface FrameSupportDispatchDispatchClass extends Enum {
    readonly isNormal: boolean;
    readonly isOperational: boolean;
    readonly isMandatory: boolean;
    readonly type: "Normal" | "Operational" | "Mandatory";
  }

  /**
   * @name FrameSupportDispatchPays (24)
   */
  interface FrameSupportDispatchPays extends Enum {
    readonly isYes: boolean;
    readonly isNo: boolean;
    readonly type: "Yes" | "No";
  }

  /**
   * @name SpRuntimeDispatchError (25)
   */
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
    readonly asArithmetic: SpRuntimeArithmeticError;
    readonly isTransactional: boolean;
    readonly asTransactional: SpRuntimeTransactionalError;
    readonly isExhausted: boolean;
    readonly isCorruption: boolean;
    readonly isUnavailable: boolean;
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
      | "Unavailable";
  }

  /**
   * @name SpRuntimeModuleError (26)
   */
  interface SpRuntimeModuleError extends Struct {
    readonly index: u8;
    readonly error: U8aFixed;
  }

  /**
   * @name SpRuntimeTokenError (27)
   */
  interface SpRuntimeTokenError extends Enum {
    readonly isNoFunds: boolean;
    readonly isWouldDie: boolean;
    readonly isBelowMinimum: boolean;
    readonly isCannotCreate: boolean;
    readonly isUnknownAsset: boolean;
    readonly isFrozen: boolean;
    readonly isUnsupported: boolean;
    readonly type:
      | "NoFunds"
      | "WouldDie"
      | "BelowMinimum"
      | "CannotCreate"
      | "UnknownAsset"
      | "Frozen"
      | "Unsupported";
  }

  /**
   * @name SpRuntimeArithmeticError (28)
   */
  interface SpRuntimeArithmeticError extends Enum {
    readonly isUnderflow: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly type: "Underflow" | "Overflow" | "DivisionByZero";
  }

  /**
   * @name SpRuntimeTransactionalError (29)
   */
  interface SpRuntimeTransactionalError extends Enum {
    readonly isLimitReached: boolean;
    readonly isNoLayer: boolean;
    readonly type: "LimitReached" | "NoLayer";
  }

  /**
   * @name CumulusPalletParachainSystemEvent (30)
   */
  interface CumulusPalletParachainSystemEvent extends Enum {
    readonly isValidationFunctionStored: boolean;
    readonly isValidationFunctionApplied: boolean;
    readonly asValidationFunctionApplied: {
      readonly relayChainBlockNum: u32;
    } & Struct;
    readonly isValidationFunctionDiscarded: boolean;
    readonly isUpgradeAuthorized: boolean;
    readonly asUpgradeAuthorized: {
      readonly codeHash: H256;
    } & Struct;
    readonly isDownwardMessagesReceived: boolean;
    readonly asDownwardMessagesReceived: {
      readonly count: u32;
    } & Struct;
    readonly isDownwardMessagesProcessed: boolean;
    readonly asDownwardMessagesProcessed: {
      readonly weightUsed: SpWeightsWeightV2Weight;
      readonly dmqHead: H256;
    } & Struct;
    readonly type:
      | "ValidationFunctionStored"
      | "ValidationFunctionApplied"
      | "ValidationFunctionDiscarded"
      | "UpgradeAuthorized"
      | "DownwardMessagesReceived"
      | "DownwardMessagesProcessed";
  }

  /**
   * @name PalletBalancesEvent (31)
   */
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
      | "Slashed";
  }

  /**
   * @name FrameSupportTokensMiscBalanceStatus (32)
   */
  interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean;
    readonly isReserved: boolean;
    readonly type: "Free" | "Reserved";
  }

  /**
   * @name PalletTransactionPaymentEvent (33)
   */
  interface PalletTransactionPaymentEvent extends Enum {
    readonly isTransactionFeePaid: boolean;
    readonly asTransactionFeePaid: {
      readonly who: AccountId20;
      readonly actualFee: u128;
      readonly tip: u128;
    } & Struct;
    readonly type: "TransactionFeePaid";
  }

  /**
   * @name PalletParachainStakingEvent (34)
   */
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
    readonly isReservedForParachainBond: boolean;
    readonly asReservedForParachainBond: {
      readonly account: AccountId20;
      readonly value: u128;
    } & Struct;
    readonly isParachainBondAccountSet: boolean;
    readonly asParachainBondAccountSet: {
      readonly old: AccountId20;
      readonly new_: AccountId20;
    } & Struct;
    readonly isParachainBondReservePercentSet: boolean;
    readonly asParachainBondReservePercentSet: {
      readonly old: Percent;
      readonly new_: Percent;
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
      | "ReservedForParachainBond"
      | "ParachainBondAccountSet"
      | "ParachainBondReservePercentSet"
      | "InflationSet"
      | "StakeExpectationsSet"
      | "TotalSelectedSet"
      | "CollatorCommissionSet"
      | "BlocksPerRoundSet"
      | "AutoCompoundSet"
      | "Compounded";
  }

  /**
   * @name PalletParachainStakingDelegationRequestsCancelledScheduledRequest (36)
   */
  interface PalletParachainStakingDelegationRequestsCancelledScheduledRequest extends Struct {
    readonly whenExecutable: u32;
    readonly action: PalletParachainStakingDelegationRequestsDelegationAction;
  }

  /**
   * @name PalletParachainStakingDelegationRequestsDelegationAction (37)
   */
  interface PalletParachainStakingDelegationRequestsDelegationAction extends Enum {
    readonly isRevoke: boolean;
    readonly asRevoke: u128;
    readonly isDecrease: boolean;
    readonly asDecrease: u128;
    readonly type: "Revoke" | "Decrease";
  }

  /**
   * @name PalletParachainStakingDelegatorAdded (38)
   */
  interface PalletParachainStakingDelegatorAdded extends Enum {
    readonly isAddedToTop: boolean;
    readonly asAddedToTop: {
      readonly newTotal: u128;
    } & Struct;
    readonly isAddedToBottom: boolean;
    readonly type: "AddedToTop" | "AddedToBottom";
  }

  /**
   * @name PalletAuthorSlotFilterEvent (41)
   */
  interface PalletAuthorSlotFilterEvent extends Enum {
    readonly isEligibleUpdated: boolean;
    readonly asEligibleUpdated: u32;
    readonly type: "EligibleUpdated";
  }

  /**
   * @name PalletAuthorMappingEvent (43)
   */
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

  /**
   * @name NimbusPrimitivesNimbusCryptoPublic (44)
   */
  interface NimbusPrimitivesNimbusCryptoPublic extends SpCoreSr25519Public {}

  /**
   * @name SpCoreSr25519Public (45)
   */
  interface SpCoreSr25519Public extends U8aFixed {}

  /**
   * @name SessionKeysPrimitivesVrfVrfCryptoPublic (46)
   */
  interface SessionKeysPrimitivesVrfVrfCryptoPublic extends SpCoreSr25519Public {}

  /**
   * @name PalletMoonbeamOrbitersEvent (47)
   */
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

  /**
   * @name PalletUtilityEvent (49)
   */
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

  /**
   * @name PalletProxyEvent (52)
   */
  interface PalletProxyEvent extends Enum {
    readonly isProxyExecuted: boolean;
    readonly asProxyExecuted: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isPureCreated: boolean;
    readonly asPureCreated: {
      readonly pure: AccountId20;
      readonly who: AccountId20;
      readonly proxyType: MoonbeamRuntimeProxyType;
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
      readonly proxyType: MoonbeamRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly isProxyRemoved: boolean;
    readonly asProxyRemoved: {
      readonly delegator: AccountId20;
      readonly delegatee: AccountId20;
      readonly proxyType: MoonbeamRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly type: "ProxyExecuted" | "PureCreated" | "Announced" | "ProxyAdded" | "ProxyRemoved";
  }

  /**
   * @name MoonbeamRuntimeProxyType (53)
   */
  interface MoonbeamRuntimeProxyType extends Enum {
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

  /**
   * @name PalletMaintenanceModeEvent (55)
   */
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

  /**
   * @name PalletIdentityEvent (56)
   */
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
    readonly type:
      | "IdentitySet"
      | "IdentityCleared"
      | "IdentityKilled"
      | "JudgementRequested"
      | "JudgementUnrequested"
      | "JudgementGiven"
      | "RegistrarAdded"
      | "SubIdentityAdded"
      | "SubIdentityRemoved"
      | "SubIdentityRevoked";
  }

  /**
   * @name PalletMigrationsEvent (57)
   */
  interface PalletMigrationsEvent extends Enum {
    readonly isRuntimeUpgradeStarted: boolean;
    readonly isRuntimeUpgradeCompleted: boolean;
    readonly asRuntimeUpgradeCompleted: {
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isMigrationStarted: boolean;
    readonly asMigrationStarted: {
      readonly migrationName: Bytes;
    } & Struct;
    readonly isMigrationCompleted: boolean;
    readonly asMigrationCompleted: {
      readonly migrationName: Bytes;
      readonly consumedWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isFailedToSuspendIdleXcmExecution: boolean;
    readonly asFailedToSuspendIdleXcmExecution: {
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isFailedToResumeIdleXcmExecution: boolean;
    readonly asFailedToResumeIdleXcmExecution: {
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly type:
      | "RuntimeUpgradeStarted"
      | "RuntimeUpgradeCompleted"
      | "MigrationStarted"
      | "MigrationCompleted"
      | "FailedToSuspendIdleXcmExecution"
      | "FailedToResumeIdleXcmExecution";
  }

  /**
   * @name PalletEvmEvent (58)
   */
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

  /**
   * @name EthereumLog (59)
   */
  interface EthereumLog extends Struct {
    readonly address: H160;
    readonly topics: Vec<H256>;
    readonly data: Bytes;
  }

  /**
   * @name PalletEthereumEvent (62)
   */
  interface PalletEthereumEvent extends Enum {
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly from: H160;
      readonly to: H160;
      readonly transactionHash: H256;
      readonly exitReason: EvmCoreErrorExitReason;
    } & Struct;
    readonly type: "Executed";
  }

  /**
   * @name EvmCoreErrorExitReason (63)
   */
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

  /**
   * @name EvmCoreErrorExitSucceed (64)
   */
  interface EvmCoreErrorExitSucceed extends Enum {
    readonly isStopped: boolean;
    readonly isReturned: boolean;
    readonly isSuicided: boolean;
    readonly type: "Stopped" | "Returned" | "Suicided";
  }

  /**
   * @name EvmCoreErrorExitError (65)
   */
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
      | "InvalidCode";
  }

  /**
   * @name EvmCoreErrorExitRevert (69)
   */
  interface EvmCoreErrorExitRevert extends Enum {
    readonly isReverted: boolean;
    readonly type: "Reverted";
  }

  /**
   * @name EvmCoreErrorExitFatal (70)
   */
  interface EvmCoreErrorExitFatal extends Enum {
    readonly isNotSupported: boolean;
    readonly isUnhandledInterrupt: boolean;
    readonly isCallErrorAsFatal: boolean;
    readonly asCallErrorAsFatal: EvmCoreErrorExitError;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly type: "NotSupported" | "UnhandledInterrupt" | "CallErrorAsFatal" | "Other";
  }

  /**
   * @name PalletSchedulerEvent (71)
   */
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
    readonly isPermanentlyOverweight: boolean;
    readonly asPermanentlyOverweight: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly type:
      | "Scheduled"
      | "Canceled"
      | "Dispatched"
      | "CallUnavailable"
      | "PeriodicFailed"
      | "PermanentlyOverweight";
  }

  /**
   * @name PalletDemocracyEvent (74)
   */
  interface PalletDemocracyEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly proposalIndex: u32;
      readonly deposit: u128;
    } & Struct;
    readonly isTabled: boolean;
    readonly asTabled: {
      readonly proposalIndex: u32;
      readonly deposit: u128;
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
    readonly isBlacklisted: boolean;
    readonly asBlacklisted: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isVoted: boolean;
    readonly asVoted: {
      readonly voter: AccountId20;
      readonly refIndex: u32;
      readonly vote: PalletDemocracyVoteAccountVote;
    } & Struct;
    readonly isSeconded: boolean;
    readonly asSeconded: {
      readonly seconder: AccountId20;
      readonly propIndex: u32;
    } & Struct;
    readonly isProposalCanceled: boolean;
    readonly asProposalCanceled: {
      readonly propIndex: u32;
    } & Struct;
    readonly type:
      | "Proposed"
      | "Tabled"
      | "ExternalTabled"
      | "Started"
      | "Passed"
      | "NotPassed"
      | "Cancelled"
      | "Delegated"
      | "Undelegated"
      | "Vetoed"
      | "Blacklisted"
      | "Voted"
      | "Seconded"
      | "ProposalCanceled";
  }

  /**
   * @name PalletDemocracyVoteThreshold (75)
   */
  interface PalletDemocracyVoteThreshold extends Enum {
    readonly isSuperMajorityApprove: boolean;
    readonly isSuperMajorityAgainst: boolean;
    readonly isSimpleMajority: boolean;
    readonly type: "SuperMajorityApprove" | "SuperMajorityAgainst" | "SimpleMajority";
  }

  /**
   * @name PalletDemocracyVoteAccountVote (76)
   */
  interface PalletDemocracyVoteAccountVote extends Enum {
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
    readonly type: "Standard" | "Split";
  }

  /**
   * @name PalletPreimageEvent (78)
   */
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

  /**
   * @name PalletCollectiveEvent (79)
   */
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
    readonly type:
      | "Proposed"
      | "Voted"
      | "Approved"
      | "Disapproved"
      | "Executed"
      | "MemberExecuted"
      | "Closed";
  }

  /**
   * @name PalletTreasuryEvent (82)
   */
  interface PalletTreasuryEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly proposalIndex: u32;
    } & Struct;
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
    readonly isRejected: boolean;
    readonly asRejected: {
      readonly proposalIndex: u32;
      readonly slashed: u128;
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
    readonly type:
      | "Proposed"
      | "Spending"
      | "Awarded"
      | "Rejected"
      | "Burnt"
      | "Rollover"
      | "Deposit"
      | "SpendApproved";
  }

  /**
   * @name PalletCrowdloanRewardsEvent (83)
   */
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

  /**
   * @name CumulusPalletXcmpQueueEvent (84)
   */
  interface CumulusPalletXcmpQueueEvent extends Enum {
    readonly isSuccess: boolean;
    readonly asSuccess: {
      readonly messageHash: Option<H256>;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isFail: boolean;
    readonly asFail: {
      readonly messageHash: Option<H256>;
      readonly error: XcmV2TraitsError;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isBadVersion: boolean;
    readonly asBadVersion: {
      readonly messageHash: Option<H256>;
    } & Struct;
    readonly isBadFormat: boolean;
    readonly asBadFormat: {
      readonly messageHash: Option<H256>;
    } & Struct;
    readonly isUpwardMessageSent: boolean;
    readonly asUpwardMessageSent: {
      readonly messageHash: Option<H256>;
    } & Struct;
    readonly isXcmpMessageSent: boolean;
    readonly asXcmpMessageSent: {
      readonly messageHash: Option<H256>;
    } & Struct;
    readonly isOverweightEnqueued: boolean;
    readonly asOverweightEnqueued: {
      readonly sender: u32;
      readonly sentAt: u32;
      readonly index: u64;
      readonly required: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isOverweightServiced: boolean;
    readonly asOverweightServiced: {
      readonly index: u64;
      readonly used: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type:
      | "Success"
      | "Fail"
      | "BadVersion"
      | "BadFormat"
      | "UpwardMessageSent"
      | "XcmpMessageSent"
      | "OverweightEnqueued"
      | "OverweightServiced";
  }

  /**
   * @name XcmV2TraitsError (86)
   */
  interface XcmV2TraitsError extends Enum {
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
    readonly isMaxWeightInvalid: boolean;
    readonly isNotHoldingFees: boolean;
    readonly isTooExpensive: boolean;
    readonly isTrap: boolean;
    readonly asTrap: u64;
    readonly isUnhandledXcmVersion: boolean;
    readonly isWeightLimitReached: boolean;
    readonly asWeightLimitReached: u64;
    readonly isBarrier: boolean;
    readonly isWeightNotComputable: boolean;
    readonly type:
      | "Overflow"
      | "Unimplemented"
      | "UntrustedReserveLocation"
      | "UntrustedTeleportLocation"
      | "MultiLocationFull"
      | "MultiLocationNotInvertible"
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
      | "UnhandledXcmVersion"
      | "WeightLimitReached"
      | "Barrier"
      | "WeightNotComputable";
  }

  /**
   * @name CumulusPalletXcmEvent (88)
   */
  interface CumulusPalletXcmEvent extends Enum {
    readonly isInvalidFormat: boolean;
    readonly asInvalidFormat: U8aFixed;
    readonly isUnsupportedVersion: boolean;
    readonly asUnsupportedVersion: U8aFixed;
    readonly isExecutedDownward: boolean;
    readonly asExecutedDownward: ITuple<[U8aFixed, XcmV2TraitsOutcome]>;
    readonly type: "InvalidFormat" | "UnsupportedVersion" | "ExecutedDownward";
  }

  /**
   * @name XcmV2TraitsOutcome (90)
   */
  interface XcmV2TraitsOutcome extends Enum {
    readonly isComplete: boolean;
    readonly asComplete: u64;
    readonly isIncomplete: boolean;
    readonly asIncomplete: ITuple<[u64, XcmV2TraitsError]>;
    readonly isError: boolean;
    readonly asError: XcmV2TraitsError;
    readonly type: "Complete" | "Incomplete" | "Error";
  }

  /**
   * @name CumulusPalletDmpQueueEvent (91)
   */
  interface CumulusPalletDmpQueueEvent extends Enum {
    readonly isInvalidFormat: boolean;
    readonly asInvalidFormat: {
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isUnsupportedVersion: boolean;
    readonly asUnsupportedVersion: {
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isExecutedDownward: boolean;
    readonly asExecutedDownward: {
      readonly messageId: U8aFixed;
      readonly outcome: XcmV2TraitsOutcome;
    } & Struct;
    readonly isWeightExhausted: boolean;
    readonly asWeightExhausted: {
      readonly messageId: U8aFixed;
      readonly remainingWeight: SpWeightsWeightV2Weight;
      readonly requiredWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isOverweightEnqueued: boolean;
    readonly asOverweightEnqueued: {
      readonly messageId: U8aFixed;
      readonly overweightIndex: u64;
      readonly requiredWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isOverweightServiced: boolean;
    readonly asOverweightServiced: {
      readonly overweightIndex: u64;
      readonly weightUsed: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type:
      | "InvalidFormat"
      | "UnsupportedVersion"
      | "ExecutedDownward"
      | "WeightExhausted"
      | "OverweightEnqueued"
      | "OverweightServiced";
  }

  /**
   * @name PalletXcmEvent (92)
   */
  interface PalletXcmEvent extends Enum {
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
    readonly asNotifyOverweight: ITuple<
      [u64, u8, u8, SpWeightsWeightV2Weight, SpWeightsWeightV2Weight]
    >;
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
    readonly isAssetsClaimed: boolean;
    readonly asAssetsClaimed: ITuple<[H256, XcmV1MultiLocation, XcmVersionedMultiAssets]>;
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
      | "AssetsClaimed";
  }

  /**
   * @name XcmV1MultiLocation (93)
   */
  interface XcmV1MultiLocation extends Struct {
    readonly parents: u8;
    readonly interior: XcmV1MultilocationJunctions;
  }

  /**
   * @name XcmV1MultilocationJunctions (94)
   */
  interface XcmV1MultilocationJunctions extends Enum {
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
    readonly asX5: ITuple<
      [XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction]
    >;
    readonly isX6: boolean;
    readonly asX6: ITuple<
      [XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction, XcmV1Junction]
    >;
    readonly isX7: boolean;
    readonly asX7: ITuple<
      [
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction
      ]
    >;
    readonly isX8: boolean;
    readonly asX8: ITuple<
      [
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction,
        XcmV1Junction
      ]
    >;
    readonly type: "Here" | "X1" | "X2" | "X3" | "X4" | "X5" | "X6" | "X7" | "X8";
  }

  /**
   * @name XcmV1Junction (95)
   */
  interface XcmV1Junction extends Enum {
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
    readonly type:
      | "Parachain"
      | "AccountId32"
      | "AccountIndex64"
      | "AccountKey20"
      | "PalletInstance"
      | "GeneralIndex"
      | "GeneralKey"
      | "OnlyChild"
      | "Plurality";
  }

  /**
   * @name XcmV0JunctionNetworkId (97)
   */
  interface XcmV0JunctionNetworkId extends Enum {
    readonly isAny: boolean;
    readonly isNamed: boolean;
    readonly asNamed: Bytes;
    readonly isPolkadot: boolean;
    readonly isKusama: boolean;
    readonly type: "Any" | "Named" | "Polkadot" | "Kusama";
  }

  /**
   * @name XcmV0JunctionBodyId (100)
   */
  interface XcmV0JunctionBodyId extends Enum {
    readonly isUnit: boolean;
    readonly isNamed: boolean;
    readonly asNamed: Bytes;
    readonly isIndex: boolean;
    readonly asIndex: Compact<u32>;
    readonly isExecutive: boolean;
    readonly isTechnical: boolean;
    readonly isLegislative: boolean;
    readonly isJudicial: boolean;
    readonly type:
      | "Unit"
      | "Named"
      | "Index"
      | "Executive"
      | "Technical"
      | "Legislative"
      | "Judicial";
  }

  /**
   * @name XcmV0JunctionBodyPart (101)
   */
  interface XcmV0JunctionBodyPart extends Enum {
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

  /**
   * @name XcmV2Xcm (102)
   */
  interface XcmV2Xcm extends Vec<XcmV2Instruction> {}

  /**
   * @name XcmV2Instruction (104)
   */
  interface XcmV2Instruction extends Enum {
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
      | "QueryHolding"
      | "BuyExecution"
      | "RefundSurplus"
      | "SetErrorHandler"
      | "SetAppendix"
      | "ClearError"
      | "ClaimAsset"
      | "Trap"
      | "SubscribeVersion"
      | "UnsubscribeVersion";
  }

  /**
   * @name XcmV1MultiassetMultiAssets (105)
   */
  interface XcmV1MultiassetMultiAssets extends Vec<XcmV1MultiAsset> {}

  /**
   * @name XcmV1MultiAsset (107)
   */
  interface XcmV1MultiAsset extends Struct {
    readonly id: XcmV1MultiassetAssetId;
    readonly fun: XcmV1MultiassetFungibility;
  }

  /**
   * @name XcmV1MultiassetAssetId (108)
   */
  interface XcmV1MultiassetAssetId extends Enum {
    readonly isConcrete: boolean;
    readonly asConcrete: XcmV1MultiLocation;
    readonly isAbstract: boolean;
    readonly asAbstract: Bytes;
    readonly type: "Concrete" | "Abstract";
  }

  /**
   * @name XcmV1MultiassetFungibility (109)
   */
  interface XcmV1MultiassetFungibility extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: Compact<u128>;
    readonly isNonFungible: boolean;
    readonly asNonFungible: XcmV1MultiassetAssetInstance;
    readonly type: "Fungible" | "NonFungible";
  }

  /**
   * @name XcmV1MultiassetAssetInstance (110)
   */
  interface XcmV1MultiassetAssetInstance extends Enum {
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
    readonly type: "Undefined" | "Index" | "Array4" | "Array8" | "Array16" | "Array32" | "Blob";
  }

  /**
   * @name XcmV2Response (112)
   */
  interface XcmV2Response extends Enum {
    readonly isNull: boolean;
    readonly isAssets: boolean;
    readonly asAssets: XcmV1MultiassetMultiAssets;
    readonly isExecutionResult: boolean;
    readonly asExecutionResult: Option<ITuple<[u32, XcmV2TraitsError]>>;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly type: "Null" | "Assets" | "ExecutionResult" | "Version";
  }

  /**
   * @name XcmV0OriginKind (115)
   */
  interface XcmV0OriginKind extends Enum {
    readonly isNative: boolean;
    readonly isSovereignAccount: boolean;
    readonly isSuperuser: boolean;
    readonly isXcm: boolean;
    readonly type: "Native" | "SovereignAccount" | "Superuser" | "Xcm";
  }

  /**
   * @name XcmDoubleEncoded (116)
   */
  interface XcmDoubleEncoded extends Struct {
    readonly encoded: Bytes;
  }

  /**
   * @name XcmV1MultiassetMultiAssetFilter (117)
   */
  interface XcmV1MultiassetMultiAssetFilter extends Enum {
    readonly isDefinite: boolean;
    readonly asDefinite: XcmV1MultiassetMultiAssets;
    readonly isWild: boolean;
    readonly asWild: XcmV1MultiassetWildMultiAsset;
    readonly type: "Definite" | "Wild";
  }

  /**
   * @name XcmV1MultiassetWildMultiAsset (118)
   */
  interface XcmV1MultiassetWildMultiAsset extends Enum {
    readonly isAll: boolean;
    readonly isAllOf: boolean;
    readonly asAllOf: {
      readonly id: XcmV1MultiassetAssetId;
      readonly fun: XcmV1MultiassetWildFungibility;
    } & Struct;
    readonly type: "All" | "AllOf";
  }

  /**
   * @name XcmV1MultiassetWildFungibility (119)
   */
  interface XcmV1MultiassetWildFungibility extends Enum {
    readonly isFungible: boolean;
    readonly isNonFungible: boolean;
    readonly type: "Fungible" | "NonFungible";
  }

  /**
   * @name XcmV2WeightLimit (120)
   */
  interface XcmV2WeightLimit extends Enum {
    readonly isUnlimited: boolean;
    readonly isLimited: boolean;
    readonly asLimited: Compact<u64>;
    readonly type: "Unlimited" | "Limited";
  }

  /**
   * @name XcmVersionedMultiAssets (122)
   */
  interface XcmVersionedMultiAssets extends Enum {
    readonly isV0: boolean;
    readonly asV0: Vec<XcmV0MultiAsset>;
    readonly isV1: boolean;
    readonly asV1: XcmV1MultiassetMultiAssets;
    readonly type: "V0" | "V1";
  }

  /**
   * @name XcmV0MultiAsset (124)
   */
  interface XcmV0MultiAsset extends Enum {
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
    readonly type:
      | "None"
      | "All"
      | "AllFungible"
      | "AllNonFungible"
      | "AllAbstractFungible"
      | "AllAbstractNonFungible"
      | "AllConcreteFungible"
      | "AllConcreteNonFungible"
      | "AbstractFungible"
      | "AbstractNonFungible"
      | "ConcreteFungible"
      | "ConcreteNonFungible";
  }

  /**
   * @name XcmV0MultiLocation (125)
   */
  interface XcmV0MultiLocation extends Enum {
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
    readonly asX5: ITuple<
      [XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction]
    >;
    readonly isX6: boolean;
    readonly asX6: ITuple<
      [XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction, XcmV0Junction]
    >;
    readonly isX7: boolean;
    readonly asX7: ITuple<
      [
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction
      ]
    >;
    readonly isX8: boolean;
    readonly asX8: ITuple<
      [
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction,
        XcmV0Junction
      ]
    >;
    readonly type: "Null" | "X1" | "X2" | "X3" | "X4" | "X5" | "X6" | "X7" | "X8";
  }

  /**
   * @name XcmV0Junction (126)
   */
  interface XcmV0Junction extends Enum {
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
    readonly type:
      | "Parent"
      | "Parachain"
      | "AccountId32"
      | "AccountIndex64"
      | "AccountKey20"
      | "PalletInstance"
      | "GeneralIndex"
      | "GeneralKey"
      | "OnlyChild"
      | "Plurality";
  }

  /**
   * @name XcmVersionedMultiLocation (127)
   */
  interface XcmVersionedMultiLocation extends Enum {
    readonly isV0: boolean;
    readonly asV0: XcmV0MultiLocation;
    readonly isV1: boolean;
    readonly asV1: XcmV1MultiLocation;
    readonly type: "V0" | "V1";
  }

  /**
   * @name PalletAssetsEvent (128)
   */
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
      | "Destroyed"
      | "ForceCreated"
      | "MetadataSet"
      | "MetadataCleared"
      | "ApprovedTransfer"
      | "ApprovalCancelled"
      | "TransferredApproved"
      | "AssetStatusChanged";
  }

  /**
   * @name PalletAssetManagerEvent (129)
   */
  interface PalletAssetManagerEvent extends Enum {
    readonly isForeignAssetRegistered: boolean;
    readonly asForeignAssetRegistered: {
      readonly assetId: u128;
      readonly asset: MoonbeamRuntimeXcmConfigAssetType;
      readonly metadata: MoonbeamRuntimeAssetConfigAssetRegistrarMetadata;
    } & Struct;
    readonly isUnitsPerSecondChanged: boolean;
    readonly asUnitsPerSecondChanged: {
      readonly assetType: MoonbeamRuntimeXcmConfigAssetType;
      readonly unitsPerSecond: u128;
    } & Struct;
    readonly isForeignAssetTypeChanged: boolean;
    readonly asForeignAssetTypeChanged: {
      readonly assetId: u128;
      readonly newAssetType: MoonbeamRuntimeXcmConfigAssetType;
    } & Struct;
    readonly isForeignAssetRemoved: boolean;
    readonly asForeignAssetRemoved: {
      readonly assetId: u128;
      readonly assetType: MoonbeamRuntimeXcmConfigAssetType;
    } & Struct;
    readonly isSupportedAssetRemoved: boolean;
    readonly asSupportedAssetRemoved: {
      readonly assetType: MoonbeamRuntimeXcmConfigAssetType;
    } & Struct;
    readonly isLocalAssetRegistered: boolean;
    readonly asLocalAssetRegistered: {
      readonly assetId: u128;
      readonly creator: AccountId20;
      readonly owner: AccountId20;
    } & Struct;
    readonly isForeignAssetDestroyed: boolean;
    readonly asForeignAssetDestroyed: {
      readonly assetId: u128;
      readonly assetType: MoonbeamRuntimeXcmConfigAssetType;
    } & Struct;
    readonly isLocalAssetDestroyed: boolean;
    readonly asLocalAssetDestroyed: {
      readonly assetId: u128;
    } & Struct;
    readonly type:
      | "ForeignAssetRegistered"
      | "UnitsPerSecondChanged"
      | "ForeignAssetTypeChanged"
      | "ForeignAssetRemoved"
      | "SupportedAssetRemoved"
      | "LocalAssetRegistered"
      | "ForeignAssetDestroyed"
      | "LocalAssetDestroyed";
  }

  /**
   * @name MoonbeamRuntimeXcmConfigAssetType (130)
   */
  interface MoonbeamRuntimeXcmConfigAssetType extends Enum {
    readonly isXcm: boolean;
    readonly asXcm: XcmV1MultiLocation;
    readonly type: "Xcm";
  }

  /**
   * @name MoonbeamRuntimeAssetConfigAssetRegistrarMetadata (131)
   */
  interface MoonbeamRuntimeAssetConfigAssetRegistrarMetadata extends Struct {
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly decimals: u8;
    readonly isFrozen: bool;
  }

  /**
   * @name OrmlXtokensModuleEvent (132)
   */
  interface OrmlXtokensModuleEvent extends Enum {
    readonly isTransferredMultiAssets: boolean;
    readonly asTransferredMultiAssets: {
      readonly sender: AccountId20;
      readonly assets: XcmV1MultiassetMultiAssets;
      readonly fee: XcmV1MultiAsset;
      readonly dest: XcmV1MultiLocation;
    } & Struct;
    readonly type: "TransferredMultiAssets";
  }

  /**
   * @name PalletXcmTransactorEvent (133)
   */
  interface PalletXcmTransactorEvent extends Enum {
    readonly isTransactedDerivative: boolean;
    readonly asTransactedDerivative: {
      readonly accountId: AccountId20;
      readonly dest: XcmV1MultiLocation;
      readonly call: Bytes;
      readonly index: u16;
    } & Struct;
    readonly isTransactedSovereign: boolean;
    readonly asTransactedSovereign: {
      readonly feePayer: AccountId20;
      readonly dest: XcmV1MultiLocation;
      readonly call: Bytes;
    } & Struct;
    readonly isTransactedSigned: boolean;
    readonly asTransactedSigned: {
      readonly feePayer: AccountId20;
      readonly dest: XcmV1MultiLocation;
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
      readonly error: XcmV2TraitsError;
    } & Struct;
    readonly isTransactInfoChanged: boolean;
    readonly asTransactInfoChanged: {
      readonly location: XcmV1MultiLocation;
      readonly remoteInfo: PalletXcmTransactorRemoteTransactInfoWithMaxWeight;
    } & Struct;
    readonly isTransactInfoRemoved: boolean;
    readonly asTransactInfoRemoved: {
      readonly location: XcmV1MultiLocation;
    } & Struct;
    readonly isDestFeePerSecondChanged: boolean;
    readonly asDestFeePerSecondChanged: {
      readonly location: XcmV1MultiLocation;
      readonly feePerSecond: u128;
    } & Struct;
    readonly isDestFeePerSecondRemoved: boolean;
    readonly asDestFeePerSecondRemoved: {
      readonly location: XcmV1MultiLocation;
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

  /**
   * @name PalletXcmTransactorRemoteTransactInfoWithMaxWeight (134)
   */
  interface PalletXcmTransactorRemoteTransactInfoWithMaxWeight extends Struct {
    readonly transactExtraWeight: u64;
    readonly maxWeight: u64;
    readonly transactExtraWeightSigned: Option<u64>;
  }

  /**
   * @name PalletXcmTransactorHrmpOperation (136)
   */
  interface PalletXcmTransactorHrmpOperation extends Enum {
    readonly isInitOpen: boolean;
    readonly asInitOpen: PalletXcmTransactorHrmpInitParams;
    readonly isAccept: boolean;
    readonly asAccept: {
      readonly paraId: u32;
    } & Struct;
    readonly isClose: boolean;
    readonly asClose: PolkadotParachainPrimitivesHrmpChannelId;
    readonly type: "InitOpen" | "Accept" | "Close";
  }

  /**
   * @name PalletXcmTransactorHrmpInitParams (137)
   */
  interface PalletXcmTransactorHrmpInitParams extends Struct {
    readonly paraId: u32;
    readonly proposedMaxCapacity: u32;
    readonly proposedMaxMessageSize: u32;
  }

  /**
   * @name PolkadotParachainPrimitivesHrmpChannelId (138)
   */
  interface PolkadotParachainPrimitivesHrmpChannelId extends Struct {
    readonly sender: u32;
    readonly recipient: u32;
  }

  /**
   * @name PalletRandomnessEvent (140)
   */
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

  /**
   * @name FrameSystemPhase (141)
   */
  interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean;
    readonly asApplyExtrinsic: u32;
    readonly isFinalization: boolean;
    readonly isInitialization: boolean;
    readonly type: "ApplyExtrinsic" | "Finalization" | "Initialization";
  }

  /**
   * @name FrameSystemLastRuntimeUpgradeInfo (143)
   */
  interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>;
    readonly specName: Text;
  }

  /**
   * @name FrameSystemCall (144)
   */
  interface FrameSystemCall extends Enum {
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
    readonly type:
      | "FillBlock"
      | "Remark"
      | "SetHeapPages"
      | "SetCode"
      | "SetCodeWithoutChecks"
      | "SetStorage"
      | "KillStorage"
      | "KillPrefix"
      | "RemarkWithEvent";
  }

  /**
   * @name FrameSystemLimitsBlockWeights (148)
   */
  interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: SpWeightsWeightV2Weight;
    readonly maxBlock: SpWeightsWeightV2Weight;
    readonly perClass: FrameSupportDispatchPerDispatchClassWeightsPerClass;
  }

  /**
   * @name FrameSupportDispatchPerDispatchClassWeightsPerClass (149)
   */
  interface FrameSupportDispatchPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass;
    readonly operational: FrameSystemLimitsWeightsPerClass;
    readonly mandatory: FrameSystemLimitsWeightsPerClass;
  }

  /**
   * @name FrameSystemLimitsWeightsPerClass (150)
   */
  interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: SpWeightsWeightV2Weight;
    readonly maxExtrinsic: Option<SpWeightsWeightV2Weight>;
    readonly maxTotal: Option<SpWeightsWeightV2Weight>;
    readonly reserved: Option<SpWeightsWeightV2Weight>;
  }

  /**
   * @name FrameSystemLimitsBlockLength (152)
   */
  interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportDispatchPerDispatchClassU32;
  }

  /**
   * @name FrameSupportDispatchPerDispatchClassU32 (153)
   */
  interface FrameSupportDispatchPerDispatchClassU32 extends Struct {
    readonly normal: u32;
    readonly operational: u32;
    readonly mandatory: u32;
  }

  /**
   * @name SpWeightsRuntimeDbWeight (154)
   */
  interface SpWeightsRuntimeDbWeight extends Struct {
    readonly read: u64;
    readonly write: u64;
  }

  /**
   * @name SpVersionRuntimeVersion (155)
   */
  interface SpVersionRuntimeVersion extends Struct {
    readonly specName: Text;
    readonly implName: Text;
    readonly authoringVersion: u32;
    readonly specVersion: u32;
    readonly implVersion: u32;
    readonly apis: Vec<ITuple<[U8aFixed, u32]>>;
    readonly transactionVersion: u32;
    readonly stateVersion: u8;
  }

  /**
   * @name FrameSystemError (159)
   */
  interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean;
    readonly isSpecVersionNeedsToIncrease: boolean;
    readonly isFailedToExtractRuntimeVersion: boolean;
    readonly isNonDefaultComposite: boolean;
    readonly isNonZeroRefCount: boolean;
    readonly isCallFiltered: boolean;
    readonly type:
      | "InvalidSpecName"
      | "SpecVersionNeedsToIncrease"
      | "FailedToExtractRuntimeVersion"
      | "NonDefaultComposite"
      | "NonZeroRefCount"
      | "CallFiltered";
  }

  /**
   * @name PolkadotPrimitivesV2PersistedValidationData (160)
   */
  interface PolkadotPrimitivesV2PersistedValidationData extends Struct {
    readonly parentHead: Bytes;
    readonly relayParentNumber: u32;
    readonly relayParentStorageRoot: H256;
    readonly maxPovSize: u32;
  }

  /**
   * @name PolkadotPrimitivesV2UpgradeRestriction (163)
   */
  interface PolkadotPrimitivesV2UpgradeRestriction extends Enum {
    readonly isPresent: boolean;
    readonly type: "Present";
  }

  /**
   * @name SpTrieStorageProof (164)
   */
  interface SpTrieStorageProof extends Struct {
    readonly trieNodes: BTreeSet<Bytes>;
  }

  /**
   * @name CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot (166)
   */
  interface CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot extends Struct {
    readonly dmqMqcHead: H256;
    readonly relayDispatchQueueSize: ITuple<[u32, u32]>;
    readonly ingressChannels: Vec<ITuple<[u32, PolkadotPrimitivesV2AbridgedHrmpChannel]>>;
    readonly egressChannels: Vec<ITuple<[u32, PolkadotPrimitivesV2AbridgedHrmpChannel]>>;
  }

  /**
   * @name PolkadotPrimitivesV2AbridgedHrmpChannel (169)
   */
  interface PolkadotPrimitivesV2AbridgedHrmpChannel extends Struct {
    readonly maxCapacity: u32;
    readonly maxTotalSize: u32;
    readonly maxMessageSize: u32;
    readonly msgCount: u32;
    readonly totalSize: u32;
    readonly mqcHead: Option<H256>;
  }

  /**
   * @name PolkadotPrimitivesV2AbridgedHostConfiguration (170)
   */
  interface PolkadotPrimitivesV2AbridgedHostConfiguration extends Struct {
    readonly maxCodeSize: u32;
    readonly maxHeadDataSize: u32;
    readonly maxUpwardQueueCount: u32;
    readonly maxUpwardQueueSize: u32;
    readonly maxUpwardMessageSize: u32;
    readonly maxUpwardMessageNumPerCandidate: u32;
    readonly hrmpMaxMessageNumPerCandidate: u32;
    readonly validationUpgradeCooldown: u32;
    readonly validationUpgradeDelay: u32;
  }

  /**
   * @name PolkadotCorePrimitivesOutboundHrmpMessage (176)
   */
  interface PolkadotCorePrimitivesOutboundHrmpMessage extends Struct {
    readonly recipient: u32;
    readonly data: Bytes;
  }

  /**
   * @name CumulusPalletParachainSystemCall (177)
   */
  interface CumulusPalletParachainSystemCall extends Enum {
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
    readonly type:
      | "SetValidationData"
      | "SudoSendUpwardMessage"
      | "AuthorizeUpgrade"
      | "EnactAuthorizedUpgrade";
  }

  /**
   * @name CumulusPrimitivesParachainInherentParachainInherentData (178)
   */
  interface CumulusPrimitivesParachainInherentParachainInherentData extends Struct {
    readonly validationData: PolkadotPrimitivesV2PersistedValidationData;
    readonly relayChainState: SpTrieStorageProof;
    readonly downwardMessages: Vec<PolkadotCorePrimitivesInboundDownwardMessage>;
    readonly horizontalMessages: BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>;
  }

  /**
   * @name PolkadotCorePrimitivesInboundDownwardMessage (180)
   */
  interface PolkadotCorePrimitivesInboundDownwardMessage extends Struct {
    readonly sentAt: u32;
    readonly msg: Bytes;
  }

  /**
   * @name PolkadotCorePrimitivesInboundHrmpMessage (183)
   */
  interface PolkadotCorePrimitivesInboundHrmpMessage extends Struct {
    readonly sentAt: u32;
    readonly data: Bytes;
  }

  /**
   * @name CumulusPalletParachainSystemError (186)
   */
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

  /**
   * @name PalletTimestampCall (188)
   */
  interface PalletTimestampCall extends Enum {
    readonly isSet: boolean;
    readonly asSet: {
      readonly now: Compact<u64>;
    } & Struct;
    readonly type: "Set";
  }

  /**
   * @name PalletBalancesBalanceLock (190)
   */
  interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
    readonly reasons: PalletBalancesReasons;
  }

  /**
   * @name PalletBalancesReasons (191)
   */
  interface PalletBalancesReasons extends Enum {
    readonly isFee: boolean;
    readonly isMisc: boolean;
    readonly isAll: boolean;
    readonly type: "Fee" | "Misc" | "All";
  }

  /**
   * @name PalletBalancesReserveData (194)
   */
  interface PalletBalancesReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /**
   * @name PalletBalancesReleases (196)
   */
  interface PalletBalancesReleases extends Enum {
    readonly isV100: boolean;
    readonly isV200: boolean;
    readonly type: "V100" | "V200";
  }

  /**
   * @name PalletBalancesCall (197)
   */
  interface PalletBalancesCall extends Enum {
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
    readonly type:
      | "Transfer"
      | "SetBalance"
      | "ForceTransfer"
      | "TransferKeepAlive"
      | "TransferAll"
      | "ForceUnreserve";
  }

  /**
   * @name PalletBalancesError (198)
   */
  interface PalletBalancesError extends Enum {
    readonly isVestingBalance: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isKeepAlive: boolean;
    readonly isExistingVestingSchedule: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly type:
      | "VestingBalance"
      | "LiquidityRestrictions"
      | "InsufficientBalance"
      | "ExistentialDeposit"
      | "KeepAlive"
      | "ExistingVestingSchedule"
      | "DeadAccount"
      | "TooManyReserves";
  }

  /**
   * @name PalletTransactionPaymentReleases (200)
   */
  interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean;
    readonly isV2: boolean;
    readonly type: "V1Ancient" | "V2";
  }

  /**
   * @name PalletParachainStakingParachainBondConfig (201)
   */
  interface PalletParachainStakingParachainBondConfig extends Struct {
    readonly account: AccountId20;
    readonly percent: Percent;
  }

  /**
   * @name PalletParachainStakingRoundInfo (202)
   */
  interface PalletParachainStakingRoundInfo extends Struct {
    readonly current: u32;
    readonly first: u32;
    readonly length: u32;
  }

  /**
   * @name PalletParachainStakingDelegator (203)
   */
  interface PalletParachainStakingDelegator extends Struct {
    readonly id: AccountId20;
    readonly delegations: PalletParachainStakingSetOrderedSet;
    readonly total: u128;
    readonly lessTotal: u128;
    readonly status: PalletParachainStakingDelegatorStatus;
  }

  /**
   * @name PalletParachainStakingSetOrderedSet (204)
   */
  interface PalletParachainStakingSetOrderedSet extends Vec<PalletParachainStakingBond> {}

  /**
   * @name PalletParachainStakingBond (205)
   */
  interface PalletParachainStakingBond extends Struct {
    readonly owner: AccountId20;
    readonly amount: u128;
  }

  /**
   * @name PalletParachainStakingDelegatorStatus (207)
   */
  interface PalletParachainStakingDelegatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isLeaving: boolean;
    readonly asLeaving: u32;
    readonly type: "Active" | "Leaving";
  }

  /**
   * @name PalletParachainStakingCandidateMetadata (208)
   */
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

  /**
   * @name PalletParachainStakingCapacityStatus (209)
   */
  interface PalletParachainStakingCapacityStatus extends Enum {
    readonly isFull: boolean;
    readonly isEmpty: boolean;
    readonly isPartial: boolean;
    readonly type: "Full" | "Empty" | "Partial";
  }

  /**
   * @name PalletParachainStakingCandidateBondLessRequest (211)
   */
  interface PalletParachainStakingCandidateBondLessRequest extends Struct {
    readonly amount: u128;
    readonly whenExecutable: u32;
  }

  /**
   * @name PalletParachainStakingCollatorStatus (212)
   */
  interface PalletParachainStakingCollatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isIdle: boolean;
    readonly isLeaving: boolean;
    readonly asLeaving: u32;
    readonly type: "Active" | "Idle" | "Leaving";
  }

  /**
   * @name PalletParachainStakingDelegationRequestsScheduledRequest (214)
   */
  interface PalletParachainStakingDelegationRequestsScheduledRequest extends Struct {
    readonly delegator: AccountId20;
    readonly whenExecutable: u32;
    readonly action: PalletParachainStakingDelegationRequestsDelegationAction;
  }

  /**
   * @name PalletParachainStakingAutoCompoundAutoCompoundConfig (216)
   */
  interface PalletParachainStakingAutoCompoundAutoCompoundConfig extends Struct {
    readonly delegator: AccountId20;
    readonly value: Percent;
  }

  /**
   * @name PalletParachainStakingDelegations (217)
   */
  interface PalletParachainStakingDelegations extends Struct {
    readonly delegations: Vec<PalletParachainStakingBond>;
    readonly total: u128;
  }

  /**
   * @name PalletParachainStakingCollatorSnapshot (220)
   */
  interface PalletParachainStakingCollatorSnapshot extends Struct {
    readonly bond: u128;
    readonly delegations: Vec<PalletParachainStakingBondWithAutoCompound>;
    readonly total: u128;
  }

  /**
   * @name PalletParachainStakingBondWithAutoCompound (222)
   */
  interface PalletParachainStakingBondWithAutoCompound extends Struct {
    readonly owner: AccountId20;
    readonly amount: u128;
    readonly autoCompound: Percent;
  }

  /**
   * @name PalletParachainStakingDelayedPayout (223)
   */
  interface PalletParachainStakingDelayedPayout extends Struct {
    readonly roundIssuance: u128;
    readonly totalStakingReward: u128;
    readonly collatorCommission: Perbill;
  }

  /**
   * @name PalletParachainStakingInflationInflationInfo (224)
   */
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

  /**
   * @name PalletParachainStakingCall (227)
   */
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
    readonly isDelegateWithAutoCompound: boolean;
    readonly asDelegateWithAutoCompound: {
      readonly candidate: AccountId20;
      readonly amount: u128;
      readonly autoCompound: Percent;
      readonly candidateDelegationCount: u32;
      readonly candidateAutoCompoundingDelegationCount: u32;
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
    readonly isSetAutoCompound: boolean;
    readonly asSetAutoCompound: {
      readonly candidate: AccountId20;
      readonly value: Percent;
      readonly candidateAutoCompoundingDelegationCountHint: u32;
      readonly delegationCountHint: u32;
    } & Struct;
    readonly isHotfixRemoveDelegationRequestsExitedCandidates: boolean;
    readonly asHotfixRemoveDelegationRequestsExitedCandidates: {
      readonly candidates: Vec<AccountId20>;
    } & Struct;
    readonly type:
      | "SetStakingExpectations"
      | "SetInflation"
      | "SetParachainBondAccount"
      | "SetParachainBondReservePercent"
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
      | "Delegate"
      | "DelegateWithAutoCompound"
      | "ScheduleLeaveDelegators"
      | "ExecuteLeaveDelegators"
      | "CancelLeaveDelegators"
      | "ScheduleRevokeDelegation"
      | "DelegatorBondMore"
      | "ScheduleDelegatorBondLess"
      | "ExecuteDelegationRequest"
      | "CancelDelegationRequest"
      | "SetAutoCompound"
      | "HotfixRemoveDelegationRequestsExitedCandidates";
  }

  /**
   * @name PalletParachainStakingError (228)
   */
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
      | "TooLowCandidateAutoCompoundingDelegationCountToDelegate";
  }

  /**
   * @name PalletAuthorInherentCall (229)
   */
  interface PalletAuthorInherentCall extends Enum {
    readonly isKickOffAuthorshipValidation: boolean;
    readonly type: "KickOffAuthorshipValidation";
  }

  /**
   * @name PalletAuthorInherentError (230)
   */
  interface PalletAuthorInherentError extends Enum {
    readonly isAuthorAlreadySet: boolean;
    readonly isNoAccountId: boolean;
    readonly isCannotBeAuthor: boolean;
    readonly type: "AuthorAlreadySet" | "NoAccountId" | "CannotBeAuthor";
  }

  /**
   * @name PalletAuthorSlotFilterCall (231)
   */
  interface PalletAuthorSlotFilterCall extends Enum {
    readonly isSetEligible: boolean;
    readonly asSetEligible: {
      readonly new_: u32;
    } & Struct;
    readonly type: "SetEligible";
  }

  /**
   * @name PalletAuthorMappingRegistrationInfo (232)
   */
  interface PalletAuthorMappingRegistrationInfo extends Struct {
    readonly account: AccountId20;
    readonly deposit: u128;
    readonly keys_: SessionKeysPrimitivesVrfVrfCryptoPublic;
  }

  /**
   * @name PalletAuthorMappingCall (233)
   */
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

  /**
   * @name PalletAuthorMappingError (234)
   */
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

  /**
   * @name PalletMoonbeamOrbitersCollatorPoolInfo (235)
   */
  interface PalletMoonbeamOrbitersCollatorPoolInfo extends Struct {
    readonly orbiters: Vec<AccountId20>;
    readonly maybeCurrentOrbiter: Option<PalletMoonbeamOrbitersCurrentOrbiter>;
    readonly nextOrbiter: u32;
  }

  /**
   * @name PalletMoonbeamOrbitersCurrentOrbiter (237)
   */
  interface PalletMoonbeamOrbitersCurrentOrbiter extends Struct {
    readonly accountId: AccountId20;
    readonly removed: bool;
  }

  /**
   * @name PalletMoonbeamOrbitersCall (238)
   */
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

  /**
   * @name PalletMoonbeamOrbitersError (239)
   */
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

  /**
   * @name PalletUtilityCall (240)
   */
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
      readonly asOrigin: MoonbeamRuntimeOriginCaller;
      readonly call: Call;
    } & Struct;
    readonly isForceBatch: boolean;
    readonly asForceBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly type: "Batch" | "AsDerivative" | "BatchAll" | "DispatchAs" | "ForceBatch";
  }

  /**
   * @name PalletProxyCall (243)
   */
  interface PalletProxyCall extends Enum {
    readonly isProxy: boolean;
    readonly asProxy: {
      readonly real: AccountId20;
      readonly forceProxyType: Option<MoonbeamRuntimeProxyType>;
      readonly call: Call;
    } & Struct;
    readonly isAddProxy: boolean;
    readonly asAddProxy: {
      readonly delegate: AccountId20;
      readonly proxyType: MoonbeamRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly isRemoveProxy: boolean;
    readonly asRemoveProxy: {
      readonly delegate: AccountId20;
      readonly proxyType: MoonbeamRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly isRemoveProxies: boolean;
    readonly isCreatePure: boolean;
    readonly asCreatePure: {
      readonly proxyType: MoonbeamRuntimeProxyType;
      readonly delay: u32;
      readonly index: u16;
    } & Struct;
    readonly isKillPure: boolean;
    readonly asKillPure: {
      readonly spawner: AccountId20;
      readonly proxyType: MoonbeamRuntimeProxyType;
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
      readonly forceProxyType: Option<MoonbeamRuntimeProxyType>;
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

  /**
   * @name PalletMaintenanceModeCall (245)
   */
  interface PalletMaintenanceModeCall extends Enum {
    readonly isEnterMaintenanceMode: boolean;
    readonly isResumeNormalOperation: boolean;
    readonly type: "EnterMaintenanceMode" | "ResumeNormalOperation";
  }

  /**
   * @name PalletIdentityCall (246)
   */
  interface PalletIdentityCall extends Enum {
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
      | "QuitSub";
  }

  /**
   * @name PalletIdentityIdentityInfo (247)
   */
  interface PalletIdentityIdentityInfo extends Struct {
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

  /**
   * @name PalletIdentityBitFlags (283)
   */
  interface PalletIdentityBitFlags extends Set {
    readonly isDisplay: boolean;
    readonly isLegal: boolean;
    readonly isWeb: boolean;
    readonly isRiot: boolean;
    readonly isEmail: boolean;
    readonly isPgpFingerprint: boolean;
    readonly isImage: boolean;
    readonly isTwitter: boolean;
  }

  /**
   * @name PalletIdentityIdentityField (284)
   */
  interface PalletIdentityIdentityField extends Enum {
    readonly isDisplay: boolean;
    readonly isLegal: boolean;
    readonly isWeb: boolean;
    readonly isRiot: boolean;
    readonly isEmail: boolean;
    readonly isPgpFingerprint: boolean;
    readonly isImage: boolean;
    readonly isTwitter: boolean;
    readonly type:
      | "Display"
      | "Legal"
      | "Web"
      | "Riot"
      | "Email"
      | "PgpFingerprint"
      | "Image"
      | "Twitter";
  }

  /**
   * @name PalletIdentityJudgement (285)
   */
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

  /**
   * @name PalletMigrationsCall (286)
   */
  interface PalletMigrationsCall extends Enum {
    readonly isMigrateDemocracyPreimage: boolean;
    readonly asMigrateDemocracyPreimage: {
      readonly proposalHash: H256;
      readonly proposalLenUpperBound: Compact<u32>;
    } & Struct;
    readonly type: "MigrateDemocracyPreimage";
  }

  /**
   * @name PalletEvmCall (287)
   */
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

  /**
   * @name PalletEthereumCall (293)
   */
  interface PalletEthereumCall extends Enum {
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly transaction: EthereumTransactionTransactionV2;
    } & Struct;
    readonly type: "Transact";
  }

  /**
   * @name EthereumTransactionTransactionV2 (294)
   */
  interface EthereumTransactionTransactionV2 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumTransactionLegacyTransaction;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumTransactionEip2930Transaction;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumTransactionEip1559Transaction;
    readonly type: "Legacy" | "Eip2930" | "Eip1559";
  }

  /**
   * @name EthereumTransactionLegacyTransaction (295)
   */
  interface EthereumTransactionLegacyTransaction extends Struct {
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly signature: EthereumTransactionTransactionSignature;
  }

  /**
   * @name EthereumTransactionTransactionAction (296)
   */
  interface EthereumTransactionTransactionAction extends Enum {
    readonly isCall: boolean;
    readonly asCall: H160;
    readonly isCreate: boolean;
    readonly type: "Call" | "Create";
  }

  /**
   * @name EthereumTransactionTransactionSignature (297)
   */
  interface EthereumTransactionTransactionSignature extends Struct {
    readonly v: u64;
    readonly r: H256;
    readonly s: H256;
  }

  /**
   * @name EthereumTransactionEip2930Transaction (299)
   */
  interface EthereumTransactionEip2930Transaction extends Struct {
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

  /**
   * @name EthereumTransactionAccessListItem (301)
   */
  interface EthereumTransactionAccessListItem extends Struct {
    readonly address: H160;
    readonly storageKeys: Vec<H256>;
  }

  /**
   * @name EthereumTransactionEip1559Transaction (302)
   */
  interface EthereumTransactionEip1559Transaction extends Struct {
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

  /**
   * @name PalletSchedulerCall (303)
   */
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
    readonly type:
      | "Schedule"
      | "Cancel"
      | "ScheduleNamed"
      | "CancelNamed"
      | "ScheduleAfter"
      | "ScheduleNamedAfter";
  }

  /**
   * @name PalletDemocracyCall (305)
   */
  interface PalletDemocracyCall extends Enum {
    readonly isPropose: boolean;
    readonly asPropose: {
      readonly proposal: FrameSupportPreimagesBounded;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isSecond: boolean;
    readonly asSecond: {
      readonly proposal: Compact<u32>;
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
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isExternalProposeMajority: boolean;
    readonly asExternalProposeMajority: {
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isExternalProposeDefault: boolean;
    readonly asExternalProposeDefault: {
      readonly proposal: FrameSupportPreimagesBounded;
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
    readonly isDelegate: boolean;
    readonly asDelegate: {
      readonly to: AccountId20;
      readonly conviction: PalletDemocracyConviction;
      readonly balance: u128;
    } & Struct;
    readonly isUndelegate: boolean;
    readonly isClearPublicProposals: boolean;
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
    readonly isBlacklist: boolean;
    readonly asBlacklist: {
      readonly proposalHash: H256;
      readonly maybeRefIndex: Option<u32>;
    } & Struct;
    readonly isCancelProposal: boolean;
    readonly asCancelProposal: {
      readonly propIndex: Compact<u32>;
    } & Struct;
    readonly type:
      | "Propose"
      | "Second"
      | "Vote"
      | "EmergencyCancel"
      | "ExternalPropose"
      | "ExternalProposeMajority"
      | "ExternalProposeDefault"
      | "FastTrack"
      | "VetoExternal"
      | "CancelReferendum"
      | "Delegate"
      | "Undelegate"
      | "ClearPublicProposals"
      | "Unlock"
      | "RemoveVote"
      | "RemoveOtherVote"
      | "Blacklist"
      | "CancelProposal";
  }

  /**
   * @name FrameSupportPreimagesBounded (306)
   */
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

  /**
   * @name PalletDemocracyConviction (308)
   */
  interface PalletDemocracyConviction extends Enum {
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

  /**
   * @name PalletPreimageCall (310)
   */
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
    readonly type: "NotePreimage" | "UnnotePreimage" | "RequestPreimage" | "UnrequestPreimage";
  }

  /**
   * @name PalletCollectiveCall (311)
   */
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
    readonly isCloseOldWeight: boolean;
    readonly asCloseOldWeight: {
      readonly proposalHash: H256;
      readonly index: Compact<u32>;
      readonly proposalWeightBound: Compact<u64>;
      readonly lengthBound: Compact<u32>;
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
    readonly type:
      | "SetMembers"
      | "Execute"
      | "Propose"
      | "Vote"
      | "CloseOldWeight"
      | "DisapproveProposal"
      | "Close";
  }

  /**
   * @name PalletTreasuryCall (316)
   */
  interface PalletTreasuryCall extends Enum {
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
    readonly isSpend: boolean;
    readonly asSpend: {
      readonly amount: Compact<u128>;
      readonly beneficiary: AccountId20;
    } & Struct;
    readonly isRemoveApproval: boolean;
    readonly asRemoveApproval: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly type:
      | "ProposeSpend"
      | "RejectProposal"
      | "ApproveProposal"
      | "Spend"
      | "RemoveApproval";
  }

  /**
   * @name PalletCrowdloanRewardsCall (317)
   */
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

  /**
   * @name SpRuntimeMultiSignature (318)
   */
  interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean;
    readonly asEd25519: SpCoreEd25519Signature;
    readonly isSr25519: boolean;
    readonly asSr25519: SpCoreSr25519Signature;
    readonly isEcdsa: boolean;
    readonly asEcdsa: SpCoreEcdsaSignature;
    readonly type: "Ed25519" | "Sr25519" | "Ecdsa";
  }

  /**
   * @name SpCoreEd25519Signature (319)
   */
  interface SpCoreEd25519Signature extends U8aFixed {}

  /**
   * @name SpCoreSr25519Signature (321)
   */
  interface SpCoreSr25519Signature extends U8aFixed {}

  /**
   * @name SpCoreEcdsaSignature (322)
   */
  interface SpCoreEcdsaSignature extends U8aFixed {}

  /**
   * @name CumulusPalletDmpQueueCall (328)
   */
  interface CumulusPalletDmpQueueCall extends Enum {
    readonly isServiceOverweight: boolean;
    readonly asServiceOverweight: {
      readonly index: u64;
      readonly weightLimit: u64;
    } & Struct;
    readonly type: "ServiceOverweight";
  }

  /**
   * @name PalletXcmCall (329)
   */
  interface PalletXcmCall extends Enum {
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
      | "LimitedTeleportAssets";
  }

  /**
   * @name XcmVersionedXcm (330)
   */
  interface XcmVersionedXcm extends Enum {
    readonly isV0: boolean;
    readonly asV0: XcmV0Xcm;
    readonly isV1: boolean;
    readonly asV1: XcmV1Xcm;
    readonly isV2: boolean;
    readonly asV2: XcmV2Xcm;
    readonly type: "V0" | "V1" | "V2";
  }

  /**
   * @name XcmV0Xcm (331)
   */
  interface XcmV0Xcm extends Enum {
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
    readonly type:
      | "WithdrawAsset"
      | "ReserveAssetDeposit"
      | "TeleportAsset"
      | "QueryResponse"
      | "TransferAsset"
      | "TransferReserveAsset"
      | "Transact"
      | "HrmpNewChannelOpenRequest"
      | "HrmpChannelAccepted"
      | "HrmpChannelClosing"
      | "RelayedFrom";
  }

  /**
   * @name XcmV0Order (333)
   */
  interface XcmV0Order extends Enum {
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
    readonly type:
      | "Null"
      | "DepositAsset"
      | "DepositReserveAsset"
      | "ExchangeAsset"
      | "InitiateReserveWithdraw"
      | "InitiateTeleport"
      | "QueryHolding"
      | "BuyExecution";
  }

  /**
   * @name XcmV0Response (335)
   */
  interface XcmV0Response extends Enum {
    readonly isAssets: boolean;
    readonly asAssets: Vec<XcmV0MultiAsset>;
    readonly type: "Assets";
  }

  /**
   * @name XcmV1Xcm (336)
   */
  interface XcmV1Xcm extends Enum {
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
      | "RelayedFrom"
      | "SubscribeVersion"
      | "UnsubscribeVersion";
  }

  /**
   * @name XcmV1Order (338)
   */
  interface XcmV1Order extends Enum {
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
    readonly type:
      | "Noop"
      | "DepositAsset"
      | "DepositReserveAsset"
      | "ExchangeAsset"
      | "InitiateReserveWithdraw"
      | "InitiateTeleport"
      | "QueryHolding"
      | "BuyExecution";
  }

  /**
   * @name XcmV1Response (340)
   */
  interface XcmV1Response extends Enum {
    readonly isAssets: boolean;
    readonly asAssets: XcmV1MultiassetMultiAssets;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly type: "Assets" | "Version";
  }

  /**
   * @name PalletAssetsCall (354)
   */
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
    readonly isTouch: boolean;
    readonly asTouch: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isRefund: boolean;
    readonly asRefund: {
      readonly id: Compact<u128>;
      readonly allowBurn: bool;
    } & Struct;
    readonly type:
      | "Create"
      | "ForceCreate"
      | "Destroy"
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
      | "Refund";
  }

  /**
   * @name PalletAssetsDestroyWitness (355)
   */
  interface PalletAssetsDestroyWitness extends Struct {
    readonly accounts: Compact<u32>;
    readonly sufficients: Compact<u32>;
    readonly approvals: Compact<u32>;
  }

  /**
   * @name PalletAssetManagerCall (356)
   */
  interface PalletAssetManagerCall extends Enum {
    readonly isRegisterForeignAsset: boolean;
    readonly asRegisterForeignAsset: {
      readonly asset: MoonbeamRuntimeXcmConfigAssetType;
      readonly metadata: MoonbeamRuntimeAssetConfigAssetRegistrarMetadata;
      readonly minAmount: u128;
      readonly isSufficient: bool;
    } & Struct;
    readonly isSetAssetUnitsPerSecond: boolean;
    readonly asSetAssetUnitsPerSecond: {
      readonly assetType: MoonbeamRuntimeXcmConfigAssetType;
      readonly unitsPerSecond: u128;
      readonly numAssetsWeightHint: u32;
    } & Struct;
    readonly isChangeExistingAssetType: boolean;
    readonly asChangeExistingAssetType: {
      readonly assetId: u128;
      readonly newAssetType: MoonbeamRuntimeXcmConfigAssetType;
      readonly numAssetsWeightHint: u32;
    } & Struct;
    readonly isRemoveSupportedAsset: boolean;
    readonly asRemoveSupportedAsset: {
      readonly assetType: MoonbeamRuntimeXcmConfigAssetType;
      readonly numAssetsWeightHint: u32;
    } & Struct;
    readonly isRemoveExistingAssetType: boolean;
    readonly asRemoveExistingAssetType: {
      readonly assetId: u128;
      readonly numAssetsWeightHint: u32;
    } & Struct;
    readonly isRegisterLocalAsset: boolean;
    readonly asRegisterLocalAsset: {
      readonly creator: AccountId20;
      readonly owner: AccountId20;
      readonly isSufficient: bool;
      readonly minBalance: u128;
    } & Struct;
    readonly isDestroyForeignAsset: boolean;
    readonly asDestroyForeignAsset: {
      readonly assetId: u128;
      readonly destroyAssetWitness: PalletAssetsDestroyWitness;
      readonly numAssetsWeightHint: u32;
    } & Struct;
    readonly isDestroyLocalAsset: boolean;
    readonly asDestroyLocalAsset: {
      readonly assetId: u128;
      readonly destroyAssetWitness: PalletAssetsDestroyWitness;
    } & Struct;
    readonly type:
      | "RegisterForeignAsset"
      | "SetAssetUnitsPerSecond"
      | "ChangeExistingAssetType"
      | "RemoveSupportedAsset"
      | "RemoveExistingAssetType"
      | "RegisterLocalAsset"
      | "DestroyForeignAsset"
      | "DestroyLocalAsset";
  }

  /**
   * @name OrmlXtokensModuleCall (357)
   */
  interface OrmlXtokensModuleCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly currencyId: MoonbeamRuntimeXcmConfigCurrencyId;
      readonly amount: u128;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly isTransferMultiasset: boolean;
    readonly asTransferMultiasset: {
      readonly asset: XcmVersionedMultiAsset;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly isTransferWithFee: boolean;
    readonly asTransferWithFee: {
      readonly currencyId: MoonbeamRuntimeXcmConfigCurrencyId;
      readonly amount: u128;
      readonly fee: u128;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly isTransferMultiassetWithFee: boolean;
    readonly asTransferMultiassetWithFee: {
      readonly asset: XcmVersionedMultiAsset;
      readonly fee: XcmVersionedMultiAsset;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly isTransferMulticurrencies: boolean;
    readonly asTransferMulticurrencies: {
      readonly currencies: Vec<ITuple<[MoonbeamRuntimeXcmConfigCurrencyId, u128]>>;
      readonly feeItem: u32;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly isTransferMultiassets: boolean;
    readonly asTransferMultiassets: {
      readonly assets: XcmVersionedMultiAssets;
      readonly feeItem: u32;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly type:
      | "Transfer"
      | "TransferMultiasset"
      | "TransferWithFee"
      | "TransferMultiassetWithFee"
      | "TransferMulticurrencies"
      | "TransferMultiassets";
  }

  /**
   * @name MoonbeamRuntimeXcmConfigCurrencyId (358)
   */
  interface MoonbeamRuntimeXcmConfigCurrencyId extends Enum {
    readonly isSelfReserve: boolean;
    readonly isForeignAsset: boolean;
    readonly asForeignAsset: u128;
    readonly isLocalAssetReserve: boolean;
    readonly asLocalAssetReserve: u128;
    readonly type: "SelfReserve" | "ForeignAsset" | "LocalAssetReserve";
  }

  /**
   * @name XcmVersionedMultiAsset (359)
   */
  interface XcmVersionedMultiAsset extends Enum {
    readonly isV0: boolean;
    readonly asV0: XcmV0MultiAsset;
    readonly isV1: boolean;
    readonly asV1: XcmV1MultiAsset;
    readonly type: "V0" | "V1";
  }

  /**
   * @name PalletXcmTransactorCall (362)
   */
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
      readonly dest: MoonbeamRuntimeXcmConfigTransactors;
      readonly index: u16;
      readonly fee: PalletXcmTransactorCurrencyPayment;
      readonly innerCall: Bytes;
      readonly weightInfo: PalletXcmTransactorTransactWeights;
    } & Struct;
    readonly isTransactThroughSovereign: boolean;
    readonly asTransactThroughSovereign: {
      readonly dest: XcmVersionedMultiLocation;
      readonly feePayer: AccountId20;
      readonly fee: PalletXcmTransactorCurrencyPayment;
      readonly call: Bytes;
      readonly originKind: XcmV0OriginKind;
      readonly weightInfo: PalletXcmTransactorTransactWeights;
    } & Struct;
    readonly isSetTransactInfo: boolean;
    readonly asSetTransactInfo: {
      readonly location: XcmVersionedMultiLocation;
      readonly transactExtraWeight: u64;
      readonly maxWeight: u64;
      readonly transactExtraWeightSigned: Option<u64>;
    } & Struct;
    readonly isRemoveTransactInfo: boolean;
    readonly asRemoveTransactInfo: {
      readonly location: XcmVersionedMultiLocation;
    } & Struct;
    readonly isTransactThroughSigned: boolean;
    readonly asTransactThroughSigned: {
      readonly dest: XcmVersionedMultiLocation;
      readonly fee: PalletXcmTransactorCurrencyPayment;
      readonly call: Bytes;
      readonly weightInfo: PalletXcmTransactorTransactWeights;
    } & Struct;
    readonly isSetFeePerSecond: boolean;
    readonly asSetFeePerSecond: {
      readonly assetLocation: XcmVersionedMultiLocation;
      readonly feePerSecond: u128;
    } & Struct;
    readonly isRemoveFeePerSecond: boolean;
    readonly asRemoveFeePerSecond: {
      readonly assetLocation: XcmVersionedMultiLocation;
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

  /**
   * @name MoonbeamRuntimeXcmConfigTransactors (363)
   */
  interface MoonbeamRuntimeXcmConfigTransactors extends Enum {
    readonly isRelay: boolean;
    readonly type: "Relay";
  }

  /**
   * @name PalletXcmTransactorCurrencyPayment (364)
   */
  interface PalletXcmTransactorCurrencyPayment extends Struct {
    readonly currency: PalletXcmTransactorCurrency;
    readonly feeAmount: Option<u128>;
  }

  /**
   * @name PalletXcmTransactorCurrency (365)
   */
  interface PalletXcmTransactorCurrency extends Enum {
    readonly isAsCurrencyId: boolean;
    readonly asAsCurrencyId: MoonbeamRuntimeXcmConfigCurrencyId;
    readonly isAsMultiLocation: boolean;
    readonly asAsMultiLocation: XcmVersionedMultiLocation;
    readonly type: "AsCurrencyId" | "AsMultiLocation";
  }

  /**
   * @name PalletXcmTransactorTransactWeights (367)
   */
  interface PalletXcmTransactorTransactWeights extends Struct {
    readonly transactRequiredWeightAtMost: u64;
    readonly overallWeight: Option<u64>;
  }

  /**
   * @name PalletRandomnessCall (369)
   */
  interface PalletRandomnessCall extends Enum {
    readonly isSetBabeRandomnessResults: boolean;
    readonly type: "SetBabeRandomnessResults";
  }

  /**
   * @name MoonbeamRuntimeOriginCaller (370)
   */
  interface MoonbeamRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean;
    readonly asSystem: FrameSupportDispatchRawOrigin;
    readonly isVoid: boolean;
    readonly isEthereum: boolean;
    readonly asEthereum: PalletEthereumRawOrigin;
    readonly isCouncilCollective: boolean;
    readonly asCouncilCollective: PalletCollectiveRawOrigin;
    readonly isTechCommitteeCollective: boolean;
    readonly asTechCommitteeCollective: PalletCollectiveRawOrigin;
    readonly isTreasuryCouncilCollective: boolean;
    readonly asTreasuryCouncilCollective: PalletCollectiveRawOrigin;
    readonly isCumulusXcm: boolean;
    readonly asCumulusXcm: CumulusPalletXcmOrigin;
    readonly isPolkadotXcm: boolean;
    readonly asPolkadotXcm: PalletXcmOrigin;
    readonly type:
      | "System"
      | "Void"
      | "Ethereum"
      | "CouncilCollective"
      | "TechCommitteeCollective"
      | "TreasuryCouncilCollective"
      | "CumulusXcm"
      | "PolkadotXcm";
  }

  /**
   * @name FrameSupportDispatchRawOrigin (371)
   */
  interface FrameSupportDispatchRawOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId20;
    readonly isNone: boolean;
    readonly type: "Root" | "Signed" | "None";
  }

  /**
   * @name PalletEthereumRawOrigin (372)
   */
  interface PalletEthereumRawOrigin extends Enum {
    readonly isEthereumTransaction: boolean;
    readonly asEthereumTransaction: H160;
    readonly type: "EthereumTransaction";
  }

  /**
   * @name PalletCollectiveRawOrigin (373)
   */
  interface PalletCollectiveRawOrigin extends Enum {
    readonly isMembers: boolean;
    readonly asMembers: ITuple<[u32, u32]>;
    readonly isMember: boolean;
    readonly asMember: AccountId20;
    readonly isPhantom: boolean;
    readonly type: "Members" | "Member" | "Phantom";
  }

  /**
   * @name CumulusPalletXcmOrigin (376)
   */
  interface CumulusPalletXcmOrigin extends Enum {
    readonly isRelay: boolean;
    readonly isSiblingParachain: boolean;
    readonly asSiblingParachain: u32;
    readonly type: "Relay" | "SiblingParachain";
  }

  /**
   * @name PalletXcmOrigin (377)
   */
  interface PalletXcmOrigin extends Enum {
    readonly isXcm: boolean;
    readonly asXcm: XcmV1MultiLocation;
    readonly isResponse: boolean;
    readonly asResponse: XcmV1MultiLocation;
    readonly type: "Xcm" | "Response";
  }

  /**
   * @name SpCoreVoid (378)
   */
  type SpCoreVoid = Null;

  /**
   * @name PalletUtilityError (379)
   */
  interface PalletUtilityError extends Enum {
    readonly isTooManyCalls: boolean;
    readonly type: "TooManyCalls";
  }

  /**
   * @name PalletProxyProxyDefinition (382)
   */
  interface PalletProxyProxyDefinition extends Struct {
    readonly delegate: AccountId20;
    readonly proxyType: MoonbeamRuntimeProxyType;
    readonly delay: u32;
  }

  /**
   * @name PalletProxyAnnouncement (386)
   */
  interface PalletProxyAnnouncement extends Struct {
    readonly real: AccountId20;
    readonly callHash: H256;
    readonly height: u32;
  }

  /**
   * @name PalletProxyError (388)
   */
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

  /**
   * @name PalletMaintenanceModeError (389)
   */
  interface PalletMaintenanceModeError extends Enum {
    readonly isAlreadyInMaintenanceMode: boolean;
    readonly isNotInMaintenanceMode: boolean;
    readonly type: "AlreadyInMaintenanceMode" | "NotInMaintenanceMode";
  }

  /**
   * @name PalletIdentityRegistration (390)
   */
  interface PalletIdentityRegistration extends Struct {
    readonly judgements: Vec<ITuple<[u32, PalletIdentityJudgement]>>;
    readonly deposit: u128;
    readonly info: PalletIdentityIdentityInfo;
  }

  /**
   * @name PalletIdentityRegistrarInfo (398)
   */
  interface PalletIdentityRegistrarInfo extends Struct {
    readonly account: AccountId20;
    readonly fee: u128;
    readonly fields: PalletIdentityBitFlags;
  }

  /**
   * @name PalletIdentityError (400)
   */
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
    readonly isTooManyFields: boolean;
    readonly isTooManyRegistrars: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isNotSub: boolean;
    readonly isNotOwned: boolean;
    readonly isJudgementForDifferentIdentity: boolean;
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
      | "TooManyFields"
      | "TooManyRegistrars"
      | "AlreadyClaimed"
      | "NotSub"
      | "NotOwned"
      | "JudgementForDifferentIdentity";
  }

  /**
   * @name PalletMigrationsError (401)
   */
  interface PalletMigrationsError extends Enum {
    readonly isPreimageMissing: boolean;
    readonly isWrongUpperBound: boolean;
    readonly isPreimageIsTooBig: boolean;
    readonly isPreimageAlreadyExists: boolean;
    readonly type:
      | "PreimageMissing"
      | "WrongUpperBound"
      | "PreimageIsTooBig"
      | "PreimageAlreadyExists";
  }

  /**
   * @name PalletEvmError (403)
   */
  interface PalletEvmError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isFeeOverflow: boolean;
    readonly isPaymentOverflow: boolean;
    readonly isWithdrawFailed: boolean;
    readonly isGasPriceTooLow: boolean;
    readonly isInvalidNonce: boolean;
    readonly isGasLimitTooLow: boolean;
    readonly isGasLimitTooHigh: boolean;
    readonly isUndefined: boolean;
    readonly isReentrancy: boolean;
    readonly type:
      | "BalanceLow"
      | "FeeOverflow"
      | "PaymentOverflow"
      | "WithdrawFailed"
      | "GasPriceTooLow"
      | "InvalidNonce"
      | "GasLimitTooLow"
      | "GasLimitTooHigh"
      | "Undefined"
      | "Reentrancy";
  }

  /**
   * @name FpRpcTransactionStatus (406)
   */
  interface FpRpcTransactionStatus extends Struct {
    readonly transactionHash: H256;
    readonly transactionIndex: u32;
    readonly from: H160;
    readonly to: Option<H160>;
    readonly contractAddress: Option<H160>;
    readonly logs: Vec<EthereumLog>;
    readonly logsBloom: EthbloomBloom;
  }

  /**
   * @name EthbloomBloom (409)
   */
  interface EthbloomBloom extends U8aFixed {}

  /**
   * @name EthereumReceiptReceiptV3 (411)
   */
  interface EthereumReceiptReceiptV3 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumReceiptEip658ReceiptData;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumReceiptEip658ReceiptData;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumReceiptEip658ReceiptData;
    readonly type: "Legacy" | "Eip2930" | "Eip1559";
  }

  /**
   * @name EthereumReceiptEip658ReceiptData (412)
   */
  interface EthereumReceiptEip658ReceiptData extends Struct {
    readonly statusCode: u8;
    readonly usedGas: U256;
    readonly logsBloom: EthbloomBloom;
    readonly logs: Vec<EthereumLog>;
  }

  /**
   * @name EthereumBlock (413)
   */
  interface EthereumBlock extends Struct {
    readonly header: EthereumHeader;
    readonly transactions: Vec<EthereumTransactionTransactionV2>;
    readonly ommers: Vec<EthereumHeader>;
  }

  /**
   * @name EthereumHeader (414)
   */
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

  /**
   * @name EthereumTypesHashH64 (415)
   */
  interface EthereumTypesHashH64 extends U8aFixed {}

  /**
   * @name PalletEthereumError (420)
   */
  interface PalletEthereumError extends Enum {
    readonly isInvalidSignature: boolean;
    readonly isPreLogExists: boolean;
    readonly type: "InvalidSignature" | "PreLogExists";
  }

  /**
   * @name PalletSchedulerScheduled (423)
   */
  interface PalletSchedulerScheduled extends Struct {
    readonly maybeId: Option<U8aFixed>;
    readonly priority: u8;
    readonly call: FrameSupportPreimagesBounded;
    readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
    readonly origin: MoonbeamRuntimeOriginCaller;
  }

  /**
   * @name PalletSchedulerError (425)
   */
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

  /**
   * @name PalletDemocracyReferendumInfo (430)
   */
  interface PalletDemocracyReferendumInfo extends Enum {
    readonly isOngoing: boolean;
    readonly asOngoing: PalletDemocracyReferendumStatus;
    readonly isFinished: boolean;
    readonly asFinished: {
      readonly approved: bool;
      readonly end: u32;
    } & Struct;
    readonly type: "Ongoing" | "Finished";
  }

  /**
   * @name PalletDemocracyReferendumStatus (431)
   */
  interface PalletDemocracyReferendumStatus extends Struct {
    readonly end: u32;
    readonly proposal: FrameSupportPreimagesBounded;
    readonly threshold: PalletDemocracyVoteThreshold;
    readonly delay: u32;
    readonly tally: PalletDemocracyTally;
  }

  /**
   * @name PalletDemocracyTally (432)
   */
  interface PalletDemocracyTally extends Struct {
    readonly ayes: u128;
    readonly nays: u128;
    readonly turnout: u128;
  }

  /**
   * @name PalletDemocracyVoteVoting (433)
   */
  interface PalletDemocracyVoteVoting extends Enum {
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
    readonly type: "Direct" | "Delegating";
  }

  /**
   * @name PalletDemocracyDelegations (437)
   */
  interface PalletDemocracyDelegations extends Struct {
    readonly votes: u128;
    readonly capital: u128;
  }

  /**
   * @name PalletDemocracyVotePriorLock (438)
   */
  interface PalletDemocracyVotePriorLock extends ITuple<[u32, u128]> {}

  /**
   * @name PalletDemocracyError (441)
   */
  interface PalletDemocracyError extends Enum {
    readonly isValueLow: boolean;
    readonly isProposalMissing: boolean;
    readonly isAlreadyCanceled: boolean;
    readonly isDuplicateProposal: boolean;
    readonly isProposalBlacklisted: boolean;
    readonly isNotSimpleMajority: boolean;
    readonly isInvalidHash: boolean;
    readonly isNoProposal: boolean;
    readonly isAlreadyVetoed: boolean;
    readonly isReferendumInvalid: boolean;
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
    readonly isTooMany: boolean;
    readonly isVotingPeriodLow: boolean;
    readonly type:
      | "ValueLow"
      | "ProposalMissing"
      | "AlreadyCanceled"
      | "DuplicateProposal"
      | "ProposalBlacklisted"
      | "NotSimpleMajority"
      | "InvalidHash"
      | "NoProposal"
      | "AlreadyVetoed"
      | "ReferendumInvalid"
      | "NoneWaiting"
      | "NotVoter"
      | "NoPermission"
      | "AlreadyDelegating"
      | "InsufficientFunds"
      | "NotDelegating"
      | "VotesExist"
      | "InstantNotAllowed"
      | "Nonsense"
      | "WrongUpperBound"
      | "MaxVotesReached"
      | "TooMany"
      | "VotingPeriodLow";
  }

  /**
   * @name PalletPreimageRequestStatus (442)
   */
  interface PalletPreimageRequestStatus extends Enum {
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

  /**
   * @name PalletPreimageError (447)
   */
  interface PalletPreimageError extends Enum {
    readonly isTooBig: boolean;
    readonly isAlreadyNoted: boolean;
    readonly isNotAuthorized: boolean;
    readonly isNotNoted: boolean;
    readonly isRequested: boolean;
    readonly isNotRequested: boolean;
    readonly type:
      | "TooBig"
      | "AlreadyNoted"
      | "NotAuthorized"
      | "NotNoted"
      | "Requested"
      | "NotRequested";
  }

  /**
   * @name PalletCollectiveVotes (449)
   */
  interface PalletCollectiveVotes extends Struct {
    readonly index: u32;
    readonly threshold: u32;
    readonly ayes: Vec<AccountId20>;
    readonly nays: Vec<AccountId20>;
    readonly end: u32;
  }

  /**
   * @name PalletCollectiveError (450)
   */
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
      | "WrongProposalLength";
  }

  /**
   * @name PalletTreasuryProposal (454)
   */
  interface PalletTreasuryProposal extends Struct {
    readonly proposer: AccountId20;
    readonly value: u128;
    readonly beneficiary: AccountId20;
    readonly bond: u128;
  }

  /**
   * @name FrameSupportPalletId (458)
   */
  interface FrameSupportPalletId extends U8aFixed {}

  /**
   * @name PalletTreasuryError (459)
   */
  interface PalletTreasuryError extends Enum {
    readonly isInsufficientProposersBalance: boolean;
    readonly isInvalidIndex: boolean;
    readonly isTooManyApprovals: boolean;
    readonly isInsufficientPermission: boolean;
    readonly isProposalNotApproved: boolean;
    readonly type:
      | "InsufficientProposersBalance"
      | "InvalidIndex"
      | "TooManyApprovals"
      | "InsufficientPermission"
      | "ProposalNotApproved";
  }

  /**
   * @name PalletCrowdloanRewardsRewardInfo (460)
   */
  interface PalletCrowdloanRewardsRewardInfo extends Struct {
    readonly totalReward: u128;
    readonly claimedReward: u128;
    readonly contributedRelayAddresses: Vec<U8aFixed>;
  }

  /**
   * @name PalletCrowdloanRewardsError (462)
   */
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

  /**
   * @name CumulusPalletXcmpQueueInboundChannelDetails (464)
   */
  interface CumulusPalletXcmpQueueInboundChannelDetails extends Struct {
    readonly sender: u32;
    readonly state: CumulusPalletXcmpQueueInboundState;
    readonly messageMetadata: Vec<ITuple<[u32, PolkadotParachainPrimitivesXcmpMessageFormat]>>;
  }

  /**
   * @name CumulusPalletXcmpQueueInboundState (465)
   */
  interface CumulusPalletXcmpQueueInboundState extends Enum {
    readonly isOk: boolean;
    readonly isSuspended: boolean;
    readonly type: "Ok" | "Suspended";
  }

  /**
   * @name PolkadotParachainPrimitivesXcmpMessageFormat (468)
   */
  interface PolkadotParachainPrimitivesXcmpMessageFormat extends Enum {
    readonly isConcatenatedVersionedXcm: boolean;
    readonly isConcatenatedEncodedBlob: boolean;
    readonly isSignals: boolean;
    readonly type: "ConcatenatedVersionedXcm" | "ConcatenatedEncodedBlob" | "Signals";
  }

  /**
   * @name CumulusPalletXcmpQueueOutboundChannelDetails (471)
   */
  interface CumulusPalletXcmpQueueOutboundChannelDetails extends Struct {
    readonly recipient: u32;
    readonly state: CumulusPalletXcmpQueueOutboundState;
    readonly signalsExist: bool;
    readonly firstIndex: u16;
    readonly lastIndex: u16;
  }

  /**
   * @name CumulusPalletXcmpQueueOutboundState (472)
   */
  interface CumulusPalletXcmpQueueOutboundState extends Enum {
    readonly isOk: boolean;
    readonly isSuspended: boolean;
    readonly type: "Ok" | "Suspended";
  }

  /**
   * @name CumulusPalletXcmpQueueQueueConfigData (474)
   */
  interface CumulusPalletXcmpQueueQueueConfigData extends Struct {
    readonly suspendThreshold: u32;
    readonly dropThreshold: u32;
    readonly resumeThreshold: u32;
    readonly thresholdWeight: SpWeightsWeightV2Weight;
    readonly weightRestrictDecay: SpWeightsWeightV2Weight;
    readonly xcmpMaxIndividualWeight: SpWeightsWeightV2Weight;
  }

  /**
   * @name CumulusPalletXcmpQueueError (476)
   */
  interface CumulusPalletXcmpQueueError extends Enum {
    readonly isFailedToSend: boolean;
    readonly isBadXcmOrigin: boolean;
    readonly isBadXcm: boolean;
    readonly isBadOverweightIndex: boolean;
    readonly isWeightOverLimit: boolean;
    readonly type:
      | "FailedToSend"
      | "BadXcmOrigin"
      | "BadXcm"
      | "BadOverweightIndex"
      | "WeightOverLimit";
  }

  /**
   * @name CumulusPalletXcmError (477)
   */
  type CumulusPalletXcmError = Null;

  /**
   * @name CumulusPalletDmpQueueConfigData (478)
   */
  interface CumulusPalletDmpQueueConfigData extends Struct {
    readonly maxIndividual: SpWeightsWeightV2Weight;
  }

  /**
   * @name CumulusPalletDmpQueuePageIndexData (479)
   */
  interface CumulusPalletDmpQueuePageIndexData extends Struct {
    readonly beginUsed: u32;
    readonly endUsed: u32;
    readonly overweightCount: u64;
  }

  /**
   * @name CumulusPalletDmpQueueError (482)
   */
  interface CumulusPalletDmpQueueError extends Enum {
    readonly isUnknown: boolean;
    readonly isOverLimit: boolean;
    readonly type: "Unknown" | "OverLimit";
  }

  /**
   * @name PalletXcmQueryStatus (483)
   */
  interface PalletXcmQueryStatus extends Enum {
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
    readonly type: "Pending" | "VersionNotifier" | "Ready";
  }

  /**
   * @name XcmVersionedResponse (486)
   */
  interface XcmVersionedResponse extends Enum {
    readonly isV0: boolean;
    readonly asV0: XcmV0Response;
    readonly isV1: boolean;
    readonly asV1: XcmV1Response;
    readonly isV2: boolean;
    readonly asV2: XcmV2Response;
    readonly type: "V0" | "V1" | "V2";
  }

  /**
   * @name PalletXcmVersionMigrationStage (492)
   */
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

  /**
   * @name PalletXcmError (494)
   */
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
      | "AlreadySubscribed";
  }

  /**
   * @name PalletAssetsAssetDetails (495)
   */
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
    readonly isFrozen: bool;
  }

  /**
   * @name PalletAssetsAssetAccount (497)
   */
  interface PalletAssetsAssetAccount extends Struct {
    readonly balance: u128;
    readonly isFrozen: bool;
    readonly reason: PalletAssetsExistenceReason;
    readonly extra: Null;
  }

  /**
   * @name PalletAssetsExistenceReason (498)
   */
  interface PalletAssetsExistenceReason extends Enum {
    readonly isConsumer: boolean;
    readonly isSufficient: boolean;
    readonly isDepositHeld: boolean;
    readonly asDepositHeld: u128;
    readonly isDepositRefunded: boolean;
    readonly type: "Consumer" | "Sufficient" | "DepositHeld" | "DepositRefunded";
  }

  /**
   * @name PalletAssetsApproval (500)
   */
  interface PalletAssetsApproval extends Struct {
    readonly amount: u128;
    readonly deposit: u128;
  }

  /**
   * @name PalletAssetsAssetMetadata (501)
   */
  interface PalletAssetsAssetMetadata extends Struct {
    readonly deposit: u128;
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly decimals: u8;
    readonly isFrozen: bool;
  }

  /**
   * @name PalletAssetsError (503)
   */
  interface PalletAssetsError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isNoAccount: boolean;
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
    readonly isAlreadyExists: boolean;
    readonly isNoDeposit: boolean;
    readonly isWouldBurn: boolean;
    readonly type:
      | "BalanceLow"
      | "NoAccount"
      | "NoPermission"
      | "Unknown"
      | "Frozen"
      | "InUse"
      | "BadWitness"
      | "MinBalanceZero"
      | "NoProvider"
      | "BadMetadata"
      | "Unapproved"
      | "WouldDie"
      | "AlreadyExists"
      | "NoDeposit"
      | "WouldBurn";
  }

  /**
   * @name PalletAssetManagerAssetInfo (504)
   */
  interface PalletAssetManagerAssetInfo extends Struct {
    readonly creator: AccountId20;
    readonly deposit: u128;
  }

  /**
   * @name PalletAssetManagerError (506)
   */
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

  /**
   * @name OrmlXtokensModuleError (507)
   */
  interface OrmlXtokensModuleError extends Enum {
    readonly isAssetHasNoReserve: boolean;
    readonly isNotCrossChainTransfer: boolean;
    readonly isInvalidDest: boolean;
    readonly isNotCrossChainTransferableCurrency: boolean;
    readonly isUnweighableMessage: boolean;
    readonly isXcmExecutionFailed: boolean;
    readonly isCannotReanchor: boolean;
    readonly isInvalidAncestry: boolean;
    readonly isInvalidAsset: boolean;
    readonly isDestinationNotInvertible: boolean;
    readonly isBadVersion: boolean;
    readonly isDistinctReserveForAssetAndFee: boolean;
    readonly isZeroFee: boolean;
    readonly isZeroAmount: boolean;
    readonly isTooManyAssetsBeingSent: boolean;
    readonly isAssetIndexNonExistent: boolean;
    readonly isFeeNotEnough: boolean;
    readonly isNotSupportedMultiLocation: boolean;
    readonly isMinXcmFeeNotDefined: boolean;
    readonly type:
      | "AssetHasNoReserve"
      | "NotCrossChainTransfer"
      | "InvalidDest"
      | "NotCrossChainTransferableCurrency"
      | "UnweighableMessage"
      | "XcmExecutionFailed"
      | "CannotReanchor"
      | "InvalidAncestry"
      | "InvalidAsset"
      | "DestinationNotInvertible"
      | "BadVersion"
      | "DistinctReserveForAssetAndFee"
      | "ZeroFee"
      | "ZeroAmount"
      | "TooManyAssetsBeingSent"
      | "AssetIndexNonExistent"
      | "FeeNotEnough"
      | "NotSupportedMultiLocation"
      | "MinXcmFeeNotDefined";
  }

  /**
   * @name PalletXcmTransactorError (508)
   */
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
    readonly isFeePerSecondNotSet: boolean;
    readonly isSignedTransactNotAllowedForDestination: boolean;
    readonly isFailedMultiLocationToJunction: boolean;
    readonly isHrmpHandlerNotImplemented: boolean;
    readonly isTooMuchFeeUsed: boolean;
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
      | "ErrorSending"
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
      | "TooMuchFeeUsed";
  }

  /**
   * @name PalletRandomnessRequestState (510)
   */
  interface PalletRandomnessRequestState extends Struct {
    readonly request: PalletRandomnessRequest;
    readonly deposit: u128;
  }

  /**
   * @name PalletRandomnessRequest (511)
   */
  interface PalletRandomnessRequest extends Struct {
    readonly refundAddress: H160;
    readonly contractAddress: H160;
    readonly fee: u128;
    readonly gasLimit: u64;
    readonly numWords: u8;
    readonly salt: H256;
    readonly info: PalletRandomnessRequestInfo;
  }

  /**
   * @name PalletRandomnessRequestInfo (512)
   */
  interface PalletRandomnessRequestInfo extends Enum {
    readonly isBabeEpoch: boolean;
    readonly asBabeEpoch: ITuple<[u64, u64]>;
    readonly isLocal: boolean;
    readonly asLocal: ITuple<[u32, u32]>;
    readonly type: "BabeEpoch" | "Local";
  }

  /**
   * @name PalletRandomnessRequestType (513)
   */
  interface PalletRandomnessRequestType extends Enum {
    readonly isBabeEpoch: boolean;
    readonly asBabeEpoch: u64;
    readonly isLocal: boolean;
    readonly asLocal: u32;
    readonly type: "BabeEpoch" | "Local";
  }

  /**
   * @name PalletRandomnessRandomnessResult (514)
   */
  interface PalletRandomnessRandomnessResult extends Struct {
    readonly randomness: Option<H256>;
    readonly requestCount: u64;
  }

  /**
   * @name PalletRandomnessError (515)
   */
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

  /**
   * @name AccountEthereumSignature (517)
   */
  interface AccountEthereumSignature extends SpCoreEcdsaSignature {}

  /**
   * @name FrameSystemExtensionsCheckSpecVersion (519)
   */
  type FrameSystemExtensionsCheckSpecVersion = Null;

  /**
   * @name FrameSystemExtensionsCheckTxVersion (520)
   */
  type FrameSystemExtensionsCheckTxVersion = Null;

  /**
   * @name FrameSystemExtensionsCheckGenesis (521)
   */
  type FrameSystemExtensionsCheckGenesis = Null;

  /**
   * @name FrameSystemExtensionsCheckNonce (524)
   */
  interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /**
   * @name FrameSystemExtensionsCheckWeight (525)
   */
  type FrameSystemExtensionsCheckWeight = Null;

  /**
   * @name PalletTransactionPaymentChargeTransactionPayment (526)
   */
  interface PalletTransactionPaymentChargeTransactionPayment extends Compact<u128> {}

  /**
   * @name MoonbeamRuntimeRuntime (528)
   */
  type MoonbeamRuntimeRuntime = Null;
} // declare module
