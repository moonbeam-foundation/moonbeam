// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

/* eslint-disable sort-keys */

export default {
  /**
   * Lookup3: frame_system::AccountInfo<Index, pallet_balances::AccountData<Balance>>
   */
  FrameSystemAccountInfo: {
    nonce: "u32",
    consumers: "u32",
    providers: "u32",
    sufficients: "u32",
    data: "PalletBalancesAccountData",
  },
  /**
   * Lookup5: pallet_balances::AccountData<Balance>
   */
  PalletBalancesAccountData: {
    free: "u128",
    reserved: "u128",
    miscFrozen: "u128",
    feeFrozen: "u128",
  },
  /**
   * Lookup7: frame_support::weights::PerDispatchClass<T>
   */
  FrameSupportWeightsPerDispatchClassU64: {
    normal: "u64",
    operational: "u64",
    mandatory: "u64",
  },
  /**
   * Lookup12: sp_runtime::generic::digest::Digest
   */
  SpRuntimeDigest: {
    logs: "Vec<SpRuntimeDigestDigestItem>",
  },
  /**
   * Lookup14: sp_runtime::generic::digest::DigestItem
   */
  SpRuntimeDigestDigestItem: {
    _enum: {
      Other: "Bytes",
      __Unused1: "Null",
      __Unused2: "Null",
      __Unused3: "Null",
      Consensus: "([u8;4],Bytes)",
      Seal: "([u8;4],Bytes)",
      PreRuntime: "([u8;4],Bytes)",
      __Unused7: "Null",
      RuntimeEnvironmentUpdated: "Null",
    },
  },
  /**
   * Lookup17: frame_system::EventRecord<moonbeam_runtime::Event, primitive_types::H256>
   */
  FrameSystemEventRecord: {
    phase: "FrameSystemPhase",
    event: "Event",
    topics: "Vec<H256>",
  },
  /**
   * Lookup19: frame_system::pallet::Event<T>
   */
  FrameSystemEvent: {
    _enum: {
      ExtrinsicSuccess: {
        dispatchInfo: "FrameSupportWeightsDispatchInfo",
      },
      ExtrinsicFailed: {
        dispatchError: "SpRuntimeDispatchError",
        dispatchInfo: "FrameSupportWeightsDispatchInfo",
      },
      CodeUpdated: "Null",
      NewAccount: {
        account: "AccountId20",
      },
      KilledAccount: {
        account: "AccountId20",
      },
      Remarked: {
        _alias: {
          hash_: "hash",
        },
        sender: "AccountId20",
        hash_: "H256",
      },
    },
  },
  /**
   * Lookup20: frame_support::weights::DispatchInfo
   */
  FrameSupportWeightsDispatchInfo: {
    weight: "u64",
    class: "FrameSupportWeightsDispatchClass",
    paysFee: "FrameSupportWeightsPays",
  },
  /**
   * Lookup21: frame_support::weights::DispatchClass
   */
  FrameSupportWeightsDispatchClass: {
    _enum: ["Normal", "Operational", "Mandatory"],
  },
  /**
   * Lookup22: frame_support::weights::Pays
   */
  FrameSupportWeightsPays: {
    _enum: ["Yes", "No"],
  },
  /**
   * Lookup23: sp_runtime::DispatchError
   */
  SpRuntimeDispatchError: {
    _enum: {
      Other: "Null",
      CannotLookup: "Null",
      BadOrigin: "Null",
      Module: "SpRuntimeModuleError",
      ConsumerRemaining: "Null",
      NoProviders: "Null",
      TooManyConsumers: "Null",
      Token: "SpRuntimeTokenError",
      Arithmetic: "SpRuntimeArithmeticError",
      Transactional: "SpRuntimeTransactionalError",
    },
  },
  /**
   * Lookup24: sp_runtime::ModuleError
   */
  SpRuntimeModuleError: {
    index: "u8",
    error: "[u8;4]",
  },
  /**
   * Lookup25: sp_runtime::TokenError
   */
  SpRuntimeTokenError: {
    _enum: [
      "NoFunds",
      "WouldDie",
      "BelowMinimum",
      "CannotCreate",
      "UnknownAsset",
      "Frozen",
      "Unsupported",
    ],
  },
  /**
   * Lookup26: sp_runtime::ArithmeticError
   */
  SpRuntimeArithmeticError: {
    _enum: ["Underflow", "Overflow", "DivisionByZero"],
  },
  /**
   * Lookup27: sp_runtime::TransactionalError
   */
  SpRuntimeTransactionalError: {
    _enum: ["LimitReached", "NoLayer"],
  },
  /**
   * Lookup28: cumulus_pallet_parachain_system::pallet::Event<T>
   */
  CumulusPalletParachainSystemEvent: {
    _enum: {
      ValidationFunctionStored: "Null",
      ValidationFunctionApplied: "u32",
      ValidationFunctionDiscarded: "Null",
      UpgradeAuthorized: "H256",
      DownwardMessagesReceived: "u32",
      DownwardMessagesProcessed: "(u64,H256)",
    },
  },
  /**
   * Lookup29: pallet_balances::pallet::Event<T, I>
   */
  PalletBalancesEvent: {
    _enum: {
      Endowed: {
        account: "AccountId20",
        freeBalance: "u128",
      },
      DustLost: {
        account: "AccountId20",
        amount: "u128",
      },
      Transfer: {
        from: "AccountId20",
        to: "AccountId20",
        amount: "u128",
      },
      BalanceSet: {
        who: "AccountId20",
        free: "u128",
        reserved: "u128",
      },
      Reserved: {
        who: "AccountId20",
        amount: "u128",
      },
      Unreserved: {
        who: "AccountId20",
        amount: "u128",
      },
      ReserveRepatriated: {
        from: "AccountId20",
        to: "AccountId20",
        amount: "u128",
        destinationStatus: "FrameSupportTokensMiscBalanceStatus",
      },
      Deposit: {
        who: "AccountId20",
        amount: "u128",
      },
      Withdraw: {
        who: "AccountId20",
        amount: "u128",
      },
      Slashed: {
        who: "AccountId20",
        amount: "u128",
      },
    },
  },
  /**
   * Lookup30: frame_support::traits::tokens::misc::BalanceStatus
   */
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ["Free", "Reserved"],
  },
  /**
   * Lookup31: parachain_staking::pallet::Event<T>
   */
  ParachainStakingEvent: {
    _enum: {
      NewRound: {
        startingBlock: "u32",
        round: "u32",
        selectedCollatorsNumber: "u32",
        totalBalance: "u128",
      },
      JoinedCollatorCandidates: {
        account: "AccountId20",
        amountLocked: "u128",
        newTotalAmtLocked: "u128",
      },
      CollatorChosen: {
        round: "u32",
        collatorAccount: "AccountId20",
        totalExposedAmount: "u128",
      },
      CandidateBondLessRequested: {
        candidate: "AccountId20",
        amountToDecrease: "u128",
        executeRound: "u32",
      },
      CandidateBondedMore: {
        candidate: "AccountId20",
        amount: "u128",
        newTotalBond: "u128",
      },
      CandidateBondedLess: {
        candidate: "AccountId20",
        amount: "u128",
        newBond: "u128",
      },
      CandidateWentOffline: {
        candidate: "AccountId20",
      },
      CandidateBackOnline: {
        candidate: "AccountId20",
      },
      CandidateScheduledExit: {
        exitAllowedRound: "u32",
        candidate: "AccountId20",
        scheduledExit: "u32",
      },
      CancelledCandidateExit: {
        candidate: "AccountId20",
      },
      CancelledCandidateBondLess: {
        candidate: "AccountId20",
        amount: "u128",
        executeRound: "u32",
      },
      CandidateLeft: {
        exCandidate: "AccountId20",
        unlockedAmount: "u128",
        newTotalAmtLocked: "u128",
      },
      DelegationDecreaseScheduled: {
        delegator: "AccountId20",
        candidate: "AccountId20",
        amountToDecrease: "u128",
        executeRound: "u32",
      },
      DelegationIncreased: {
        delegator: "AccountId20",
        candidate: "AccountId20",
        amount: "u128",
        inTop: "bool",
      },
      DelegationDecreased: {
        delegator: "AccountId20",
        candidate: "AccountId20",
        amount: "u128",
        inTop: "bool",
      },
      DelegatorExitScheduled: {
        round: "u32",
        delegator: "AccountId20",
        scheduledExit: "u32",
      },
      DelegationRevocationScheduled: {
        round: "u32",
        delegator: "AccountId20",
        candidate: "AccountId20",
        scheduledExit: "u32",
      },
      DelegatorLeft: {
        delegator: "AccountId20",
        unstakedAmount: "u128",
      },
      DelegationRevoked: {
        delegator: "AccountId20",
        candidate: "AccountId20",
        unstakedAmount: "u128",
      },
      DelegationKicked: {
        delegator: "AccountId20",
        candidate: "AccountId20",
        unstakedAmount: "u128",
      },
      DelegatorExitCancelled: {
        delegator: "AccountId20",
      },
      CancelledDelegationRequest: {
        delegator: "AccountId20",
        cancelledRequest: "ParachainStakingDelegationRequestsCancelledScheduledRequest",
        collator: "AccountId20",
      },
      Delegation: {
        delegator: "AccountId20",
        lockedAmount: "u128",
        candidate: "AccountId20",
        delegatorPosition: "ParachainStakingDelegatorAdded",
      },
      DelegatorLeftCandidate: {
        delegator: "AccountId20",
        candidate: "AccountId20",
        unstakedAmount: "u128",
        totalCandidateStaked: "u128",
      },
      Rewarded: {
        account: "AccountId20",
        rewards: "u128",
      },
      ReservedForParachainBond: {
        account: "AccountId20",
        value: "u128",
      },
      ParachainBondAccountSet: {
        _alias: {
          new_: "new",
        },
        old: "AccountId20",
        new_: "AccountId20",
      },
      ParachainBondReservePercentSet: {
        _alias: {
          new_: "new",
        },
        old: "Percent",
        new_: "Percent",
      },
      InflationSet: {
        annualMin: "Perbill",
        annualIdeal: "Perbill",
        annualMax: "Perbill",
        roundMin: "Perbill",
        roundIdeal: "Perbill",
        roundMax: "Perbill",
      },
      StakeExpectationsSet: {
        expectMin: "u128",
        expectIdeal: "u128",
        expectMax: "u128",
      },
      TotalSelectedSet: {
        _alias: {
          new_: "new",
        },
        old: "u32",
        new_: "u32",
      },
      CollatorCommissionSet: {
        _alias: {
          new_: "new",
        },
        old: "Perbill",
        new_: "Perbill",
      },
      BlocksPerRoundSet: {
        _alias: {
          new_: "new",
        },
        currentRound: "u32",
        firstBlock: "u32",
        old: "u32",
        new_: "u32",
        newPerRoundInflationMin: "Perbill",
        newPerRoundInflationIdeal: "Perbill",
        newPerRoundInflationMax: "Perbill",
      },
    },
  },
  /**
   * Lookup33: parachain_staking::delegation_requests::CancelledScheduledRequest<Balance>
   */
  ParachainStakingDelegationRequestsCancelledScheduledRequest: {
    whenExecutable: "u32",
    action: "ParachainStakingDelegationRequestsDelegationAction",
  },
  /**
   * Lookup34: parachain_staking::delegation_requests::DelegationAction<Balance>
   */
  ParachainStakingDelegationRequestsDelegationAction: {
    _enum: {
      Revoke: "u128",
      Decrease: "u128",
    },
  },
  /**
   * Lookup35: parachain_staking::types::DelegatorAdded<B>
   */
  ParachainStakingDelegatorAdded: {
    _enum: {
      AddedToTop: {
        newTotal: "u128",
      },
      AddedToBottom: "Null",
    },
  },
  /**
   * Lookup38: pallet_author_slot_filter::pallet::Event
   */
  PalletAuthorSlotFilterEvent: {
    _enum: {
      EligibleUpdated: "u32",
    },
  },
  /**
   * Lookup40: pallet_author_mapping::pallet::Event<T>
   */
  PalletAuthorMappingEvent: {
    _enum: {
      KeysRegistered: {
        _alias: {
          keys_: "keys",
        },
        nimbusId: "NimbusPrimitivesNimbusCryptoPublic",
        accountId: "AccountId20",
        keys_: "SessionKeysPrimitivesVrfVrfCryptoPublic",
      },
      KeysRemoved: {
        _alias: {
          keys_: "keys",
        },
        nimbusId: "NimbusPrimitivesNimbusCryptoPublic",
        accountId: "AccountId20",
        keys_: "SessionKeysPrimitivesVrfVrfCryptoPublic",
      },
      KeysRotated: {
        newNimbusId: "NimbusPrimitivesNimbusCryptoPublic",
        accountId: "AccountId20",
        newKeys: "SessionKeysPrimitivesVrfVrfCryptoPublic",
      },
    },
  },
  /**
   * Lookup41: nimbus_primitives::nimbus_crypto::Public
   */
  NimbusPrimitivesNimbusCryptoPublic: "SpCoreSr25519Public",
  /**
   * Lookup42: sp_core::sr25519::Public
   */
  SpCoreSr25519Public: "[u8;32]",
  /**
   * Lookup43: session_keys_primitives::vrf::vrf_crypto::Public
   */
  SessionKeysPrimitivesVrfVrfCryptoPublic: "SpCoreSr25519Public",
  /**
   * Lookup44: pallet_moonbeam_orbiters::pallet::Event<T>
   */
  PalletMoonbeamOrbitersEvent: {
    _enum: {
      OrbiterJoinCollatorPool: {
        collator: "AccountId20",
        orbiter: "AccountId20",
      },
      OrbiterLeaveCollatorPool: {
        collator: "AccountId20",
        orbiter: "AccountId20",
      },
      OrbiterRewarded: {
        account: "AccountId20",
        rewards: "u128",
      },
      OrbiterRotation: {
        collator: "AccountId20",
        oldOrbiter: "Option<AccountId20>",
        newOrbiter: "Option<AccountId20>",
      },
      OrbiterRegistered: {
        account: "AccountId20",
        deposit: "u128",
      },
      OrbiterUnregistered: {
        account: "AccountId20",
      },
    },
  },
  /**
   * Lookup46: pallet_utility::pallet::Event
   */
  PalletUtilityEvent: {
    _enum: {
      BatchInterrupted: {
        index: "u32",
        error: "SpRuntimeDispatchError",
      },
      BatchCompleted: "Null",
      ItemCompleted: "Null",
      DispatchedAs: {
        result: "Result<Null, SpRuntimeDispatchError>",
      },
    },
  },
  /**
   * Lookup49: pallet_proxy::pallet::Event<T>
   */
  PalletProxyEvent: {
    _enum: {
      ProxyExecuted: {
        result: "Result<Null, SpRuntimeDispatchError>",
      },
      AnonymousCreated: {
        anonymous: "AccountId20",
        who: "AccountId20",
        proxyType: "MoonbeamRuntimeProxyType",
        disambiguationIndex: "u16",
      },
      Announced: {
        real: "AccountId20",
        proxy: "AccountId20",
        callHash: "H256",
      },
      ProxyAdded: {
        delegator: "AccountId20",
        delegatee: "AccountId20",
        proxyType: "MoonbeamRuntimeProxyType",
        delay: "u32",
      },
      ProxyRemoved: {
        delegator: "AccountId20",
        delegatee: "AccountId20",
        proxyType: "MoonbeamRuntimeProxyType",
        delay: "u32",
      },
    },
  },
  /**
   * Lookup50: moonbeam_runtime::ProxyType
   */
  MoonbeamRuntimeProxyType: {
    _enum: [
      "Any",
      "NonTransfer",
      "Governance",
      "Staking",
      "CancelProxy",
      "Balances",
      "AuthorMapping",
    ],
  },
  /**
   * Lookup52: pallet_maintenance_mode::pallet::Event
   */
  PalletMaintenanceModeEvent: {
    _enum: {
      EnteredMaintenanceMode: "Null",
      NormalOperationResumed: "Null",
      FailedToSuspendIdleXcmExecution: {
        error: "SpRuntimeDispatchError",
      },
      FailedToResumeIdleXcmExecution: {
        error: "SpRuntimeDispatchError",
      },
    },
  },
  /**
   * Lookup53: pallet_identity::pallet::Event<T>
   */
  PalletIdentityEvent: {
    _enum: {
      IdentitySet: {
        who: "AccountId20",
      },
      IdentityCleared: {
        who: "AccountId20",
        deposit: "u128",
      },
      IdentityKilled: {
        who: "AccountId20",
        deposit: "u128",
      },
      JudgementRequested: {
        who: "AccountId20",
        registrarIndex: "u32",
      },
      JudgementUnrequested: {
        who: "AccountId20",
        registrarIndex: "u32",
      },
      JudgementGiven: {
        target: "AccountId20",
        registrarIndex: "u32",
      },
      RegistrarAdded: {
        registrarIndex: "u32",
      },
      SubIdentityAdded: {
        sub: "AccountId20",
        main: "AccountId20",
        deposit: "u128",
      },
      SubIdentityRemoved: {
        sub: "AccountId20",
        main: "AccountId20",
        deposit: "u128",
      },
      SubIdentityRevoked: {
        sub: "AccountId20",
        main: "AccountId20",
        deposit: "u128",
      },
    },
  },
  /**
   * Lookup54: pallet_migrations::pallet::Event<T>
   */
  PalletMigrationsEvent: {
    _enum: {
      RuntimeUpgradeStarted: "Null",
      RuntimeUpgradeCompleted: {
        weight: "u64",
      },
      MigrationStarted: {
        migrationName: "Bytes",
      },
      MigrationCompleted: {
        migrationName: "Bytes",
        consumedWeight: "u64",
      },
    },
  },
  /**
   * Lookup55: pallet_evm::pallet::Event<T>
   */
  PalletEvmEvent: {
    _enum: {
      Log: "EthereumLog",
      Created: "H160",
      CreatedFailed: "H160",
      Executed: "H160",
      ExecutedFailed: "H160",
      BalanceDeposit: "(AccountId20,H160,U256)",
      BalanceWithdraw: "(AccountId20,H160,U256)",
    },
  },
  /**
   * Lookup56: ethereum::log::Log
   */
  EthereumLog: {
    address: "H160",
    topics: "Vec<H256>",
    data: "Bytes",
  },
  /**
   * Lookup61: pallet_ethereum::pallet::Event
   */
  PalletEthereumEvent: {
    _enum: {
      Executed: "(H160,H160,H256,EvmCoreErrorExitReason)",
    },
  },
  /**
   * Lookup62: evm_core::error::ExitReason
   */
  EvmCoreErrorExitReason: {
    _enum: {
      Succeed: "EvmCoreErrorExitSucceed",
      Error: "EvmCoreErrorExitError",
      Revert: "EvmCoreErrorExitRevert",
      Fatal: "EvmCoreErrorExitFatal",
    },
  },
  /**
   * Lookup63: evm_core::error::ExitSucceed
   */
  EvmCoreErrorExitSucceed: {
    _enum: ["Stopped", "Returned", "Suicided"],
  },
  /**
   * Lookup64: evm_core::error::ExitError
   */
  EvmCoreErrorExitError: {
    _enum: {
      StackUnderflow: "Null",
      StackOverflow: "Null",
      InvalidJump: "Null",
      InvalidRange: "Null",
      DesignatedInvalid: "Null",
      CallTooDeep: "Null",
      CreateCollision: "Null",
      CreateContractLimit: "Null",
      OutOfOffset: "Null",
      OutOfGas: "Null",
      OutOfFund: "Null",
      PCUnderflow: "Null",
      CreateEmpty: "Null",
      Other: "Text",
      InvalidCode: "Null",
    },
  },
  /**
   * Lookup67: evm_core::error::ExitRevert
   */
  EvmCoreErrorExitRevert: {
    _enum: ["Reverted"],
  },
  /**
   * Lookup68: evm_core::error::ExitFatal
   */
  EvmCoreErrorExitFatal: {
    _enum: {
      NotSupported: "Null",
      UnhandledInterrupt: "Null",
      CallErrorAsFatal: "EvmCoreErrorExitError",
      Other: "Text",
    },
  },
  /**
   * Lookup69: pallet_base_fee::pallet::Event
   */
  PalletBaseFeeEvent: {
    _enum: {
      NewBaseFeePerGas: "U256",
      BaseFeeOverflow: "Null",
      IsActive: "bool",
      NewElasticity: "Permill",
    },
  },
  /**
   * Lookup71: pallet_scheduler::pallet::Event<T>
   */
  PalletSchedulerEvent: {
    _enum: {
      Scheduled: {
        when: "u32",
        index: "u32",
      },
      Canceled: {
        when: "u32",
        index: "u32",
      },
      Dispatched: {
        task: "(u32,u32)",
        id: "Option<Bytes>",
        result: "Result<Null, SpRuntimeDispatchError>",
      },
      CallLookupFailed: {
        task: "(u32,u32)",
        id: "Option<Bytes>",
        error: "FrameSupportScheduleLookupError",
      },
    },
  },
  /**
   * Lookup74: frame_support::traits::schedule::LookupError
   */
  FrameSupportScheduleLookupError: {
    _enum: ["Unknown", "BadFormat"],
  },
  /**
   * Lookup75: pallet_democracy::pallet::Event<T>
   */
  PalletDemocracyEvent: {
    _enum: {
      Proposed: {
        proposalIndex: "u32",
        deposit: "u128",
      },
      Tabled: {
        proposalIndex: "u32",
        deposit: "u128",
        depositors: "Vec<AccountId20>",
      },
      ExternalTabled: "Null",
      Started: {
        refIndex: "u32",
        threshold: "PalletDemocracyVoteThreshold",
      },
      Passed: {
        refIndex: "u32",
      },
      NotPassed: {
        refIndex: "u32",
      },
      Cancelled: {
        refIndex: "u32",
      },
      Executed: {
        refIndex: "u32",
        result: "Result<Null, SpRuntimeDispatchError>",
      },
      Delegated: {
        who: "AccountId20",
        target: "AccountId20",
      },
      Undelegated: {
        account: "AccountId20",
      },
      Vetoed: {
        who: "AccountId20",
        proposalHash: "H256",
        until: "u32",
      },
      PreimageNoted: {
        proposalHash: "H256",
        who: "AccountId20",
        deposit: "u128",
      },
      PreimageUsed: {
        proposalHash: "H256",
        provider: "AccountId20",
        deposit: "u128",
      },
      PreimageInvalid: {
        proposalHash: "H256",
        refIndex: "u32",
      },
      PreimageMissing: {
        proposalHash: "H256",
        refIndex: "u32",
      },
      PreimageReaped: {
        proposalHash: "H256",
        provider: "AccountId20",
        deposit: "u128",
        reaper: "AccountId20",
      },
      Blacklisted: {
        proposalHash: "H256",
      },
      Voted: {
        voter: "AccountId20",
        refIndex: "u32",
        vote: "PalletDemocracyVoteAccountVote",
      },
      Seconded: {
        seconder: "AccountId20",
        propIndex: "u32",
      },
    },
  },
  /**
   * Lookup77: pallet_democracy::vote_threshold::VoteThreshold
   */
  PalletDemocracyVoteThreshold: {
    _enum: ["SuperMajorityApprove", "SuperMajorityAgainst", "SimpleMajority"],
  },
  /**
   * Lookup78: pallet_democracy::vote::AccountVote<Balance>
   */
  PalletDemocracyVoteAccountVote: {
    _enum: {
      Standard: {
        vote: "Vote",
        balance: "u128",
      },
      Split: {
        aye: "u128",
        nay: "u128",
      },
    },
  },
  /**
   * Lookup80: pallet_collective::pallet::Event<T, I>
   */
  PalletCollectiveEvent: {
    _enum: {
      Proposed: {
        account: "AccountId20",
        proposalIndex: "u32",
        proposalHash: "H256",
        threshold: "u32",
      },
      Voted: {
        account: "AccountId20",
        proposalHash: "H256",
        voted: "bool",
        yes: "u32",
        no: "u32",
      },
      Approved: {
        proposalHash: "H256",
      },
      Disapproved: {
        proposalHash: "H256",
      },
      Executed: {
        proposalHash: "H256",
        result: "Result<Null, SpRuntimeDispatchError>",
      },
      MemberExecuted: {
        proposalHash: "H256",
        result: "Result<Null, SpRuntimeDispatchError>",
      },
      Closed: {
        proposalHash: "H256",
        yes: "u32",
        no: "u32",
      },
    },
  },
  /**
   * Lookup82: pallet_treasury::pallet::Event<T, I>
   */
  PalletTreasuryEvent: {
    _enum: {
      Proposed: {
        proposalIndex: "u32",
      },
      Spending: {
        budgetRemaining: "u128",
      },
      Awarded: {
        proposalIndex: "u32",
        award: "u128",
        account: "AccountId20",
      },
      Rejected: {
        proposalIndex: "u32",
        slashed: "u128",
      },
      Burnt: {
        burntFunds: "u128",
      },
      Rollover: {
        rolloverBalance: "u128",
      },
      Deposit: {
        value: "u128",
      },
    },
  },
  /**
   * Lookup83: pallet_crowdloan_rewards::pallet::Event<T>
   */
  PalletCrowdloanRewardsEvent: {
    _enum: {
      InitialPaymentMade: "(AccountId20,u128)",
      NativeIdentityAssociated: "([u8;32],AccountId20,u128)",
      RewardsPaid: "(AccountId20,u128)",
      RewardAddressUpdated: "(AccountId20,AccountId20)",
      InitializedAlreadyInitializedAccount: "([u8;32],Option<AccountId20>,u128)",
      InitializedAccountWithNotEnoughContribution: "([u8;32],Option<AccountId20>,u128)",
    },
  },
  /**
   * Lookup84: cumulus_pallet_xcmp_queue::pallet::Event<T>
   */
  CumulusPalletXcmpQueueEvent: {
    _enum: {
      Success: "Option<H256>",
      Fail: "(Option<H256>,XcmV2TraitsError)",
      BadVersion: "Option<H256>",
      BadFormat: "Option<H256>",
      UpwardMessageSent: "Option<H256>",
      XcmpMessageSent: "Option<H256>",
      OverweightEnqueued: "(u32,u32,u64,u64)",
      OverweightServiced: "(u64,u64)",
    },
  },
  /**
   * Lookup86: xcm::v2::traits::Error
   */
  XcmV2TraitsError: {
    _enum: {
      Overflow: "Null",
      Unimplemented: "Null",
      UntrustedReserveLocation: "Null",
      UntrustedTeleportLocation: "Null",
      MultiLocationFull: "Null",
      MultiLocationNotInvertible: "Null",
      BadOrigin: "Null",
      InvalidLocation: "Null",
      AssetNotFound: "Null",
      FailedToTransactAsset: "Null",
      NotWithdrawable: "Null",
      LocationCannotHold: "Null",
      ExceedsMaxMessageSize: "Null",
      DestinationUnsupported: "Null",
      Transport: "Null",
      Unroutable: "Null",
      UnknownClaim: "Null",
      FailedToDecode: "Null",
      MaxWeightInvalid: "Null",
      NotHoldingFees: "Null",
      TooExpensive: "Null",
      Trap: "u64",
      UnhandledXcmVersion: "Null",
      WeightLimitReached: "u64",
      Barrier: "Null",
      WeightNotComputable: "Null",
    },
  },
  /**
   * Lookup88: cumulus_pallet_xcm::pallet::Event<T>
   */
  CumulusPalletXcmEvent: {
    _enum: {
      InvalidFormat: "[u8;8]",
      UnsupportedVersion: "[u8;8]",
      ExecutedDownward: "([u8;8],XcmV2TraitsOutcome)",
    },
  },
  /**
   * Lookup90: xcm::v2::traits::Outcome
   */
  XcmV2TraitsOutcome: {
    _enum: {
      Complete: "u64",
      Incomplete: "(u64,XcmV2TraitsError)",
      Error: "XcmV2TraitsError",
    },
  },
  /**
   * Lookup91: cumulus_pallet_dmp_queue::pallet::Event<T>
   */
  CumulusPalletDmpQueueEvent: {
    _enum: {
      InvalidFormat: "[u8;32]",
      UnsupportedVersion: "[u8;32]",
      ExecutedDownward: "([u8;32],XcmV2TraitsOutcome)",
      WeightExhausted: "([u8;32],u64,u64)",
      OverweightEnqueued: "([u8;32],u64,u64)",
      OverweightServiced: "(u64,u64)",
    },
  },
  /**
   * Lookup92: pallet_xcm::pallet::Event<T>
   */
  PalletXcmEvent: {
    _enum: {
      Attempted: "XcmV2TraitsOutcome",
      Sent: "(XcmV1MultiLocation,XcmV1MultiLocation,XcmV2Xcm)",
      UnexpectedResponse: "(XcmV1MultiLocation,u64)",
      ResponseReady: "(u64,XcmV2Response)",
      Notified: "(u64,u8,u8)",
      NotifyOverweight: "(u64,u8,u8,u64,u64)",
      NotifyDispatchError: "(u64,u8,u8)",
      NotifyDecodeFailed: "(u64,u8,u8)",
      InvalidResponder: "(XcmV1MultiLocation,u64,Option<XcmV1MultiLocation>)",
      InvalidResponderVersion: "(XcmV1MultiLocation,u64)",
      ResponseTaken: "u64",
      AssetsTrapped: "(H256,XcmV1MultiLocation,XcmVersionedMultiAssets)",
      VersionChangeNotified: "(XcmV1MultiLocation,u32)",
      SupportedVersionChanged: "(XcmV1MultiLocation,u32)",
      NotifyTargetSendFail: "(XcmV1MultiLocation,u64,XcmV2TraitsError)",
      NotifyTargetMigrationFail: "(XcmVersionedMultiLocation,u64)",
    },
  },
  /**
   * Lookup93: xcm::v1::multilocation::MultiLocation
   */
  XcmV1MultiLocation: {
    parents: "u8",
    interior: "XcmV1MultilocationJunctions",
  },
  /**
   * Lookup94: xcm::v1::multilocation::Junctions
   */
  XcmV1MultilocationJunctions: {
    _enum: {
      Here: "Null",
      X1: "XcmV1Junction",
      X2: "(XcmV1Junction,XcmV1Junction)",
      X3: "(XcmV1Junction,XcmV1Junction,XcmV1Junction)",
      X4: "(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)",
      X5: "(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)",
      X6: "(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)",
      X7: "(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)",
      X8: "(XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction,XcmV1Junction)",
    },
  },
  /**
   * Lookup95: xcm::v1::junction::Junction
   */
  XcmV1Junction: {
    _enum: {
      Parachain: "Compact<u32>",
      AccountId32: {
        network: "XcmV0JunctionNetworkId",
        id: "[u8;32]",
      },
      AccountIndex64: {
        network: "XcmV0JunctionNetworkId",
        index: "Compact<u64>",
      },
      AccountKey20: {
        network: "XcmV0JunctionNetworkId",
        key: "[u8;20]",
      },
      PalletInstance: "u8",
      GeneralIndex: "Compact<u128>",
      GeneralKey: "Bytes",
      OnlyChild: "Null",
      Plurality: {
        id: "XcmV0JunctionBodyId",
        part: "XcmV0JunctionBodyPart",
      },
    },
  },
  /**
   * Lookup97: xcm::v0::junction::NetworkId
   */
  XcmV0JunctionNetworkId: {
    _enum: {
      Any: "Null",
      Named: "Bytes",
      Polkadot: "Null",
      Kusama: "Null",
    },
  },
  /**
   * Lookup100: xcm::v0::junction::BodyId
   */
  XcmV0JunctionBodyId: {
    _enum: {
      Unit: "Null",
      Named: "Bytes",
      Index: "Compact<u32>",
      Executive: "Null",
      Technical: "Null",
      Legislative: "Null",
      Judicial: "Null",
    },
  },
  /**
   * Lookup101: xcm::v0::junction::BodyPart
   */
  XcmV0JunctionBodyPart: {
    _enum: {
      Voice: "Null",
      Members: {
        count: "Compact<u32>",
      },
      Fraction: {
        nom: "Compact<u32>",
        denom: "Compact<u32>",
      },
      AtLeastProportion: {
        nom: "Compact<u32>",
        denom: "Compact<u32>",
      },
      MoreThanProportion: {
        nom: "Compact<u32>",
        denom: "Compact<u32>",
      },
    },
  },
  /**
   * Lookup102: xcm::v2::Xcm<Call>
   */
  XcmV2Xcm: "Vec<XcmV2Instruction>",
  /**
   * Lookup104: xcm::v2::Instruction<Call>
   */
  XcmV2Instruction: {
    _enum: {
      WithdrawAsset: "XcmV1MultiassetMultiAssets",
      ReserveAssetDeposited: "XcmV1MultiassetMultiAssets",
      ReceiveTeleportedAsset: "XcmV1MultiassetMultiAssets",
      QueryResponse: {
        queryId: "Compact<u64>",
        response: "XcmV2Response",
        maxWeight: "Compact<u64>",
      },
      TransferAsset: {
        assets: "XcmV1MultiassetMultiAssets",
        beneficiary: "XcmV1MultiLocation",
      },
      TransferReserveAsset: {
        assets: "XcmV1MultiassetMultiAssets",
        dest: "XcmV1MultiLocation",
        xcm: "XcmV2Xcm",
      },
      Transact: {
        originType: "XcmV0OriginKind",
        requireWeightAtMost: "Compact<u64>",
        call: "XcmDoubleEncoded",
      },
      HrmpNewChannelOpenRequest: {
        sender: "Compact<u32>",
        maxMessageSize: "Compact<u32>",
        maxCapacity: "Compact<u32>",
      },
      HrmpChannelAccepted: {
        recipient: "Compact<u32>",
      },
      HrmpChannelClosing: {
        initiator: "Compact<u32>",
        sender: "Compact<u32>",
        recipient: "Compact<u32>",
      },
      ClearOrigin: "Null",
      DescendOrigin: "XcmV1MultilocationJunctions",
      ReportError: {
        queryId: "Compact<u64>",
        dest: "XcmV1MultiLocation",
        maxResponseWeight: "Compact<u64>",
      },
      DepositAsset: {
        assets: "XcmV1MultiassetMultiAssetFilter",
        maxAssets: "Compact<u32>",
        beneficiary: "XcmV1MultiLocation",
      },
      DepositReserveAsset: {
        assets: "XcmV1MultiassetMultiAssetFilter",
        maxAssets: "Compact<u32>",
        dest: "XcmV1MultiLocation",
        xcm: "XcmV2Xcm",
      },
      ExchangeAsset: {
        give: "XcmV1MultiassetMultiAssetFilter",
        receive: "XcmV1MultiassetMultiAssets",
      },
      InitiateReserveWithdraw: {
        assets: "XcmV1MultiassetMultiAssetFilter",
        reserve: "XcmV1MultiLocation",
        xcm: "XcmV2Xcm",
      },
      InitiateTeleport: {
        assets: "XcmV1MultiassetMultiAssetFilter",
        dest: "XcmV1MultiLocation",
        xcm: "XcmV2Xcm",
      },
      QueryHolding: {
        queryId: "Compact<u64>",
        dest: "XcmV1MultiLocation",
        assets: "XcmV1MultiassetMultiAssetFilter",
        maxResponseWeight: "Compact<u64>",
      },
      BuyExecution: {
        fees: "XcmV1MultiAsset",
        weightLimit: "XcmV2WeightLimit",
      },
      RefundSurplus: "Null",
      SetErrorHandler: "XcmV2Xcm",
      SetAppendix: "XcmV2Xcm",
      ClearError: "Null",
      ClaimAsset: {
        assets: "XcmV1MultiassetMultiAssets",
        ticket: "XcmV1MultiLocation",
      },
      Trap: "Compact<u64>",
      SubscribeVersion: {
        queryId: "Compact<u64>",
        maxResponseWeight: "Compact<u64>",
      },
      UnsubscribeVersion: "Null",
    },
  },
  /**
   * Lookup105: xcm::v1::multiasset::MultiAssets
   */
  XcmV1MultiassetMultiAssets: "Vec<XcmV1MultiAsset>",
  /**
   * Lookup107: xcm::v1::multiasset::MultiAsset
   */
  XcmV1MultiAsset: {
    id: "XcmV1MultiassetAssetId",
    fun: "XcmV1MultiassetFungibility",
  },
  /**
   * Lookup108: xcm::v1::multiasset::AssetId
   */
  XcmV1MultiassetAssetId: {
    _enum: {
      Concrete: "XcmV1MultiLocation",
      Abstract: "Bytes",
    },
  },
  /**
   * Lookup109: xcm::v1::multiasset::Fungibility
   */
  XcmV1MultiassetFungibility: {
    _enum: {
      Fungible: "Compact<u128>",
      NonFungible: "XcmV1MultiassetAssetInstance",
    },
  },
  /**
   * Lookup110: xcm::v1::multiasset::AssetInstance
   */
  XcmV1MultiassetAssetInstance: {
    _enum: {
      Undefined: "Null",
      Index: "Compact<u128>",
      Array4: "[u8;4]",
      Array8: "[u8;8]",
      Array16: "[u8;16]",
      Array32: "[u8;32]",
      Blob: "Bytes",
    },
  },
  /**
   * Lookup112: xcm::v2::Response
   */
  XcmV2Response: {
    _enum: {
      Null: "Null",
      Assets: "XcmV1MultiassetMultiAssets",
      ExecutionResult: "Option<(u32,XcmV2TraitsError)>",
      Version: "u32",
    },
  },
  /**
   * Lookup115: xcm::v0::OriginKind
   */
  XcmV0OriginKind: {
    _enum: ["Native", "SovereignAccount", "Superuser", "Xcm"],
  },
  /**
   * Lookup116: xcm::double_encoded::DoubleEncoded<T>
   */
  XcmDoubleEncoded: {
    encoded: "Bytes",
  },
  /**
   * Lookup117: xcm::v1::multiasset::MultiAssetFilter
   */
  XcmV1MultiassetMultiAssetFilter: {
    _enum: {
      Definite: "XcmV1MultiassetMultiAssets",
      Wild: "XcmV1MultiassetWildMultiAsset",
    },
  },
  /**
   * Lookup118: xcm::v1::multiasset::WildMultiAsset
   */
  XcmV1MultiassetWildMultiAsset: {
    _enum: {
      All: "Null",
      AllOf: {
        id: "XcmV1MultiassetAssetId",
        fun: "XcmV1MultiassetWildFungibility",
      },
    },
  },
  /**
   * Lookup119: xcm::v1::multiasset::WildFungibility
   */
  XcmV1MultiassetWildFungibility: {
    _enum: ["Fungible", "NonFungible"],
  },
  /**
   * Lookup120: xcm::v2::WeightLimit
   */
  XcmV2WeightLimit: {
    _enum: {
      Unlimited: "Null",
      Limited: "Compact<u64>",
    },
  },
  /**
   * Lookup122: xcm::VersionedMultiAssets
   */
  XcmVersionedMultiAssets: {
    _enum: {
      V0: "Vec<XcmV0MultiAsset>",
      V1: "XcmV1MultiassetMultiAssets",
    },
  },
  /**
   * Lookup124: xcm::v0::multi_asset::MultiAsset
   */
  XcmV0MultiAsset: {
    _enum: {
      None: "Null",
      All: "Null",
      AllFungible: "Null",
      AllNonFungible: "Null",
      AllAbstractFungible: {
        id: "Bytes",
      },
      AllAbstractNonFungible: {
        class: "Bytes",
      },
      AllConcreteFungible: {
        id: "XcmV0MultiLocation",
      },
      AllConcreteNonFungible: {
        class: "XcmV0MultiLocation",
      },
      AbstractFungible: {
        id: "Bytes",
        amount: "Compact<u128>",
      },
      AbstractNonFungible: {
        class: "Bytes",
        instance: "XcmV1MultiassetAssetInstance",
      },
      ConcreteFungible: {
        id: "XcmV0MultiLocation",
        amount: "Compact<u128>",
      },
      ConcreteNonFungible: {
        class: "XcmV0MultiLocation",
        instance: "XcmV1MultiassetAssetInstance",
      },
    },
  },
  /**
   * Lookup125: xcm::v0::multi_location::MultiLocation
   */
  XcmV0MultiLocation: {
    _enum: {
      Null: "Null",
      X1: "XcmV0Junction",
      X2: "(XcmV0Junction,XcmV0Junction)",
      X3: "(XcmV0Junction,XcmV0Junction,XcmV0Junction)",
      X4: "(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)",
      X5: "(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)",
      X6: "(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)",
      X7: "(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)",
      X8: "(XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction,XcmV0Junction)",
    },
  },
  /**
   * Lookup126: xcm::v0::junction::Junction
   */
  XcmV0Junction: {
    _enum: {
      Parent: "Null",
      Parachain: "Compact<u32>",
      AccountId32: {
        network: "XcmV0JunctionNetworkId",
        id: "[u8;32]",
      },
      AccountIndex64: {
        network: "XcmV0JunctionNetworkId",
        index: "Compact<u64>",
      },
      AccountKey20: {
        network: "XcmV0JunctionNetworkId",
        key: "[u8;20]",
      },
      PalletInstance: "u8",
      GeneralIndex: "Compact<u128>",
      GeneralKey: "Bytes",
      OnlyChild: "Null",
      Plurality: {
        id: "XcmV0JunctionBodyId",
        part: "XcmV0JunctionBodyPart",
      },
    },
  },
  /**
   * Lookup127: xcm::VersionedMultiLocation
   */
  XcmVersionedMultiLocation: {
    _enum: {
      V0: "XcmV0MultiLocation",
      V1: "XcmV1MultiLocation",
    },
  },
  /**
   * Lookup128: pallet_assets::pallet::Event<T, I>
   */
  PalletAssetsEvent: {
    _enum: {
      Created: {
        assetId: "u128",
        creator: "AccountId20",
        owner: "AccountId20",
      },
      Issued: {
        assetId: "u128",
        owner: "AccountId20",
        totalSupply: "u128",
      },
      Transferred: {
        assetId: "u128",
        from: "AccountId20",
        to: "AccountId20",
        amount: "u128",
      },
      Burned: {
        assetId: "u128",
        owner: "AccountId20",
        balance: "u128",
      },
      TeamChanged: {
        assetId: "u128",
        issuer: "AccountId20",
        admin: "AccountId20",
        freezer: "AccountId20",
      },
      OwnerChanged: {
        assetId: "u128",
        owner: "AccountId20",
      },
      Frozen: {
        assetId: "u128",
        who: "AccountId20",
      },
      Thawed: {
        assetId: "u128",
        who: "AccountId20",
      },
      AssetFrozen: {
        assetId: "u128",
      },
      AssetThawed: {
        assetId: "u128",
      },
      Destroyed: {
        assetId: "u128",
      },
      ForceCreated: {
        assetId: "u128",
        owner: "AccountId20",
      },
      MetadataSet: {
        assetId: "u128",
        name: "Bytes",
        symbol: "Bytes",
        decimals: "u8",
        isFrozen: "bool",
      },
      MetadataCleared: {
        assetId: "u128",
      },
      ApprovedTransfer: {
        assetId: "u128",
        source: "AccountId20",
        delegate: "AccountId20",
        amount: "u128",
      },
      ApprovalCancelled: {
        assetId: "u128",
        owner: "AccountId20",
        delegate: "AccountId20",
      },
      TransferredApproved: {
        assetId: "u128",
        owner: "AccountId20",
        delegate: "AccountId20",
        destination: "AccountId20",
        amount: "u128",
      },
      AssetStatusChanged: {
        assetId: "u128",
      },
    },
  },
  /**
   * Lookup129: pallet_asset_manager::pallet::Event<T>
   */
  PalletAssetManagerEvent: {
    _enum: {
      ForeignAssetRegistered: {
        assetId: "u128",
        asset: "MoonbeamRuntimeXcmConfigAssetType",
        metadata: "MoonbeamRuntimeAssetConfigAssetRegistrarMetadata",
      },
      UnitsPerSecondChanged: {
        assetType: "MoonbeamRuntimeXcmConfigAssetType",
        unitsPerSecond: "u128",
      },
      ForeignAssetTypeChanged: {
        assetId: "u128",
        newAssetType: "MoonbeamRuntimeXcmConfigAssetType",
      },
      ForeignAssetRemoved: {
        assetId: "u128",
        assetType: "MoonbeamRuntimeXcmConfigAssetType",
      },
      SupportedAssetRemoved: {
        assetType: "MoonbeamRuntimeXcmConfigAssetType",
      },
      LocalAssetRegistered: {
        assetId: "u128",
        creator: "AccountId20",
        owner: "AccountId20",
      },
      ForeignAssetDestroyed: {
        assetId: "u128",
        assetType: "MoonbeamRuntimeXcmConfigAssetType",
      },
      LocalAssetDestroyed: {
        assetId: "u128",
      },
    },
  },
  /**
   * Lookup130: moonbeam_runtime::xcm_config::AssetType
   */
  MoonbeamRuntimeXcmConfigAssetType: {
    _enum: {
      Xcm: "XcmV1MultiLocation",
    },
  },
  /**
   * Lookup131: moonbeam_runtime::asset_config::AssetRegistrarMetadata
   */
  MoonbeamRuntimeAssetConfigAssetRegistrarMetadata: {
    name: "Bytes",
    symbol: "Bytes",
    decimals: "u8",
    isFrozen: "bool",
  },
  /**
   * Lookup132: orml_xtokens::module::Event<T>
   */
  OrmlXtokensModuleEvent: {
    _enum: {
      TransferredMultiAssets: {
        sender: "AccountId20",
        assets: "XcmV1MultiassetMultiAssets",
        fee: "XcmV1MultiAsset",
        dest: "XcmV1MultiLocation",
      },
    },
  },
  /**
   * Lookup133: xcm_transactor::pallet::Event<T>
   */
  XcmTransactorEvent: {
    _enum: {
      TransactedDerivative: {
        accountId: "AccountId20",
        dest: "XcmV1MultiLocation",
        call: "Bytes",
        index: "u16",
      },
      TransactedSovereign: {
        feePayer: "AccountId20",
        dest: "XcmV1MultiLocation",
        call: "Bytes",
      },
      TransactedSigned: {
        feePayer: "AccountId20",
        dest: "XcmV1MultiLocation",
        call: "Bytes",
      },
      RegisteredDerivative: {
        accountId: "AccountId20",
        index: "u16",
      },
      DeRegisteredDerivative: {
        index: "u16",
      },
      TransactFailed: {
        error: "XcmV2TraitsError",
      },
      TransactInfoChanged: {
        location: "XcmV1MultiLocation",
        remoteInfo: "XcmTransactorRemoteTransactInfoWithMaxWeight",
      },
      TransactInfoRemoved: {
        location: "XcmV1MultiLocation",
      },
      DestFeePerSecondChanged: {
        location: "XcmV1MultiLocation",
        feePerSecond: "u128",
      },
      DestFeePerSecondRemoved: {
        location: "XcmV1MultiLocation",
      },
    },
  },
  /**
   * Lookup134: xcm_transactor::pallet::RemoteTransactInfoWithMaxWeight
   */
  XcmTransactorRemoteTransactInfoWithMaxWeight: {
    transactExtraWeight: "u64",
    maxWeight: "u64",
    transactExtraWeightSigned: "Option<u64>",
  },
  /**
   * Lookup137: frame_system::Phase
   */
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: "u32",
      Finalization: "Null",
      Initialization: "Null",
    },
  },
  /**
   * Lookup139: frame_system::LastRuntimeUpgradeInfo
   */
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: "Compact<u32>",
    specName: "Text",
  },
  /**
   * Lookup140: frame_system::pallet::Call<T>
   */
  FrameSystemCall: {
    _enum: {
      fill_block: {
        ratio: "Perbill",
      },
      remark: {
        remark: "Bytes",
      },
      set_heap_pages: {
        pages: "u64",
      },
      set_code: {
        code: "Bytes",
      },
      set_code_without_checks: {
        code: "Bytes",
      },
      set_storage: {
        items: "Vec<(Bytes,Bytes)>",
      },
      kill_storage: {
        _alias: {
          keys_: "keys",
        },
        keys_: "Vec<Bytes>",
      },
      kill_prefix: {
        prefix: "Bytes",
        subkeys: "u32",
      },
      remark_with_event: {
        remark: "Bytes",
      },
    },
  },
  /**
   * Lookup144: frame_system::limits::BlockWeights
   */
  FrameSystemLimitsBlockWeights: {
    baseBlock: "u64",
    maxBlock: "u64",
    perClass: "FrameSupportWeightsPerDispatchClassWeightsPerClass",
  },
  /**
   * Lookup145:
   * frame_support::weights::PerDispatchClass<frame_system::limits::WeightsPerClass>
   */
  FrameSupportWeightsPerDispatchClassWeightsPerClass: {
    normal: "FrameSystemLimitsWeightsPerClass",
    operational: "FrameSystemLimitsWeightsPerClass",
    mandatory: "FrameSystemLimitsWeightsPerClass",
  },
  /**
   * Lookup146: frame_system::limits::WeightsPerClass
   */
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: "u64",
    maxExtrinsic: "Option<u64>",
    maxTotal: "Option<u64>",
    reserved: "Option<u64>",
  },
  /**
   * Lookup147: frame_system::limits::BlockLength
   */
  FrameSystemLimitsBlockLength: {
    max: "FrameSupportWeightsPerDispatchClassU32",
  },
  /**
   * Lookup148: frame_support::weights::PerDispatchClass<T>
   */
  FrameSupportWeightsPerDispatchClassU32: {
    normal: "u32",
    operational: "u32",
    mandatory: "u32",
  },
  /**
   * Lookup149: frame_support::weights::RuntimeDbWeight
   */
  FrameSupportWeightsRuntimeDbWeight: {
    read: "u64",
    write: "u64",
  },
  /**
   * Lookup150: sp_version::RuntimeVersion
   */
  SpVersionRuntimeVersion: {
    specName: "Text",
    implName: "Text",
    authoringVersion: "u32",
    specVersion: "u32",
    implVersion: "u32",
    apis: "Vec<([u8;8],u32)>",
    transactionVersion: "u32",
    stateVersion: "u8",
  },
  /**
   * Lookup154: frame_system::pallet::Error<T>
   */
  FrameSystemError: {
    _enum: [
      "InvalidSpecName",
      "SpecVersionNeedsToIncrease",
      "FailedToExtractRuntimeVersion",
      "NonDefaultComposite",
      "NonZeroRefCount",
      "CallFiltered",
    ],
  },
  /**
   * Lookup155: polkadot_primitives::v2::PersistedValidationData<primitive_types::H256, N>
   */
  PolkadotPrimitivesV2PersistedValidationData: {
    parentHead: "Bytes",
    relayParentNumber: "u32",
    relayParentStorageRoot: "H256",
    maxPovSize: "u32",
  },
  /**
   * Lookup158: polkadot_primitives::v2::UpgradeRestriction
   */
  PolkadotPrimitivesV2UpgradeRestriction: {
    _enum: ["Present"],
  },
  /**
   * Lookup159: sp_trie::storage_proof::StorageProof
   */
  SpTrieStorageProof: {
    trieNodes: "BTreeSet<Bytes>",
  },
  /**
   * Lookup161:
   * cumulus_pallet_parachain_system::relay_state_snapshot::MessagingStateSnapshot
   */
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: {
    dmqMqcHead: "H256",
    relayDispatchQueueSize: "(u32,u32)",
    ingressChannels: "Vec<(u32,PolkadotPrimitivesV2AbridgedHrmpChannel)>",
    egressChannels: "Vec<(u32,PolkadotPrimitivesV2AbridgedHrmpChannel)>",
  },
  /**
   * Lookup164: polkadot_primitives::v2::AbridgedHrmpChannel
   */
  PolkadotPrimitivesV2AbridgedHrmpChannel: {
    maxCapacity: "u32",
    maxTotalSize: "u32",
    maxMessageSize: "u32",
    msgCount: "u32",
    totalSize: "u32",
    mqcHead: "Option<H256>",
  },
  /**
   * Lookup165: polkadot_primitives::v2::AbridgedHostConfiguration
   */
  PolkadotPrimitivesV2AbridgedHostConfiguration: {
    maxCodeSize: "u32",
    maxHeadDataSize: "u32",
    maxUpwardQueueCount: "u32",
    maxUpwardQueueSize: "u32",
    maxUpwardMessageSize: "u32",
    maxUpwardMessageNumPerCandidate: "u32",
    hrmpMaxMessageNumPerCandidate: "u32",
    validationUpgradeCooldown: "u32",
    validationUpgradeDelay: "u32",
  },
  /**
   * Lookup171:
   * polkadot_core_primitives::OutboundHrmpMessage<polkadot_parachain::primitives::Id>
   */
  PolkadotCorePrimitivesOutboundHrmpMessage: {
    recipient: "u32",
    data: "Bytes",
  },
  /**
   * Lookup172: cumulus_pallet_parachain_system::pallet::Call<T>
   */
  CumulusPalletParachainSystemCall: {
    _enum: {
      set_validation_data: {
        data: "CumulusPrimitivesParachainInherentParachainInherentData",
      },
      sudo_send_upward_message: {
        message: "Bytes",
      },
      authorize_upgrade: {
        codeHash: "H256",
      },
      enact_authorized_upgrade: {
        code: "Bytes",
      },
    },
  },
  /**
   * Lookup173: cumulus_primitives_parachain_inherent::ParachainInherentData
   */
  CumulusPrimitivesParachainInherentParachainInherentData: {
    validationData: "PolkadotPrimitivesV2PersistedValidationData",
    relayChainState: "SpTrieStorageProof",
    downwardMessages: "Vec<PolkadotCorePrimitivesInboundDownwardMessage>",
    horizontalMessages: "BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>",
  },
  /**
   * Lookup175: polkadot_core_primitives::InboundDownwardMessage<BlockNumber>
   */
  PolkadotCorePrimitivesInboundDownwardMessage: {
    sentAt: "u32",
    msg: "Bytes",
  },
  /**
   * Lookup178: polkadot_core_primitives::InboundHrmpMessage<BlockNumber>
   */
  PolkadotCorePrimitivesInboundHrmpMessage: {
    sentAt: "u32",
    data: "Bytes",
  },
  /**
   * Lookup181: cumulus_pallet_parachain_system::pallet::Error<T>
   */
  CumulusPalletParachainSystemError: {
    _enum: [
      "OverlappingUpgrades",
      "ProhibitedByPolkadot",
      "TooBig",
      "ValidationDataNotAvailable",
      "HostConfigurationNotAvailable",
      "NotScheduled",
      "NothingAuthorized",
      "Unauthorized",
    ],
  },
  /**
   * Lookup183: pallet_timestamp::pallet::Call<T>
   */
  PalletTimestampCall: {
    _enum: {
      set: {
        now: "Compact<u64>",
      },
    },
  },
  /**
   * Lookup185: pallet_balances::BalanceLock<Balance>
   */
  PalletBalancesBalanceLock: {
    id: "[u8;8]",
    amount: "u128",
    reasons: "PalletBalancesReasons",
  },
  /**
   * Lookup186: pallet_balances::Reasons
   */
  PalletBalancesReasons: {
    _enum: ["Fee", "Misc", "All"],
  },
  /**
   * Lookup189: pallet_balances::ReserveData<ReserveIdentifier, Balance>
   */
  PalletBalancesReserveData: {
    id: "[u8;4]",
    amount: "u128",
  },
  /**
   * Lookup191: pallet_balances::Releases
   */
  PalletBalancesReleases: {
    _enum: ["V1_0_0", "V2_0_0"],
  },
  /**
   * Lookup192: pallet_balances::pallet::Call<T, I>
   */
  PalletBalancesCall: {
    _enum: {
      transfer: {
        dest: "AccountId20",
        value: "Compact<u128>",
      },
      set_balance: {
        who: "AccountId20",
        newFree: "Compact<u128>",
        newReserved: "Compact<u128>",
      },
      force_transfer: {
        source: "AccountId20",
        dest: "AccountId20",
        value: "Compact<u128>",
      },
      transfer_keep_alive: {
        dest: "AccountId20",
        value: "Compact<u128>",
      },
      transfer_all: {
        dest: "AccountId20",
        keepAlive: "bool",
      },
      force_unreserve: {
        who: "AccountId20",
        amount: "u128",
      },
    },
  },
  /**
   * Lookup193: pallet_balances::pallet::Error<T, I>
   */
  PalletBalancesError: {
    _enum: [
      "VestingBalance",
      "LiquidityRestrictions",
      "InsufficientBalance",
      "ExistentialDeposit",
      "KeepAlive",
      "ExistingVestingSchedule",
      "DeadAccount",
      "TooManyReserves",
    ],
  },
  /**
   * Lookup195: pallet_transaction_payment::Releases
   */
  PalletTransactionPaymentReleases: {
    _enum: ["V1Ancient", "V2"],
  },
  /**
   * Lookup197: frame_support::weights::WeightToFeeCoefficient<Balance>
   */
  FrameSupportWeightsWeightToFeeCoefficient: {
    coeffInteger: "u128",
    coeffFrac: "Perbill",
    negative: "bool",
    degree: "u8",
  },
  /**
   * Lookup198:
   * parachain_staking::types::ParachainBondConfig[account::AccountId20](account::AccountId20)
   */
  ParachainStakingParachainBondConfig: {
    account: "AccountId20",
    percent: "Percent",
  },
  /**
   * Lookup199: parachain_staking::types::RoundInfo<BlockNumber>
   */
  ParachainStakingRoundInfo: {
    current: "u32",
    first: "u32",
    length: "u32",
  },
  /**
   * Lookup200: parachain_staking::types::Nominator2<account::AccountId20, Balance>
   */
  ParachainStakingNominator2: {
    delegations: "ParachainStakingSetOrderedSetBond",
    revocations: "ParachainStakingSetOrderedSetAccountId20",
    total: "u128",
    scheduledRevocationsCount: "u32",
    scheduledRevocationsTotal: "u128",
    status: "ParachainStakingDelegatorStatus",
  },
  /**
   * Lookup201:
   * parachain_staking::set::OrderedSet<parachain_staking::types::Bond<account::AccountId20,
   * Balance>>
   */
  ParachainStakingSetOrderedSetBond: "Vec<ParachainStakingBond>",
  /**
   * Lookup202: parachain_staking::types::Bond<account::AccountId20, Balance>
   */
  ParachainStakingBond: {
    owner: "AccountId20",
    amount: "u128",
  },
  /**
   * Lookup204:
   * parachain_staking::set::OrderedSet[account::AccountId20](account::AccountId20)
   */
  ParachainStakingSetOrderedSetAccountId20: "Vec<AccountId20>",
  /**
   * Lookup205: parachain_staking::types::DelegatorStatus
   */
  ParachainStakingDelegatorStatus: {
    _enum: {
      Active: "Null",
      Leaving: "u32",
    },
  },
  /**
   * Lookup206: parachain_staking::types::Delegator<account::AccountId20, Balance>
   */
  ParachainStakingDelegator: {
    id: "AccountId20",
    delegations: "ParachainStakingSetOrderedSetBond",
    total: "u128",
    lessTotal: "u128",
    status: "ParachainStakingDelegatorStatus",
  },
  /**
   * Lookup207: parachain_staking::types::CollatorCandidate<account::AccountId20, Balance>
   */
  ParachainStakingCollatorCandidate: {
    id: "AccountId20",
    bond: "u128",
    delegators: "ParachainStakingSetOrderedSetAccountId20",
    topDelegations: "Vec<ParachainStakingBond>",
    bottomDelegations: "Vec<ParachainStakingBond>",
    totalCounted: "u128",
    totalBacking: "u128",
    request: "Option<ParachainStakingCandidateBondLessRequest>",
    state: "ParachainStakingCollatorStatus",
  },
  /**
   * Lookup209: parachain_staking::types::CandidateBondLessRequest<Balance>
   */
  ParachainStakingCandidateBondLessRequest: {
    amount: "u128",
    whenExecutable: "u32",
  },
  /**
   * Lookup210: parachain_staking::types::CollatorStatus
   */
  ParachainStakingCollatorStatus: {
    _enum: {
      Active: "Null",
      Idle: "Null",
      Leaving: "u32",
    },
  },
  /**
   * Lookup211: parachain_staking::types::CandidateMetadata<Balance>
   */
  ParachainStakingCandidateMetadata: {
    bond: "u128",
    delegationCount: "u32",
    totalCounted: "u128",
    lowestTopDelegationAmount: "u128",
    highestBottomDelegationAmount: "u128",
    lowestBottomDelegationAmount: "u128",
    topCapacity: "ParachainStakingCapacityStatus",
    bottomCapacity: "ParachainStakingCapacityStatus",
    request: "Option<ParachainStakingCandidateBondLessRequest>",
    status: "ParachainStakingCollatorStatus",
  },
  /**
   * Lookup212: parachain_staking::types::CapacityStatus
   */
  ParachainStakingCapacityStatus: {
    _enum: ["Full", "Empty", "Partial"],
  },
  /**
   * Lookup214:
   * parachain_staking::delegation_requests::ScheduledRequest<account::AccountId20,
   * Balance>
   */
  ParachainStakingDelegationRequestsScheduledRequest: {
    delegator: "AccountId20",
    whenExecutable: "u32",
    action: "ParachainStakingDelegationRequestsDelegationAction",
  },
  /**
   * Lookup215: parachain_staking::types::Delegations<account::AccountId20, Balance>
   */
  ParachainStakingDelegations: {
    delegations: "Vec<ParachainStakingBond>",
    total: "u128",
  },
  /**
   * Lookup216: parachain_staking::types::Collator2<account::AccountId20, Balance>
   */
  ParachainStakingCollator2: {
    id: "AccountId20",
    bond: "u128",
    nominators: "ParachainStakingSetOrderedSetAccountId20",
    topNominators: "Vec<ParachainStakingBond>",
    bottomNominators: "Vec<ParachainStakingBond>",
    totalCounted: "u128",
    totalBacking: "u128",
    state: "ParachainStakingCollatorStatus",
  },
  /**
   * Lookup218: parachain_staking::types::CollatorSnapshot<account::AccountId20, Balance>
   */
  ParachainStakingCollatorSnapshot: {
    bond: "u128",
    delegations: "Vec<ParachainStakingBond>",
    total: "u128",
  },
  /**
   * Lookup219: parachain_staking::types::DelayedPayout<Balance>
   */
  ParachainStakingDelayedPayout: {
    roundIssuance: "u128",
    totalStakingReward: "u128",
    collatorCommission: "Perbill",
  },
  /**
   * Lookup220: parachain_staking::inflation::InflationInfo<Balance>
   */
  ParachainStakingInflationInflationInfo: {
    expect: {
      min: "u128",
      ideal: "u128",
      max: "u128",
    },
    annual: {
      min: "Perbill",
      ideal: "Perbill",
      max: "Perbill",
    },
    round: {
      min: "Perbill",
      ideal: "Perbill",
      max: "Perbill",
    },
  },
  /**
   * Lookup223: parachain_staking::pallet::Call<T>
   */
  ParachainStakingCall: {
    _enum: {
      set_staking_expectations: {
        expectations: {
          min: "u128",
          ideal: "u128",
          max: "u128",
        },
      },
      set_inflation: {
        schedule: {
          min: "Perbill",
          ideal: "Perbill",
          max: "Perbill",
        },
      },
      set_parachain_bond_account: {
        _alias: {
          new_: "new",
        },
        new_: "AccountId20",
      },
      set_parachain_bond_reserve_percent: {
        _alias: {
          new_: "new",
        },
        new_: "Percent",
      },
      set_total_selected: {
        _alias: {
          new_: "new",
        },
        new_: "u32",
      },
      set_collator_commission: {
        _alias: {
          new_: "new",
        },
        new_: "Perbill",
      },
      set_blocks_per_round: {
        _alias: {
          new_: "new",
        },
        new_: "u32",
      },
      join_candidates: {
        bond: "u128",
        candidateCount: "u32",
      },
      schedule_leave_candidates: {
        candidateCount: "u32",
      },
      execute_leave_candidates: {
        candidate: "AccountId20",
        candidateDelegationCount: "u32",
      },
      cancel_leave_candidates: {
        candidateCount: "u32",
      },
      go_offline: "Null",
      go_online: "Null",
      candidate_bond_more: {
        more: "u128",
      },
      schedule_candidate_bond_less: {
        less: "u128",
      },
      execute_candidate_bond_less: {
        candidate: "AccountId20",
      },
      cancel_candidate_bond_less: "Null",
      delegate: {
        candidate: "AccountId20",
        amount: "u128",
        candidateDelegationCount: "u32",
        delegationCount: "u32",
      },
      schedule_leave_delegators: "Null",
      execute_leave_delegators: {
        delegator: "AccountId20",
        delegationCount: "u32",
      },
      cancel_leave_delegators: "Null",
      schedule_revoke_delegation: {
        collator: "AccountId20",
      },
      delegator_bond_more: {
        candidate: "AccountId20",
        more: "u128",
      },
      schedule_delegator_bond_less: {
        candidate: "AccountId20",
        less: "u128",
      },
      execute_delegation_request: {
        delegator: "AccountId20",
        candidate: "AccountId20",
      },
      cancel_delegation_request: {
        candidate: "AccountId20",
      },
    },
  },
  /**
   * Lookup224: parachain_staking::pallet::Error<T>
   */
  ParachainStakingError: {
    _enum: [
      "DelegatorDNE",
      "DelegatorDNEinTopNorBottom",
      "DelegatorDNEInDelegatorSet",
      "CandidateDNE",
      "DelegationDNE",
      "DelegatorExists",
      "CandidateExists",
      "CandidateBondBelowMin",
      "InsufficientBalance",
      "DelegatorBondBelowMin",
      "DelegationBelowMin",
      "AlreadyOffline",
      "AlreadyActive",
      "DelegatorAlreadyLeaving",
      "DelegatorNotLeaving",
      "DelegatorCannotLeaveYet",
      "CannotDelegateIfLeaving",
      "CandidateAlreadyLeaving",
      "CandidateNotLeaving",
      "CandidateCannotLeaveYet",
      "CannotGoOnlineIfLeaving",
      "ExceedMaxDelegationsPerDelegator",
      "AlreadyDelegatedCandidate",
      "InvalidSchedule",
      "CannotSetBelowMin",
      "RoundLengthMustBeAtLeastTotalSelectedCollators",
      "NoWritingSameValue",
      "TooLowCandidateCountWeightHintJoinCandidates",
      "TooLowCandidateCountWeightHintCancelLeaveCandidates",
      "TooLowCandidateCountToLeaveCandidates",
      "TooLowDelegationCountToDelegate",
      "TooLowCandidateDelegationCountToDelegate",
      "TooLowCandidateDelegationCountToLeaveCandidates",
      "TooLowDelegationCountToLeaveDelegators",
      "PendingCandidateRequestsDNE",
      "PendingCandidateRequestAlreadyExists",
      "PendingCandidateRequestNotDueYet",
      "PendingDelegationRequestDNE",
      "PendingDelegationRequestAlreadyExists",
      "PendingDelegationRequestNotDueYet",
      "CannotDelegateLessThanOrEqualToLowestBottomWhenFull",
      "PendingDelegationRevoke",
    ],
  },
  /**
   * Lookup225: pallet_author_inherent::pallet::Call<T>
   */
  PalletAuthorInherentCall: {
    _enum: ["kick_off_authorship_validation"],
  },
  /**
   * Lookup226: pallet_author_inherent::pallet::Error<T>
   */
  PalletAuthorInherentError: {
    _enum: ["AuthorAlreadySet", "NoAccountId", "CannotBeAuthor"],
  },
  /**
   * Lookup227: pallet_author_slot_filter::pallet::Call<T>
   */
  PalletAuthorSlotFilterCall: {
    _enum: {
      set_eligible: {
        _alias: {
          new_: "new",
        },
        new_: "u32",
      },
    },
  },
  /**
   * Lookup228: pallet_author_mapping::pallet::RegistrationInfo<T>
   */
  PalletAuthorMappingRegistrationInfo: {
    _alias: {
      keys_: "keys",
    },
    account: "AccountId20",
    deposit: "u128",
    keys_: "SessionKeysPrimitivesVrfVrfCryptoPublic",
  },
  /**
   * Lookup229: pallet_author_mapping::pallet::Call<T>
   */
  PalletAuthorMappingCall: {
    _enum: {
      add_association: {
        nimbusId: "NimbusPrimitivesNimbusCryptoPublic",
      },
      update_association: {
        oldNimbusId: "NimbusPrimitivesNimbusCryptoPublic",
        newNimbusId: "NimbusPrimitivesNimbusCryptoPublic",
      },
      clear_association: {
        nimbusId: "NimbusPrimitivesNimbusCryptoPublic",
      },
      remove_keys: "Null",
      set_keys: {
        _alias: {
          keys_: "keys",
        },
        keys_: "Bytes",
      },
    },
  },
  /**
   * Lookup230: pallet_author_mapping::pallet::Error<T>
   */
  PalletAuthorMappingError: {
    _enum: [
      "AssociationNotFound",
      "NotYourAssociation",
      "CannotAffordSecurityDeposit",
      "AlreadyAssociated",
      "OldAuthorIdNotFound",
      "WrongKeySize",
      "DecodeNimbusFailed",
      "DecodeKeysFailed",
    ],
  },
  /**
   * Lookup231:
   * pallet_moonbeam_orbiters::types::CollatorPoolInfo[account::AccountId20](account::AccountId20)
   */
  PalletMoonbeamOrbitersCollatorPoolInfo: {
    orbiters: "Vec<AccountId20>",
    maybeCurrentOrbiter: "Option<PalletMoonbeamOrbitersCurrentOrbiter>",
    nextOrbiter: "u32",
  },
  /**
   * Lookup233:
   * pallet_moonbeam_orbiters::types::CurrentOrbiter[account::AccountId20](account::AccountId20)
   */
  PalletMoonbeamOrbitersCurrentOrbiter: {
    accountId: "AccountId20",
    removed: "bool",
  },
  /**
   * Lookup234: pallet_moonbeam_orbiters::pallet::Call<T>
   */
  PalletMoonbeamOrbitersCall: {
    _enum: {
      collator_add_orbiter: {
        orbiter: "AccountId20",
      },
      collator_remove_orbiter: {
        orbiter: "AccountId20",
      },
      orbiter_leave_collator_pool: {
        collator: "AccountId20",
      },
      orbiter_register: "Null",
      orbiter_unregister: {
        collatorsPoolCount: "u32",
      },
      add_collator: {
        collator: "AccountId20",
      },
      remove_collator: {
        collator: "AccountId20",
      },
    },
  },
  /**
   * Lookup235: pallet_moonbeam_orbiters::pallet::Error<T>
   */
  PalletMoonbeamOrbitersError: {
    _enum: [
      "CollatorAlreadyAdded",
      "CollatorNotFound",
      "CollatorPoolTooLarge",
      "CollatorsPoolCountTooLow",
      "MinOrbiterDepositNotSet",
      "OrbiterAlreadyInPool",
      "OrbiterDepositNotFound",
      "OrbiterNotFound",
      "OrbiterStillInAPool",
    ],
  },
  /**
   * Lookup236: pallet_utility::pallet::Call<T>
   */
  PalletUtilityCall: {
    _enum: {
      batch: {
        calls: "Vec<Call>",
      },
      as_derivative: {
        index: "u16",
        call: "Call",
      },
      batch_all: {
        calls: "Vec<Call>",
      },
      dispatch_as: {
        asOrigin: "MoonbeamRuntimeOriginCaller",
        call: "Call",
      },
    },
  },
  /**
   * Lookup239: pallet_proxy::pallet::Call<T>
   */
  PalletProxyCall: {
    _enum: {
      proxy: {
        real: "AccountId20",
        forceProxyType: "Option<MoonbeamRuntimeProxyType>",
        call: "Call",
      },
      add_proxy: {
        delegate: "AccountId20",
        proxyType: "MoonbeamRuntimeProxyType",
        delay: "u32",
      },
      remove_proxy: {
        delegate: "AccountId20",
        proxyType: "MoonbeamRuntimeProxyType",
        delay: "u32",
      },
      remove_proxies: "Null",
      anonymous: {
        proxyType: "MoonbeamRuntimeProxyType",
        delay: "u32",
        index: "u16",
      },
      kill_anonymous: {
        spawner: "AccountId20",
        proxyType: "MoonbeamRuntimeProxyType",
        index: "u16",
        height: "Compact<u32>",
        extIndex: "Compact<u32>",
      },
      announce: {
        real: "AccountId20",
        callHash: "H256",
      },
      remove_announcement: {
        real: "AccountId20",
        callHash: "H256",
      },
      reject_announcement: {
        delegate: "AccountId20",
        callHash: "H256",
      },
      proxy_announced: {
        delegate: "AccountId20",
        real: "AccountId20",
        forceProxyType: "Option<MoonbeamRuntimeProxyType>",
        call: "Call",
      },
    },
  },
  /**
   * Lookup241: pallet_maintenance_mode::pallet::Call<T>
   */
  PalletMaintenanceModeCall: {
    _enum: ["enter_maintenance_mode", "resume_normal_operation"],
  },
  /**
   * Lookup242: pallet_identity::pallet::Call<T>
   */
  PalletIdentityCall: {
    _enum: {
      add_registrar: {
        account: "AccountId20",
      },
      set_identity: {
        info: "PalletIdentityIdentityInfo",
      },
      set_subs: {
        subs: "Vec<(AccountId20,Data)>",
      },
      clear_identity: "Null",
      request_judgement: {
        regIndex: "Compact<u32>",
        maxFee: "Compact<u128>",
      },
      cancel_request: {
        regIndex: "u32",
      },
      set_fee: {
        index: "Compact<u32>",
        fee: "Compact<u128>",
      },
      set_account_id: {
        _alias: {
          new_: "new",
        },
        index: "Compact<u32>",
        new_: "AccountId20",
      },
      set_fields: {
        index: "Compact<u32>",
        fields: "PalletIdentityBitFlags",
      },
      provide_judgement: {
        regIndex: "Compact<u32>",
        target: "AccountId20",
        judgement: "PalletIdentityJudgement",
      },
      kill_identity: {
        target: "AccountId20",
      },
      add_sub: {
        sub: "AccountId20",
        data: "Data",
      },
      rename_sub: {
        sub: "AccountId20",
        data: "Data",
      },
      remove_sub: {
        sub: "AccountId20",
      },
      quit_sub: "Null",
    },
  },
  /**
   * Lookup243: pallet_identity::types::IdentityInfo<FieldLimit>
   */
  PalletIdentityIdentityInfo: {
    additional: "Vec<(Data,Data)>",
    display: "Data",
    legal: "Data",
    web: "Data",
    riot: "Data",
    email: "Data",
    pgpFingerprint: "Option<[u8;20]>",
    image: "Data",
    twitter: "Data",
  },
  /**
   * Lookup279: pallet_identity::types::BitFlags<pallet_identity::types::IdentityField>
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
   * Lookup280: pallet_identity::types::IdentityField
   */
  PalletIdentityIdentityField: {
    _enum: [
      "__Unused0",
      "Display",
      "Legal",
      "__Unused3",
      "Web",
      "__Unused5",
      "__Unused6",
      "__Unused7",
      "Riot",
      "__Unused9",
      "__Unused10",
      "__Unused11",
      "__Unused12",
      "__Unused13",
      "__Unused14",
      "__Unused15",
      "Email",
      "__Unused17",
      "__Unused18",
      "__Unused19",
      "__Unused20",
      "__Unused21",
      "__Unused22",
      "__Unused23",
      "__Unused24",
      "__Unused25",
      "__Unused26",
      "__Unused27",
      "__Unused28",
      "__Unused29",
      "__Unused30",
      "__Unused31",
      "PgpFingerprint",
      "__Unused33",
      "__Unused34",
      "__Unused35",
      "__Unused36",
      "__Unused37",
      "__Unused38",
      "__Unused39",
      "__Unused40",
      "__Unused41",
      "__Unused42",
      "__Unused43",
      "__Unused44",
      "__Unused45",
      "__Unused46",
      "__Unused47",
      "__Unused48",
      "__Unused49",
      "__Unused50",
      "__Unused51",
      "__Unused52",
      "__Unused53",
      "__Unused54",
      "__Unused55",
      "__Unused56",
      "__Unused57",
      "__Unused58",
      "__Unused59",
      "__Unused60",
      "__Unused61",
      "__Unused62",
      "__Unused63",
      "Image",
      "__Unused65",
      "__Unused66",
      "__Unused67",
      "__Unused68",
      "__Unused69",
      "__Unused70",
      "__Unused71",
      "__Unused72",
      "__Unused73",
      "__Unused74",
      "__Unused75",
      "__Unused76",
      "__Unused77",
      "__Unused78",
      "__Unused79",
      "__Unused80",
      "__Unused81",
      "__Unused82",
      "__Unused83",
      "__Unused84",
      "__Unused85",
      "__Unused86",
      "__Unused87",
      "__Unused88",
      "__Unused89",
      "__Unused90",
      "__Unused91",
      "__Unused92",
      "__Unused93",
      "__Unused94",
      "__Unused95",
      "__Unused96",
      "__Unused97",
      "__Unused98",
      "__Unused99",
      "__Unused100",
      "__Unused101",
      "__Unused102",
      "__Unused103",
      "__Unused104",
      "__Unused105",
      "__Unused106",
      "__Unused107",
      "__Unused108",
      "__Unused109",
      "__Unused110",
      "__Unused111",
      "__Unused112",
      "__Unused113",
      "__Unused114",
      "__Unused115",
      "__Unused116",
      "__Unused117",
      "__Unused118",
      "__Unused119",
      "__Unused120",
      "__Unused121",
      "__Unused122",
      "__Unused123",
      "__Unused124",
      "__Unused125",
      "__Unused126",
      "__Unused127",
      "Twitter",
    ],
  },
  /**
   * Lookup281: pallet_identity::types::Judgement<Balance>
   */
  PalletIdentityJudgement: {
    _enum: {
      Unknown: "Null",
      FeePaid: "u128",
      Reasonable: "Null",
      KnownGood: "Null",
      OutOfDate: "Null",
      LowQuality: "Null",
      Erroneous: "Null",
    },
  },
  /**
   * Lookup282: pallet_evm::pallet::Call<T>
   */
  PalletEvmCall: {
    _enum: {
      withdraw: {
        address: "H160",
        value: "u128",
      },
      call: {
        source: "H160",
        target: "H160",
        input: "Bytes",
        value: "U256",
        gasLimit: "u64",
        maxFeePerGas: "U256",
        maxPriorityFeePerGas: "Option<U256>",
        nonce: "Option<U256>",
        accessList: "Vec<(H160,Vec<H256>)>",
      },
      create: {
        source: "H160",
        init: "Bytes",
        value: "U256",
        gasLimit: "u64",
        maxFeePerGas: "U256",
        maxPriorityFeePerGas: "Option<U256>",
        nonce: "Option<U256>",
        accessList: "Vec<(H160,Vec<H256>)>",
      },
      create2: {
        source: "H160",
        init: "Bytes",
        salt: "H256",
        value: "U256",
        gasLimit: "u64",
        maxFeePerGas: "U256",
        maxPriorityFeePerGas: "Option<U256>",
        nonce: "Option<U256>",
        accessList: "Vec<(H160,Vec<H256>)>",
      },
      hotfix_inc_account_sufficients: {
        addresses: "Vec<H160>",
      },
    },
  },
  /**
   * Lookup287: pallet_ethereum::pallet::Call<T>
   */
  PalletEthereumCall: {
    _enum: {
      transact: {
        transaction: "EthereumTransactionTransactionV2",
      },
    },
  },
  /**
   * Lookup288: ethereum::transaction::TransactionV2
   */
  EthereumTransactionTransactionV2: {
    _enum: {
      Legacy: "EthereumTransactionLegacyTransaction",
      EIP2930: "EthereumTransactionEip2930Transaction",
      EIP1559: "EthereumTransactionEip1559Transaction",
    },
  },
  /**
   * Lookup289: ethereum::transaction::LegacyTransaction
   */
  EthereumTransactionLegacyTransaction: {
    nonce: "U256",
    gasPrice: "U256",
    gasLimit: "U256",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    signature: "EthereumTransactionTransactionSignature",
  },
  /**
   * Lookup290: ethereum::transaction::TransactionAction
   */
  EthereumTransactionTransactionAction: {
    _enum: {
      Call: "H160",
      Create: "Null",
    },
  },
  /**
   * Lookup291: ethereum::transaction::TransactionSignature
   */
  EthereumTransactionTransactionSignature: {
    v: "u64",
    r: "H256",
    s: "H256",
  },
  /**
   * Lookup293: ethereum::transaction::EIP2930Transaction
   */
  EthereumTransactionEip2930Transaction: {
    chainId: "u64",
    nonce: "U256",
    gasPrice: "U256",
    gasLimit: "U256",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    accessList: "Vec<EthereumTransactionAccessListItem>",
    oddYParity: "bool",
    r: "H256",
    s: "H256",
  },
  /**
   * Lookup295: ethereum::transaction::AccessListItem
   */
  EthereumTransactionAccessListItem: {
    address: "H160",
    storageKeys: "Vec<H256>",
  },
  /**
   * Lookup296: ethereum::transaction::EIP1559Transaction
   */
  EthereumTransactionEip1559Transaction: {
    chainId: "u64",
    nonce: "U256",
    maxPriorityFeePerGas: "U256",
    maxFeePerGas: "U256",
    gasLimit: "U256",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    accessList: "Vec<EthereumTransactionAccessListItem>",
    oddYParity: "bool",
    r: "H256",
    s: "H256",
  },
  /**
   * Lookup297: pallet_base_fee::pallet::Call<T>
   */
  PalletBaseFeeCall: {
    _enum: {
      set_base_fee_per_gas: {
        fee: "U256",
      },
      set_is_active: {
        isActive: "bool",
      },
      set_elasticity: {
        elasticity: "Permill",
      },
    },
  },
  /**
   * Lookup298: pallet_scheduler::pallet::Call<T>
   */
  PalletSchedulerCall: {
    _enum: {
      schedule: {
        when: "u32",
        maybePeriodic: "Option<(u32,u32)>",
        priority: "u8",
        call: "FrameSupportScheduleMaybeHashed",
      },
      cancel: {
        when: "u32",
        index: "u32",
      },
      schedule_named: {
        id: "Bytes",
        when: "u32",
        maybePeriodic: "Option<(u32,u32)>",
        priority: "u8",
        call: "FrameSupportScheduleMaybeHashed",
      },
      cancel_named: {
        id: "Bytes",
      },
      schedule_after: {
        after: "u32",
        maybePeriodic: "Option<(u32,u32)>",
        priority: "u8",
        call: "FrameSupportScheduleMaybeHashed",
      },
      schedule_named_after: {
        id: "Bytes",
        after: "u32",
        maybePeriodic: "Option<(u32,u32)>",
        priority: "u8",
        call: "FrameSupportScheduleMaybeHashed",
      },
    },
  },
  /**
   * Lookup300:
   * frame_support::traits::schedule::MaybeHashed<moonbeam_runtime::Call,
   * primitive_types::H256>
   */
  FrameSupportScheduleMaybeHashed: {
    _enum: {
      Value: "Call",
      Hash: "H256",
    },
  },
  /**
   * Lookup301: pallet_democracy::pallet::Call<T>
   */
  PalletDemocracyCall: {
    _enum: {
      propose: {
        proposalHash: "H256",
        value: "Compact<u128>",
      },
      second: {
        proposal: "Compact<u32>",
        secondsUpperBound: "Compact<u32>",
      },
      vote: {
        refIndex: "Compact<u32>",
        vote: "PalletDemocracyVoteAccountVote",
      },
      emergency_cancel: {
        refIndex: "u32",
      },
      external_propose: {
        proposalHash: "H256",
      },
      external_propose_majority: {
        proposalHash: "H256",
      },
      external_propose_default: {
        proposalHash: "H256",
      },
      fast_track: {
        proposalHash: "H256",
        votingPeriod: "u32",
        delay: "u32",
      },
      veto_external: {
        proposalHash: "H256",
      },
      cancel_referendum: {
        refIndex: "Compact<u32>",
      },
      cancel_queued: {
        which: "u32",
      },
      delegate: {
        to: "AccountId20",
        conviction: "PalletDemocracyConviction",
        balance: "u128",
      },
      undelegate: "Null",
      clear_public_proposals: "Null",
      note_preimage: {
        encodedProposal: "Bytes",
      },
      note_preimage_operational: {
        encodedProposal: "Bytes",
      },
      note_imminent_preimage: {
        encodedProposal: "Bytes",
      },
      note_imminent_preimage_operational: {
        encodedProposal: "Bytes",
      },
      reap_preimage: {
        proposalHash: "H256",
        proposalLenUpperBound: "Compact<u32>",
      },
      unlock: {
        target: "AccountId20",
      },
      remove_vote: {
        index: "u32",
      },
      remove_other_vote: {
        target: "AccountId20",
        index: "u32",
      },
      enact_proposal: {
        proposalHash: "H256",
        index: "u32",
      },
      blacklist: {
        proposalHash: "H256",
        maybeRefIndex: "Option<u32>",
      },
      cancel_proposal: {
        propIndex: "Compact<u32>",
      },
    },
  },
  /**
   * Lookup302: pallet_democracy::conviction::Conviction
   */
  PalletDemocracyConviction: {
    _enum: ["None", "Locked1x", "Locked2x", "Locked3x", "Locked4x", "Locked5x", "Locked6x"],
  },
  /**
   * Lookup304: pallet_collective::pallet::Call<T, I>
   */
  PalletCollectiveCall: {
    _enum: {
      set_members: {
        newMembers: "Vec<AccountId20>",
        prime: "Option<AccountId20>",
        oldCount: "u32",
      },
      execute: {
        proposal: "Call",
        lengthBound: "Compact<u32>",
      },
      propose: {
        threshold: "Compact<u32>",
        proposal: "Call",
        lengthBound: "Compact<u32>",
      },
      vote: {
        proposal: "H256",
        index: "Compact<u32>",
        approve: "bool",
      },
      close: {
        proposalHash: "H256",
        index: "Compact<u32>",
        proposalWeightBound: "Compact<u64>",
        lengthBound: "Compact<u32>",
      },
      disapprove_proposal: {
        proposalHash: "H256",
      },
    },
  },
  /**
   * Lookup306: pallet_treasury::pallet::Call<T, I>
   */
  PalletTreasuryCall: {
    _enum: {
      propose_spend: {
        value: "Compact<u128>",
        beneficiary: "AccountId20",
      },
      reject_proposal: {
        proposalId: "Compact<u32>",
      },
      approve_proposal: {
        proposalId: "Compact<u32>",
      },
    },
  },
  /**
   * Lookup307: pallet_crowdloan_rewards::pallet::Call<T>
   */
  PalletCrowdloanRewardsCall: {
    _enum: {
      associate_native_identity: {
        rewardAccount: "AccountId20",
        relayAccount: "[u8;32]",
        proof: "SpRuntimeMultiSignature",
      },
      change_association_with_relay_keys: {
        rewardAccount: "AccountId20",
        previousAccount: "AccountId20",
        proofs: "Vec<([u8;32],SpRuntimeMultiSignature)>",
      },
      claim: "Null",
      update_reward_address: {
        newRewardAccount: "AccountId20",
      },
      complete_initialization: {
        leaseEndingBlock: "u32",
      },
      initialize_reward_vec: {
        rewards: "Vec<([u8;32],Option<AccountId20>,u128)>",
      },
    },
  },
  /**
   * Lookup308: sp_runtime::MultiSignature
   */
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: "SpCoreEd25519Signature",
      Sr25519: "SpCoreSr25519Signature",
      Ecdsa: "SpCoreEcdsaSignature",
    },
  },
  /**
   * Lookup309: sp_core::ed25519::Signature
   */
  SpCoreEd25519Signature: "[u8;64]",
  /**
   * Lookup311: sp_core::sr25519::Signature
   */
  SpCoreSr25519Signature: "[u8;64]",
  /**
   * Lookup312: sp_core::ecdsa::Signature
   */
  SpCoreEcdsaSignature: "[u8;65]",
  /**
   * Lookup318: cumulus_pallet_dmp_queue::pallet::Call<T>
   */
  CumulusPalletDmpQueueCall: {
    _enum: {
      service_overweight: {
        index: "u64",
        weightLimit: "u64",
      },
    },
  },
  /**
   * Lookup319: pallet_xcm::pallet::Call<T>
   */
  PalletXcmCall: {
    _enum: {
      send: {
        dest: "XcmVersionedMultiLocation",
        message: "XcmVersionedXcm",
      },
      teleport_assets: {
        dest: "XcmVersionedMultiLocation",
        beneficiary: "XcmVersionedMultiLocation",
        assets: "XcmVersionedMultiAssets",
        feeAssetItem: "u32",
      },
      reserve_transfer_assets: {
        dest: "XcmVersionedMultiLocation",
        beneficiary: "XcmVersionedMultiLocation",
        assets: "XcmVersionedMultiAssets",
        feeAssetItem: "u32",
      },
      execute: {
        message: "XcmVersionedXcm",
        maxWeight: "u64",
      },
      force_xcm_version: {
        location: "XcmV1MultiLocation",
        xcmVersion: "u32",
      },
      force_default_xcm_version: {
        maybeXcmVersion: "Option<u32>",
      },
      force_subscribe_version_notify: {
        location: "XcmVersionedMultiLocation",
      },
      force_unsubscribe_version_notify: {
        location: "XcmVersionedMultiLocation",
      },
      limited_reserve_transfer_assets: {
        dest: "XcmVersionedMultiLocation",
        beneficiary: "XcmVersionedMultiLocation",
        assets: "XcmVersionedMultiAssets",
        feeAssetItem: "u32",
        weightLimit: "XcmV2WeightLimit",
      },
      limited_teleport_assets: {
        dest: "XcmVersionedMultiLocation",
        beneficiary: "XcmVersionedMultiLocation",
        assets: "XcmVersionedMultiAssets",
        feeAssetItem: "u32",
        weightLimit: "XcmV2WeightLimit",
      },
    },
  },
  /**
   * Lookup320: xcm::VersionedXcm<Call>
   */
  XcmVersionedXcm: {
    _enum: {
      V0: "XcmV0Xcm",
      V1: "XcmV1Xcm",
      V2: "XcmV2Xcm",
    },
  },
  /**
   * Lookup321: xcm::v0::Xcm<Call>
   */
  XcmV0Xcm: {
    _enum: {
      WithdrawAsset: {
        assets: "Vec<XcmV0MultiAsset>",
        effects: "Vec<XcmV0Order>",
      },
      ReserveAssetDeposit: {
        assets: "Vec<XcmV0MultiAsset>",
        effects: "Vec<XcmV0Order>",
      },
      TeleportAsset: {
        assets: "Vec<XcmV0MultiAsset>",
        effects: "Vec<XcmV0Order>",
      },
      QueryResponse: {
        queryId: "Compact<u64>",
        response: "XcmV0Response",
      },
      TransferAsset: {
        assets: "Vec<XcmV0MultiAsset>",
        dest: "XcmV0MultiLocation",
      },
      TransferReserveAsset: {
        assets: "Vec<XcmV0MultiAsset>",
        dest: "XcmV0MultiLocation",
        effects: "Vec<XcmV0Order>",
      },
      Transact: {
        originType: "XcmV0OriginKind",
        requireWeightAtMost: "u64",
        call: "XcmDoubleEncoded",
      },
      HrmpNewChannelOpenRequest: {
        sender: "Compact<u32>",
        maxMessageSize: "Compact<u32>",
        maxCapacity: "Compact<u32>",
      },
      HrmpChannelAccepted: {
        recipient: "Compact<u32>",
      },
      HrmpChannelClosing: {
        initiator: "Compact<u32>",
        sender: "Compact<u32>",
        recipient: "Compact<u32>",
      },
      RelayedFrom: {
        who: "XcmV0MultiLocation",
        message: "XcmV0Xcm",
      },
    },
  },
  /**
   * Lookup323: xcm::v0::order::Order<Call>
   */
  XcmV0Order: {
    _enum: {
      Null: "Null",
      DepositAsset: {
        assets: "Vec<XcmV0MultiAsset>",
        dest: "XcmV0MultiLocation",
      },
      DepositReserveAsset: {
        assets: "Vec<XcmV0MultiAsset>",
        dest: "XcmV0MultiLocation",
        effects: "Vec<XcmV0Order>",
      },
      ExchangeAsset: {
        give: "Vec<XcmV0MultiAsset>",
        receive: "Vec<XcmV0MultiAsset>",
      },
      InitiateReserveWithdraw: {
        assets: "Vec<XcmV0MultiAsset>",
        reserve: "XcmV0MultiLocation",
        effects: "Vec<XcmV0Order>",
      },
      InitiateTeleport: {
        assets: "Vec<XcmV0MultiAsset>",
        dest: "XcmV0MultiLocation",
        effects: "Vec<XcmV0Order>",
      },
      QueryHolding: {
        queryId: "Compact<u64>",
        dest: "XcmV0MultiLocation",
        assets: "Vec<XcmV0MultiAsset>",
      },
      BuyExecution: {
        fees: "XcmV0MultiAsset",
        weight: "u64",
        debt: "u64",
        haltOnError: "bool",
        xcm: "Vec<XcmV0Xcm>",
      },
    },
  },
  /**
   * Lookup325: xcm::v0::Response
   */
  XcmV0Response: {
    _enum: {
      Assets: "Vec<XcmV0MultiAsset>",
    },
  },
  /**
   * Lookup326: xcm::v1::Xcm<Call>
   */
  XcmV1Xcm: {
    _enum: {
      WithdrawAsset: {
        assets: "XcmV1MultiassetMultiAssets",
        effects: "Vec<XcmV1Order>",
      },
      ReserveAssetDeposited: {
        assets: "XcmV1MultiassetMultiAssets",
        effects: "Vec<XcmV1Order>",
      },
      ReceiveTeleportedAsset: {
        assets: "XcmV1MultiassetMultiAssets",
        effects: "Vec<XcmV1Order>",
      },
      QueryResponse: {
        queryId: "Compact<u64>",
        response: "XcmV1Response",
      },
      TransferAsset: {
        assets: "XcmV1MultiassetMultiAssets",
        beneficiary: "XcmV1MultiLocation",
      },
      TransferReserveAsset: {
        assets: "XcmV1MultiassetMultiAssets",
        dest: "XcmV1MultiLocation",
        effects: "Vec<XcmV1Order>",
      },
      Transact: {
        originType: "XcmV0OriginKind",
        requireWeightAtMost: "u64",
        call: "XcmDoubleEncoded",
      },
      HrmpNewChannelOpenRequest: {
        sender: "Compact<u32>",
        maxMessageSize: "Compact<u32>",
        maxCapacity: "Compact<u32>",
      },
      HrmpChannelAccepted: {
        recipient: "Compact<u32>",
      },
      HrmpChannelClosing: {
        initiator: "Compact<u32>",
        sender: "Compact<u32>",
        recipient: "Compact<u32>",
      },
      RelayedFrom: {
        who: "XcmV1MultilocationJunctions",
        message: "XcmV1Xcm",
      },
      SubscribeVersion: {
        queryId: "Compact<u64>",
        maxResponseWeight: "Compact<u64>",
      },
      UnsubscribeVersion: "Null",
    },
  },
  /**
   * Lookup328: xcm::v1::order::Order<Call>
   */
  XcmV1Order: {
    _enum: {
      Noop: "Null",
      DepositAsset: {
        assets: "XcmV1MultiassetMultiAssetFilter",
        maxAssets: "u32",
        beneficiary: "XcmV1MultiLocation",
      },
      DepositReserveAsset: {
        assets: "XcmV1MultiassetMultiAssetFilter",
        maxAssets: "u32",
        dest: "XcmV1MultiLocation",
        effects: "Vec<XcmV1Order>",
      },
      ExchangeAsset: {
        give: "XcmV1MultiassetMultiAssetFilter",
        receive: "XcmV1MultiassetMultiAssets",
      },
      InitiateReserveWithdraw: {
        assets: "XcmV1MultiassetMultiAssetFilter",
        reserve: "XcmV1MultiLocation",
        effects: "Vec<XcmV1Order>",
      },
      InitiateTeleport: {
        assets: "XcmV1MultiassetMultiAssetFilter",
        dest: "XcmV1MultiLocation",
        effects: "Vec<XcmV1Order>",
      },
      QueryHolding: {
        queryId: "Compact<u64>",
        dest: "XcmV1MultiLocation",
        assets: "XcmV1MultiassetMultiAssetFilter",
      },
      BuyExecution: {
        fees: "XcmV1MultiAsset",
        weight: "u64",
        debt: "u64",
        haltOnError: "bool",
        instructions: "Vec<XcmV1Xcm>",
      },
    },
  },
  /**
   * Lookup330: xcm::v1::Response
   */
  XcmV1Response: {
    _enum: {
      Assets: "XcmV1MultiassetMultiAssets",
      Version: "u32",
    },
  },
  /**
   * Lookup344: pallet_assets::pallet::Call<T, I>
   */
  PalletAssetsCall: {
    _enum: {
      create: {
        id: "Compact<u128>",
        admin: "AccountId20",
        minBalance: "u128",
      },
      force_create: {
        id: "Compact<u128>",
        owner: "AccountId20",
        isSufficient: "bool",
        minBalance: "Compact<u128>",
      },
      destroy: {
        id: "Compact<u128>",
        witness: "PalletAssetsDestroyWitness",
      },
      mint: {
        id: "Compact<u128>",
        beneficiary: "AccountId20",
        amount: "Compact<u128>",
      },
      burn: {
        id: "Compact<u128>",
        who: "AccountId20",
        amount: "Compact<u128>",
      },
      transfer: {
        id: "Compact<u128>",
        target: "AccountId20",
        amount: "Compact<u128>",
      },
      transfer_keep_alive: {
        id: "Compact<u128>",
        target: "AccountId20",
        amount: "Compact<u128>",
      },
      force_transfer: {
        id: "Compact<u128>",
        source: "AccountId20",
        dest: "AccountId20",
        amount: "Compact<u128>",
      },
      freeze: {
        id: "Compact<u128>",
        who: "AccountId20",
      },
      thaw: {
        id: "Compact<u128>",
        who: "AccountId20",
      },
      freeze_asset: {
        id: "Compact<u128>",
      },
      thaw_asset: {
        id: "Compact<u128>",
      },
      transfer_ownership: {
        id: "Compact<u128>",
        owner: "AccountId20",
      },
      set_team: {
        id: "Compact<u128>",
        issuer: "AccountId20",
        admin: "AccountId20",
        freezer: "AccountId20",
      },
      set_metadata: {
        id: "Compact<u128>",
        name: "Bytes",
        symbol: "Bytes",
        decimals: "u8",
      },
      clear_metadata: {
        id: "Compact<u128>",
      },
      force_set_metadata: {
        id: "Compact<u128>",
        name: "Bytes",
        symbol: "Bytes",
        decimals: "u8",
        isFrozen: "bool",
      },
      force_clear_metadata: {
        id: "Compact<u128>",
      },
      force_asset_status: {
        id: "Compact<u128>",
        owner: "AccountId20",
        issuer: "AccountId20",
        admin: "AccountId20",
        freezer: "AccountId20",
        minBalance: "Compact<u128>",
        isSufficient: "bool",
        isFrozen: "bool",
      },
      approve_transfer: {
        id: "Compact<u128>",
        delegate: "AccountId20",
        amount: "Compact<u128>",
      },
      cancel_approval: {
        id: "Compact<u128>",
        delegate: "AccountId20",
      },
      force_cancel_approval: {
        id: "Compact<u128>",
        owner: "AccountId20",
        delegate: "AccountId20",
      },
      transfer_approved: {
        id: "Compact<u128>",
        owner: "AccountId20",
        destination: "AccountId20",
        amount: "Compact<u128>",
      },
      touch: {
        id: "Compact<u128>",
      },
      refund: {
        id: "Compact<u128>",
        allowBurn: "bool",
      },
    },
  },
  /**
   * Lookup345: pallet_assets::types::DestroyWitness
   */
  PalletAssetsDestroyWitness: {
    accounts: "Compact<u32>",
    sufficients: "Compact<u32>",
    approvals: "Compact<u32>",
  },
  /**
   * Lookup346: pallet_asset_manager::pallet::Call<T>
   */
  PalletAssetManagerCall: {
    _enum: {
      register_foreign_asset: {
        asset: "MoonbeamRuntimeXcmConfigAssetType",
        metadata: "MoonbeamRuntimeAssetConfigAssetRegistrarMetadata",
        minAmount: "u128",
        isSufficient: "bool",
      },
      set_asset_units_per_second: {
        assetType: "MoonbeamRuntimeXcmConfigAssetType",
        unitsPerSecond: "u128",
        numAssetsWeightHint: "u32",
      },
      change_existing_asset_type: {
        assetId: "u128",
        newAssetType: "MoonbeamRuntimeXcmConfigAssetType",
        numAssetsWeightHint: "u32",
      },
      remove_supported_asset: {
        assetType: "MoonbeamRuntimeXcmConfigAssetType",
        numAssetsWeightHint: "u32",
      },
      remove_existing_asset_type: {
        assetId: "u128",
        numAssetsWeightHint: "u32",
      },
      register_local_asset: {
        creator: "AccountId20",
        owner: "AccountId20",
        isSufficient: "bool",
        minBalance: "u128",
      },
      destroy_foreign_asset: {
        assetId: "u128",
        destroyAssetWitness: "PalletAssetsDestroyWitness",
        numAssetsWeightHint: "u32",
      },
      destroy_local_asset: {
        assetId: "u128",
        destroyAssetWitness: "PalletAssetsDestroyWitness",
      },
    },
  },
  /**
   * Lookup347: orml_xtokens::module::Call<T>
   */
  OrmlXtokensModuleCall: {
    _enum: {
      transfer: {
        currencyId: "MoonbeamRuntimeXcmConfigCurrencyId",
        amount: "u128",
        dest: "XcmVersionedMultiLocation",
        destWeight: "u64",
      },
      transfer_multiasset: {
        asset: "XcmVersionedMultiAsset",
        dest: "XcmVersionedMultiLocation",
        destWeight: "u64",
      },
      transfer_with_fee: {
        currencyId: "MoonbeamRuntimeXcmConfigCurrencyId",
        amount: "u128",
        fee: "u128",
        dest: "XcmVersionedMultiLocation",
        destWeight: "u64",
      },
      transfer_multiasset_with_fee: {
        asset: "XcmVersionedMultiAsset",
        fee: "XcmVersionedMultiAsset",
        dest: "XcmVersionedMultiLocation",
        destWeight: "u64",
      },
      transfer_multicurrencies: {
        currencies: "Vec<(MoonbeamRuntimeXcmConfigCurrencyId,u128)>",
        feeItem: "u32",
        dest: "XcmVersionedMultiLocation",
        destWeight: "u64",
      },
      transfer_multiassets: {
        assets: "XcmVersionedMultiAssets",
        feeItem: "u32",
        dest: "XcmVersionedMultiLocation",
        destWeight: "u64",
      },
    },
  },
  /**
   * Lookup348: moonbeam_runtime::xcm_config::CurrencyId
   */
  MoonbeamRuntimeXcmConfigCurrencyId: {
    _enum: {
      SelfReserve: "Null",
      ForeignAsset: "u128",
      LocalAssetReserve: "u128",
    },
  },
  /**
   * Lookup349: xcm::VersionedMultiAsset
   */
  XcmVersionedMultiAsset: {
    _enum: {
      V0: "XcmV0MultiAsset",
      V1: "XcmV1MultiAsset",
    },
  },
  /**
   * Lookup352: xcm_transactor::pallet::Call<T>
   */
  XcmTransactorCall: {
    _enum: {
      register: {
        who: "AccountId20",
        index: "u16",
      },
      deregister: {
        index: "u16",
      },
      transact_through_derivative_multilocation: {
        dest: "MoonbeamRuntimeXcmConfigTransactors",
        index: "u16",
        feeLocation: "XcmVersionedMultiLocation",
        destWeight: "u64",
        innerCall: "Bytes",
      },
      transact_through_derivative: {
        dest: "MoonbeamRuntimeXcmConfigTransactors",
        index: "u16",
        currencyId: "MoonbeamRuntimeXcmConfigCurrencyId",
        destWeight: "u64",
        innerCall: "Bytes",
      },
      transact_through_sovereign: {
        dest: "XcmVersionedMultiLocation",
        feePayer: "AccountId20",
        feeLocation: "XcmVersionedMultiLocation",
        destWeight: "u64",
        call: "Bytes",
        originKind: "XcmV0OriginKind",
      },
      set_transact_info: {
        location: "XcmVersionedMultiLocation",
        transactExtraWeight: "u64",
        maxWeight: "u64",
        transactExtraWeightSigned: "Option<u64>",
      },
      remove_transact_info: {
        location: "XcmVersionedMultiLocation",
      },
      transact_through_signed: {
        dest: "XcmVersionedMultiLocation",
        feeCurrencyId: "MoonbeamRuntimeXcmConfigCurrencyId",
        destWeight: "u64",
        call: "Bytes",
      },
      transact_through_signed_multilocation: {
        dest: "XcmVersionedMultiLocation",
        feeLocation: "XcmVersionedMultiLocation",
        destWeight: "u64",
        call: "Bytes",
      },
      set_fee_per_second: {
        assetLocation: "XcmVersionedMultiLocation",
        feePerSecond: "u128",
      },
      remove_fee_per_second: {
        assetLocation: "XcmVersionedMultiLocation",
      },
    },
  },
  /**
   * Lookup353: moonbeam_runtime::xcm_config::Transactors
   */
  MoonbeamRuntimeXcmConfigTransactors: {
    _enum: ["Relay"],
  },
  /**
   * Lookup355: moonbeam_runtime::OriginCaller
   */
  MoonbeamRuntimeOriginCaller: {
    _enum: {
      system: "FrameSupportDispatchRawOrigin",
      __Unused1: "Null",
      __Unused2: "Null",
      __Unused3: "Null",
      __Unused4: "Null",
      __Unused5: "Null",
      Void: "SpCoreVoid",
      __Unused7: "Null",
      __Unused8: "Null",
      __Unused9: "Null",
      __Unused10: "Null",
      __Unused11: "Null",
      __Unused12: "Null",
      __Unused13: "Null",
      __Unused14: "Null",
      __Unused15: "Null",
      __Unused16: "Null",
      __Unused17: "Null",
      __Unused18: "Null",
      __Unused19: "Null",
      __Unused20: "Null",
      __Unused21: "Null",
      __Unused22: "Null",
      __Unused23: "Null",
      __Unused24: "Null",
      __Unused25: "Null",
      __Unused26: "Null",
      __Unused27: "Null",
      __Unused28: "Null",
      __Unused29: "Null",
      __Unused30: "Null",
      __Unused31: "Null",
      __Unused32: "Null",
      __Unused33: "Null",
      __Unused34: "Null",
      __Unused35: "Null",
      __Unused36: "Null",
      __Unused37: "Null",
      __Unused38: "Null",
      __Unused39: "Null",
      __Unused40: "Null",
      __Unused41: "Null",
      __Unused42: "Null",
      __Unused43: "Null",
      __Unused44: "Null",
      __Unused45: "Null",
      __Unused46: "Null",
      __Unused47: "Null",
      __Unused48: "Null",
      __Unused49: "Null",
      __Unused50: "Null",
      __Unused51: "Null",
      Ethereum: "PalletEthereumRawOrigin",
      __Unused53: "Null",
      __Unused54: "Null",
      __Unused55: "Null",
      __Unused56: "Null",
      __Unused57: "Null",
      __Unused58: "Null",
      __Unused59: "Null",
      __Unused60: "Null",
      __Unused61: "Null",
      __Unused62: "Null",
      __Unused63: "Null",
      __Unused64: "Null",
      __Unused65: "Null",
      __Unused66: "Null",
      __Unused67: "Null",
      __Unused68: "Null",
      __Unused69: "Null",
      CouncilCollective: "PalletCollectiveRawOrigin",
      TechCommitteeCollective: "PalletCollectiveRawOrigin",
      __Unused72: "Null",
      __Unused73: "Null",
      __Unused74: "Null",
      __Unused75: "Null",
      __Unused76: "Null",
      __Unused77: "Null",
      __Unused78: "Null",
      __Unused79: "Null",
      __Unused80: "Null",
      __Unused81: "Null",
      __Unused82: "Null",
      __Unused83: "Null",
      __Unused84: "Null",
      __Unused85: "Null",
      __Unused86: "Null",
      __Unused87: "Null",
      __Unused88: "Null",
      __Unused89: "Null",
      __Unused90: "Null",
      __Unused91: "Null",
      __Unused92: "Null",
      __Unused93: "Null",
      __Unused94: "Null",
      __Unused95: "Null",
      __Unused96: "Null",
      __Unused97: "Null",
      __Unused98: "Null",
      __Unused99: "Null",
      __Unused100: "Null",
      CumulusXcm: "CumulusPalletXcmOrigin",
      __Unused102: "Null",
      PolkadotXcm: "PalletXcmOrigin",
    },
  },
  /**
   * Lookup356:
   * frame_support::dispatch::RawOrigin[account::AccountId20](account::AccountId20)
   */
  FrameSupportDispatchRawOrigin: {
    _enum: {
      Root: "Null",
      Signed: "AccountId20",
      None: "Null",
    },
  },
  /**
   * Lookup357: pallet_ethereum::RawOrigin
   */
  PalletEthereumRawOrigin: {
    _enum: {
      EthereumTransaction: "H160",
    },
  },
  /**
   * Lookup358: pallet_collective::RawOrigin<account::AccountId20, I>
   */
  PalletCollectiveRawOrigin: {
    _enum: {
      Members: "(u32,u32)",
      Member: "AccountId20",
      _Phantom: "Null",
    },
  },
  /**
   * Lookup360: cumulus_pallet_xcm::pallet::Origin
   */
  CumulusPalletXcmOrigin: {
    _enum: {
      Relay: "Null",
      SiblingParachain: "u32",
    },
  },
  /**
   * Lookup361: pallet_xcm::pallet::Origin
   */
  PalletXcmOrigin: {
    _enum: {
      Xcm: "XcmV1MultiLocation",
      Response: "XcmV1MultiLocation",
    },
  },
  /**
   * Lookup362: sp_core::Void
   */
  SpCoreVoid: "Null",
  /**
   * Lookup363: pallet_utility::pallet::Error<T>
   */
  PalletUtilityError: {
    _enum: ["TooManyCalls"],
  },
  /**
   * Lookup366: pallet_proxy::ProxyDefinition<account::AccountId20,
   * moonbeam_runtime::ProxyType, BlockNumber>
   */
  PalletProxyProxyDefinition: {
    delegate: "AccountId20",
    proxyType: "MoonbeamRuntimeProxyType",
    delay: "u32",
  },
  /**
   * Lookup370: pallet_proxy::Announcement<account::AccountId20,
   * primitive_types::H256, BlockNumber>
   */
  PalletProxyAnnouncement: {
    real: "AccountId20",
    callHash: "H256",
    height: "u32",
  },
  /**
   * Lookup372: pallet_proxy::pallet::Error<T>
   */
  PalletProxyError: {
    _enum: [
      "TooMany",
      "NotFound",
      "NotProxy",
      "Unproxyable",
      "Duplicate",
      "NoPermission",
      "Unannounced",
      "NoSelfProxy",
    ],
  },
  /**
   * Lookup373: pallet_maintenance_mode::pallet::Error<T>
   */
  PalletMaintenanceModeError: {
    _enum: ["AlreadyInMaintenanceMode", "NotInMaintenanceMode"],
  },
  /**
   * Lookup374: pallet_identity::types::Registration<Balance, MaxJudgements,
   * MaxAdditionalFields>
   */
  PalletIdentityRegistration: {
    judgements: "Vec<(u32,PalletIdentityJudgement)>",
    deposit: "u128",
    info: "PalletIdentityIdentityInfo",
  },
  /**
   * Lookup382: pallet_identity::types::RegistrarInfo<Balance, account::AccountId20>
   */
  PalletIdentityRegistrarInfo: {
    account: "AccountId20",
    fee: "u128",
    fields: "PalletIdentityBitFlags",
  },
  /**
   * Lookup384: pallet_identity::pallet::Error<T>
   */
  PalletIdentityError: {
    _enum: [
      "TooManySubAccounts",
      "NotFound",
      "NotNamed",
      "EmptyIndex",
      "FeeChanged",
      "NoIdentity",
      "StickyJudgement",
      "JudgementGiven",
      "InvalidJudgement",
      "InvalidIndex",
      "InvalidTarget",
      "TooManyFields",
      "TooManyRegistrars",
      "AlreadyClaimed",
      "NotSub",
      "NotOwned",
    ],
  },
  /**
   * Lookup386: pallet_evm::pallet::Error<T>
   */
  PalletEvmError: {
    _enum: [
      "BalanceLow",
      "FeeOverflow",
      "PaymentOverflow",
      "WithdrawFailed",
      "GasPriceTooLow",
      "InvalidNonce",
      "MaxAddressCountExceeded",
    ],
  },
  /**
   * Lookup389: fp_rpc::TransactionStatus
   */
  FpRpcTransactionStatus: {
    transactionHash: "H256",
    transactionIndex: "u32",
    from: "H160",
    to: "Option<H160>",
    contractAddress: "Option<H160>",
    logs: "Vec<EthereumLog>",
    logsBloom: "EthbloomBloom",
  },
  /**
   * Lookup392: ethbloom::Bloom
   */
  EthbloomBloom: "[u8;256]",
  /**
   * Lookup394: ethereum::receipt::ReceiptV3
   */
  EthereumReceiptReceiptV3: {
    _enum: {
      Legacy: "EthereumReceiptEip658ReceiptData",
      EIP2930: "EthereumReceiptEip658ReceiptData",
      EIP1559: "EthereumReceiptEip658ReceiptData",
    },
  },
  /**
   * Lookup395: ethereum::receipt::EIP658ReceiptData
   */
  EthereumReceiptEip658ReceiptData: {
    statusCode: "u8",
    usedGas: "U256",
    logsBloom: "EthbloomBloom",
    logs: "Vec<EthereumLog>",
  },
  /**
   * Lookup396:
   * ethereum::block::Block[ethereum::transaction::TransactionV2](ethereum::transaction::TransactionV2)
   */
  EthereumBlock: {
    header: "EthereumHeader",
    transactions: "Vec<EthereumTransactionTransactionV2>",
    ommers: "Vec<EthereumHeader>",
  },
  /**
   * Lookup397: ethereum::header::Header
   */
  EthereumHeader: {
    parentHash: "H256",
    ommersHash: "H256",
    beneficiary: "H160",
    stateRoot: "H256",
    transactionsRoot: "H256",
    receiptsRoot: "H256",
    logsBloom: "EthbloomBloom",
    difficulty: "U256",
    number: "U256",
    gasLimit: "U256",
    gasUsed: "U256",
    timestamp: "u64",
    extraData: "Bytes",
    mixHash: "H256",
    nonce: "EthereumTypesHashH64",
  },
  /**
   * Lookup398: ethereum_types::hash::H64
   */
  EthereumTypesHashH64: "[u8;8]",
  /**
   * Lookup403: pallet_ethereum::pallet::Error<T>
   */
  PalletEthereumError: {
    _enum: ["InvalidSignature", "PreLogExists"],
  },
  /**
   * Lookup406:
   * pallet_scheduler::ScheduledV3<frame_support::traits::schedule::MaybeHashed<moonbeam_runtime::Call,
   * primitive_types::H256>, BlockNumber, moonbeam_runtime::OriginCaller,
   * account::AccountId20>
   */
  PalletSchedulerScheduledV3: {
    maybeId: "Option<Bytes>",
    priority: "u8",
    call: "FrameSupportScheduleMaybeHashed",
    maybePeriodic: "Option<(u32,u32)>",
    origin: "MoonbeamRuntimeOriginCaller",
  },
  /**
   * Lookup407: pallet_scheduler::pallet::Error<T>
   */
  PalletSchedulerError: {
    _enum: ["FailedToSchedule", "NotFound", "TargetBlockNumberInPast", "RescheduleNoChange"],
  },
  /**
   * Lookup411: pallet_democracy::PreimageStatus<account::AccountId20, Balance,
   * BlockNumber>
   */
  PalletDemocracyPreimageStatus: {
    _enum: {
      Missing: "u32",
      Available: {
        data: "Bytes",
        provider: "AccountId20",
        deposit: "u128",
        since: "u32",
        expiry: "Option<u32>",
      },
    },
  },
  /**
   * Lookup412: pallet_democracy::types::ReferendumInfo<BlockNumber,
   * primitive_types::H256, Balance>
   */
  PalletDemocracyReferendumInfo: {
    _enum: {
      Ongoing: "PalletDemocracyReferendumStatus",
      Finished: {
        approved: "bool",
        end: "u32",
      },
    },
  },
  /**
   * Lookup413: pallet_democracy::types::ReferendumStatus<BlockNumber,
   * primitive_types::H256, Balance>
   */
  PalletDemocracyReferendumStatus: {
    end: "u32",
    proposalHash: "H256",
    threshold: "PalletDemocracyVoteThreshold",
    delay: "u32",
    tally: "PalletDemocracyTally",
  },
  /**
   * Lookup414: pallet_democracy::types::Tally<Balance>
   */
  PalletDemocracyTally: {
    ayes: "u128",
    nays: "u128",
    turnout: "u128",
  },
  /**
   * Lookup415: pallet_democracy::vote::Voting<Balance, account::AccountId20, BlockNumber>
   */
  PalletDemocracyVoteVoting: {
    _enum: {
      Direct: {
        votes: "Vec<(u32,PalletDemocracyVoteAccountVote)>",
        delegations: "PalletDemocracyDelegations",
        prior: "PalletDemocracyVotePriorLock",
      },
      Delegating: {
        balance: "u128",
        target: "AccountId20",
        conviction: "PalletDemocracyConviction",
        delegations: "PalletDemocracyDelegations",
        prior: "PalletDemocracyVotePriorLock",
      },
    },
  },
  /**
   * Lookup418: pallet_democracy::types::Delegations<Balance>
   */
  PalletDemocracyDelegations: {
    votes: "u128",
    capital: "u128",
  },
  /**
   * Lookup419: pallet_democracy::vote::PriorLock<BlockNumber, Balance>
   */
  PalletDemocracyVotePriorLock: "(u32,u128)",
  /**
   * Lookup422: pallet_democracy::Releases
   */
  PalletDemocracyReleases: {
    _enum: ["V1"],
  },
  /**
   * Lookup423: pallet_democracy::pallet::Error<T>
   */
  PalletDemocracyError: {
    _enum: [
      "ValueLow",
      "ProposalMissing",
      "AlreadyCanceled",
      "DuplicateProposal",
      "ProposalBlacklisted",
      "NotSimpleMajority",
      "InvalidHash",
      "NoProposal",
      "AlreadyVetoed",
      "DuplicatePreimage",
      "NotImminent",
      "TooEarly",
      "Imminent",
      "PreimageMissing",
      "ReferendumInvalid",
      "PreimageInvalid",
      "NoneWaiting",
      "NotVoter",
      "NoPermission",
      "AlreadyDelegating",
      "InsufficientFunds",
      "NotDelegating",
      "VotesExist",
      "InstantNotAllowed",
      "Nonsense",
      "WrongUpperBound",
      "MaxVotesReached",
      "TooManyProposals",
    ],
  },
  /**
   * Lookup425: pallet_collective::Votes<account::AccountId20, BlockNumber>
   */
  PalletCollectiveVotes: {
    index: "u32",
    threshold: "u32",
    ayes: "Vec<AccountId20>",
    nays: "Vec<AccountId20>",
    end: "u32",
  },
  /**
   * Lookup426: pallet_collective::pallet::Error<T, I>
   */
  PalletCollectiveError: {
    _enum: [
      "NotMember",
      "DuplicateProposal",
      "ProposalMissing",
      "WrongIndex",
      "DuplicateVote",
      "AlreadyInitialized",
      "TooEarly",
      "TooManyProposals",
      "WrongProposalWeight",
      "WrongProposalLength",
    ],
  },
  /**
   * Lookup428: pallet_treasury::Proposal<account::AccountId20, Balance>
   */
  PalletTreasuryProposal: {
    proposer: "AccountId20",
    value: "u128",
    beneficiary: "AccountId20",
    bond: "u128",
  },
  /**
   * Lookup432: frame_support::PalletId
   */
  FrameSupportPalletId: "[u8;8]",
  /**
   * Lookup433: pallet_treasury::pallet::Error<T, I>
   */
  PalletTreasuryError: {
    _enum: ["InsufficientProposersBalance", "InvalidIndex", "TooManyApprovals"],
  },
  /**
   * Lookup434: pallet_crowdloan_rewards::pallet::RewardInfo<T>
   */
  PalletCrowdloanRewardsRewardInfo: {
    totalReward: "u128",
    claimedReward: "u128",
    contributedRelayAddresses: "Vec<[u8;32]>",
  },
  /**
   * Lookup436: pallet_crowdloan_rewards::pallet::Error<T>
   */
  PalletCrowdloanRewardsError: {
    _enum: [
      "AlreadyAssociated",
      "BatchBeyondFundPot",
      "FirstClaimAlreadyDone",
      "RewardNotHighEnough",
      "InvalidClaimSignature",
      "InvalidFreeClaimSignature",
      "NoAssociatedClaim",
      "RewardsAlreadyClaimed",
      "RewardVecAlreadyInitialized",
      "RewardVecNotFullyInitializedYet",
      "RewardsDoNotMatchFund",
      "TooManyContributors",
      "VestingPeriodNonValid",
      "NonContributedAddressProvided",
      "InsufficientNumberOfValidProofs",
    ],
  },
  /**
   * Lookup438: cumulus_pallet_xcmp_queue::InboundChannelDetails
   */
  CumulusPalletXcmpQueueInboundChannelDetails: {
    sender: "u32",
    state: "CumulusPalletXcmpQueueInboundState",
    messageMetadata: "Vec<(u32,PolkadotParachainPrimitivesXcmpMessageFormat)>",
  },
  /**
   * Lookup439: cumulus_pallet_xcmp_queue::InboundState
   */
  CumulusPalletXcmpQueueInboundState: {
    _enum: ["Ok", "Suspended"],
  },
  /**
   * Lookup442: polkadot_parachain::primitives::XcmpMessageFormat
   */
  PolkadotParachainPrimitivesXcmpMessageFormat: {
    _enum: ["ConcatenatedVersionedXcm", "ConcatenatedEncodedBlob", "Signals"],
  },
  /**
   * Lookup445: cumulus_pallet_xcmp_queue::OutboundChannelDetails
   */
  CumulusPalletXcmpQueueOutboundChannelDetails: {
    recipient: "u32",
    state: "CumulusPalletXcmpQueueOutboundState",
    signalsExist: "bool",
    firstIndex: "u16",
    lastIndex: "u16",
  },
  /**
   * Lookup446: cumulus_pallet_xcmp_queue::OutboundState
   */
  CumulusPalletXcmpQueueOutboundState: {
    _enum: ["Ok", "Suspended"],
  },
  /**
   * Lookup448: cumulus_pallet_xcmp_queue::QueueConfigData
   */
  CumulusPalletXcmpQueueQueueConfigData: {
    suspendThreshold: "u32",
    dropThreshold: "u32",
    resumeThreshold: "u32",
    thresholdWeight: "u64",
    weightRestrictDecay: "u64",
    xcmpMaxIndividualWeight: "u64",
  },
  /**
   * Lookup450: cumulus_pallet_xcmp_queue::pallet::Error<T>
   */
  CumulusPalletXcmpQueueError: {
    _enum: ["FailedToSend", "BadXcmOrigin", "BadXcm", "BadOverweightIndex", "WeightOverLimit"],
  },
  /**
   * Lookup451: cumulus_pallet_xcm::pallet::Error<T>
   */
  CumulusPalletXcmError: "Null",
  /**
   * Lookup452: cumulus_pallet_dmp_queue::ConfigData
   */
  CumulusPalletDmpQueueConfigData: {
    maxIndividual: "u64",
  },
  /**
   * Lookup453: cumulus_pallet_dmp_queue::PageIndexData
   */
  CumulusPalletDmpQueuePageIndexData: {
    beginUsed: "u32",
    endUsed: "u32",
    overweightCount: "u64",
  },
  /**
   * Lookup456: cumulus_pallet_dmp_queue::pallet::Error<T>
   */
  CumulusPalletDmpQueueError: {
    _enum: ["Unknown", "OverLimit"],
  },
  /**
   * Lookup457: pallet_xcm::pallet::QueryStatus<BlockNumber>
   */
  PalletXcmQueryStatus: {
    _enum: {
      Pending: {
        responder: "XcmVersionedMultiLocation",
        maybeNotify: "Option<(u8,u8)>",
        timeout: "u32",
      },
      VersionNotifier: {
        origin: "XcmVersionedMultiLocation",
        isActive: "bool",
      },
      Ready: {
        response: "XcmVersionedResponse",
        at: "u32",
      },
    },
  },
  /**
   * Lookup460: xcm::VersionedResponse
   */
  XcmVersionedResponse: {
    _enum: {
      V0: "XcmV0Response",
      V1: "XcmV1Response",
      V2: "XcmV2Response",
    },
  },
  /**
   * Lookup466: pallet_xcm::pallet::VersionMigrationStage
   */
  PalletXcmVersionMigrationStage: {
    _enum: {
      MigrateSupportedVersion: "Null",
      MigrateVersionNotifiers: "Null",
      NotifyCurrentTargets: "Option<Bytes>",
      MigrateAndNotifyOldTargets: "Null",
    },
  },
  /**
   * Lookup467: pallet_xcm::pallet::Error<T>
   */
  PalletXcmError: {
    _enum: [
      "Unreachable",
      "SendFailure",
      "Filtered",
      "UnweighableMessage",
      "DestinationNotInvertible",
      "Empty",
      "CannotReanchor",
      "TooManyAssets",
      "InvalidOrigin",
      "BadVersion",
      "BadLocation",
      "NoSubscription",
      "AlreadySubscribed",
    ],
  },
  /**
   * Lookup468: pallet_assets::types::AssetDetails<Balance,
   * account::AccountId20, DepositBalance>
   */
  PalletAssetsAssetDetails: {
    owner: "AccountId20",
    issuer: "AccountId20",
    admin: "AccountId20",
    freezer: "AccountId20",
    supply: "u128",
    deposit: "u128",
    minBalance: "u128",
    isSufficient: "bool",
    accounts: "u32",
    sufficients: "u32",
    approvals: "u32",
    isFrozen: "bool",
  },
  /**
   * Lookup470: pallet_assets::types::AssetAccount<Balance, DepositBalance, Extra>
   */
  PalletAssetsAssetAccount: {
    balance: "u128",
    isFrozen: "bool",
    reason: "PalletAssetsExistenceReason",
    extra: "Null",
  },
  /**
   * Lookup471: pallet_assets::types::ExistenceReason<Balance>
   */
  PalletAssetsExistenceReason: {
    _enum: {
      Consumer: "Null",
      Sufficient: "Null",
      DepositHeld: "u128",
      DepositRefunded: "Null",
    },
  },
  /**
   * Lookup473: pallet_assets::types::Approval<Balance, DepositBalance>
   */
  PalletAssetsApproval: {
    amount: "u128",
    deposit: "u128",
  },
  /**
   * Lookup474: pallet_assets::types::AssetMetadata<DepositBalance,
   * frame_support::storage::bounded_vec::BoundedVec<T, S>>
   */
  PalletAssetsAssetMetadata: {
    deposit: "u128",
    name: "Bytes",
    symbol: "Bytes",
    decimals: "u8",
    isFrozen: "bool",
  },
  /**
   * Lookup476: pallet_assets::pallet::Error<T, I>
   */
  PalletAssetsError: {
    _enum: [
      "BalanceLow",
      "NoAccount",
      "NoPermission",
      "Unknown",
      "Frozen",
      "InUse",
      "BadWitness",
      "MinBalanceZero",
      "NoProvider",
      "BadMetadata",
      "Unapproved",
      "WouldDie",
      "AlreadyExists",
      "NoDeposit",
      "WouldBurn",
    ],
  },
  /**
   * Lookup477: pallet_asset_manager::pallet::AssetInfo<T>
   */
  PalletAssetManagerAssetInfo: {
    creator: "AccountId20",
    deposit: "u128",
  },
  /**
   * Lookup479: pallet_asset_manager::pallet::Error<T>
   */
  PalletAssetManagerError: {
    _enum: [
      "ErrorCreatingAsset",
      "AssetAlreadyExists",
      "AssetDoesNotExist",
      "TooLowNumAssetsWeightHint",
      "LocalAssetLimitReached",
      "ErrorDestroyingAsset",
      "NotSufficientDeposit",
      "NonExistentLocalAsset",
    ],
  },
  /**
   * Lookup480: orml_xtokens::module::Error<T>
   */
  OrmlXtokensModuleError: {
    _enum: [
      "AssetHasNoReserve",
      "NotCrossChainTransfer",
      "InvalidDest",
      "NotCrossChainTransferableCurrency",
      "UnweighableMessage",
      "XcmExecutionFailed",
      "CannotReanchor",
      "InvalidAncestry",
      "InvalidAsset",
      "DestinationNotInvertible",
      "BadVersion",
      "DistinctReserveForAssetAndFee",
      "ZeroFee",
      "ZeroAmount",
      "TooManyAssetsBeingSent",
      "AssetIndexNonExistent",
      "FeeNotEnough",
      "NotSupportedMultiLocation",
    ],
  },
  /**
   * Lookup481: xcm_transactor::pallet::Error<T>
   */
  XcmTransactorError: {
    _enum: [
      "IndexAlreadyClaimed",
      "UnclaimedIndex",
      "NotOwner",
      "UnweighableMessage",
      "CannotReanchor",
      "AssetHasNoReserve",
      "InvalidDest",
      "NotCrossChainTransfer",
      "AssetIsNotReserveInDestination",
      "DestinationNotInvertible",
      "ErrorSending",
      "DispatchWeightBiggerThanTotalWeight",
      "WeightOverflow",
      "AmountOverflow",
      "TransactorInfoNotSet",
      "NotCrossChainTransferableCurrency",
      "XcmExecuteError",
      "BadVersion",
      "MaxWeightTransactReached",
      "UnableToWithdrawAsset",
      "FeePerSecondNotSet",
      "SignedTransactNotAllowedForDestination",
      "FailedMultiLocationToJunction",
    ],
  },
  /**
   * Lookup484: account::EthereumSignature
   */
  AccountEthereumSignature: "SpCoreEcdsaSignature",
  /**
   * Lookup486: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
   */
  FrameSystemExtensionsCheckSpecVersion: "Null",
  /**
   * Lookup487: frame_system::extensions::check_tx_version::CheckTxVersion<T>
   */
  FrameSystemExtensionsCheckTxVersion: "Null",
  /**
   * Lookup488: frame_system::extensions::check_genesis::CheckGenesis<T>
   */
  FrameSystemExtensionsCheckGenesis: "Null",
  /**
   * Lookup491: frame_system::extensions::check_nonce::CheckNonce<T>
   */
  FrameSystemExtensionsCheckNonce: "Compact<u32>",
  /**
   * Lookup492: frame_system::extensions::check_weight::CheckWeight<T>
   */
  FrameSystemExtensionsCheckWeight: "Null",
  /**
   * Lookup493: pallet_transaction_payment::ChargeTransactionPayment<T>
   */
  PalletTransactionPaymentChargeTransactionPayment: "Compact<u128>",
  /**
   * Lookup495: moonbeam_runtime::Runtime
   */
  MoonbeamRuntimeRuntime: "Null",
};
