// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

/* eslint-disable sort-keys */

export default {
  /**
   * Lookup3: frame_system::AccountInfo<Index, pallet_balances::AccountData<Balance>>
   */
  FrameSystemAccountInfo: {
    nonce: 'u32',
    consumers: 'u32',
    providers: 'u32',
    sufficients: 'u32',
    data: 'PalletBalancesAccountData',
  },
  /**
   * Lookup5: pallet_balances::AccountData<Balance>
   */
  PalletBalancesAccountData: {
    free: 'u128',
    reserved: 'u128',
    miscFrozen: 'u128',
    feeFrozen: 'u128',
  },
  /**
   * Lookup7: frame_support::weights::PerDispatchClass<T>
   */
  FrameSupportWeightsPerDispatchClassU64: {
    normal: 'u64',
    operational: 'u64',
    mandatory: 'u64',
  },
  /**
   * Lookup12: sp_runtime::generic::digest::Digest
   */
  SpRuntimeDigest: {
    logs: 'Vec<SpRuntimeDigestDigestItem>',
  },
  /**
   * Lookup14: sp_runtime::generic::digest::DigestItem
   */
  SpRuntimeDigestDigestItem: {
    _enum: {
      Other: 'Bytes',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      Consensus: '([u8;4],Bytes)',
      Seal: '([u8;4],Bytes)',
      PreRuntime: '([u8;4],Bytes)',
      __Unused7: 'Null',
      RuntimeEnvironmentUpdated: 'Null',
    },
  },
  /**
   * Lookup17: frame_system::EventRecord<moonbeam_runtime::Event, primitive_types::H256>
   */
  FrameSystemEventRecord: {
    phase: 'FrameSystemPhase',
    event: 'Event',
    topics: 'Vec<H256>',
  },
  /**
   * Lookup19: frame_system::pallet::Event<T>
   */
  FrameSystemEvent: {
    _enum: {
      ExtrinsicSuccess: 'FrameSupportWeightsDispatchInfo',
      ExtrinsicFailed: '(SpRuntimeDispatchError,FrameSupportWeightsDispatchInfo)',
      CodeUpdated: 'Null',
      NewAccount: 'AccountId20',
      KilledAccount: 'AccountId20',
      Remarked: '(AccountId20,H256)',
    },
  },
  /**
   * Lookup20: frame_support::weights::DispatchInfo
   */
  FrameSupportWeightsDispatchInfo: {
    weight: 'u64',
    class: 'FrameSupportWeightsDispatchClass',
    paysFee: 'FrameSupportWeightsPays',
  },
  /**
   * Lookup21: frame_support::weights::DispatchClass
   */
  FrameSupportWeightsDispatchClass: {
    _enum: ['Normal', 'Operational', 'Mandatory'],
  },
  /**
   * Lookup22: frame_support::weights::Pays
   */
  FrameSupportWeightsPays: {
    _enum: ['Yes', 'No'],
  },
  /**
   * Lookup23: sp_runtime::DispatchError
   */
  SpRuntimeDispatchError: {
    _enum: {
      Other: 'Null',
      CannotLookup: 'Null',
      BadOrigin: 'Null',
      Module: {
        index: 'u8',
        error: 'u8',
      },
      ConsumerRemaining: 'Null',
      NoProviders: 'Null',
      Token: 'SpRuntimeTokenError',
      Arithmetic: 'SpRuntimeArithmeticError',
    },
  },
  /**
   * Lookup24: sp_runtime::TokenError
   */
  SpRuntimeTokenError: {
    _enum: [
      'NoFunds',
      'WouldDie',
      'BelowMinimum',
      'CannotCreate',
      'UnknownAsset',
      'Frozen',
      'Unsupported',
    ],
  },
  /**
   * Lookup25: sp_runtime::ArithmeticError
   */
  SpRuntimeArithmeticError: {
    _enum: ['Underflow', 'Overflow', 'DivisionByZero'],
  },
  /**
   * Lookup26: cumulus_pallet_parachain_system::pallet::Event<T>
   */
  CumulusPalletParachainSystemEvent: {
    _enum: {
      ValidationFunctionStored: 'Null',
      ValidationFunctionApplied: 'u32',
      ValidationFunctionDiscarded: 'Null',
      UpgradeAuthorized: 'H256',
      DownwardMessagesReceived: 'u32',
      DownwardMessagesProcessed: '(u64,H256)',
    },
  },
  /**
   * Lookup27: pallet_balances::pallet::Event<T, I>
   */
  PalletBalancesEvent: {
    _enum: {
      Endowed: {
        account: 'AccountId20',
        freeBalance: 'u128',
      },
      DustLost: {
        account: 'AccountId20',
        amount: 'u128',
      },
      Transfer: {
        from: 'AccountId20',
        to: 'AccountId20',
        amount: 'u128',
      },
      BalanceSet: {
        who: 'AccountId20',
        free: 'u128',
        reserved: 'u128',
      },
      Reserved: {
        who: 'AccountId20',
        amount: 'u128',
      },
      Unreserved: {
        who: 'AccountId20',
        amount: 'u128',
      },
      ReserveRepatriated: {
        from: 'AccountId20',
        to: 'AccountId20',
        amount: 'u128',
        destinationStatus: 'FrameSupportTokensMiscBalanceStatus',
      },
      Deposit: {
        who: 'AccountId20',
        amount: 'u128',
      },
      Withdraw: {
        who: 'AccountId20',
        amount: 'u128',
      },
      Slashed: {
        who: 'AccountId20',
        amount: 'u128',
      },
    },
  },
  /**
   * Lookup28: frame_support::traits::tokens::misc::BalanceStatus
   */
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ['Free', 'Reserved'],
  },
  /**
   * Lookup29: parachain_staking::pallet::Event<T>
   */
  ParachainStakingEvent: {
    _enum: {
      NewRound: '(u32,u32,u32,u128)',
      JoinedCollatorCandidates: '(AccountId20,u128,u128)',
      CollatorChosen: '(u32,AccountId20,u128)',
      CandidateBondLessRequested: '(AccountId20,u128,u32)',
      CandidateBondedMore: '(AccountId20,u128,u128)',
      CandidateBondedLess: '(AccountId20,u128,u128)',
      CandidateWentOffline: 'AccountId20',
      CandidateBackOnline: 'AccountId20',
      CandidateScheduledExit: '(u32,AccountId20,u32)',
      CancelledCandidateExit: 'AccountId20',
      CancelledCandidateBondLess: '(AccountId20,u128,u32)',
      CandidateLeft: '(AccountId20,u128,u128)',
      DelegationDecreaseScheduled: '(AccountId20,AccountId20,u128,u32)',
      DelegationIncreased: '(AccountId20,AccountId20,u128,bool)',
      DelegationDecreased: '(AccountId20,AccountId20,u128,bool)',
      DelegatorExitScheduled: '(u32,AccountId20,u32)',
      DelegationRevocationScheduled: '(u32,AccountId20,AccountId20,u32)',
      DelegatorLeft: '(AccountId20,u128)',
      DelegationRevoked: '(AccountId20,AccountId20,u128)',
      DelegationKicked: '(AccountId20,AccountId20,u128)',
      DelegatorExitCancelled: 'AccountId20',
      CancelledDelegationRequest: '(AccountId20,ParachainStakingDelegationRequest)',
      Delegation: '(AccountId20,u128,AccountId20,ParachainStakingDelegatorAdded)',
      DelegatorLeftCandidate: '(AccountId20,AccountId20,u128,u128)',
      Rewarded: '(AccountId20,u128)',
      ReservedForParachainBond: '(AccountId20,u128)',
      ParachainBondAccountSet: '(AccountId20,AccountId20)',
      ParachainBondReservePercentSet: '(Percent,Percent)',
      InflationSet: '(Perbill,Perbill,Perbill,Perbill,Perbill,Perbill)',
      StakeExpectationsSet: '(u128,u128,u128)',
      TotalSelectedSet: '(u32,u32)',
      CollatorCommissionSet: '(Perbill,Perbill)',
      BlocksPerRoundSet: '(u32,u32,u32,u32,Perbill,Perbill,Perbill)',
    },
  },
  /**
   * Lookup31: parachain_staking::pallet::DelegationRequest<account::AccountId20, Balance>
   */
  ParachainStakingDelegationRequest: {
    collator: 'AccountId20',
    amount: 'u128',
    whenExecutable: 'u32',
    action: 'ParachainStakingDelegationChange',
  },
  /**
   * Lookup32: parachain_staking::pallet::DelegationChange
   */
  ParachainStakingDelegationChange: {
    _enum: ['Revoke', 'Decrease'],
  },
  /**
   * Lookup33: parachain_staking::pallet::DelegatorAdded<B>
   */
  ParachainStakingDelegatorAdded: {
    _enum: {
      AddedToTop: {
        newTotal: 'u128',
      },
      AddedToBottom: 'Null',
    },
  },
  /**
   * Lookup36: pallet_author_slot_filter::pallet::Event
   */
  PalletAuthorSlotFilterEvent: {
    _enum: {
      EligibleUpdated: 'Percent',
    },
  },
  /**
   * Lookup37: pallet_author_mapping::pallet::Event<T>
   */
  PalletAuthorMappingEvent: {
    _enum: {
      AuthorRegistered: '(NimbusPrimitivesNimbusCryptoPublic,AccountId20)',
      AuthorDeRegistered: 'NimbusPrimitivesNimbusCryptoPublic',
      AuthorRotated: '(NimbusPrimitivesNimbusCryptoPublic,AccountId20)',
      DefunctAuthorBusted: '(NimbusPrimitivesNimbusCryptoPublic,AccountId20)',
    },
  },
  /**
   * Lookup38: nimbus_primitives::nimbus_crypto::Public
   */
  NimbusPrimitivesNimbusCryptoPublic: 'SpCoreSr25519Public',
  /**
   * Lookup39: sp_core::sr25519::Public
   */
  SpCoreSr25519Public: '[u8;32]',
  /**
   * Lookup40: pallet_utility::pallet::Event
   */
  PalletUtilityEvent: {
    _enum: {
      BatchInterrupted: {
        index: 'u32',
        error: 'SpRuntimeDispatchError',
      },
      BatchCompleted: 'Null',
      ItemCompleted: 'Null',
      DispatchedAs: 'Result<Null, SpRuntimeDispatchError>',
    },
  },
  /**
   * Lookup43: pallet_proxy::pallet::Event<T>
   */
  PalletProxyEvent: {
    _enum: {
      ProxyExecuted: {
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      AnonymousCreated: {
        anonymous: 'AccountId20',
        who: 'AccountId20',
        proxyType: 'MoonbeamRuntimeProxyType',
        disambiguationIndex: 'u16',
      },
      Announced: {
        real: 'AccountId20',
        proxy: 'AccountId20',
        callHash: 'H256',
      },
      ProxyAdded: {
        delegator: 'AccountId20',
        delegatee: 'AccountId20',
        proxyType: 'MoonbeamRuntimeProxyType',
        delay: 'u32',
      },
    },
  },
  /**
   * Lookup44: moonbeam_runtime::ProxyType
   */
  MoonbeamRuntimeProxyType: {
    _enum: [
      'Any',
      'NonTransfer',
      'Governance',
      'Staking',
      'CancelProxy',
      'Balances',
      'AuthorMapping',
    ],
  },
  /**
   * Lookup46: pallet_maintenance_mode::pallet::Event
   */
  PalletMaintenanceModeEvent: {
    _enum: ['EnteredMaintenanceMode', 'NormalOperationResumed'],
  },
  /**
   * Lookup47: pallet_identity::pallet::Event<T>
   */
  PalletIdentityEvent: {
    _enum: {
      IdentitySet: {
        who: 'AccountId20',
      },
      IdentityCleared: {
        who: 'AccountId20',
        deposit: 'u128',
      },
      IdentityKilled: {
        who: 'AccountId20',
        deposit: 'u128',
      },
      JudgementRequested: {
        who: 'AccountId20',
        registrarIndex: 'u32',
      },
      JudgementUnrequested: {
        who: 'AccountId20',
        registrarIndex: 'u32',
      },
      JudgementGiven: {
        target: 'AccountId20',
        registrarIndex: 'u32',
      },
      RegistrarAdded: {
        registrarIndex: 'u32',
      },
      SubIdentityAdded: {
        sub: 'AccountId20',
        main: 'AccountId20',
        deposit: 'u128',
      },
      SubIdentityRemoved: {
        sub: 'AccountId20',
        main: 'AccountId20',
        deposit: 'u128',
      },
      SubIdentityRevoked: {
        sub: 'AccountId20',
        main: 'AccountId20',
        deposit: 'u128',
      },
    },
  },
  /**
   * Lookup48: pallet_migrations::pallet::Event<T>
   */
  PalletMigrationsEvent: {
    _enum: {
      RuntimeUpgradeStarted: 'Null',
      RuntimeUpgradeCompleted: 'u64',
      MigrationStarted: 'Bytes',
      MigrationCompleted: '(Bytes,u64)',
    },
  },
  /**
   * Lookup49: pallet_evm::pallet::Event<T>
   */
  PalletEvmEvent: {
    _enum: {
      Log: 'EthereumLog',
      Created: 'H160',
      CreatedFailed: 'H160',
      Executed: 'H160',
      ExecutedFailed: 'H160',
      BalanceDeposit: '(AccountId20,H160,U256)',
      BalanceWithdraw: '(AccountId20,H160,U256)',
    },
  },
  /**
   * Lookup50: ethereum::log::Log
   */
  EthereumLog: {
    address: 'H160',
    topics: 'Vec<H256>',
    data: 'Bytes',
  },
  /**
   * Lookup55: pallet_ethereum::pallet::Event
   */
  PalletEthereumEvent: {
    _enum: {
      Executed: '(H160,H160,H256,EvmCoreErrorExitReason)',
    },
  },
  /**
   * Lookup56: evm_core::error::ExitReason
   */
  EvmCoreErrorExitReason: {
    _enum: {
      Succeed: 'EvmCoreErrorExitSucceed',
      Error: 'EvmCoreErrorExitError',
      Revert: 'EvmCoreErrorExitRevert',
      Fatal: 'EvmCoreErrorExitFatal',
    },
  },
  /**
   * Lookup57: evm_core::error::ExitSucceed
   */
  EvmCoreErrorExitSucceed: {
    _enum: ['Stopped', 'Returned', 'Suicided'],
  },
  /**
   * Lookup58: evm_core::error::ExitError
   */
  EvmCoreErrorExitError: {
    _enum: {
      StackUnderflow: 'Null',
      StackOverflow: 'Null',
      InvalidJump: 'Null',
      InvalidRange: 'Null',
      DesignatedInvalid: 'Null',
      CallTooDeep: 'Null',
      CreateCollision: 'Null',
      CreateContractLimit: 'Null',
      OutOfOffset: 'Null',
      OutOfGas: 'Null',
      OutOfFund: 'Null',
      PCUnderflow: 'Null',
      CreateEmpty: 'Null',
      Other: 'Text',
      InvalidCode: 'Null',
    },
  },
  /**
   * Lookup61: evm_core::error::ExitRevert
   */
  EvmCoreErrorExitRevert: {
    _enum: ['Reverted'],
  },
  /**
   * Lookup62: evm_core::error::ExitFatal
   */
  EvmCoreErrorExitFatal: {
    _enum: {
      NotSupported: 'Null',
      UnhandledInterrupt: 'Null',
      CallErrorAsFatal: 'EvmCoreErrorExitError',
      Other: 'Text',
    },
  },
  /**
   * Lookup63: pallet_base_fee::pallet::Event
   */
  PalletBaseFeeEvent: {
    _enum: {
      NewBaseFeePerGas: 'U256',
      BaseFeeOverflow: 'Null',
      IsActive: 'bool',
      NewElasticity: 'Permill',
    },
  },
  /**
   * Lookup65: pallet_scheduler::pallet::Event<T>
   */
  PalletSchedulerEvent: {
    _enum: {
      Scheduled: '(u32,u32)',
      Canceled: '(u32,u32)',
      Dispatched: '((u32,u32),Option<Bytes>,Result<Null, SpRuntimeDispatchError>)',
    },
  },
  /**
   * Lookup68: pallet_democracy::pallet::Event<T>
   */
  PalletDemocracyEvent: {
    _enum: {
      Proposed: {
        proposalIndex: 'u32',
        deposit: 'u128',
      },
      Tabled: {
        proposalIndex: 'u32',
        deposit: 'u128',
        depositors: 'Vec<AccountId20>',
      },
      ExternalTabled: 'Null',
      Started: {
        refIndex: 'u32',
        threshold: 'PalletDemocracyVoteThreshold',
      },
      Passed: {
        refIndex: 'u32',
      },
      NotPassed: {
        refIndex: 'u32',
      },
      Cancelled: {
        refIndex: 'u32',
      },
      Executed: {
        refIndex: 'u32',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      Delegated: {
        who: 'AccountId20',
        target: 'AccountId20',
      },
      Undelegated: {
        account: 'AccountId20',
      },
      Vetoed: {
        who: 'AccountId20',
        proposalHash: 'H256',
        until: 'u32',
      },
      PreimageNoted: {
        proposalHash: 'H256',
        who: 'AccountId20',
        deposit: 'u128',
      },
      PreimageUsed: {
        proposalHash: 'H256',
        provider: 'AccountId20',
        deposit: 'u128',
      },
      PreimageInvalid: {
        proposalHash: 'H256',
        refIndex: 'u32',
      },
      PreimageMissing: {
        proposalHash: 'H256',
        refIndex: 'u32',
      },
      PreimageReaped: {
        proposalHash: 'H256',
        provider: 'AccountId20',
        deposit: 'u128',
        reaper: 'AccountId20',
      },
      Blacklisted: {
        proposalHash: 'H256',
      },
      Voted: {
        who: 'AccountId20',
        refIndex: 'u32',
        vote: 'PalletDemocracyVoteAccountVote',
      },
      Seconded: {
        who: 'AccountId20',
        proposalIndex: 'u32',
      },
    },
  },
  /**
   * Lookup70: pallet_democracy::vote_threshold::VoteThreshold
   */
  PalletDemocracyVoteThreshold: {
    _enum: ['SuperMajorityApprove', 'SuperMajorityAgainst', 'SimpleMajority'],
  },
  /**
   * Lookup71: pallet_democracy::vote::AccountVote<Balance>
   */
  PalletDemocracyVoteAccountVote: {
    _enum: {
      Standard: {
        vote: 'Vote',
        balance: 'u128',
      },
      Split: {
        aye: 'u128',
        nay: 'u128',
      },
    },
  },
  /**
   * Lookup73: pallet_collective::pallet::Event<T, I>
   */
  PalletCollectiveEvent: {
    _enum: {
      Proposed: {
        account: 'AccountId20',
        proposalIndex: 'u32',
        proposalHash: 'H256',
        threshold: 'u32',
      },
      Voted: {
        account: 'AccountId20',
        proposalHash: 'H256',
        voted: 'bool',
        yes: 'u32',
        no: 'u32',
      },
      Approved: {
        proposalHash: 'H256',
      },
      Disapproved: {
        proposalHash: 'H256',
      },
      Executed: {
        proposalHash: 'H256',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      MemberExecuted: {
        proposalHash: 'H256',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      Closed: {
        proposalHash: 'H256',
        yes: 'u32',
        no: 'u32',
      },
    },
  },
  /**
   * Lookup75: pallet_treasury::pallet::Event<T, I>
   */
  PalletTreasuryEvent: {
    _enum: {
      Proposed: 'u32',
      Spending: 'u128',
      Awarded: '(u32,u128,AccountId20)',
      Rejected: '(u32,u128)',
      Burnt: 'u128',
      Rollover: 'u128',
      Deposit: 'u128',
    },
  },
  /**
   * Lookup76: pallet_crowdloan_rewards::pallet::Event<T>
   */
  PalletCrowdloanRewardsEvent: {
    _enum: {
      InitialPaymentMade: '(AccountId20,u128)',
      NativeIdentityAssociated: '([u8;32],AccountId20,u128)',
      RewardsPaid: '(AccountId20,u128)',
      RewardAddressUpdated: '(AccountId20,AccountId20)',
      InitializedAlreadyInitializedAccount: '([u8;32],Option<AccountId20>,u128)',
      InitializedAccountWithNotEnoughContribution: '([u8;32],Option<AccountId20>,u128)',
    },
  },
  /**
   * Lookup78: cumulus_pallet_xcmp_queue::pallet::Event<T>
   */
  CumulusPalletXcmpQueueEvent: {
    _enum: {
      Success: 'Option<H256>',
      Fail: '(Option<H256>,XcmV2TraitsError)',
      BadVersion: 'Option<H256>',
      BadFormat: 'Option<H256>',
      UpwardMessageSent: 'Option<H256>',
      XcmpMessageSent: 'Option<H256>',
    },
  },
  /**
   * Lookup80: xcm::v2::traits::Error
   */
  XcmV2TraitsError: {
    _enum: {
      Overflow: 'Null',
      Unimplemented: 'Null',
      UntrustedReserveLocation: 'Null',
      UntrustedTeleportLocation: 'Null',
      MultiLocationFull: 'Null',
      MultiLocationNotInvertible: 'Null',
      BadOrigin: 'Null',
      InvalidLocation: 'Null',
      AssetNotFound: 'Null',
      FailedToTransactAsset: 'Null',
      NotWithdrawable: 'Null',
      LocationCannotHold: 'Null',
      ExceedsMaxMessageSize: 'Null',
      DestinationUnsupported: 'Null',
      Transport: 'Null',
      Unroutable: 'Null',
      UnknownClaim: 'Null',
      FailedToDecode: 'Null',
      TooMuchWeightRequired: 'Null',
      NotHoldingFees: 'Null',
      TooExpensive: 'Null',
      Trap: 'u64',
      UnhandledXcmVersion: 'Null',
      WeightLimitReached: 'u64',
      Barrier: 'Null',
      WeightNotComputable: 'Null',
    },
  },
  /**
   * Lookup81: cumulus_pallet_xcm::pallet::Event<T>
   */
  CumulusPalletXcmEvent: {
    _enum: {
      InvalidFormat: '[u8;8]',
      UnsupportedVersion: '[u8;8]',
      ExecutedDownward: '([u8;8],XcmV2TraitsOutcome)',
    },
  },
  /**
   * Lookup83: xcm::v2::traits::Outcome
   */
  XcmV2TraitsOutcome: {
    _enum: {
      Complete: 'u64',
      Incomplete: '(u64,XcmV2TraitsError)',
      Error: 'XcmV2TraitsError',
    },
  },
  /**
   * Lookup84: cumulus_pallet_dmp_queue::pallet::Event<T>
   */
  CumulusPalletDmpQueueEvent: {
    _enum: {
      InvalidFormat: '[u8;32]',
      UnsupportedVersion: '[u8;32]',
      ExecutedDownward: '([u8;32],XcmV2TraitsOutcome)',
      WeightExhausted: '([u8;32],u64,u64)',
      OverweightEnqueued: '([u8;32],u64,u64)',
      OverweightServiced: '(u64,u64)',
    },
  },
  /**
   * Lookup85: pallet_xcm::pallet::Event<T>
   */
  PalletXcmEvent: {
    _enum: {
      Attempted: 'XcmV2TraitsOutcome',
      Sent: '(XcmV1MultiLocation,XcmV1MultiLocation,XcmV2Xcm)',
      UnexpectedResponse: '(XcmV1MultiLocation,u64)',
      ResponseReady: '(u64,XcmV2Response)',
      Notified: '(u64,u8,u8)',
      NotifyOverweight: '(u64,u8,u8,u64,u64)',
      NotifyDispatchError: '(u64,u8,u8)',
      NotifyDecodeFailed: '(u64,u8,u8)',
      InvalidResponder: '(XcmV1MultiLocation,u64,Option<XcmV1MultiLocation>)',
      InvalidResponderVersion: '(XcmV1MultiLocation,u64)',
      ResponseTaken: 'u64',
      AssetsTrapped: '(H256,XcmV1MultiLocation,XcmVersionedMultiAssets)',
      VersionChangeNotified: '(XcmV1MultiLocation,u32)',
      SupportedVersionChanged: '(XcmV1MultiLocation,u32)',
      NotifyTargetSendFail: '(XcmV1MultiLocation,u64,XcmV2TraitsError)',
      NotifyTargetMigrationFail: '(XcmVersionedMultiLocation,u64)',
    },
  },
  /**
   * Lookup86: xcm::v1::multilocation::MultiLocation
   */
  XcmV1MultiLocation: {
    parents: 'u8',
    interior: 'XcmV1MultilocationJunctions',
  },
  /**
   * Lookup87: xcm::v1::multilocation::Junctions
   */
  XcmV1MultilocationJunctions: {
    _enum: {
      Here: 'Null',
      X1: 'XcmV1Junction',
      X2: '(XcmV1Junction,XcmV1Junction)',
      X3: '(XcmV1Junction,XcmV1Junction,XcmV1Junction)',
      X4: '(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)',
      X5: '(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)',
      X6: '(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)',
      X7: '(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)',
      X8: '(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)',
    },
  },
  /**
   * Lookup88: xcm::v1::junction::Junction
   */
  XcmV1Junction: {
    _enum: {
      Parachain: 'Compact<u32>',
      AccountId32: {
        network: 'XcmV0JunctionNetworkId',
        id: '[u8;32]',
      },
      AccountIndex64: {
        network: 'XcmV0JunctionNetworkId',
        index: 'Compact<u64>',
      },
      AccountKey20: {
        network: 'XcmV0JunctionNetworkId',
        key: '[u8;20]',
      },
      PalletInstance: 'u8',
      GeneralIndex: 'Compact<u128>',
      GeneralKey: 'Bytes',
      OnlyChild: 'Null',
      Plurality: {
        id: 'XcmV0JunctionBodyId',
        part: 'XcmV0JunctionBodyPart',
      },
    },
  },
  /**
   * Lookup90: xcm::v0::junction::NetworkId
   */
  XcmV0JunctionNetworkId: {
    _enum: {
      Any: 'Null',
      Named: 'Bytes',
      Polkadot: 'Null',
      Kusama: 'Null',
    },
  },
  /**
   * Lookup93: xcm::v0::junction::BodyId
   */
  XcmV0JunctionBodyId: {
    _enum: {
      Unit: 'Null',
      Named: 'Bytes',
      Index: 'Compact<u32>',
      Executive: 'Null',
      Technical: 'Null',
      Legislative: 'Null',
      Judicial: 'Null',
    },
  },
  /**
   * Lookup94: xcm::v0::junction::BodyPart
   */
  XcmV0JunctionBodyPart: {
    _enum: {
      Voice: 'Null',
      Members: {
        count: 'Compact<u32>',
      },
      Fraction: {
        nom: 'Compact<u32>',
        denom: 'Compact<u32>',
      },
      AtLeastProportion: {
        nom: 'Compact<u32>',
        denom: 'Compact<u32>',
      },
      MoreThanProportion: {
        nom: 'Compact<u32>',
        denom: 'Compact<u32>',
      },
    },
  },
  /**
   * Lookup95: xcm::v2::Xcm<Call>
   */
  XcmV2Xcm: 'Vec<XcmV2Instruction>',
  /**
   * Lookup97: xcm::v2::Instruction<Call>
   */
  XcmV2Instruction: {
    _enum: {
      WithdrawAsset: 'XcmV1MultiassetMultiAssets',
      ReserveAssetDeposited: 'XcmV1MultiassetMultiAssets',
      ReceiveTeleportedAsset: 'XcmV1MultiassetMultiAssets',
      QueryResponse: {
        queryId: 'Compact<u64>',
        response: 'XcmV2Response',
        maxWeight: 'Compact<u64>',
      },
      TransferAsset: {
        assets: 'XcmV1MultiassetMultiAssets',
        beneficiary: 'XcmV1MultiLocation',
      },
      TransferReserveAsset: {
        assets: 'XcmV1MultiassetMultiAssets',
        dest: 'XcmV1MultiLocation',
        xcm: 'XcmV2Xcm',
      },
      Transact: {
        originType: 'XcmV0OriginKind',
        requireWeightAtMost: 'Compact<u64>',
        call: 'XcmDoubleEncoded',
      },
      HrmpNewChannelOpenRequest: {
        sender: 'Compact<u32>',
        maxMessageSize: 'Compact<u32>',
        maxCapacity: 'Compact<u32>',
      },
      HrmpChannelAccepted: {
        recipient: 'Compact<u32>',
      },
      HrmpChannelClosing: {
        initiator: 'Compact<u32>',
        sender: 'Compact<u32>',
        recipient: 'Compact<u32>',
      },
      ClearOrigin: 'Null',
      DescendOrigin: 'XcmV1MultilocationJunctions',
      ReportError: {
        queryId: 'Compact<u64>',
        dest: 'XcmV1MultiLocation',
        maxResponseWeight: 'Compact<u64>',
      },
      DepositAsset: {
        assets: 'XcmV1MultiassetMultiAssetFilter',
        maxAssets: 'Compact<u32>',
        beneficiary: 'XcmV1MultiLocation',
      },
      DepositReserveAsset: {
        assets: 'XcmV1MultiassetMultiAssetFilter',
        maxAssets: 'Compact<u32>',
        dest: 'XcmV1MultiLocation',
        xcm: 'XcmV2Xcm',
      },
      ExchangeAsset: {
        give: 'XcmV1MultiassetMultiAssetFilter',
        receive: 'XcmV1MultiassetMultiAssets',
      },
      InitiateReserveWithdraw: {
        assets: 'XcmV1MultiassetMultiAssetFilter',
        reserve: 'XcmV1MultiLocation',
        xcm: 'XcmV2Xcm',
      },
      InitiateTeleport: {
        assets: 'XcmV1MultiassetMultiAssetFilter',
        dest: 'XcmV1MultiLocation',
        xcm: 'XcmV2Xcm',
      },
      QueryHolding: {
        queryId: 'Compact<u64>',
        dest: 'XcmV1MultiLocation',
        assets: 'XcmV1MultiassetMultiAssetFilter',
        maxResponseWeight: 'Compact<u64>',
      },
      BuyExecution: {
        fees: 'XcmV1MultiAsset',
        weightLimit: 'XcmV2WeightLimit',
      },
      RefundSurplus: 'Null',
      SetErrorHandler: 'XcmV2Xcm',
      SetAppendix: 'XcmV2Xcm',
      ClearError: 'Null',
      ClaimAsset: {
        assets: 'XcmV1MultiassetMultiAssets',
        ticket: 'XcmV1MultiLocation',
      },
      Trap: 'Compact<u64>',
      SubscribeVersion: {
        queryId: 'Compact<u64>',
        maxResponseWeight: 'Compact<u64>',
      },
      UnsubscribeVersion: 'Null',
    },
  },
  /**
   * Lookup98: xcm::v1::multiasset::MultiAssets
   */
  XcmV1MultiassetMultiAssets: 'Vec<XcmV1MultiAsset>',
  /**
   * Lookup100: xcm::v1::multiasset::MultiAsset
   */
  XcmV1MultiAsset: {
    id: 'XcmV1MultiassetAssetId',
    fun: 'XcmV1MultiassetFungibility',
  },
  /**
   * Lookup101: xcm::v1::multiasset::AssetId
   */
  XcmV1MultiassetAssetId: {
    _enum: {
      Concrete: 'XcmV1MultiLocation',
      Abstract: 'Bytes',
    },
  },
  /**
   * Lookup102: xcm::v1::multiasset::Fungibility
   */
  XcmV1MultiassetFungibility: {
    _enum: {
      Fungible: 'Compact<u128>',
      NonFungible: 'XcmV1MultiassetAssetInstance',
    },
  },
  /**
   * Lookup103: xcm::v1::multiasset::AssetInstance
   */
  XcmV1MultiassetAssetInstance: {
    _enum: {
      Undefined: 'Null',
      Index: 'Compact<u128>',
      Array4: '[u8;4]',
      Array8: '[u8;8]',
      Array16: '[u8;16]',
      Array32: '[u8;32]',
      Blob: 'Bytes',
    },
  },
  /**
   * Lookup105: xcm::v2::Response
   */
  XcmV2Response: {
    _enum: {
      Null: 'Null',
      Assets: 'XcmV1MultiassetMultiAssets',
      ExecutionResult: 'Option<(u32,XcmV2TraitsError)>',
      Version: 'u32',
    },
  },
  /**
   * Lookup108: xcm::v0::OriginKind
   */
  XcmV0OriginKind: {
    _enum: ['Native', 'SovereignAccount', 'Superuser', 'Xcm'],
  },
  /**
   * Lookup109: xcm::double_encoded::DoubleEncoded<T>
   */
  XcmDoubleEncoded: {
    encoded: 'Bytes',
  },
  /**
   * Lookup110: xcm::v1::multiasset::MultiAssetFilter
   */
  XcmV1MultiassetMultiAssetFilter: {
    _enum: {
      Definite: 'XcmV1MultiassetMultiAssets',
      Wild: 'XcmV1MultiassetWildMultiAsset',
    },
  },
  /**
   * Lookup111: xcm::v1::multiasset::WildMultiAsset
   */
  XcmV1MultiassetWildMultiAsset: {
    _enum: {
      All: 'Null',
      AllOf: {
        id: 'XcmV1MultiassetAssetId',
        fun: 'XcmV1MultiassetWildFungibility',
      },
    },
  },
  /**
   * Lookup112: xcm::v1::multiasset::WildFungibility
   */
  XcmV1MultiassetWildFungibility: {
    _enum: ['Fungible', 'NonFungible'],
  },
  /**
   * Lookup113: xcm::v2::WeightLimit
   */
  XcmV2WeightLimit: {
    _enum: {
      Unlimited: 'Null',
      Limited: 'Compact<u64>',
    },
  },
  /**
   * Lookup115: xcm::VersionedMultiAssets
   */
  XcmVersionedMultiAssets: {
    _enum: {
      V0: 'Vec<XcmV0MultiAsset>',
      V1: 'XcmV1MultiassetMultiAssets',
    },
  },
  /**
   * Lookup117: xcm::v0::multi_asset::MultiAsset
   */
  XcmV0MultiAsset: {
    _enum: {
      None: 'Null',
      All: 'Null',
      AllFungible: 'Null',
      AllNonFungible: 'Null',
      AllAbstractFungible: {
        id: 'Bytes',
      },
      AllAbstractNonFungible: {
        class: 'Bytes',
      },
      AllConcreteFungible: {
        id: 'XcmV0MultiLocation',
      },
      AllConcreteNonFungible: {
        class: 'XcmV0MultiLocation',
      },
      AbstractFungible: {
        id: 'Bytes',
        amount: 'Compact<u128>',
      },
      AbstractNonFungible: {
        class: 'Bytes',
        instance: 'XcmV1MultiassetAssetInstance',
      },
      ConcreteFungible: {
        id: 'XcmV0MultiLocation',
        amount: 'Compact<u128>',
      },
      ConcreteNonFungible: {
        class: 'XcmV0MultiLocation',
        instance: 'XcmV1MultiassetAssetInstance',
      },
    },
  },
  /**
   * Lookup118: xcm::v0::multi_location::MultiLocation
   */
  XcmV0MultiLocation: {
    _enum: {
      Null: 'Null',
      X1: 'XcmV0Junction',
      X2: '(XcmV0Junction,XcmV0Junction)',
      X3: '(XcmV0Junction,XcmV0Junction,XcmV0Junction)',
      X4: '(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)',
      X5: '(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)',
      X6: '(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)',
      X7: '(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)',
      X8: '(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)',
    },
  },
  /**
   * Lookup119: xcm::v0::junction::Junction
   */
  XcmV0Junction: {
    _enum: {
      Parent: 'Null',
      Parachain: 'Compact<u32>',
      AccountId32: {
        network: 'XcmV0JunctionNetworkId',
        id: '[u8;32]',
      },
      AccountIndex64: {
        network: 'XcmV0JunctionNetworkId',
        index: 'Compact<u64>',
      },
      AccountKey20: {
        network: 'XcmV0JunctionNetworkId',
        key: '[u8;20]',
      },
      PalletInstance: 'u8',
      GeneralIndex: 'Compact<u128>',
      GeneralKey: 'Bytes',
      OnlyChild: 'Null',
      Plurality: {
        id: 'XcmV0JunctionBodyId',
        part: 'XcmV0JunctionBodyPart',
      },
    },
  },
  /**
   * Lookup120: xcm::VersionedMultiLocation
   */
  XcmVersionedMultiLocation: {
    _enum: {
      V0: 'XcmV0MultiLocation',
      V1: 'XcmV1MultiLocation',
    },
  },
  /**
   * Lookup121: pallet_assets::pallet::Event<T, I>
   */
  PalletAssetsEvent: {
    _enum: {
      Created: {
        assetId: 'u128',
        creator: 'AccountId20',
        owner: 'AccountId20',
      },
      Issued: {
        assetId: 'u128',
        owner: 'AccountId20',
        totalSupply: 'u128',
      },
      Transferred: {
        assetId: 'u128',
        from: 'AccountId20',
        to: 'AccountId20',
        amount: 'u128',
      },
      Burned: {
        assetId: 'u128',
        owner: 'AccountId20',
        balance: 'u128',
      },
      TeamChanged: {
        assetId: 'u128',
        issuer: 'AccountId20',
        admin: 'AccountId20',
        freezer: 'AccountId20',
      },
      OwnerChanged: {
        assetId: 'u128',
        owner: 'AccountId20',
      },
      Frozen: {
        assetId: 'u128',
        who: 'AccountId20',
      },
      Thawed: {
        assetId: 'u128',
        who: 'AccountId20',
      },
      AssetFrozen: {
        assetId: 'u128',
      },
      AssetThawed: {
        assetId: 'u128',
      },
      Destroyed: {
        assetId: 'u128',
      },
      ForceCreated: {
        assetId: 'u128',
        owner: 'AccountId20',
      },
      MetadataSet: {
        assetId: 'u128',
        name: 'Bytes',
        symbol: 'Bytes',
        decimals: 'u8',
        isFrozen: 'bool',
      },
      MetadataCleared: {
        assetId: 'u128',
      },
      ApprovedTransfer: {
        assetId: 'u128',
        source: 'AccountId20',
        delegate: 'AccountId20',
        amount: 'u128',
      },
      ApprovalCancelled: {
        assetId: 'u128',
        owner: 'AccountId20',
        delegate: 'AccountId20',
      },
      TransferredApproved: {
        assetId: 'u128',
        owner: 'AccountId20',
        delegate: 'AccountId20',
        destination: 'AccountId20',
        amount: 'u128',
      },
      AssetStatusChanged: {
        assetId: 'u128',
      },
    },
  },
  /**
   * Lookup122: pallet_asset_manager::pallet::Event<T>
   */
  PalletAssetManagerEvent: {
    _enum: {
      AssetRegistered: '(u128,MoonbeamRuntimeAssetType,MoonbeamRuntimeAssetRegistrarMetadata)',
      UnitsPerSecondChanged: '(MoonbeamRuntimeAssetType,u128)',
      AssetTypeChanged: '(u128,MoonbeamRuntimeAssetType)',
    },
  },
  /**
   * Lookup123: moonbeam_runtime::AssetType
   */
  MoonbeamRuntimeAssetType: {
    _enum: {
      Xcm: 'XcmV1MultiLocation',
    },
  },
  /**
   * Lookup124: moonbeam_runtime::AssetRegistrarMetadata
   */
  MoonbeamRuntimeAssetRegistrarMetadata: {
    name: 'Bytes',
    symbol: 'Bytes',
    decimals: 'u8',
    isFrozen: 'bool',
  },
  /**
   * Lookup125: orml_xtokens::module::Event<T>
   */
  OrmlXtokensModuleEvent: {
    _enum: {
      Transferred: '(AccountId20,MoonbeamRuntimeCurrencyId,u128,XcmV1MultiLocation)',
      TransferredWithFee: '(AccountId20,MoonbeamRuntimeCurrencyId,u128,u128,XcmV1MultiLocation)',
      TransferredMultiAsset: '(AccountId20,XcmV1MultiAsset,XcmV1MultiLocation)',
      TransferredMultiAssetWithFee:
        '(AccountId20,XcmV1MultiAsset,XcmV1MultiAsset,XcmV1MultiLocation)',
    },
  },
  /**
   * Lookup126: moonbeam_runtime::CurrencyId
   */
  MoonbeamRuntimeCurrencyId: {
    _enum: {
      SelfReserve: 'Null',
      OtherReserve: 'u128',
    },
  },
  /**
   * Lookup127: xcm_transactor::pallet::Event<T>
   */
  XcmTransactorEvent: {
    _enum: {
      TransactedDerivative: '(AccountId20,XcmV1MultiLocation,Bytes,u16)',
      TransactedSovereign: '(AccountId20,XcmV1MultiLocation,Bytes)',
      RegisterdDerivative: '(AccountId20,u16)',
      TransactFailed: 'XcmV2TraitsError',
      TransactInfoChanged: '(XcmV1MultiLocation,XcmTransactorRemoteTransactInfoWithMaxWeight)',
    },
  },
  /**
   * Lookup128: xcm_transactor::pallet::RemoteTransactInfoWithMaxWeight
   */
  XcmTransactorRemoteTransactInfoWithMaxWeight: {
    transactExtraWeight: 'u64',
    feePerSecond: 'u128',
    maxWeight: 'u64',
  },
  /**
   * Lookup129: frame_system::Phase
   */
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: 'u32',
      Finalization: 'Null',
      Initialization: 'Null',
    },
  },
  /**
   * Lookup131: frame_system::LastRuntimeUpgradeInfo
   */
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: 'Compact<u32>',
    specName: 'Text',
  },
  /**
   * Lookup132: frame_system::pallet::Call<T>
   */
  FrameSystemCall: {
    _enum: {
      fill_block: {
        ratio: 'Perbill',
      },
      remark: {
        remark: 'Bytes',
      },
      set_heap_pages: {
        pages: 'u64',
      },
      set_code: {
        code: 'Bytes',
      },
      set_code_without_checks: {
        code: 'Bytes',
      },
      set_storage: {
        items: 'Vec<(Bytes,Bytes)>',
      },
      kill_storage: {
        _alias: {
          keys_: 'keys',
        },
        keys_: 'Vec<Bytes>',
      },
      kill_prefix: {
        prefix: 'Bytes',
        subkeys: 'u32',
      },
      remark_with_event: {
        remark: 'Bytes',
      },
    },
  },
  /**
   * Lookup136: frame_system::limits::BlockWeights
   */
  FrameSystemLimitsBlockWeights: {
    baseBlock: 'u64',
    maxBlock: 'u64',
    perClass: 'FrameSupportWeightsPerDispatchClassWeightsPerClass',
  },
  /**
   * Lookup137:
   * frame_support::weights::PerDispatchClass<frame_system::limits::WeightsPerClass>
   */
  FrameSupportWeightsPerDispatchClassWeightsPerClass: {
    normal: 'FrameSystemLimitsWeightsPerClass',
    operational: 'FrameSystemLimitsWeightsPerClass',
    mandatory: 'FrameSystemLimitsWeightsPerClass',
  },
  /**
   * Lookup138: frame_system::limits::WeightsPerClass
   */
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: 'u64',
    maxExtrinsic: 'Option<u64>',
    maxTotal: 'Option<u64>',
    reserved: 'Option<u64>',
  },
  /**
   * Lookup140: frame_system::limits::BlockLength
   */
  FrameSystemLimitsBlockLength: {
    max: 'FrameSupportWeightsPerDispatchClassU32',
  },
  /**
   * Lookup141: frame_support::weights::PerDispatchClass<T>
   */
  FrameSupportWeightsPerDispatchClassU32: {
    normal: 'u32',
    operational: 'u32',
    mandatory: 'u32',
  },
  /**
   * Lookup142: frame_support::weights::RuntimeDbWeight
   */
  FrameSupportWeightsRuntimeDbWeight: {
    read: 'u64',
    write: 'u64',
  },
  /**
   * Lookup143: sp_version::RuntimeVersion
   */
  SpVersionRuntimeVersion: {
    specName: 'Text',
    implName: 'Text',
    authoringVersion: 'u32',
    specVersion: 'u32',
    implVersion: 'u32',
    apis: 'Vec<([u8;8],u32)>',
    transactionVersion: 'u32',
  },
  /**
   * Lookup147: frame_system::pallet::Error<T>
   */
  FrameSystemError: {
    _enum: [
      'InvalidSpecName',
      'SpecVersionNeedsToIncrease',
      'FailedToExtractRuntimeVersion',
      'NonDefaultComposite',
      'NonZeroRefCount',
      'CallFiltered',
    ],
  },
  /**
   * Lookup148: polkadot_primitives::v1::PersistedValidationData<primitive_types::H256, N>
   */
  PolkadotPrimitivesV1PersistedValidationData: {
    parentHead: 'Bytes',
    relayParentNumber: 'u32',
    relayParentStorageRoot: 'H256',
    maxPovSize: 'u32',
  },
  /**
   * Lookup151: polkadot_primitives::v1::UpgradeRestriction
   */
  PolkadotPrimitivesV1UpgradeRestriction: {
    _enum: ['Present'],
  },
  /**
   * Lookup152:
   * cumulus_pallet_parachain_system::relay_state_snapshot::MessagingStateSnapshot
   */
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: {
    dmqMqcHead: 'H256',
    relayDispatchQueueSize: '(u32,u32)',
    ingressChannels: 'Vec<(u32,PolkadotPrimitivesV1AbridgedHrmpChannel)>',
    egressChannels: 'Vec<(u32,PolkadotPrimitivesV1AbridgedHrmpChannel)>',
  },
  /**
   * Lookup156: polkadot_primitives::v1::AbridgedHrmpChannel
   */
  PolkadotPrimitivesV1AbridgedHrmpChannel: {
    maxCapacity: 'u32',
    maxTotalSize: 'u32',
    maxMessageSize: 'u32',
    msgCount: 'u32',
    totalSize: 'u32',
    mqcHead: 'Option<H256>',
  },
  /**
   * Lookup157: polkadot_primitives::v1::AbridgedHostConfiguration
   */
  PolkadotPrimitivesV1AbridgedHostConfiguration: {
    maxCodeSize: 'u32',
    maxHeadDataSize: 'u32',
    maxUpwardQueueCount: 'u32',
    maxUpwardQueueSize: 'u32',
    maxUpwardMessageSize: 'u32',
    maxUpwardMessageNumPerCandidate: 'u32',
    hrmpMaxMessageNumPerCandidate: 'u32',
    validationUpgradeFrequency: 'u32',
    validationUpgradeDelay: 'u32',
  },
  /**
   * Lookup163:
   * polkadot_core_primitives::OutboundHrmpMessage<polkadot_parachain::primitives::Id>
   */
  PolkadotCorePrimitivesOutboundHrmpMessage: {
    recipient: 'u32',
    data: 'Bytes',
  },
  /**
   * Lookup164: cumulus_pallet_parachain_system::pallet::Call<T>
   */
  CumulusPalletParachainSystemCall: {
    _enum: {
      set_validation_data: {
        data: 'CumulusPrimitivesParachainInherentParachainInherentData',
      },
      sudo_send_upward_message: {
        message: 'Bytes',
      },
      authorize_upgrade: {
        codeHash: 'H256',
      },
      enact_authorized_upgrade: {
        code: 'Bytes',
      },
    },
  },
  /**
   * Lookup165: cumulus_primitives_parachain_inherent::ParachainInherentData
   */
  CumulusPrimitivesParachainInherentParachainInherentData: {
    validationData: 'PolkadotPrimitivesV1PersistedValidationData',
    relayChainState: 'SpTrieStorageProof',
    downwardMessages: 'Vec<PolkadotCorePrimitivesInboundDownwardMessage>',
    horizontalMessages: 'BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>',
  },
  /**
   * Lookup166: sp_trie::storage_proof::StorageProof
   */
  SpTrieStorageProof: {
    trieNodes: 'Vec<Bytes>',
  },
  /**
   * Lookup168: polkadot_core_primitives::InboundDownwardMessage<BlockNumber>
   */
  PolkadotCorePrimitivesInboundDownwardMessage: {
    sentAt: 'u32',
    msg: 'Bytes',
  },
  /**
   * Lookup171: polkadot_core_primitives::InboundHrmpMessage<BlockNumber>
   */
  PolkadotCorePrimitivesInboundHrmpMessage: {
    sentAt: 'u32',
    data: 'Bytes',
  },
  /**
   * Lookup174: cumulus_pallet_parachain_system::pallet::Error<T>
   */
  CumulusPalletParachainSystemError: {
    _enum: [
      'OverlappingUpgrades',
      'ProhibitedByPolkadot',
      'TooBig',
      'ValidationDataNotAvailable',
      'HostConfigurationNotAvailable',
      'NotScheduled',
      'NothingAuthorized',
      'Unauthorized',
    ],
  },
  /**
   * Lookup175: pallet_timestamp::pallet::Call<T>
   */
  PalletTimestampCall: {
    _enum: {
      set: {
        now: 'Compact<u64>',
      },
    },
  },
  /**
   * Lookup177: pallet_balances::BalanceLock<Balance>
   */
  PalletBalancesBalanceLock: {
    id: '[u8;8]',
    amount: 'u128',
    reasons: 'PalletBalancesReasons',
  },
  /**
   * Lookup178: pallet_balances::Reasons
   */
  PalletBalancesReasons: {
    _enum: ['Fee', 'Misc', 'All'],
  },
  /**
   * Lookup181: pallet_balances::ReserveData<ReserveIdentifier, Balance>
   */
  PalletBalancesReserveData: {
    id: '[u8;4]',
    amount: 'u128',
  },
  /**
   * Lookup183: pallet_balances::Releases
   */
  PalletBalancesReleases: {
    _enum: ['V1_0_0', 'V2_0_0'],
  },
  /**
   * Lookup184: pallet_balances::pallet::Call<T, I>
   */
  PalletBalancesCall: {
    _enum: {
      transfer: {
        dest: 'AccountId20',
        value: 'Compact<u128>',
      },
      set_balance: {
        who: 'AccountId20',
        newFree: 'Compact<u128>',
        newReserved: 'Compact<u128>',
      },
      force_transfer: {
        source: 'AccountId20',
        dest: 'AccountId20',
        value: 'Compact<u128>',
      },
      transfer_keep_alive: {
        dest: 'AccountId20',
        value: 'Compact<u128>',
      },
      transfer_all: {
        dest: 'AccountId20',
        keepAlive: 'bool',
      },
      force_unreserve: {
        who: 'AccountId20',
        amount: 'u128',
      },
    },
  },
  /**
   * Lookup185: pallet_balances::pallet::Error<T, I>
   */
  PalletBalancesError: {
    _enum: [
      'VestingBalance',
      'LiquidityRestrictions',
      'InsufficientBalance',
      'ExistentialDeposit',
      'KeepAlive',
      'ExistingVestingSchedule',
      'DeadAccount',
      'TooManyReserves',
    ],
  },
  /**
   * Lookup187: pallet_transaction_payment::Releases
   */
  PalletTransactionPaymentReleases: {
    _enum: ['V1Ancient', 'V2'],
  },
  /**
   * Lookup189: frame_support::weights::WeightToFeeCoefficient<Balance>
   */
  FrameSupportWeightsWeightToFeeCoefficient: {
    coeffInteger: 'u128',
    coeffFrac: 'Perbill',
    negative: 'bool',
    degree: 'u8',
  },
  /**
   * Lookup190:
   * parachain_staking::pallet::ParachainBondConfig[account::AccountId20](account::AccountId20)
   */
  ParachainStakingParachainBondConfig: {
    account: 'AccountId20',
    percent: 'Percent',
  },
  /**
   * Lookup191: parachain_staking::pallet::RoundInfo<BlockNumber>
   */
  ParachainStakingRoundInfo: {
    current: 'u32',
    first: 'u32',
    length: 'u32',
  },
  /**
   * Lookup192: parachain_staking::pallet::Nominator2<account::AccountId20, Balance>
   */
  ParachainStakingNominator2: {
    delegations: 'ParachainStakingSetOrderedSetBond',
    revocations: 'ParachainStakingSetOrderedSetAccountId20',
    total: 'u128',
    scheduledRevocationsCount: 'u32',
    scheduledRevocationsTotal: 'u128',
    status: 'ParachainStakingDelegatorStatus',
  },
  /**
   * Lookup193:
   * parachain_staking::set::OrderedSet<parachain_staking::pallet::Bond<account::AccountId20,
   * Balance>>
   */
  ParachainStakingSetOrderedSetBond: 'Vec<ParachainStakingBond>',
  /**
   * Lookup194: parachain_staking::pallet::Bond<account::AccountId20, Balance>
   */
  ParachainStakingBond: {
    owner: 'AccountId20',
    amount: 'u128',
  },
  /**
   * Lookup196:
   * parachain_staking::set::OrderedSet[account::AccountId20](account::AccountId20)
   */
  ParachainStakingSetOrderedSetAccountId20: 'Vec<AccountId20>',
  /**
   * Lookup197: parachain_staking::pallet::DelegatorStatus
   */
  ParachainStakingDelegatorStatus: {
    _enum: {
      Active: 'Null',
      Leaving: 'u32',
    },
  },
  /**
   * Lookup198: parachain_staking::pallet::Delegator<account::AccountId20, Balance>
   */
  ParachainStakingDelegator: {
    id: 'AccountId20',
    delegations: 'ParachainStakingSetOrderedSetBond',
    total: 'u128',
    requests: 'ParachainStakingPendingDelegationRequests',
    status: 'ParachainStakingDelegatorStatus',
  },
  /**
   * Lookup199:
   * parachain_staking::pallet::PendingDelegationRequests<account::AccountId20, Balance>
   */
  ParachainStakingPendingDelegationRequests: {
    revocationsCount: 'u32',
    requests: 'BTreeMap<AccountId20, ParachainStakingDelegationRequest>',
    lessTotal: 'u128',
  },
  /**
   * Lookup203:
   * parachain_staking::pallet::CollatorCandidate<account::AccountId20, Balance>
   */
  ParachainStakingCollatorCandidate: {
    id: 'AccountId20',
    bond: 'u128',
    delegators: 'ParachainStakingSetOrderedSetAccountId20',
    topDelegations: 'Vec<ParachainStakingBond>',
    bottomDelegations: 'Vec<ParachainStakingBond>',
    totalCounted: 'u128',
    totalBacking: 'u128',
    request: 'Option<ParachainStakingCandidateBondLessRequest>',
    state: 'ParachainStakingCollatorStatus',
  },
  /**
   * Lookup205: parachain_staking::pallet::CandidateBondLessRequest<Balance>
   */
  ParachainStakingCandidateBondLessRequest: {
    amount: 'u128',
    whenExecutable: 'u32',
  },
  /**
   * Lookup206: parachain_staking::pallet::CollatorStatus
   */
  ParachainStakingCollatorStatus: {
    _enum: {
      Active: 'Null',
      Idle: 'Null',
      Leaving: 'u32',
    },
  },
  /**
   * Lookup207: parachain_staking::pallet::CandidateMetadata<Balance>
   */
  ParachainStakingCandidateMetadata: {
    bond: 'u128',
    delegationCount: 'u32',
    totalCounted: 'u128',
    lowestTopDelegationAmount: 'u128',
    highestBottomDelegationAmount: 'u128',
    lowestBottomDelegationAmount: 'u128',
    topCapacity: 'ParachainStakingCapacityStatus',
    bottomCapacity: 'ParachainStakingCapacityStatus',
    request: 'Option<ParachainStakingCandidateBondLessRequest>',
    status: 'ParachainStakingCollatorStatus',
  },
  /**
   * Lookup208: parachain_staking::pallet::CapacityStatus
   */
  ParachainStakingCapacityStatus: {
    _enum: ['Full', 'Empty', 'Partial'],
  },
  /**
   * Lookup209: parachain_staking::pallet::Delegations<account::AccountId20, Balance>
   */
  ParachainStakingDelegations: {
    delegations: 'Vec<ParachainStakingBond>',
    total: 'u128',
  },
  /**
   * Lookup210: parachain_staking::pallet::Collator2<account::AccountId20, Balance>
   */
  ParachainStakingCollator2: {
    id: 'AccountId20',
    bond: 'u128',
    nominators: 'ParachainStakingSetOrderedSetAccountId20',
    topNominators: 'Vec<ParachainStakingBond>',
    bottomNominators: 'Vec<ParachainStakingBond>',
    totalCounted: 'u128',
    totalBacking: 'u128',
    state: 'ParachainStakingCollatorStatus',
  },
  /**
   * Lookup211:
   * parachain_staking::pallet::ExitQ[account::AccountId20](account::AccountId20)
   */
  ParachainStakingExitQ: {
    candidates: 'ParachainStakingSetOrderedSetAccountId20',
    nominatorsLeaving: 'ParachainStakingSetOrderedSetAccountId20',
    candidateSchedule: 'Vec<(AccountId20,u32)>',
    nominatorSchedule: 'Vec<(AccountId20,Option<AccountId20>,u32)>',
  },
  /**
   * Lookup217: parachain_staking::pallet::CollatorSnapshot<account::AccountId20, Balance>
   */
  ParachainStakingCollatorSnapshot: {
    bond: 'u128',
    delegations: 'Vec<ParachainStakingBond>',
    total: 'u128',
  },
  /**
   * Lookup218: parachain_staking::pallet::DelayedPayout<Balance>
   */
  ParachainStakingDelayedPayout: {
    roundIssuance: 'u128',
    totalStakingReward: 'u128',
    collatorCommission: 'Perbill',
  },
  /**
   * Lookup219: parachain_staking::inflation::InflationInfo<Balance>
   */
  ParachainStakingInflationInflationInfo: {
    expect: 'ParachainStakingInflationRangeU128',
    annual: 'ParachainStakingInflationRangePerbill',
    round: 'ParachainStakingInflationRangePerbill',
  },
  /**
   * Lookup220: parachain_staking::inflation::Range<T>
   */
  ParachainStakingInflationRangeU128: {
    min: 'u128',
    ideal: 'u128',
    max: 'u128',
  },
  /**
   * Lookup221: parachain_staking::inflation::Range<sp_arithmetic::per_things::Perbill>
   */
  ParachainStakingInflationRangePerbill: {
    min: 'Perbill',
    ideal: 'Perbill',
    max: 'Perbill',
  },
  /**
   * Lookup222: parachain_staking::pallet::Call<T>
   */
  ParachainStakingCall: {
    _enum: {
      hotfix_remove_delegation_requests: {
        delegators: 'Vec<AccountId20>',
      },
      hotfix_update_candidate_pool_value: {
        candidates: 'Vec<AccountId20>',
      },
      set_staking_expectations: {
        expectations: 'ParachainStakingInflationRangeU128',
      },
      set_inflation: {
        schedule: 'ParachainStakingInflationRangePerbill',
      },
      set_parachain_bond_account: {
        _alias: {
          new_: 'new',
        },
        new_: 'AccountId20',
      },
      set_parachain_bond_reserve_percent: {
        _alias: {
          new_: 'new',
        },
        new_: 'Percent',
      },
      set_total_selected: {
        _alias: {
          new_: 'new',
        },
        new_: 'u32',
      },
      set_collator_commission: {
        _alias: {
          new_: 'new',
        },
        new_: 'Perbill',
      },
      set_blocks_per_round: {
        _alias: {
          new_: 'new',
        },
        new_: 'u32',
      },
      join_candidates: {
        bond: 'u128',
        candidateCount: 'u32',
      },
      schedule_leave_candidates: {
        candidateCount: 'u32',
      },
      execute_leave_candidates: {
        candidate: 'AccountId20',
        candidateDelegationCount: 'u32',
      },
      cancel_leave_candidates: {
        candidateCount: 'u32',
      },
      go_offline: 'Null',
      go_online: 'Null',
      candidate_bond_more: {
        more: 'u128',
      },
      schedule_candidate_bond_less: {
        less: 'u128',
      },
      execute_candidate_bond_less: {
        candidate: 'AccountId20',
      },
      cancel_candidate_bond_less: 'Null',
      delegate: {
        candidate: 'AccountId20',
        amount: 'u128',
        candidateDelegationCount: 'u32',
        delegationCount: 'u32',
      },
      schedule_leave_delegators: 'Null',
      execute_leave_delegators: {
        delegator: 'AccountId20',
        delegationCount: 'u32',
      },
      cancel_leave_delegators: 'Null',
      schedule_revoke_delegation: {
        collator: 'AccountId20',
      },
      delegator_bond_more: {
        candidate: 'AccountId20',
        more: 'u128',
      },
      schedule_delegator_bond_less: {
        candidate: 'AccountId20',
        less: 'u128',
      },
      execute_delegation_request: {
        delegator: 'AccountId20',
        candidate: 'AccountId20',
      },
      cancel_delegation_request: {
        candidate: 'AccountId20',
      },
    },
  },
  /**
   * Lookup223: parachain_staking::pallet::Error<T>
   */
  ParachainStakingError: {
    _enum: [
      'DelegatorDNE',
      'DelegatorDNEinTopNorBottom',
      'DelegatorDNEInDelegatorSet',
      'CandidateDNE',
      'DelegationDNE',
      'DelegatorExists',
      'CandidateExists',
      'CandidateBondBelowMin',
      'InsufficientBalance',
      'DelegatorBondBelowMin',
      'DelegationBelowMin',
      'AlreadyOffline',
      'AlreadyActive',
      'DelegatorAlreadyLeaving',
      'DelegatorNotLeaving',
      'DelegatorCannotLeaveYet',
      'CannotDelegateIfLeaving',
      'CandidateAlreadyLeaving',
      'CandidateNotLeaving',
      'CandidateCannotLeaveYet',
      'CannotGoOnlineIfLeaving',
      'ExceedMaxDelegationsPerDelegator',
      'AlreadyDelegatedCandidate',
      'InvalidSchedule',
      'CannotSetBelowMin',
      'RoundLengthMustBeAtLeastTotalSelectedCollators',
      'NoWritingSameValue',
      'TooLowCandidateCountWeightHintJoinCandidates',
      'TooLowCandidateCountWeightHintCancelLeaveCandidates',
      'TooLowCandidateCountToLeaveCandidates',
      'TooLowDelegationCountToDelegate',
      'TooLowCandidateDelegationCountToDelegate',
      'TooLowCandidateDelegationCountToLeaveCandidates',
      'TooLowDelegationCountToLeaveDelegators',
      'PendingCandidateRequestsDNE',
      'PendingCandidateRequestAlreadyExists',
      'PendingCandidateRequestNotDueYet',
      'PendingDelegationRequestDNE',
      'PendingDelegationRequestAlreadyExists',
      'PendingDelegationRequestNotDueYet',
      'CannotDelegateLessThanLowestBottomWhenBottomIsFull',
    ],
  },
  /**
   * Lookup224: pallet_author_inherent::pallet::Call<T>
   */
  PalletAuthorInherentCall: {
    _enum: ['kick_off_authorship_validation'],
  },
  /**
   * Lookup225: pallet_author_inherent::pallet::Error<T>
   */
  PalletAuthorInherentError: {
    _enum: ['AuthorAlreadySet', 'NoAccountId', 'CannotBeAuthor'],
  },
  /**
   * Lookup226: pallet_author_slot_filter::pallet::Call<T>
   */
  PalletAuthorSlotFilterCall: {
    _enum: {
      set_eligible: {
        _alias: {
          new_: 'new',
        },
        new_: 'Percent',
      },
    },
  },
  /**
   * Lookup227:
   * pallet_author_mapping::pallet::RegistrationInfo<account::AccountId20, Balance>
   */
  PalletAuthorMappingRegistrationInfo: {
    account: 'AccountId20',
    deposit: 'u128',
  },
  /**
   * Lookup228: pallet_author_mapping::pallet::Call<T>
   */
  PalletAuthorMappingCall: {
    _enum: {
      add_association: {
        authorId: 'NimbusPrimitivesNimbusCryptoPublic',
      },
      update_association: {
        oldAuthorId: 'NimbusPrimitivesNimbusCryptoPublic',
        newAuthorId: 'NimbusPrimitivesNimbusCryptoPublic',
      },
      clear_association: {
        authorId: 'NimbusPrimitivesNimbusCryptoPublic',
      },
    },
  },
  /**
   * Lookup229: pallet_author_mapping::pallet::Error<T>
   */
  PalletAuthorMappingError: {
    _enum: [
      'AssociationNotFound',
      'NotYourAssociation',
      'CannotAffordSecurityDeposit',
      'AlreadyAssociated',
    ],
  },
  /**
   * Lookup230: pallet_utility::pallet::Call<T>
   */
  PalletUtilityCall: {
    _enum: {
      batch: {
        calls: 'Vec<Call>',
      },
      as_derivative: {
        index: 'u16',
        call: 'Call',
      },
      batch_all: {
        calls: 'Vec<Call>',
      },
      dispatch_as: {
        asOrigin: 'MoonbeamRuntimeOriginCaller',
        call: 'Call',
      },
    },
  },
  /**
   * Lookup233: pallet_proxy::pallet::Call<T>
   */
  PalletProxyCall: {
    _enum: {
      proxy: {
        real: 'AccountId20',
        forceProxyType: 'Option<MoonbeamRuntimeProxyType>',
        call: 'Call',
      },
      add_proxy: {
        delegate: 'AccountId20',
        proxyType: 'MoonbeamRuntimeProxyType',
        delay: 'u32',
      },
      remove_proxy: {
        delegate: 'AccountId20',
        proxyType: 'MoonbeamRuntimeProxyType',
        delay: 'u32',
      },
      remove_proxies: 'Null',
      anonymous: {
        proxyType: 'MoonbeamRuntimeProxyType',
        delay: 'u32',
        index: 'u16',
      },
      kill_anonymous: {
        spawner: 'AccountId20',
        proxyType: 'MoonbeamRuntimeProxyType',
        index: 'u16',
        height: 'Compact<u32>',
        extIndex: 'Compact<u32>',
      },
      announce: {
        real: 'AccountId20',
        callHash: 'H256',
      },
      remove_announcement: {
        real: 'AccountId20',
        callHash: 'H256',
      },
      reject_announcement: {
        delegate: 'AccountId20',
        callHash: 'H256',
      },
      proxy_announced: {
        delegate: 'AccountId20',
        real: 'AccountId20',
        forceProxyType: 'Option<MoonbeamRuntimeProxyType>',
        call: 'Call',
      },
    },
  },
  /**
   * Lookup235: pallet_maintenance_mode::pallet::Call<T>
   */
  PalletMaintenanceModeCall: {
    _enum: ['enter_maintenance_mode', 'resume_normal_operation'],
  },
  /**
   * Lookup236: pallet_identity::pallet::Call<T>
   */
  PalletIdentityCall: {
    _enum: {
      add_registrar: {
        account: 'AccountId20',
      },
      set_identity: {
        info: 'PalletIdentityIdentityInfo',
      },
      set_subs: {
        subs: 'Vec<(AccountId20,Data)>',
      },
      clear_identity: 'Null',
      request_judgement: {
        regIndex: 'Compact<u32>',
        maxFee: 'Compact<u128>',
      },
      cancel_request: {
        regIndex: 'u32',
      },
      set_fee: {
        index: 'Compact<u32>',
        fee: 'Compact<u128>',
      },
      set_account_id: {
        _alias: {
          new_: 'new',
        },
        index: 'Compact<u32>',
        new_: 'AccountId20',
      },
      set_fields: {
        index: 'Compact<u32>',
        fields: 'PalletIdentityBitFlags',
      },
      provide_judgement: {
        regIndex: 'Compact<u32>',
        target: 'AccountId20',
        judgement: 'PalletIdentityJudgement',
      },
      kill_identity: {
        target: 'AccountId20',
      },
      add_sub: {
        sub: 'AccountId20',
        data: 'Data',
      },
      rename_sub: {
        sub: 'AccountId20',
        data: 'Data',
      },
      remove_sub: {
        sub: 'AccountId20',
      },
      quit_sub: 'Null',
    },
  },
  /**
   * Lookup237: pallet_identity::types::IdentityInfo<FieldLimit>
   */
  PalletIdentityIdentityInfo: {
    additional: 'Vec<(Data,Data)>',
    display: 'Data',
    legal: 'Data',
    web: 'Data',
    riot: 'Data',
    email: 'Data',
    pgpFingerprint: 'Option<[u8;20]>',
    image: 'Data',
    twitter: 'Data',
  },
  /**
   * Lookup273: pallet_identity::types::BitFlags<pallet_identity::types::IdentityField>
   */
  PalletIdentityBitFlags: {
    _bitLength: 64,
    Display: 1,
    Legal: 2,
    Web: 4,
    Riot: 8,
    Email: 16,
    PgpFingerprint: 32,
    Image: 64,
    Twitter: 128,
  },
  /**
   * Lookup274: pallet_identity::types::IdentityField
   */
  PalletIdentityIdentityField: {
    _enum: [
      '__Unused0',
      'Display',
      'Legal',
      '__Unused3',
      'Web',
      '__Unused5',
      '__Unused6',
      '__Unused7',
      'Riot',
      '__Unused9',
      '__Unused10',
      '__Unused11',
      '__Unused12',
      '__Unused13',
      '__Unused14',
      '__Unused15',
      'Email',
      '__Unused17',
      '__Unused18',
      '__Unused19',
      '__Unused20',
      '__Unused21',
      '__Unused22',
      '__Unused23',
      '__Unused24',
      '__Unused25',
      '__Unused26',
      '__Unused27',
      '__Unused28',
      '__Unused29',
      '__Unused30',
      '__Unused31',
      'PgpFingerprint',
      '__Unused33',
      '__Unused34',
      '__Unused35',
      '__Unused36',
      '__Unused37',
      '__Unused38',
      '__Unused39',
      '__Unused40',
      '__Unused41',
      '__Unused42',
      '__Unused43',
      '__Unused44',
      '__Unused45',
      '__Unused46',
      '__Unused47',
      '__Unused48',
      '__Unused49',
      '__Unused50',
      '__Unused51',
      '__Unused52',
      '__Unused53',
      '__Unused54',
      '__Unused55',
      '__Unused56',
      '__Unused57',
      '__Unused58',
      '__Unused59',
      '__Unused60',
      '__Unused61',
      '__Unused62',
      '__Unused63',
      'Image',
      '__Unused65',
      '__Unused66',
      '__Unused67',
      '__Unused68',
      '__Unused69',
      '__Unused70',
      '__Unused71',
      '__Unused72',
      '__Unused73',
      '__Unused74',
      '__Unused75',
      '__Unused76',
      '__Unused77',
      '__Unused78',
      '__Unused79',
      '__Unused80',
      '__Unused81',
      '__Unused82',
      '__Unused83',
      '__Unused84',
      '__Unused85',
      '__Unused86',
      '__Unused87',
      '__Unused88',
      '__Unused89',
      '__Unused90',
      '__Unused91',
      '__Unused92',
      '__Unused93',
      '__Unused94',
      '__Unused95',
      '__Unused96',
      '__Unused97',
      '__Unused98',
      '__Unused99',
      '__Unused100',
      '__Unused101',
      '__Unused102',
      '__Unused103',
      '__Unused104',
      '__Unused105',
      '__Unused106',
      '__Unused107',
      '__Unused108',
      '__Unused109',
      '__Unused110',
      '__Unused111',
      '__Unused112',
      '__Unused113',
      '__Unused114',
      '__Unused115',
      '__Unused116',
      '__Unused117',
      '__Unused118',
      '__Unused119',
      '__Unused120',
      '__Unused121',
      '__Unused122',
      '__Unused123',
      '__Unused124',
      '__Unused125',
      '__Unused126',
      '__Unused127',
      'Twitter',
    ],
  },
  /**
   * Lookup275: pallet_identity::types::Judgement<Balance>
   */
  PalletIdentityJudgement: {
    _enum: {
      Unknown: 'Null',
      FeePaid: 'u128',
      Reasonable: 'Null',
      KnownGood: 'Null',
      OutOfDate: 'Null',
      LowQuality: 'Null',
      Erroneous: 'Null',
    },
  },
  /**
   * Lookup276: pallet_evm::pallet::Call<T>
   */
  PalletEvmCall: {
    _enum: {
      withdraw: {
        address: 'H160',
        value: 'u128',
      },
      call: {
        source: 'H160',
        target: 'H160',
        input: 'Bytes',
        value: 'U256',
        gasLimit: 'u64',
        maxFeePerGas: 'U256',
        maxPriorityFeePerGas: 'Option<U256>',
        nonce: 'Option<U256>',
        accessList: 'Vec<(H160,Vec<H256>)>',
      },
      create: {
        source: 'H160',
        init: 'Bytes',
        value: 'U256',
        gasLimit: 'u64',
        maxFeePerGas: 'U256',
        maxPriorityFeePerGas: 'Option<U256>',
        nonce: 'Option<U256>',
        accessList: 'Vec<(H160,Vec<H256>)>',
      },
      create2: {
        source: 'H160',
        init: 'Bytes',
        salt: 'H256',
        value: 'U256',
        gasLimit: 'u64',
        maxFeePerGas: 'U256',
        maxPriorityFeePerGas: 'Option<U256>',
        nonce: 'Option<U256>',
        accessList: 'Vec<(H160,Vec<H256>)>',
      },
    },
  },
  /**
   * Lookup280: pallet_ethereum::pallet::Call<T>
   */
  PalletEthereumCall: {
    _enum: {
      transact: {
        transaction: 'EthereumTransactionTransactionV2',
      },
    },
  },
  /**
   * Lookup281: ethereum::transaction::TransactionV2
   */
  EthereumTransactionTransactionV2: {
    _enum: {
      Legacy: 'EthereumTransactionLegacyTransaction',
      EIP2930: 'EthereumTransactionEip2930Transaction',
      EIP1559: 'EthereumTransactionEip1559Transaction',
    },
  },
  /**
   * Lookup282: ethereum::transaction::LegacyTransaction
   */
  EthereumTransactionLegacyTransaction: {
    nonce: 'U256',
    gasPrice: 'U256',
    gasLimit: 'U256',
    action: 'EthereumTransactionTransactionAction',
    value: 'U256',
    input: 'Bytes',
    signature: 'EthereumTransactionTransactionSignature',
  },
  /**
   * Lookup283: ethereum::transaction::TransactionAction
   */
  EthereumTransactionTransactionAction: {
    _enum: {
      Call: 'H160',
      Create: 'Null',
    },
  },
  /**
   * Lookup284: ethereum::transaction::TransactionSignature
   */
  EthereumTransactionTransactionSignature: {
    v: 'u64',
    r: 'H256',
    s: 'H256',
  },
  /**
   * Lookup286: ethereum::transaction::EIP2930Transaction
   */
  EthereumTransactionEip2930Transaction: {
    chainId: 'u64',
    nonce: 'U256',
    gasPrice: 'U256',
    gasLimit: 'U256',
    action: 'EthereumTransactionTransactionAction',
    value: 'U256',
    input: 'Bytes',
    accessList: 'Vec<EthereumTransactionAccessListItem>',
    oddYParity: 'bool',
    r: 'H256',
    s: 'H256',
  },
  /**
   * Lookup288: ethereum::transaction::AccessListItem
   */
  EthereumTransactionAccessListItem: {
    address: 'H160',
    storageKeys: 'Vec<H256>',
  },
  /**
   * Lookup289: ethereum::transaction::EIP1559Transaction
   */
  EthereumTransactionEip1559Transaction: {
    chainId: 'u64',
    nonce: 'U256',
    maxPriorityFeePerGas: 'U256',
    maxFeePerGas: 'U256',
    gasLimit: 'U256',
    action: 'EthereumTransactionTransactionAction',
    value: 'U256',
    input: 'Bytes',
    accessList: 'Vec<EthereumTransactionAccessListItem>',
    oddYParity: 'bool',
    r: 'H256',
    s: 'H256',
  },
  /**
   * Lookup290: pallet_base_fee::pallet::Call<T>
   */
  PalletBaseFeeCall: {
    _enum: {
      set_base_fee_per_gas: {
        fee: 'U256',
      },
      set_is_active: {
        isActive: 'bool',
      },
      set_elasticity: {
        elasticity: 'Permill',
      },
    },
  },
  /**
   * Lookup291: pallet_scheduler::pallet::Call<T>
   */
  PalletSchedulerCall: {
    _enum: {
      schedule: {
        when: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      cancel: {
        when: 'u32',
        index: 'u32',
      },
      schedule_named: {
        id: 'Bytes',
        when: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      cancel_named: {
        id: 'Bytes',
      },
      schedule_after: {
        after: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      schedule_named_after: {
        id: 'Bytes',
        after: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'Call',
      },
    },
  },
  /**
   * Lookup293: pallet_democracy::pallet::Call<T>
   */
  PalletDemocracyCall: {
    _enum: {
      propose: {
        proposalHash: 'H256',
        value: 'Compact<u128>',
      },
      second: {
        proposal: 'Compact<u32>',
        secondsUpperBound: 'Compact<u32>',
      },
      vote: {
        refIndex: 'Compact<u32>',
        vote: 'PalletDemocracyVoteAccountVote',
      },
      emergency_cancel: {
        refIndex: 'u32',
      },
      external_propose: {
        proposalHash: 'H256',
      },
      external_propose_majority: {
        proposalHash: 'H256',
      },
      external_propose_default: {
        proposalHash: 'H256',
      },
      fast_track: {
        proposalHash: 'H256',
        votingPeriod: 'u32',
        delay: 'u32',
      },
      veto_external: {
        proposalHash: 'H256',
      },
      cancel_referendum: {
        refIndex: 'Compact<u32>',
      },
      cancel_queued: {
        which: 'u32',
      },
      delegate: {
        to: 'AccountId20',
        conviction: 'PalletDemocracyConviction',
        balance: 'u128',
      },
      undelegate: 'Null',
      clear_public_proposals: 'Null',
      note_preimage: {
        encodedProposal: 'Bytes',
      },
      note_preimage_operational: {
        encodedProposal: 'Bytes',
      },
      note_imminent_preimage: {
        encodedProposal: 'Bytes',
      },
      note_imminent_preimage_operational: {
        encodedProposal: 'Bytes',
      },
      reap_preimage: {
        proposalHash: 'H256',
        proposalLenUpperBound: 'Compact<u32>',
      },
      unlock: {
        target: 'AccountId20',
      },
      remove_vote: {
        index: 'u32',
      },
      remove_other_vote: {
        target: 'AccountId20',
        index: 'u32',
      },
      enact_proposal: {
        proposalHash: 'H256',
        index: 'u32',
      },
      blacklist: {
        proposalHash: 'H256',
        maybeRefIndex: 'Option<u32>',
      },
      cancel_proposal: {
        propIndex: 'Compact<u32>',
      },
    },
  },
  /**
   * Lookup294: pallet_democracy::conviction::Conviction
   */
  PalletDemocracyConviction: {
    _enum: ['None', 'Locked1x', 'Locked2x', 'Locked3x', 'Locked4x', 'Locked5x', 'Locked6x'],
  },
  /**
   * Lookup296: pallet_collective::pallet::Call<T, I>
   */
  PalletCollectiveCall: {
    _enum: {
      set_members: {
        newMembers: 'Vec<AccountId20>',
        prime: 'Option<AccountId20>',
        oldCount: 'u32',
      },
      execute: {
        proposal: 'Call',
        lengthBound: 'Compact<u32>',
      },
      propose: {
        threshold: 'Compact<u32>',
        proposal: 'Call',
        lengthBound: 'Compact<u32>',
      },
      vote: {
        proposal: 'H256',
        index: 'Compact<u32>',
        approve: 'bool',
      },
      close: {
        proposalHash: 'H256',
        index: 'Compact<u32>',
        proposalWeightBound: 'Compact<u64>',
        lengthBound: 'Compact<u32>',
      },
      disapprove_proposal: {
        proposalHash: 'H256',
      },
    },
  },
  /**
   * Lookup298: pallet_treasury::pallet::Call<T, I>
   */
  PalletTreasuryCall: {
    _enum: {
      propose_spend: {
        value: 'Compact<u128>',
        beneficiary: 'AccountId20',
      },
      reject_proposal: {
        proposalId: 'Compact<u32>',
      },
      approve_proposal: {
        proposalId: 'Compact<u32>',
      },
    },
  },
  /**
   * Lookup299: pallet_crowdloan_rewards::pallet::Call<T>
   */
  PalletCrowdloanRewardsCall: {
    _enum: {
      associate_native_identity: {
        rewardAccount: 'AccountId20',
        relayAccount: '[u8;32]',
        proof: 'SpRuntimeMultiSignature',
      },
      change_association_with_relay_keys: {
        rewardAccount: 'AccountId20',
        previousAccount: 'AccountId20',
        proofs: 'Vec<([u8;32],SpRuntimeMultiSignature)>',
      },
      claim: 'Null',
      update_reward_address: {
        newRewardAccount: 'AccountId20',
      },
      complete_initialization: {
        leaseEndingBlock: 'u32',
      },
      initialize_reward_vec: {
        rewards: 'Vec<([u8;32],Option<AccountId20>,u128)>',
      },
    },
  },
  /**
   * Lookup300: sp_runtime::MultiSignature
   */
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: 'SpCoreEd25519Signature',
      Sr25519: 'SpCoreSr25519Signature',
      Ecdsa: 'SpCoreEcdsaSignature',
    },
  },
  /**
   * Lookup301: sp_core::ed25519::Signature
   */
  SpCoreEd25519Signature: '[u8;64]',
  /**
   * Lookup303: sp_core::sr25519::Signature
   */
  SpCoreSr25519Signature: '[u8;64]',
  /**
   * Lookup304: sp_core::ecdsa::Signature
   */
  SpCoreEcdsaSignature: '[u8;65]',
  /**
   * Lookup310: cumulus_pallet_dmp_queue::pallet::Call<T>
   */
  CumulusPalletDmpQueueCall: {
    _enum: {
      service_overweight: {
        index: 'u64',
        weightLimit: 'u64',
      },
    },
  },
  /**
   * Lookup311: pallet_xcm::pallet::Call<T>
   */
  PalletXcmCall: {
    _enum: {
      send: {
        dest: 'XcmVersionedMultiLocation',
        message: 'XcmVersionedXcm',
      },
      teleport_assets: {
        dest: 'XcmVersionedMultiLocation',
        beneficiary: 'XcmVersionedMultiLocation',
        assets: 'XcmVersionedMultiAssets',
        feeAssetItem: 'u32',
      },
      reserve_transfer_assets: {
        dest: 'XcmVersionedMultiLocation',
        beneficiary: 'XcmVersionedMultiLocation',
        assets: 'XcmVersionedMultiAssets',
        feeAssetItem: 'u32',
      },
      execute: {
        message: 'XcmVersionedXcm',
        maxWeight: 'u64',
      },
      force_xcm_version: {
        location: 'XcmV1MultiLocation',
        xcmVersion: 'u32',
      },
      force_default_xcm_version: {
        maybeXcmVersion: 'Option<u32>',
      },
      force_subscribe_version_notify: {
        location: 'XcmVersionedMultiLocation',
      },
      force_unsubscribe_version_notify: {
        location: 'XcmVersionedMultiLocation',
      },
      limited_reserve_transfer_assets: {
        dest: 'XcmVersionedMultiLocation',
        beneficiary: 'XcmVersionedMultiLocation',
        assets: 'XcmVersionedMultiAssets',
        feeAssetItem: 'u32',
        weightLimit: 'XcmV2WeightLimit',
      },
      limited_teleport_assets: {
        dest: 'XcmVersionedMultiLocation',
        beneficiary: 'XcmVersionedMultiLocation',
        assets: 'XcmVersionedMultiAssets',
        feeAssetItem: 'u32',
        weightLimit: 'XcmV2WeightLimit',
      },
    },
  },
  /**
   * Lookup312: xcm::VersionedXcm<Call>
   */
  XcmVersionedXcm: {
    _enum: {
      V0: 'XcmV0Xcm',
      V1: 'XcmV1Xcm',
      V2: 'XcmV2Xcm',
    },
  },
  /**
   * Lookup313: xcm::v0::Xcm<Call>
   */
  XcmV0Xcm: {
    _enum: {
      WithdrawAsset: {
        assets: 'Vec<XcmV0MultiAsset>',
        effects: 'Vec<XcmV0Order>',
      },
      ReserveAssetDeposit: {
        assets: 'Vec<XcmV0MultiAsset>',
        effects: 'Vec<XcmV0Order>',
      },
      TeleportAsset: {
        assets: 'Vec<XcmV0MultiAsset>',
        effects: 'Vec<XcmV0Order>',
      },
      QueryResponse: {
        queryId: 'Compact<u64>',
        response: 'XcmV0Response',
      },
      TransferAsset: {
        assets: 'Vec<XcmV0MultiAsset>',
        dest: 'XcmV0MultiLocation',
      },
      TransferReserveAsset: {
        assets: 'Vec<XcmV0MultiAsset>',
        dest: 'XcmV0MultiLocation',
        effects: 'Vec<XcmV0Order>',
      },
      Transact: {
        originType: 'XcmV0OriginKind',
        requireWeightAtMost: 'u64',
        call: 'XcmDoubleEncoded',
      },
      HrmpNewChannelOpenRequest: {
        sender: 'Compact<u32>',
        maxMessageSize: 'Compact<u32>',
        maxCapacity: 'Compact<u32>',
      },
      HrmpChannelAccepted: {
        recipient: 'Compact<u32>',
      },
      HrmpChannelClosing: {
        initiator: 'Compact<u32>',
        sender: 'Compact<u32>',
        recipient: 'Compact<u32>',
      },
      RelayedFrom: {
        who: 'XcmV0MultiLocation',
        message: 'XcmV0Xcm',
      },
    },
  },
  /**
   * Lookup315: xcm::v0::order::Order<Call>
   */
  XcmV0Order: {
    _enum: {
      Null: 'Null',
      DepositAsset: {
        assets: 'Vec<XcmV0MultiAsset>',
        dest: 'XcmV0MultiLocation',
      },
      DepositReserveAsset: {
        assets: 'Vec<XcmV0MultiAsset>',
        dest: 'XcmV0MultiLocation',
        effects: 'Vec<XcmV0Order>',
      },
      ExchangeAsset: {
        give: 'Vec<XcmV0MultiAsset>',
        receive: 'Vec<XcmV0MultiAsset>',
      },
      InitiateReserveWithdraw: {
        assets: 'Vec<XcmV0MultiAsset>',
        reserve: 'XcmV0MultiLocation',
        effects: 'Vec<XcmV0Order>',
      },
      InitiateTeleport: {
        assets: 'Vec<XcmV0MultiAsset>',
        dest: 'XcmV0MultiLocation',
        effects: 'Vec<XcmV0Order>',
      },
      QueryHolding: {
        queryId: 'Compact<u64>',
        dest: 'XcmV0MultiLocation',
        assets: 'Vec<XcmV0MultiAsset>',
      },
      BuyExecution: {
        fees: 'XcmV0MultiAsset',
        weight: 'u64',
        debt: 'u64',
        haltOnError: 'bool',
        xcm: 'Vec<XcmV0Xcm>',
      },
    },
  },
  /**
   * Lookup317: xcm::v0::Response
   */
  XcmV0Response: {
    _enum: {
      Assets: 'Vec<XcmV0MultiAsset>',
    },
  },
  /**
   * Lookup318: xcm::v1::Xcm<Call>
   */
  XcmV1Xcm: {
    _enum: {
      WithdrawAsset: {
        assets: 'XcmV1MultiassetMultiAssets',
        effects: 'Vec<XcmV1Order>',
      },
      ReserveAssetDeposited: {
        assets: 'XcmV1MultiassetMultiAssets',
        effects: 'Vec<XcmV1Order>',
      },
      ReceiveTeleportedAsset: {
        assets: 'XcmV1MultiassetMultiAssets',
        effects: 'Vec<XcmV1Order>',
      },
      QueryResponse: {
        queryId: 'Compact<u64>',
        response: 'XcmV1Response',
      },
      TransferAsset: {
        assets: 'XcmV1MultiassetMultiAssets',
        beneficiary: 'XcmV1MultiLocation',
      },
      TransferReserveAsset: {
        assets: 'XcmV1MultiassetMultiAssets',
        dest: 'XcmV1MultiLocation',
        effects: 'Vec<XcmV1Order>',
      },
      Transact: {
        originType: 'XcmV0OriginKind',
        requireWeightAtMost: 'u64',
        call: 'XcmDoubleEncoded',
      },
      HrmpNewChannelOpenRequest: {
        sender: 'Compact<u32>',
        maxMessageSize: 'Compact<u32>',
        maxCapacity: 'Compact<u32>',
      },
      HrmpChannelAccepted: {
        recipient: 'Compact<u32>',
      },
      HrmpChannelClosing: {
        initiator: 'Compact<u32>',
        sender: 'Compact<u32>',
        recipient: 'Compact<u32>',
      },
      RelayedFrom: {
        who: 'XcmV1MultilocationJunctions',
        message: 'XcmV1Xcm',
      },
      SubscribeVersion: {
        queryId: 'Compact<u64>',
        maxResponseWeight: 'Compact<u64>',
      },
      UnsubscribeVersion: 'Null',
    },
  },
  /**
   * Lookup320: xcm::v1::order::Order<Call>
   */
  XcmV1Order: {
    _enum: {
      Noop: 'Null',
      DepositAsset: {
        assets: 'XcmV1MultiassetMultiAssetFilter',
        maxAssets: 'u32',
        beneficiary: 'XcmV1MultiLocation',
      },
      DepositReserveAsset: {
        assets: 'XcmV1MultiassetMultiAssetFilter',
        maxAssets: 'u32',
        dest: 'XcmV1MultiLocation',
        effects: 'Vec<XcmV1Order>',
      },
      ExchangeAsset: {
        give: 'XcmV1MultiassetMultiAssetFilter',
        receive: 'XcmV1MultiassetMultiAssets',
      },
      InitiateReserveWithdraw: {
        assets: 'XcmV1MultiassetMultiAssetFilter',
        reserve: 'XcmV1MultiLocation',
        effects: 'Vec<XcmV1Order>',
      },
      InitiateTeleport: {
        assets: 'XcmV1MultiassetMultiAssetFilter',
        dest: 'XcmV1MultiLocation',
        effects: 'Vec<XcmV1Order>',
      },
      QueryHolding: {
        queryId: 'Compact<u64>',
        dest: 'XcmV1MultiLocation',
        assets: 'XcmV1MultiassetMultiAssetFilter',
      },
      BuyExecution: {
        fees: 'XcmV1MultiAsset',
        weight: 'u64',
        debt: 'u64',
        haltOnError: 'bool',
        instructions: 'Vec<XcmV1Xcm>',
      },
    },
  },
  /**
   * Lookup322: xcm::v1::Response
   */
  XcmV1Response: {
    _enum: {
      Assets: 'XcmV1MultiassetMultiAssets',
      Version: 'u32',
    },
  },
  /**
   * Lookup336: pallet_assets::pallet::Call<T, I>
   */
  PalletAssetsCall: {
    _enum: {
      create: {
        id: 'Compact<u128>',
        admin: 'AccountId20',
        minBalance: 'u128',
      },
      force_create: {
        id: 'Compact<u128>',
        owner: 'AccountId20',
        isSufficient: 'bool',
        minBalance: 'Compact<u128>',
      },
      destroy: {
        id: 'Compact<u128>',
        witness: 'PalletAssetsDestroyWitness',
      },
      mint: {
        id: 'Compact<u128>',
        beneficiary: 'AccountId20',
        amount: 'Compact<u128>',
      },
      burn: {
        id: 'Compact<u128>',
        who: 'AccountId20',
        amount: 'Compact<u128>',
      },
      transfer: {
        id: 'Compact<u128>',
        target: 'AccountId20',
        amount: 'Compact<u128>',
      },
      transfer_keep_alive: {
        id: 'Compact<u128>',
        target: 'AccountId20',
        amount: 'Compact<u128>',
      },
      force_transfer: {
        id: 'Compact<u128>',
        source: 'AccountId20',
        dest: 'AccountId20',
        amount: 'Compact<u128>',
      },
      freeze: {
        id: 'Compact<u128>',
        who: 'AccountId20',
      },
      thaw: {
        id: 'Compact<u128>',
        who: 'AccountId20',
      },
      freeze_asset: {
        id: 'Compact<u128>',
      },
      thaw_asset: {
        id: 'Compact<u128>',
      },
      transfer_ownership: {
        id: 'Compact<u128>',
        owner: 'AccountId20',
      },
      set_team: {
        id: 'Compact<u128>',
        issuer: 'AccountId20',
        admin: 'AccountId20',
        freezer: 'AccountId20',
      },
      set_metadata: {
        id: 'Compact<u128>',
        name: 'Bytes',
        symbol: 'Bytes',
        decimals: 'u8',
      },
      clear_metadata: {
        id: 'Compact<u128>',
      },
      force_set_metadata: {
        id: 'Compact<u128>',
        name: 'Bytes',
        symbol: 'Bytes',
        decimals: 'u8',
        isFrozen: 'bool',
      },
      force_clear_metadata: {
        id: 'Compact<u128>',
      },
      force_asset_status: {
        id: 'Compact<u128>',
        owner: 'AccountId20',
        issuer: 'AccountId20',
        admin: 'AccountId20',
        freezer: 'AccountId20',
        minBalance: 'Compact<u128>',
        isSufficient: 'bool',
        isFrozen: 'bool',
      },
      approve_transfer: {
        id: 'Compact<u128>',
        delegate: 'AccountId20',
        amount: 'Compact<u128>',
      },
      cancel_approval: {
        id: 'Compact<u128>',
        delegate: 'AccountId20',
      },
      force_cancel_approval: {
        id: 'Compact<u128>',
        owner: 'AccountId20',
        delegate: 'AccountId20',
      },
      transfer_approved: {
        id: 'Compact<u128>',
        owner: 'AccountId20',
        destination: 'AccountId20',
        amount: 'Compact<u128>',
      },
    },
  },
  /**
   * Lookup337: pallet_assets::types::DestroyWitness
   */
  PalletAssetsDestroyWitness: {
    accounts: 'Compact<u32>',
    sufficients: 'Compact<u32>',
    approvals: 'Compact<u32>',
  },
  /**
   * Lookup338: pallet_asset_manager::pallet::Call<T>
   */
  PalletAssetManagerCall: {
    _enum: {
      register_asset: {
        asset: 'MoonbeamRuntimeAssetType',
        metadata: 'MoonbeamRuntimeAssetRegistrarMetadata',
        minAmount: 'u128',
        isSufficient: 'bool',
      },
      set_asset_units_per_second: {
        assetType: 'MoonbeamRuntimeAssetType',
        unitsPerSecond: 'u128',
      },
      change_existing_asset_type: {
        assetId: 'u128',
        newAssetType: 'MoonbeamRuntimeAssetType',
      },
    },
  },
  /**
   * Lookup339: orml_xtokens::module::Call<T>
   */
  OrmlXtokensModuleCall: {
    _enum: {
      transfer: {
        currencyId: 'MoonbeamRuntimeCurrencyId',
        amount: 'u128',
        dest: 'XcmVersionedMultiLocation',
        destWeight: 'u64',
      },
      transfer_multiasset: {
        asset: 'XcmVersionedMultiAsset',
        dest: 'XcmVersionedMultiLocation',
        destWeight: 'u64',
      },
      transfer_with_fee: {
        currencyId: 'MoonbeamRuntimeCurrencyId',
        amount: 'u128',
        fee: 'u128',
        dest: 'XcmVersionedMultiLocation',
        destWeight: 'u64',
      },
      transfer_multiasset_with_fee: {
        asset: 'XcmVersionedMultiAsset',
        fee: 'XcmVersionedMultiAsset',
        dest: 'XcmVersionedMultiLocation',
        destWeight: 'u64',
      },
    },
  },
  /**
   * Lookup340: xcm::VersionedMultiAsset
   */
  XcmVersionedMultiAsset: {
    _enum: {
      V0: 'XcmV0MultiAsset',
      V1: 'XcmV1MultiAsset',
    },
  },
  /**
   * Lookup341: xcm_transactor::pallet::Call<T>
   */
  XcmTransactorCall: {
    _enum: {
      register: {
        who: 'AccountId20',
        index: 'u16',
      },
      transact_through_derivative_multilocation: {
        dest: 'MoonbeamRuntimeTransactors',
        index: 'u16',
        feeLocation: 'XcmVersionedMultiLocation',
        destWeight: 'u64',
        innerCall: 'Bytes',
      },
      transact_through_derivative: {
        dest: 'MoonbeamRuntimeTransactors',
        index: 'u16',
        currencyId: 'MoonbeamRuntimeCurrencyId',
        destWeight: 'u64',
        innerCall: 'Bytes',
      },
      transact_through_sovereign: {
        dest: 'XcmVersionedMultiLocation',
        feePayer: 'AccountId20',
        feeLocation: 'XcmVersionedMultiLocation',
        destWeight: 'u64',
        call: 'Bytes',
      },
      set_transact_info: {
        location: 'XcmVersionedMultiLocation',
        transactExtraWeight: 'u64',
        feePerSecond: 'u128',
        maxWeight: 'u64',
      },
    },
  },
  /**
   * Lookup342: moonbeam_runtime::Transactors
   */
  MoonbeamRuntimeTransactors: {
    _enum: ['Relay'],
  },
  /**
   * Lookup343: moonbeam_runtime::OriginCaller
   */
  MoonbeamRuntimeOriginCaller: {
    _enum: {
      system: 'FrameSystemRawOrigin',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      __Unused4: 'Null',
      __Unused5: 'Null',
      Void: 'SpCoreVoid',
      __Unused7: 'Null',
      __Unused8: 'Null',
      __Unused9: 'Null',
      __Unused10: 'Null',
      __Unused11: 'Null',
      __Unused12: 'Null',
      __Unused13: 'Null',
      __Unused14: 'Null',
      __Unused15: 'Null',
      __Unused16: 'Null',
      __Unused17: 'Null',
      __Unused18: 'Null',
      __Unused19: 'Null',
      __Unused20: 'Null',
      __Unused21: 'Null',
      __Unused22: 'Null',
      __Unused23: 'Null',
      __Unused24: 'Null',
      __Unused25: 'Null',
      __Unused26: 'Null',
      __Unused27: 'Null',
      __Unused28: 'Null',
      __Unused29: 'Null',
      __Unused30: 'Null',
      __Unused31: 'Null',
      __Unused32: 'Null',
      __Unused33: 'Null',
      __Unused34: 'Null',
      __Unused35: 'Null',
      __Unused36: 'Null',
      __Unused37: 'Null',
      __Unused38: 'Null',
      __Unused39: 'Null',
      __Unused40: 'Null',
      __Unused41: 'Null',
      __Unused42: 'Null',
      __Unused43: 'Null',
      __Unused44: 'Null',
      __Unused45: 'Null',
      __Unused46: 'Null',
      __Unused47: 'Null',
      __Unused48: 'Null',
      __Unused49: 'Null',
      __Unused50: 'Null',
      __Unused51: 'Null',
      Ethereum: 'PalletEthereumRawOrigin',
      __Unused53: 'Null',
      __Unused54: 'Null',
      __Unused55: 'Null',
      __Unused56: 'Null',
      __Unused57: 'Null',
      __Unused58: 'Null',
      __Unused59: 'Null',
      __Unused60: 'Null',
      __Unused61: 'Null',
      __Unused62: 'Null',
      __Unused63: 'Null',
      __Unused64: 'Null',
      __Unused65: 'Null',
      __Unused66: 'Null',
      __Unused67: 'Null',
      __Unused68: 'Null',
      __Unused69: 'Null',
      CouncilCollective: 'PalletCollectiveRawOrigin',
      TechCommitteeCollective: 'PalletCollectiveRawOrigin',
      __Unused72: 'Null',
      __Unused73: 'Null',
      __Unused74: 'Null',
      __Unused75: 'Null',
      __Unused76: 'Null',
      __Unused77: 'Null',
      __Unused78: 'Null',
      __Unused79: 'Null',
      __Unused80: 'Null',
      __Unused81: 'Null',
      __Unused82: 'Null',
      __Unused83: 'Null',
      __Unused84: 'Null',
      __Unused85: 'Null',
      __Unused86: 'Null',
      __Unused87: 'Null',
      __Unused88: 'Null',
      __Unused89: 'Null',
      __Unused90: 'Null',
      __Unused91: 'Null',
      __Unused92: 'Null',
      __Unused93: 'Null',
      __Unused94: 'Null',
      __Unused95: 'Null',
      __Unused96: 'Null',
      __Unused97: 'Null',
      __Unused98: 'Null',
      __Unused99: 'Null',
      __Unused100: 'Null',
      CumulusXcm: 'CumulusPalletXcmOrigin',
      __Unused102: 'Null',
      PolkadotXcm: 'PalletXcmOrigin',
    },
  },
  /**
   * Lookup344: frame_system::RawOrigin[account::AccountId20](account::AccountId20)
   */
  FrameSystemRawOrigin: {
    _enum: {
      Root: 'Null',
      Signed: 'AccountId20',
      None: 'Null',
    },
  },
  /**
   * Lookup345: pallet_ethereum::RawOrigin
   */
  PalletEthereumRawOrigin: {
    _enum: {
      EthereumTransaction: 'H160',
    },
  },
  /**
   * Lookup346: pallet_collective::RawOrigin<account::AccountId20, I>
   */
  PalletCollectiveRawOrigin: {
    _enum: {
      Members: '(u32,u32)',
      Member: 'AccountId20',
      _Phantom: 'Null',
    },
  },
  /**
   * Lookup348: cumulus_pallet_xcm::pallet::Origin
   */
  CumulusPalletXcmOrigin: {
    _enum: {
      Relay: 'Null',
      SiblingParachain: 'u32',
    },
  },
  /**
   * Lookup349: pallet_xcm::pallet::Origin
   */
  PalletXcmOrigin: {
    _enum: {
      Xcm: 'XcmV1MultiLocation',
      Response: 'XcmV1MultiLocation',
    },
  },
  /**
   * Lookup350: sp_core::Void
   */
  SpCoreVoid: 'Null',
  /**
   * Lookup351: pallet_utility::pallet::Error<T>
   */
  PalletUtilityError: {
    _enum: ['TooManyCalls'],
  },
  /**
   * Lookup354: pallet_proxy::ProxyDefinition<account::AccountId20,
   * moonbeam_runtime::ProxyType, BlockNumber>
   */
  PalletProxyProxyDefinition: {
    delegate: 'AccountId20',
    proxyType: 'MoonbeamRuntimeProxyType',
    delay: 'u32',
  },
  /**
   * Lookup358: pallet_proxy::Announcement<account::AccountId20,
   * primitive_types::H256, BlockNumber>
   */
  PalletProxyAnnouncement: {
    real: 'AccountId20',
    callHash: 'H256',
    height: 'u32',
  },
  /**
   * Lookup360: pallet_proxy::pallet::Error<T>
   */
  PalletProxyError: {
    _enum: [
      'TooMany',
      'NotFound',
      'NotProxy',
      'Unproxyable',
      'Duplicate',
      'NoPermission',
      'Unannounced',
      'NoSelfProxy',
    ],
  },
  /**
   * Lookup361: pallet_maintenance_mode::pallet::Error<T>
   */
  PalletMaintenanceModeError: {
    _enum: ['AlreadyInMaintenanceMode', 'NotInMaintenanceMode'],
  },
  /**
   * Lookup362: pallet_identity::types::Registration<Balance, MaxJudgements,
   * MaxAdditionalFields>
   */
  PalletIdentityRegistration: {
    judgements: 'Vec<(u32,PalletIdentityJudgement)>',
    deposit: 'u128',
    info: 'PalletIdentityIdentityInfo',
  },
  /**
   * Lookup370: pallet_identity::types::RegistrarInfo<Balance, account::AccountId20>
   */
  PalletIdentityRegistrarInfo: {
    account: 'AccountId20',
    fee: 'u128',
    fields: 'PalletIdentityBitFlags',
  },
  /**
   * Lookup372: pallet_identity::pallet::Error<T>
   */
  PalletIdentityError: {
    _enum: [
      'TooManySubAccounts',
      'NotFound',
      'NotNamed',
      'EmptyIndex',
      'FeeChanged',
      'NoIdentity',
      'StickyJudgement',
      'JudgementGiven',
      'InvalidJudgement',
      'InvalidIndex',
      'InvalidTarget',
      'TooManyFields',
      'TooManyRegistrars',
      'AlreadyClaimed',
      'NotSub',
      'NotOwned',
    ],
  },
  /**
   * Lookup374: pallet_evm::pallet::Error<T>
   */
  PalletEvmError: {
    _enum: [
      'BalanceLow',
      'FeeOverflow',
      'PaymentOverflow',
      'WithdrawFailed',
      'GasPriceTooLow',
      'InvalidNonce',
    ],
  },
  /**
   * Lookup377: fp_rpc::TransactionStatus
   */
  FpRpcTransactionStatus: {
    transactionHash: 'H256',
    transactionIndex: 'u32',
    from: 'H160',
    to: 'Option<H160>',
    contractAddress: 'Option<H160>',
    logs: 'Vec<EthereumLog>',
    logsBloom: 'EthbloomBloom',
  },
  /**
   * Lookup380: ethbloom::Bloom
   */
  EthbloomBloom: '[u8;256]',
  /**
   * Lookup382: ethereum::receipt::ReceiptV3
   */
  EthereumReceiptReceiptV3: {
    _enum: {
      Legacy: 'EthereumReceiptEip658ReceiptData',
      EIP2930: 'EthereumReceiptEip658ReceiptData',
      EIP1559: 'EthereumReceiptEip658ReceiptData',
    },
  },
  /**
   * Lookup383: ethereum::receipt::EIP658ReceiptData
   */
  EthereumReceiptEip658ReceiptData: {
    statusCode: 'u8',
    usedGas: 'U256',
    logsBloom: 'EthbloomBloom',
    logs: 'Vec<EthereumLog>',
  },
  /**
   * Lookup384:
   * ethereum::block::Block[ethereum::transaction::TransactionV2](ethereum::transaction::TransactionV2)
   */
  EthereumBlock: {
    header: 'EthereumHeader',
    transactions: 'Vec<EthereumTransactionTransactionV2>',
    ommers: 'Vec<EthereumHeader>',
  },
  /**
   * Lookup385: ethereum::header::Header
   */
  EthereumHeader: {
    parentHash: 'H256',
    ommersHash: 'H256',
    beneficiary: 'H160',
    stateRoot: 'H256',
    transactionsRoot: 'H256',
    receiptsRoot: 'H256',
    logsBloom: 'EthbloomBloom',
    difficulty: 'U256',
    number: 'U256',
    gasLimit: 'U256',
    gasUsed: 'U256',
    timestamp: 'u64',
    extraData: 'Bytes',
    mixHash: 'H256',
    nonce: 'EthereumTypesHashH64',
  },
  /**
   * Lookup386: ethereum_types::hash::H64
   */
  EthereumTypesHashH64: '[u8;8]',
  /**
   * Lookup391: pallet_ethereum::pallet::Error<T>
   */
  PalletEthereumError: {
    _enum: ['InvalidSignature', 'PreLogExists'],
  },
  /**
   * Lookup394: pallet_scheduler::ScheduledV2<moonbeam_runtime::Call,
   * BlockNumber, moonbeam_runtime::OriginCaller, account::AccountId20>
   */
  PalletSchedulerScheduledV2: {
    maybeId: 'Option<Bytes>',
    priority: 'u8',
    call: 'Call',
    maybePeriodic: 'Option<(u32,u32)>',
    origin: 'MoonbeamRuntimeOriginCaller',
  },
  /**
   * Lookup395: pallet_scheduler::Releases
   */
  PalletSchedulerReleases: {
    _enum: ['V1', 'V2'],
  },
  /**
   * Lookup396: pallet_scheduler::pallet::Error<T>
   */
  PalletSchedulerError: {
    _enum: ['FailedToSchedule', 'NotFound', 'TargetBlockNumberInPast', 'RescheduleNoChange'],
  },
  /**
   * Lookup400: pallet_democracy::PreimageStatus<account::AccountId20, Balance,
   * BlockNumber>
   */
  PalletDemocracyPreimageStatus: {
    _enum: {
      Missing: 'u32',
      Available: {
        data: 'Bytes',
        provider: 'AccountId20',
        deposit: 'u128',
        since: 'u32',
        expiry: 'Option<u32>',
      },
    },
  },
  /**
   * Lookup401: pallet_democracy::types::ReferendumInfo<BlockNumber,
   * primitive_types::H256, Balance>
   */
  PalletDemocracyReferendumInfo: {
    _enum: {
      Ongoing: 'PalletDemocracyReferendumStatus',
      Finished: {
        approved: 'bool',
        end: 'u32',
      },
    },
  },
  /**
   * Lookup402: pallet_democracy::types::ReferendumStatus<BlockNumber,
   * primitive_types::H256, Balance>
   */
  PalletDemocracyReferendumStatus: {
    end: 'u32',
    proposalHash: 'H256',
    threshold: 'PalletDemocracyVoteThreshold',
    delay: 'u32',
    tally: 'PalletDemocracyTally',
  },
  /**
   * Lookup403: pallet_democracy::types::Tally<Balance>
   */
  PalletDemocracyTally: {
    ayes: 'u128',
    nays: 'u128',
    turnout: 'u128',
  },
  /**
   * Lookup404: pallet_democracy::vote::Voting<Balance, account::AccountId20, BlockNumber>
   */
  PalletDemocracyVoteVoting: {
    _enum: {
      Direct: {
        votes: 'Vec<(u32,PalletDemocracyVoteAccountVote)>',
        delegations: 'PalletDemocracyDelegations',
        prior: 'PalletDemocracyVotePriorLock',
      },
      Delegating: {
        balance: 'u128',
        target: 'AccountId20',
        conviction: 'PalletDemocracyConviction',
        delegations: 'PalletDemocracyDelegations',
        prior: 'PalletDemocracyVotePriorLock',
      },
    },
  },
  /**
   * Lookup407: pallet_democracy::types::Delegations<Balance>
   */
  PalletDemocracyDelegations: {
    votes: 'u128',
    capital: 'u128',
  },
  /**
   * Lookup408: pallet_democracy::vote::PriorLock<BlockNumber, Balance>
   */
  PalletDemocracyVotePriorLock: '(u32,u128)',
  /**
   * Lookup411: pallet_democracy::Releases
   */
  PalletDemocracyReleases: {
    _enum: ['V1'],
  },
  /**
   * Lookup412: pallet_democracy::pallet::Error<T>
   */
  PalletDemocracyError: {
    _enum: [
      'ValueLow',
      'ProposalMissing',
      'AlreadyCanceled',
      'DuplicateProposal',
      'ProposalBlacklisted',
      'NotSimpleMajority',
      'InvalidHash',
      'NoProposal',
      'AlreadyVetoed',
      'DuplicatePreimage',
      'NotImminent',
      'TooEarly',
      'Imminent',
      'PreimageMissing',
      'ReferendumInvalid',
      'PreimageInvalid',
      'NoneWaiting',
      'NotVoter',
      'NoPermission',
      'AlreadyDelegating',
      'InsufficientFunds',
      'NotDelegating',
      'VotesExist',
      'InstantNotAllowed',
      'Nonsense',
      'WrongUpperBound',
      'MaxVotesReached',
      'TooManyProposals',
    ],
  },
  /**
   * Lookup414: pallet_collective::Votes<account::AccountId20, BlockNumber>
   */
  PalletCollectiveVotes: {
    index: 'u32',
    threshold: 'u32',
    ayes: 'Vec<AccountId20>',
    nays: 'Vec<AccountId20>',
    end: 'u32',
  },
  /**
   * Lookup415: pallet_collective::pallet::Error<T, I>
   */
  PalletCollectiveError: {
    _enum: [
      'NotMember',
      'DuplicateProposal',
      'ProposalMissing',
      'WrongIndex',
      'DuplicateVote',
      'AlreadyInitialized',
      'TooEarly',
      'TooManyProposals',
      'WrongProposalWeight',
      'WrongProposalLength',
    ],
  },
  /**
   * Lookup418: pallet_treasury::Proposal<account::AccountId20, Balance>
   */
  PalletTreasuryProposal: {
    proposer: 'AccountId20',
    value: 'u128',
    beneficiary: 'AccountId20',
    bond: 'u128',
  },
  /**
   * Lookup421: frame_support::PalletId
   */
  FrameSupportPalletId: '[u8;8]',
  /**
   * Lookup422: pallet_treasury::pallet::Error<T, I>
   */
  PalletTreasuryError: {
    _enum: ['InsufficientProposersBalance', 'InvalidIndex', 'TooManyApprovals'],
  },
  /**
   * Lookup423: pallet_crowdloan_rewards::pallet::RewardInfo<T>
   */
  PalletCrowdloanRewardsRewardInfo: {
    totalReward: 'u128',
    claimedReward: 'u128',
    contributedRelayAddresses: 'Vec<[u8;32]>',
  },
  /**
   * Lookup425: pallet_crowdloan_rewards::pallet::Error<T>
   */
  PalletCrowdloanRewardsError: {
    _enum: [
      'AlreadyAssociated',
      'BatchBeyondFundPot',
      'FirstClaimAlreadyDone',
      'RewardNotHighEnough',
      'InvalidClaimSignature',
      'InvalidFreeClaimSignature',
      'NoAssociatedClaim',
      'RewardsAlreadyClaimed',
      'RewardVecAlreadyInitialized',
      'RewardVecNotFullyInitializedYet',
      'RewardsDoNotMatchFund',
      'TooManyContributors',
      'VestingPeriodNonValid',
      'NonContributedAddressProvided',
      'InsufficientNumberOfValidProofs',
    ],
  },
  /**
   * Lookup428: cumulus_pallet_xcmp_queue::InboundStatus
   */
  CumulusPalletXcmpQueueInboundStatus: {
    _enum: ['Ok', 'Suspended'],
  },
  /**
   * Lookup431: polkadot_parachain::primitives::XcmpMessageFormat
   */
  PolkadotParachainPrimitivesXcmpMessageFormat: {
    _enum: ['ConcatenatedVersionedXcm', 'ConcatenatedEncodedBlob', 'Signals'],
  },
  /**
   * Lookup435: cumulus_pallet_xcmp_queue::OutboundStatus
   */
  CumulusPalletXcmpQueueOutboundStatus: {
    _enum: ['Ok', 'Suspended'],
  },
  /**
   * Lookup437: cumulus_pallet_xcmp_queue::QueueConfigData
   */
  CumulusPalletXcmpQueueQueueConfigData: {
    suspendThreshold: 'u32',
    dropThreshold: 'u32',
    resumeThreshold: 'u32',
    thresholdWeight: 'u64',
    weightRestrictDecay: 'u64',
  },
  /**
   * Lookup438: cumulus_pallet_xcmp_queue::pallet::Error<T>
   */
  CumulusPalletXcmpQueueError: {
    _enum: ['FailedToSend', 'BadXcmOrigin', 'BadXcm'],
  },
  /**
   * Lookup439: cumulus_pallet_xcm::pallet::Error<T>
   */
  CumulusPalletXcmError: 'Null',
  /**
   * Lookup440: cumulus_pallet_dmp_queue::ConfigData
   */
  CumulusPalletDmpQueueConfigData: {
    maxIndividual: 'u64',
  },
  /**
   * Lookup441: cumulus_pallet_dmp_queue::PageIndexData
   */
  CumulusPalletDmpQueuePageIndexData: {
    beginUsed: 'u32',
    endUsed: 'u32',
    overweightCount: 'u64',
  },
  /**
   * Lookup444: cumulus_pallet_dmp_queue::pallet::Error<T>
   */
  CumulusPalletDmpQueueError: {
    _enum: ['Unknown', 'OverLimit'],
  },
  /**
   * Lookup445: pallet_xcm::pallet::QueryStatus<BlockNumber>
   */
  PalletXcmQueryStatus: {
    _enum: {
      Pending: {
        responder: 'XcmVersionedMultiLocation',
        maybeNotify: 'Option<(u8,u8)>',
        timeout: 'u32',
      },
      VersionNotifier: {
        origin: 'XcmVersionedMultiLocation',
        isActive: 'bool',
      },
      Ready: {
        response: 'XcmVersionedResponse',
        at: 'u32',
      },
    },
  },
  /**
   * Lookup448: xcm::VersionedResponse
   */
  XcmVersionedResponse: {
    _enum: {
      V0: 'XcmV0Response',
      V1: 'XcmV1Response',
      V2: 'XcmV2Response',
    },
  },
  /**
   * Lookup454: pallet_xcm::pallet::VersionMigrationStage
   */
  PalletXcmVersionMigrationStage: {
    _enum: {
      MigrateSupportedVersion: 'Null',
      MigrateVersionNotifiers: 'Null',
      NotifyCurrentTargets: 'Option<Bytes>',
      MigrateAndNotifyOldTargets: 'Null',
    },
  },
  /**
   * Lookup455: pallet_xcm::pallet::Error<T>
   */
  PalletXcmError: {
    _enum: [
      'Unreachable',
      'SendFailure',
      'Filtered',
      'UnweighableMessage',
      'DestinationNotInvertible',
      'Empty',
      'CannotReanchor',
      'TooManyAssets',
      'InvalidOrigin',
      'BadVersion',
      'BadLocation',
      'NoSubscription',
      'AlreadySubscribed',
    ],
  },
  /**
   * Lookup456: pallet_assets::types::AssetDetails<Balance,
   * account::AccountId20, DepositBalance>
   */
  PalletAssetsAssetDetails: {
    owner: 'AccountId20',
    issuer: 'AccountId20',
    admin: 'AccountId20',
    freezer: 'AccountId20',
    supply: 'u128',
    deposit: 'u128',
    minBalance: 'u128',
    isSufficient: 'bool',
    accounts: 'u32',
    sufficients: 'u32',
    approvals: 'u32',
    isFrozen: 'bool',
  },
  /**
   * Lookup458: pallet_assets::types::AssetBalance<Balance, Extra>
   */
  PalletAssetsAssetBalance: {
    balance: 'u128',
    isFrozen: 'bool',
    sufficient: 'bool',
    extra: 'Null',
  },
  /**
   * Lookup460: pallet_assets::types::Approval<Balance, DepositBalance>
   */
  PalletAssetsApproval: {
    amount: 'u128',
    deposit: 'u128',
  },
  /**
   * Lookup461: pallet_assets::types::AssetMetadata<DepositBalance,
   * frame_support::storage::bounded_vec::BoundedVec<T, S>>
   */
  PalletAssetsAssetMetadata: {
    deposit: 'u128',
    name: 'Bytes',
    symbol: 'Bytes',
    decimals: 'u8',
    isFrozen: 'bool',
  },
  /**
   * Lookup463: pallet_assets::pallet::Error<T, I>
   */
  PalletAssetsError: {
    _enum: [
      'BalanceLow',
      'BalanceZero',
      'NoPermission',
      'Unknown',
      'Frozen',
      'InUse',
      'BadWitness',
      'MinBalanceZero',
      'NoProvider',
      'BadMetadata',
      'Unapproved',
      'WouldDie',
    ],
  },
  /**
   * Lookup464: pallet_asset_manager::pallet::Error<T>
   */
  PalletAssetManagerError: {
    _enum: ['ErrorCreatingAsset', 'AssetAlreadyExists', 'AssetDoesNotExist'],
  },
  /**
   * Lookup465: orml_xtokens::module::Error<T>
   */
  OrmlXtokensModuleError: {
    _enum: [
      'AssetHasNoReserve',
      'NotCrossChainTransfer',
      'InvalidDest',
      'NotCrossChainTransferableCurrency',
      'UnweighableMessage',
      'XcmExecutionFailed',
      'CannotReanchor',
      'InvalidAncestry',
      'NotFungible',
      'DestinationNotInvertible',
      'BadVersion',
      'DistincAssetAndFeeId',
      'FeeCannotBeZero',
    ],
  },
  /**
   * Lookup466: xcm_transactor::pallet::Error<T>
   */
  XcmTransactorError: {
    _enum: [
      'IndexAlreadyClaimed',
      'UnclaimedIndex',
      'NotOwner',
      'UnweighableMessage',
      'CannotReanchor',
      'AssetHasNoReserve',
      'InvalidDest',
      'NotCrossChainTransfer',
      'AssetIsNotReserveInDestination',
      'DestinationNotInvertible',
      'ErrorSending',
      'DispatchWeightBiggerThanTotalWeight',
      'WeightOverflow',
      'AmountOverflow',
      'TransactorInfoNotSet',
      'NotCrossChainTransferableCurrency',
      'XcmExecuteError',
      'BadVersion',
      'MaxWeightTransactReached',
      'UnableToWithdrawAsset',
    ],
  },
  /**
   * Lookup468: account::EthereumSignature
   */
  AccountEthereumSignature: 'SpCoreEcdsaSignature',
  /**
   * Lookup470: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
   */
  FrameSystemExtensionsCheckSpecVersion: 'Null',
  /**
   * Lookup471: frame_system::extensions::check_tx_version::CheckTxVersion<T>
   */
  FrameSystemExtensionsCheckTxVersion: 'Null',
  /**
   * Lookup472: frame_system::extensions::check_genesis::CheckGenesis<T>
   */
  FrameSystemExtensionsCheckGenesis: 'Null',
  /**
   * Lookup475: frame_system::extensions::check_nonce::CheckNonce<T>
   */
  FrameSystemExtensionsCheckNonce: 'Compact<u32>',
  /**
   * Lookup476: frame_system::extensions::check_weight::CheckWeight<T>
   */
  FrameSystemExtensionsCheckWeight: 'Null',
  /**
   * Lookup477: pallet_transaction_payment::ChargeTransactionPayment<T>
   */
  PalletTransactionPaymentChargeTransactionPayment: 'Compact<u128>',
  /**
   * Lookup479: moonbeam_runtime::Runtime
   */
  MoonbeamRuntimeRuntime: 'Null',
};
