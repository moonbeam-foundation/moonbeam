// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

/* eslint-disable sort-keys */

export default {
  /** Lookup3: frame_system::AccountInfo<Index, pallet_balances::AccountData<Balance>> */
  FrameSystemAccountInfo: {
    nonce: "u32",
    consumers: "u32",
    providers: "u32",
    sufficients: "u32",
    data: "PalletBalancesAccountData",
  },
  /** Lookup5: pallet_balances::AccountData<Balance> */
  PalletBalancesAccountData: {
    free: "u128",
    reserved: "u128",
    miscFrozen: "u128",
    feeFrozen: "u128",
  },
  /** Lookup7: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight> */
  FrameSupportDispatchPerDispatchClassWeight: {
    normal: "SpWeightsWeightV2Weight",
    operational: "SpWeightsWeightV2Weight",
    mandatory: "SpWeightsWeightV2Weight",
  },
  /** Lookup8: sp_weights::weight_v2::Weight */
  SpWeightsWeightV2Weight: {
    refTime: "Compact<u64>",
    proofSize: "Compact<u64>",
  },
  /** Lookup14: sp_runtime::generic::digest::Digest */
  SpRuntimeDigest: {
    logs: "Vec<SpRuntimeDigestDigestItem>",
  },
  /** Lookup16: sp_runtime::generic::digest::DigestItem */
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
  /** Lookup19: frame_system::EventRecord<moonbase_runtime::RuntimeEvent, primitive_types::H256> */
  FrameSystemEventRecord: {
    phase: "FrameSystemPhase",
    event: "Event",
    topics: "Vec<H256>",
  },
  /** Lookup21: frame_system::pallet::Event<T> */
  FrameSystemEvent: {
    _enum: {
      ExtrinsicSuccess: {
        dispatchInfo: "FrameSupportDispatchDispatchInfo",
      },
      ExtrinsicFailed: {
        dispatchError: "SpRuntimeDispatchError",
        dispatchInfo: "FrameSupportDispatchDispatchInfo",
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
  /** Lookup22: frame_support::dispatch::DispatchInfo */
  FrameSupportDispatchDispatchInfo: {
    weight: "SpWeightsWeightV2Weight",
    class: "FrameSupportDispatchDispatchClass",
    paysFee: "FrameSupportDispatchPays",
  },
  /** Lookup23: frame_support::dispatch::DispatchClass */
  FrameSupportDispatchDispatchClass: {
    _enum: ["Normal", "Operational", "Mandatory"],
  },
  /** Lookup24: frame_support::dispatch::Pays */
  FrameSupportDispatchPays: {
    _enum: ["Yes", "No"],
  },
  /** Lookup25: sp_runtime::DispatchError */
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
      Arithmetic: "SpArithmeticArithmeticError",
      Transactional: "SpRuntimeTransactionalError",
      Exhausted: "Null",
      Corruption: "Null",
      Unavailable: "Null",
    },
  },
  /** Lookup26: sp_runtime::ModuleError */
  SpRuntimeModuleError: {
    index: "u8",
    error: "[u8;4]",
  },
  /** Lookup27: sp_runtime::TokenError */
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
  /** Lookup28: sp_arithmetic::ArithmeticError */
  SpArithmeticArithmeticError: {
    _enum: ["Underflow", "Overflow", "DivisionByZero"],
  },
  /** Lookup29: sp_runtime::TransactionalError */
  SpRuntimeTransactionalError: {
    _enum: ["LimitReached", "NoLayer"],
  },
  /** Lookup30: pallet_utility::pallet::Event */
  PalletUtilityEvent: {
    _enum: {
      BatchInterrupted: {
        index: "u32",
        error: "SpRuntimeDispatchError",
      },
      BatchCompleted: "Null",
      BatchCompletedWithErrors: "Null",
      ItemCompleted: "Null",
      ItemFailed: {
        error: "SpRuntimeDispatchError",
      },
      DispatchedAs: {
        result: "Result<Null, SpRuntimeDispatchError>",
      },
    },
  },
  /** Lookup33: pallet_balances::pallet::Event<T, I> */
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
  /** Lookup34: frame_support::traits::tokens::misc::BalanceStatus */
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ["Free", "Reserved"],
  },
  /** Lookup35: pallet_sudo::pallet::Event<T> */
  PalletSudoEvent: {
    _enum: {
      Sudid: {
        sudoResult: "Result<Null, SpRuntimeDispatchError>",
      },
      KeyChanged: {
        oldSudoer: "Option<AccountId20>",
      },
      SudoAsDone: {
        sudoResult: "Result<Null, SpRuntimeDispatchError>",
      },
    },
  },
  /** Lookup37: cumulus_pallet_parachain_system::pallet::Event<T> */
  CumulusPalletParachainSystemEvent: {
    _enum: {
      ValidationFunctionStored: "Null",
      ValidationFunctionApplied: {
        relayChainBlockNum: "u32",
      },
      ValidationFunctionDiscarded: "Null",
      UpgradeAuthorized: {
        codeHash: "H256",
      },
      DownwardMessagesReceived: {
        count: "u32",
      },
      DownwardMessagesProcessed: {
        weightUsed: "SpWeightsWeightV2Weight",
        dmqHead: "H256",
      },
      UpwardMessageSent: {
        messageHash: "Option<[u8;32]>",
      },
    },
  },
  /** Lookup39: pallet_transaction_payment::pallet::Event<T> */
  PalletTransactionPaymentEvent: {
    _enum: {
      TransactionFeePaid: {
        who: "AccountId20",
        actualFee: "u128",
        tip: "u128",
      },
    },
  },
  /** Lookup40: pallet_evm::pallet::Event<T> */
  PalletEvmEvent: {
    _enum: {
      Log: {
        log: "EthereumLog",
      },
      Created: {
        address: "H160",
      },
      CreatedFailed: {
        address: "H160",
      },
      Executed: {
        address: "H160",
      },
      ExecutedFailed: {
        address: "H160",
      },
    },
  },
  /** Lookup41: ethereum::log::Log */
  EthereumLog: {
    address: "H160",
    topics: "Vec<H256>",
    data: "Bytes",
  },
  /** Lookup44: pallet_ethereum::pallet::Event */
  PalletEthereumEvent: {
    _enum: {
      Executed: {
        from: "H160",
        to: "H160",
        transactionHash: "H256",
        exitReason: "EvmCoreErrorExitReason",
      },
    },
  },
  /** Lookup45: evm_core::error::ExitReason */
  EvmCoreErrorExitReason: {
    _enum: {
      Succeed: "EvmCoreErrorExitSucceed",
      Error: "EvmCoreErrorExitError",
      Revert: "EvmCoreErrorExitRevert",
      Fatal: "EvmCoreErrorExitFatal",
    },
  },
  /** Lookup46: evm_core::error::ExitSucceed */
  EvmCoreErrorExitSucceed: {
    _enum: ["Stopped", "Returned", "Suicided"],
  },
  /** Lookup47: evm_core::error::ExitError */
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
      __Unused14: "Null",
      InvalidCode: "u8",
    },
  },
  /** Lookup51: evm_core::error::ExitRevert */
  EvmCoreErrorExitRevert: {
    _enum: ["Reverted"],
  },
  /** Lookup52: evm_core::error::ExitFatal */
  EvmCoreErrorExitFatal: {
    _enum: {
      NotSupported: "Null",
      UnhandledInterrupt: "Null",
      CallErrorAsFatal: "EvmCoreErrorExitError",
      Other: "Text",
    },
  },
  /** Lookup53: pallet_parachain_staking::pallet::Event<T> */
  PalletParachainStakingEvent: {
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
        cancelledRequest: "PalletParachainStakingDelegationRequestsCancelledScheduledRequest",
        collator: "AccountId20",
      },
      Delegation: {
        delegator: "AccountId20",
        lockedAmount: "u128",
        candidate: "AccountId20",
        delegatorPosition: "PalletParachainStakingDelegatorAdded",
        autoCompound: "Percent",
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
      AutoCompoundSet: {
        candidate: "AccountId20",
        delegator: "AccountId20",
        value: "Percent",
      },
      Compounded: {
        candidate: "AccountId20",
        delegator: "AccountId20",
        amount: "u128",
      },
    },
  },
  /** Lookup55: pallet_parachain_staking::delegation_requests::CancelledScheduledRequest<Balance> */
  PalletParachainStakingDelegationRequestsCancelledScheduledRequest: {
    whenExecutable: "u32",
    action: "PalletParachainStakingDelegationRequestsDelegationAction",
  },
  /** Lookup56: pallet_parachain_staking::delegation_requests::DelegationAction<Balance> */
  PalletParachainStakingDelegationRequestsDelegationAction: {
    _enum: {
      Revoke: "u128",
      Decrease: "u128",
    },
  },
  /** Lookup57: pallet_parachain_staking::types::DelegatorAdded<B> */
  PalletParachainStakingDelegatorAdded: {
    _enum: {
      AddedToTop: {
        newTotal: "u128",
      },
      AddedToBottom: "Null",
    },
  },
  /** Lookup60: pallet_scheduler::pallet::Event<T> */
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
        id: "Option<[u8;32]>",
        result: "Result<Null, SpRuntimeDispatchError>",
      },
      CallUnavailable: {
        task: "(u32,u32)",
        id: "Option<[u8;32]>",
      },
      PeriodicFailed: {
        task: "(u32,u32)",
        id: "Option<[u8;32]>",
      },
      PermanentlyOverweight: {
        task: "(u32,u32)",
        id: "Option<[u8;32]>",
      },
    },
  },
  /** Lookup62: pallet_democracy::pallet::Event<T> */
  PalletDemocracyEvent: {
    _enum: {
      Proposed: {
        proposalIndex: "u32",
        deposit: "u128",
      },
      Tabled: {
        proposalIndex: "u32",
        deposit: "u128",
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
      ProposalCanceled: {
        propIndex: "u32",
      },
    },
  },
  /** Lookup63: pallet_democracy::vote_threshold::VoteThreshold */
  PalletDemocracyVoteThreshold: {
    _enum: ["SuperMajorityApprove", "SuperMajorityAgainst", "SimpleMajority"],
  },
  /** Lookup64: pallet_democracy::vote::AccountVote<Balance> */
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
  /** Lookup66: pallet_collective::pallet::Event<T, I> */
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
  /** Lookup68: pallet_treasury::pallet::Event<T, I> */
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
      SpendApproved: {
        proposalIndex: "u32",
        amount: "u128",
        beneficiary: "AccountId20",
      },
      UpdatedInactive: {
        reactivated: "u128",
        deactivated: "u128",
      },
    },
  },
  /** Lookup69: pallet_author_slot_filter::pallet::Event */
  PalletAuthorSlotFilterEvent: {
    _enum: {
      EligibleUpdated: "u32",
    },
  },
  /** Lookup71: pallet_crowdloan_rewards::pallet::Event<T> */
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
  /** Lookup72: pallet_author_mapping::pallet::Event<T> */
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
  /** Lookup73: nimbus_primitives::nimbus_crypto::Public */
  NimbusPrimitivesNimbusCryptoPublic: "SpCoreSr25519Public",
  /** Lookup74: sp_core::sr25519::Public */
  SpCoreSr25519Public: "[u8;32]",
  /** Lookup75: session_keys_primitives::vrf::vrf_crypto::Public */
  SessionKeysPrimitivesVrfVrfCryptoPublic: "SpCoreSr25519Public",
  /** Lookup76: pallet_proxy::pallet::Event<T> */
  PalletProxyEvent: {
    _enum: {
      ProxyExecuted: {
        result: "Result<Null, SpRuntimeDispatchError>",
      },
      PureCreated: {
        pure: "AccountId20",
        who: "AccountId20",
        proxyType: "MoonbaseRuntimeProxyType",
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
        proxyType: "MoonbaseRuntimeProxyType",
        delay: "u32",
      },
      ProxyRemoved: {
        delegator: "AccountId20",
        delegatee: "AccountId20",
        proxyType: "MoonbaseRuntimeProxyType",
        delay: "u32",
      },
    },
  },
  /** Lookup77: moonbase_runtime::ProxyType */
  MoonbaseRuntimeProxyType: {
    _enum: [
      "Any",
      "NonTransfer",
      "Governance",
      "Staking",
      "CancelProxy",
      "Balances",
      "AuthorMapping",
      "IdentityJudgement",
    ],
  },
  /** Lookup79: pallet_maintenance_mode::pallet::Event */
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
  /** Lookup80: pallet_identity::pallet::Event<T> */
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
  /** Lookup81: cumulus_pallet_xcmp_queue::pallet::Event<T> */
  CumulusPalletXcmpQueueEvent: {
    _enum: {
      Success: {
        messageHash: "Option<[u8;32]>",
        weight: "SpWeightsWeightV2Weight",
      },
      Fail: {
        messageHash: "Option<[u8;32]>",
        error: "XcmV3TraitsError",
        weight: "SpWeightsWeightV2Weight",
      },
      BadVersion: {
        messageHash: "Option<[u8;32]>",
      },
      BadFormat: {
        messageHash: "Option<[u8;32]>",
      },
      XcmpMessageSent: {
        messageHash: "Option<[u8;32]>",
      },
      OverweightEnqueued: {
        sender: "u32",
        sentAt: "u32",
        index: "u64",
        required: "SpWeightsWeightV2Weight",
      },
      OverweightServiced: {
        index: "u64",
        used: "SpWeightsWeightV2Weight",
      },
    },
  },
  /** Lookup82: xcm::v3::traits::Error */
  XcmV3TraitsError: {
    _enum: {
      Overflow: "Null",
      Unimplemented: "Null",
      UntrustedReserveLocation: "Null",
      UntrustedTeleportLocation: "Null",
      LocationFull: "Null",
      LocationNotInvertible: "Null",
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
      ExpectationFalse: "Null",
      PalletNotFound: "Null",
      NameMismatch: "Null",
      VersionIncompatible: "Null",
      HoldingWouldOverflow: "Null",
      ExportError: "Null",
      ReanchorFailed: "Null",
      NoDeal: "Null",
      FeesNotMet: "Null",
      LockError: "Null",
      NoPermission: "Null",
      Unanchored: "Null",
      NotDepositable: "Null",
      UnhandledXcmVersion: "Null",
      WeightLimitReached: "SpWeightsWeightV2Weight",
      Barrier: "Null",
      WeightNotComputable: "Null",
      ExceedsStackLimit: "Null",
    },
  },
  /** Lookup84: cumulus_pallet_xcm::pallet::Event<T> */
  CumulusPalletXcmEvent: {
    _enum: {
      InvalidFormat: "[u8;32]",
      UnsupportedVersion: "[u8;32]",
      ExecutedDownward: "([u8;32],XcmV3TraitsOutcome)",
    },
  },
  /** Lookup85: xcm::v3::traits::Outcome */
  XcmV3TraitsOutcome: {
    _enum: {
      Complete: "SpWeightsWeightV2Weight",
      Incomplete: "(SpWeightsWeightV2Weight,XcmV3TraitsError)",
      Error: "XcmV3TraitsError",
    },
  },
  /** Lookup86: cumulus_pallet_dmp_queue::pallet::Event<T> */
  CumulusPalletDmpQueueEvent: {
    _enum: {
      InvalidFormat: {
        messageId: "[u8;32]",
      },
      UnsupportedVersion: {
        messageId: "[u8;32]",
      },
      ExecutedDownward: {
        messageId: "[u8;32]",
        outcome: "XcmV3TraitsOutcome",
      },
      WeightExhausted: {
        messageId: "[u8;32]",
        remainingWeight: "SpWeightsWeightV2Weight",
        requiredWeight: "SpWeightsWeightV2Weight",
      },
      OverweightEnqueued: {
        messageId: "[u8;32]",
        overweightIndex: "u64",
        requiredWeight: "SpWeightsWeightV2Weight",
      },
      OverweightServiced: {
        overweightIndex: "u64",
        weightUsed: "SpWeightsWeightV2Weight",
      },
      MaxMessagesExhausted: {
        messageId: "[u8;32]",
      },
    },
  },
  /** Lookup87: pallet_xcm::pallet::Event<T> */
  PalletXcmEvent: {
    _enum: {
      Attempted: "XcmV3TraitsOutcome",
      Sent: "(XcmV3MultiLocation,XcmV3MultiLocation,XcmV3Xcm)",
      UnexpectedResponse: "(XcmV3MultiLocation,u64)",
      ResponseReady: "(u64,XcmV3Response)",
      Notified: "(u64,u8,u8)",
      NotifyOverweight: "(u64,u8,u8,SpWeightsWeightV2Weight,SpWeightsWeightV2Weight)",
      NotifyDispatchError: "(u64,u8,u8)",
      NotifyDecodeFailed: "(u64,u8,u8)",
      InvalidResponder: "(XcmV3MultiLocation,u64,Option<XcmV3MultiLocation>)",
      InvalidResponderVersion: "(XcmV3MultiLocation,u64)",
      ResponseTaken: "u64",
      AssetsTrapped: "(H256,XcmV3MultiLocation,XcmVersionedMultiAssets)",
      VersionChangeNotified: "(XcmV3MultiLocation,u32,XcmV3MultiassetMultiAssets)",
      SupportedVersionChanged: "(XcmV3MultiLocation,u32)",
      NotifyTargetSendFail: "(XcmV3MultiLocation,u64,XcmV3TraitsError)",
      NotifyTargetMigrationFail: "(XcmVersionedMultiLocation,u64)",
      InvalidQuerierVersion: "(XcmV3MultiLocation,u64)",
      InvalidQuerier: "(XcmV3MultiLocation,u64,XcmV3MultiLocation,Option<XcmV3MultiLocation>)",
      VersionNotifyStarted: "(XcmV3MultiLocation,XcmV3MultiassetMultiAssets)",
      VersionNotifyRequested: "(XcmV3MultiLocation,XcmV3MultiassetMultiAssets)",
      VersionNotifyUnrequested: "(XcmV3MultiLocation,XcmV3MultiassetMultiAssets)",
      FeesPaid: "(XcmV3MultiLocation,XcmV3MultiassetMultiAssets)",
      AssetsClaimed: "(H256,XcmV3MultiLocation,XcmVersionedMultiAssets)",
    },
  },
  /** Lookup88: xcm::v3::multilocation::MultiLocation */
  XcmV3MultiLocation: {
    parents: "u8",
    interior: "XcmV3Junctions",
  },
  /** Lookup89: xcm::v3::junctions::Junctions */
  XcmV3Junctions: {
    _enum: {
      Here: "Null",
      X1: "XcmV3Junction",
      X2: "(XcmV3Junction,XcmV3Junction)",
      X3: "(XcmV3Junction,XcmV3Junction,XcmV3Junction)",
      X4: "(XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction)",
      X5: "(XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction)",
      X6: "(XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction)",
      X7: "(XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction)",
      X8: "(XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction,XcmV3Junction)",
    },
  },
  /** Lookup90: xcm::v3::junction::Junction */
  XcmV3Junction: {
    _enum: {
      Parachain: "Compact<u32>",
      AccountId32: {
        network: "Option<XcmV3JunctionNetworkId>",
        id: "[u8;32]",
      },
      AccountIndex64: {
        network: "Option<XcmV3JunctionNetworkId>",
        index: "Compact<u64>",
      },
      AccountKey20: {
        network: "Option<XcmV3JunctionNetworkId>",
        key: "[u8;20]",
      },
      PalletInstance: "u8",
      GeneralIndex: "Compact<u128>",
      GeneralKey: {
        length: "u8",
        data: "[u8;32]",
      },
      OnlyChild: "Null",
      Plurality: {
        id: "XcmV3JunctionBodyId",
        part: "XcmV3JunctionBodyPart",
      },
      GlobalConsensus: "XcmV3JunctionNetworkId",
    },
  },
  /** Lookup93: xcm::v3::junction::NetworkId */
  XcmV3JunctionNetworkId: {
    _enum: {
      ByGenesis: "[u8;32]",
      ByFork: {
        blockNumber: "u64",
        blockHash: "[u8;32]",
      },
      Polkadot: "Null",
      Kusama: "Null",
      Westend: "Null",
      Rococo: "Null",
      Wococo: "Null",
      Ethereum: {
        chainId: "Compact<u64>",
      },
      BitcoinCore: "Null",
      BitcoinCash: "Null",
    },
  },
  /** Lookup95: xcm::v3::junction::BodyId */
  XcmV3JunctionBodyId: {
    _enum: {
      Unit: "Null",
      Moniker: "[u8;4]",
      Index: "Compact<u32>",
      Executive: "Null",
      Technical: "Null",
      Legislative: "Null",
      Judicial: "Null",
      Defense: "Null",
      Administration: "Null",
      Treasury: "Null",
    },
  },
  /** Lookup96: xcm::v3::junction::BodyPart */
  XcmV3JunctionBodyPart: {
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
  /** Lookup97: xcm::v3::Xcm<Call> */
  XcmV3Xcm: "Vec<XcmV3Instruction>",
  /** Lookup99: xcm::v3::Instruction<Call> */
  XcmV3Instruction: {
    _enum: {
      WithdrawAsset: "XcmV3MultiassetMultiAssets",
      ReserveAssetDeposited: "XcmV3MultiassetMultiAssets",
      ReceiveTeleportedAsset: "XcmV3MultiassetMultiAssets",
      QueryResponse: {
        queryId: "Compact<u64>",
        response: "XcmV3Response",
        maxWeight: "SpWeightsWeightV2Weight",
        querier: "Option<XcmV3MultiLocation>",
      },
      TransferAsset: {
        assets: "XcmV3MultiassetMultiAssets",
        beneficiary: "XcmV3MultiLocation",
      },
      TransferReserveAsset: {
        assets: "XcmV3MultiassetMultiAssets",
        dest: "XcmV3MultiLocation",
        xcm: "XcmV3Xcm",
      },
      Transact: {
        originKind: "XcmV2OriginKind",
        requireWeightAtMost: "SpWeightsWeightV2Weight",
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
      DescendOrigin: "XcmV3Junctions",
      ReportError: "XcmV3QueryResponseInfo",
      DepositAsset: {
        assets: "XcmV3MultiassetMultiAssetFilter",
        beneficiary: "XcmV3MultiLocation",
      },
      DepositReserveAsset: {
        assets: "XcmV3MultiassetMultiAssetFilter",
        dest: "XcmV3MultiLocation",
        xcm: "XcmV3Xcm",
      },
      ExchangeAsset: {
        give: "XcmV3MultiassetMultiAssetFilter",
        want: "XcmV3MultiassetMultiAssets",
        maximal: "bool",
      },
      InitiateReserveWithdraw: {
        assets: "XcmV3MultiassetMultiAssetFilter",
        reserve: "XcmV3MultiLocation",
        xcm: "XcmV3Xcm",
      },
      InitiateTeleport: {
        assets: "XcmV3MultiassetMultiAssetFilter",
        dest: "XcmV3MultiLocation",
        xcm: "XcmV3Xcm",
      },
      ReportHolding: {
        responseInfo: "XcmV3QueryResponseInfo",
        assets: "XcmV3MultiassetMultiAssetFilter",
      },
      BuyExecution: {
        fees: "XcmV3MultiAsset",
        weightLimit: "XcmV3WeightLimit",
      },
      RefundSurplus: "Null",
      SetErrorHandler: "XcmV3Xcm",
      SetAppendix: "XcmV3Xcm",
      ClearError: "Null",
      ClaimAsset: {
        assets: "XcmV3MultiassetMultiAssets",
        ticket: "XcmV3MultiLocation",
      },
      Trap: "Compact<u64>",
      SubscribeVersion: {
        queryId: "Compact<u64>",
        maxResponseWeight: "SpWeightsWeightV2Weight",
      },
      UnsubscribeVersion: "Null",
      BurnAsset: "XcmV3MultiassetMultiAssets",
      ExpectAsset: "XcmV3MultiassetMultiAssets",
      ExpectOrigin: "Option<XcmV3MultiLocation>",
      ExpectError: "Option<(u32,XcmV3TraitsError)>",
      ExpectTransactStatus: "XcmV3MaybeErrorCode",
      QueryPallet: {
        moduleName: "Bytes",
        responseInfo: "XcmV3QueryResponseInfo",
      },
      ExpectPallet: {
        index: "Compact<u32>",
        name: "Bytes",
        moduleName: "Bytes",
        crateMajor: "Compact<u32>",
        minCrateMinor: "Compact<u32>",
      },
      ReportTransactStatus: "XcmV3QueryResponseInfo",
      ClearTransactStatus: "Null",
      UniversalOrigin: "XcmV3Junction",
      ExportMessage: {
        network: "XcmV3JunctionNetworkId",
        destination: "XcmV3Junctions",
        xcm: "XcmV3Xcm",
      },
      LockAsset: {
        asset: "XcmV3MultiAsset",
        unlocker: "XcmV3MultiLocation",
      },
      UnlockAsset: {
        asset: "XcmV3MultiAsset",
        target: "XcmV3MultiLocation",
      },
      NoteUnlockable: {
        asset: "XcmV3MultiAsset",
        owner: "XcmV3MultiLocation",
      },
      RequestUnlock: {
        asset: "XcmV3MultiAsset",
        locker: "XcmV3MultiLocation",
      },
      SetFeesMode: {
        jitWithdraw: "bool",
      },
      SetTopic: "[u8;32]",
      ClearTopic: "Null",
      AliasOrigin: "XcmV3MultiLocation",
      UnpaidExecution: {
        weightLimit: "XcmV3WeightLimit",
        checkOrigin: "Option<XcmV3MultiLocation>",
      },
    },
  },
  /** Lookup100: xcm::v3::multiasset::MultiAssets */
  XcmV3MultiassetMultiAssets: "Vec<XcmV3MultiAsset>",
  /** Lookup102: xcm::v3::multiasset::MultiAsset */
  XcmV3MultiAsset: {
    id: "XcmV3MultiassetAssetId",
    fun: "XcmV3MultiassetFungibility",
  },
  /** Lookup103: xcm::v3::multiasset::AssetId */
  XcmV3MultiassetAssetId: {
    _enum: {
      Concrete: "XcmV3MultiLocation",
      Abstract: "[u8;32]",
    },
  },
  /** Lookup104: xcm::v3::multiasset::Fungibility */
  XcmV3MultiassetFungibility: {
    _enum: {
      Fungible: "Compact<u128>",
      NonFungible: "XcmV3MultiassetAssetInstance",
    },
  },
  /** Lookup105: xcm::v3::multiasset::AssetInstance */
  XcmV3MultiassetAssetInstance: {
    _enum: {
      Undefined: "Null",
      Index: "Compact<u128>",
      Array4: "[u8;4]",
      Array8: "[u8;8]",
      Array16: "[u8;16]",
      Array32: "[u8;32]",
    },
  },
  /** Lookup108: xcm::v3::Response */
  XcmV3Response: {
    _enum: {
      Null: "Null",
      Assets: "XcmV3MultiassetMultiAssets",
      ExecutionResult: "Option<(u32,XcmV3TraitsError)>",
      Version: "u32",
      PalletsInfo: "XcmV3VecPalletInfo",
      DispatchResult: "XcmV3MaybeErrorCode",
    },
  },
  /** Lookup111: xcm::v3::VecPalletInfo */
  XcmV3VecPalletInfo: "Vec<XcmV3PalletInfo>",
  /** Lookup113: xcm::v3::PalletInfo */
  XcmV3PalletInfo: {
    index: "Compact<u32>",
    name: "Bytes",
    moduleName: "Bytes",
    major: "Compact<u32>",
    minor: "Compact<u32>",
    patch: "Compact<u32>",
  },
  /** Lookup114: xcm::v3::MaybeErrorCode */
  XcmV3MaybeErrorCode: {
    _enum: {
      Success: "Null",
      Error: "Bytes",
      TruncatedError: "Bytes",
    },
  },
  /** Lookup116: xcm::v2::OriginKind */
  XcmV2OriginKind: {
    _enum: ["Native", "SovereignAccount", "Superuser", "Xcm"],
  },
  /** Lookup117: xcm::double_encoded::DoubleEncoded<T> */
  XcmDoubleEncoded: {
    encoded: "Bytes",
  },
  /** Lookup118: xcm::v3::QueryResponseInfo */
  XcmV3QueryResponseInfo: {
    destination: "XcmV3MultiLocation",
    queryId: "Compact<u64>",
    maxWeight: "SpWeightsWeightV2Weight",
  },
  /** Lookup119: xcm::v3::multiasset::MultiAssetFilter */
  XcmV3MultiassetMultiAssetFilter: {
    _enum: {
      Definite: "XcmV3MultiassetMultiAssets",
      Wild: "XcmV3MultiassetWildMultiAsset",
    },
  },
  /** Lookup120: xcm::v3::multiasset::WildMultiAsset */
  XcmV3MultiassetWildMultiAsset: {
    _enum: {
      All: "Null",
      AllOf: {
        id: "XcmV3MultiassetAssetId",
        fun: "XcmV3MultiassetWildFungibility",
      },
      AllCounted: "Compact<u32>",
      AllOfCounted: {
        id: "XcmV3MultiassetAssetId",
        fun: "XcmV3MultiassetWildFungibility",
        count: "Compact<u32>",
      },
    },
  },
  /** Lookup121: xcm::v3::multiasset::WildFungibility */
  XcmV3MultiassetWildFungibility: {
    _enum: ["Fungible", "NonFungible"],
  },
  /** Lookup122: xcm::v3::WeightLimit */
  XcmV3WeightLimit: {
    _enum: {
      Unlimited: "Null",
      Limited: "SpWeightsWeightV2Weight",
    },
  },
  /** Lookup123: xcm::VersionedMultiAssets */
  XcmVersionedMultiAssets: {
    _enum: {
      __Unused0: "Null",
      V2: "XcmV2MultiassetMultiAssets",
      __Unused2: "Null",
      V3: "XcmV3MultiassetMultiAssets",
    },
  },
  /** Lookup124: xcm::v2::multiasset::MultiAssets */
  XcmV2MultiassetMultiAssets: "Vec<XcmV2MultiAsset>",
  /** Lookup126: xcm::v2::multiasset::MultiAsset */
  XcmV2MultiAsset: {
    id: "XcmV2MultiassetAssetId",
    fun: "XcmV2MultiassetFungibility",
  },
  /** Lookup127: xcm::v2::multiasset::AssetId */
  XcmV2MultiassetAssetId: {
    _enum: {
      Concrete: "XcmV2MultiLocation",
      Abstract: "Bytes",
    },
  },
  /** Lookup128: xcm::v2::multilocation::MultiLocation */
  XcmV2MultiLocation: {
    parents: "u8",
    interior: "XcmV2MultilocationJunctions",
  },
  /** Lookup129: xcm::v2::multilocation::Junctions */
  XcmV2MultilocationJunctions: {
    _enum: {
      Here: "Null",
      X1: "XcmV2Junction",
      X2: "(XcmV2Junction,XcmV2Junction)",
      X3: "(XcmV2Junction,XcmV2Junction,XcmV2Junction)",
      X4: "(XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction)",
      X5: "(XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction)",
      X6: "(XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction)",
      X7: "(XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction)",
      X8: "(XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction,XcmV2Junction)",
    },
  },
  /** Lookup130: xcm::v2::junction::Junction */
  XcmV2Junction: {
    _enum: {
      Parachain: "Compact<u32>",
      AccountId32: {
        network: "XcmV2NetworkId",
        id: "[u8;32]",
      },
      AccountIndex64: {
        network: "XcmV2NetworkId",
        index: "Compact<u64>",
      },
      AccountKey20: {
        network: "XcmV2NetworkId",
        key: "[u8;20]",
      },
      PalletInstance: "u8",
      GeneralIndex: "Compact<u128>",
      GeneralKey: "Bytes",
      OnlyChild: "Null",
      Plurality: {
        id: "XcmV2BodyId",
        part: "XcmV2BodyPart",
      },
    },
  },
  /** Lookup131: xcm::v2::NetworkId */
  XcmV2NetworkId: {
    _enum: {
      Any: "Null",
      Named: "Bytes",
      Polkadot: "Null",
      Kusama: "Null",
    },
  },
  /** Lookup133: xcm::v2::BodyId */
  XcmV2BodyId: {
    _enum: {
      Unit: "Null",
      Named: "Bytes",
      Index: "Compact<u32>",
      Executive: "Null",
      Technical: "Null",
      Legislative: "Null",
      Judicial: "Null",
      Defense: "Null",
      Administration: "Null",
      Treasury: "Null",
    },
  },
  /** Lookup134: xcm::v2::BodyPart */
  XcmV2BodyPart: {
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
  /** Lookup135: xcm::v2::multiasset::Fungibility */
  XcmV2MultiassetFungibility: {
    _enum: {
      Fungible: "Compact<u128>",
      NonFungible: "XcmV2MultiassetAssetInstance",
    },
  },
  /** Lookup136: xcm::v2::multiasset::AssetInstance */
  XcmV2MultiassetAssetInstance: {
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
  /** Lookup137: xcm::VersionedMultiLocation */
  XcmVersionedMultiLocation: {
    _enum: {
      __Unused0: "Null",
      V2: "XcmV2MultiLocation",
      __Unused2: "Null",
      V3: "XcmV3MultiLocation",
    },
  },
  /** Lookup138: pallet_assets::pallet::Event<T, I> */
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
        amount: "u128",
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
      AccountsDestroyed: {
        assetId: "u128",
        accountsDestroyed: "u32",
        accountsRemaining: "u32",
      },
      ApprovalsDestroyed: {
        assetId: "u128",
        approvalsDestroyed: "u32",
        approvalsRemaining: "u32",
      },
      DestructionStarted: {
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
  /** Lookup139: orml_xtokens::module::Event<T> */
  OrmlXtokensModuleEvent: {
    _enum: {
      TransferredMultiAssets: {
        sender: "AccountId20",
        assets: "XcmV3MultiassetMultiAssets",
        fee: "XcmV3MultiAsset",
        dest: "XcmV3MultiLocation",
      },
    },
  },
  /** Lookup140: pallet_asset_manager::pallet::Event<T> */
  PalletAssetManagerEvent: {
    _enum: {
      ForeignAssetRegistered: {
        assetId: "u128",
        asset: "MoonbaseRuntimeXcmConfigAssetType",
        metadata: "MoonbaseRuntimeAssetConfigAssetRegistrarMetadata",
      },
      UnitsPerSecondChanged: {
        assetType: "MoonbaseRuntimeXcmConfigAssetType",
        unitsPerSecond: "u128",
      },
      ForeignAssetTypeChanged: {
        assetId: "u128",
        newAssetType: "MoonbaseRuntimeXcmConfigAssetType",
      },
      ForeignAssetRemoved: {
        assetId: "u128",
        assetType: "MoonbaseRuntimeXcmConfigAssetType",
      },
      SupportedAssetRemoved: {
        assetType: "MoonbaseRuntimeXcmConfigAssetType",
      },
      LocalAssetRegistered: {
        assetId: "u128",
        creator: "AccountId20",
        owner: "AccountId20",
      },
      ForeignAssetDestroyed: {
        assetId: "u128",
        assetType: "MoonbaseRuntimeXcmConfigAssetType",
      },
      LocalAssetDestroyed: {
        assetId: "u128",
      },
    },
  },
  /** Lookup141: moonbase_runtime::xcm_config::AssetType */
  MoonbaseRuntimeXcmConfigAssetType: {
    _enum: {
      Xcm: "XcmV3MultiLocation",
    },
  },
  /** Lookup142: moonbase_runtime::asset_config::AssetRegistrarMetadata */
  MoonbaseRuntimeAssetConfigAssetRegistrarMetadata: {
    name: "Bytes",
    symbol: "Bytes",
    decimals: "u8",
    isFrozen: "bool",
  },
  /** Lookup143: pallet_migrations::pallet::Event<T> */
  PalletMigrationsEvent: {
    _enum: {
      RuntimeUpgradeStarted: "Null",
      RuntimeUpgradeCompleted: {
        weight: "SpWeightsWeightV2Weight",
      },
      MigrationStarted: {
        migrationName: "Bytes",
      },
      MigrationCompleted: {
        migrationName: "Bytes",
        consumedWeight: "SpWeightsWeightV2Weight",
      },
      FailedToSuspendIdleXcmExecution: {
        error: "SpRuntimeDispatchError",
      },
      FailedToResumeIdleXcmExecution: {
        error: "SpRuntimeDispatchError",
      },
    },
  },
  /** Lookup144: pallet_xcm_transactor::pallet::Event<T> */
  PalletXcmTransactorEvent: {
    _enum: {
      TransactedDerivative: {
        accountId: "AccountId20",
        dest: "XcmV3MultiLocation",
        call: "Bytes",
        index: "u16",
      },
      TransactedSovereign: {
        feePayer: "AccountId20",
        dest: "XcmV3MultiLocation",
        call: "Bytes",
      },
      TransactedSigned: {
        feePayer: "AccountId20",
        dest: "XcmV3MultiLocation",
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
        error: "XcmV3TraitsError",
      },
      TransactInfoChanged: {
        location: "XcmV3MultiLocation",
        remoteInfo: "PalletXcmTransactorRemoteTransactInfoWithMaxWeight",
      },
      TransactInfoRemoved: {
        location: "XcmV3MultiLocation",
      },
      DestFeePerSecondChanged: {
        location: "XcmV3MultiLocation",
        feePerSecond: "u128",
      },
      DestFeePerSecondRemoved: {
        location: "XcmV3MultiLocation",
      },
      HrmpManagementSent: {
        action: "PalletXcmTransactorHrmpOperation",
      },
    },
  },
  /** Lookup145: pallet_xcm_transactor::pallet::RemoteTransactInfoWithMaxWeight */
  PalletXcmTransactorRemoteTransactInfoWithMaxWeight: {
    transactExtraWeight: "SpWeightsWeightV2Weight",
    maxWeight: "SpWeightsWeightV2Weight",
    transactExtraWeightSigned: "Option<SpWeightsWeightV2Weight>",
  },
  /** Lookup147: pallet_xcm_transactor::pallet::HrmpOperation */
  PalletXcmTransactorHrmpOperation: {
    _enum: {
      InitOpen: "PalletXcmTransactorHrmpInitParams",
      Accept: {
        paraId: "u32",
      },
      Close: "PolkadotParachainPrimitivesHrmpChannelId",
      Cancel: {
        channelId: "PolkadotParachainPrimitivesHrmpChannelId",
        openRequests: "u32",
      },
    },
  },
  /** Lookup148: pallet_xcm_transactor::pallet::HrmpInitParams */
  PalletXcmTransactorHrmpInitParams: {
    paraId: "u32",
    proposedMaxCapacity: "u32",
    proposedMaxMessageSize: "u32",
  },
  /** Lookup149: polkadot_parachain::primitives::HrmpChannelId */
  PolkadotParachainPrimitivesHrmpChannelId: {
    sender: "u32",
    recipient: "u32",
  },
  /** Lookup151: pallet_moonbeam_orbiters::pallet::Event<T> */
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
  /** Lookup152: pallet_randomness::pallet::Event<T> */
  PalletRandomnessEvent: {
    _enum: {
      RandomnessRequestedBabeEpoch: {
        id: "u64",
        refundAddress: "H160",
        contractAddress: "H160",
        fee: "u128",
        gasLimit: "u64",
        numWords: "u8",
        salt: "H256",
        earliestEpoch: "u64",
      },
      RandomnessRequestedLocal: {
        id: "u64",
        refundAddress: "H160",
        contractAddress: "H160",
        fee: "u128",
        gasLimit: "u64",
        numWords: "u8",
        salt: "H256",
        earliestBlock: "u32",
      },
      RequestFulfilled: {
        id: "u64",
      },
      RequestFeeIncreased: {
        id: "u64",
        newFee: "u128",
      },
      RequestExpirationExecuted: {
        id: "u64",
      },
    },
  },
  /** Lookup154: pallet_conviction_voting::pallet::Event<T, I> */
  PalletConvictionVotingEvent: {
    _enum: {
      Delegated: "(AccountId20,AccountId20)",
      Undelegated: "AccountId20",
    },
  },
  /** Lookup155: pallet_referenda::pallet::Event<T, I> */
  PalletReferendaEvent: {
    _enum: {
      Submitted: {
        index: "u32",
        track: "u16",
        proposal: "FrameSupportPreimagesBounded",
      },
      DecisionDepositPlaced: {
        index: "u32",
        who: "AccountId20",
        amount: "u128",
      },
      DecisionDepositRefunded: {
        index: "u32",
        who: "AccountId20",
        amount: "u128",
      },
      DepositSlashed: {
        who: "AccountId20",
        amount: "u128",
      },
      DecisionStarted: {
        index: "u32",
        track: "u16",
        proposal: "FrameSupportPreimagesBounded",
        tally: "PalletConvictionVotingTally",
      },
      ConfirmStarted: {
        index: "u32",
      },
      ConfirmAborted: {
        index: "u32",
      },
      Confirmed: {
        index: "u32",
        tally: "PalletConvictionVotingTally",
      },
      Approved: {
        index: "u32",
      },
      Rejected: {
        index: "u32",
        tally: "PalletConvictionVotingTally",
      },
      TimedOut: {
        index: "u32",
        tally: "PalletConvictionVotingTally",
      },
      Cancelled: {
        index: "u32",
        tally: "PalletConvictionVotingTally",
      },
      Killed: {
        index: "u32",
        tally: "PalletConvictionVotingTally",
      },
      SubmissionDepositRefunded: {
        index: "u32",
        who: "AccountId20",
        amount: "u128",
      },
    },
  },
  /** Lookup156: frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall> */
  FrameSupportPreimagesBounded: {
    _enum: {
      Legacy: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
      },
      Inline: "Bytes",
      Lookup: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
        len: "u32",
      },
    },
  },
  /** Lookup158: frame_system::pallet::Call<T> */
  FrameSystemCall: {
    _enum: {
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
  /** Lookup162: pallet_utility::pallet::Call<T> */
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
        asOrigin: "MoonbaseRuntimeOriginCaller",
        call: "Call",
      },
      force_batch: {
        calls: "Vec<Call>",
      },
      with_weight: {
        call: "Call",
        weight: "SpWeightsWeightV2Weight",
      },
    },
  },
  /** Lookup164: moonbase_runtime::OriginCaller */
  MoonbaseRuntimeOriginCaller: {
    _enum: {
      system: "FrameSupportDispatchRawOrigin",
      __Unused1: "Null",
      __Unused2: "Null",
      __Unused3: "Null",
      __Unused4: "Null",
      __Unused5: "Null",
      __Unused6: "Null",
      __Unused7: "Null",
      __Unused8: "Null",
      __Unused9: "Null",
      Void: "SpCoreVoid",
      Ethereum: "PalletEthereumRawOrigin",
      __Unused12: "Null",
      __Unused13: "Null",
      __Unused14: "Null",
      CouncilCollective: "PalletCollectiveRawOrigin",
      TechCommitteeCollective: "PalletCollectiveRawOrigin",
      __Unused17: "Null",
      __Unused18: "Null",
      __Unused19: "Null",
      __Unused20: "Null",
      __Unused21: "Null",
      __Unused22: "Null",
      __Unused23: "Null",
      __Unused24: "Null",
      __Unused25: "Null",
      CumulusXcm: "CumulusPalletXcmOrigin",
      __Unused27: "Null",
      PolkadotXcm: "PalletXcmOrigin",
      __Unused29: "Null",
      __Unused30: "Null",
      __Unused31: "Null",
      __Unused32: "Null",
      __Unused33: "Null",
      __Unused34: "Null",
      __Unused35: "Null",
      __Unused36: "Null",
      __Unused37: "Null",
      EthereumXcm: "PalletEthereumXcmRawOrigin",
      __Unused39: "Null",
      TreasuryCouncilCollective: "PalletCollectiveRawOrigin",
      __Unused41: "Null",
      __Unused42: "Null",
      Origins: "MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin",
      __Unused44: "Null",
      __Unused45: "Null",
      OpenTechCommitteeCollective: "PalletCollectiveRawOrigin",
    },
  },
  /** Lookup165: frame_support::dispatch::RawOrigin[account::AccountId20](account::AccountId20) */
  FrameSupportDispatchRawOrigin: {
    _enum: {
      Root: "Null",
      Signed: "AccountId20",
      None: "Null",
    },
  },
  /** Lookup166: pallet_ethereum::RawOrigin */
  PalletEthereumRawOrigin: {
    _enum: {
      EthereumTransaction: "H160",
    },
  },
  /** Lookup167: pallet_collective::RawOrigin<account::AccountId20, I> */
  PalletCollectiveRawOrigin: {
    _enum: {
      Members: "(u32,u32)",
      Member: "AccountId20",
      _Phantom: "Null",
    },
  },
  /** Lookup169: cumulus_pallet_xcm::pallet::Origin */
  CumulusPalletXcmOrigin: {
    _enum: {
      Relay: "Null",
      SiblingParachain: "u32",
    },
  },
  /** Lookup170: pallet_xcm::pallet::Origin */
  PalletXcmOrigin: {
    _enum: {
      Xcm: "XcmV3MultiLocation",
      Response: "XcmV3MultiLocation",
    },
  },
  /** Lookup171: pallet_ethereum_xcm::RawOrigin */
  PalletEthereumXcmRawOrigin: {
    _enum: {
      XcmEthereumTransaction: "H160",
    },
  },
  /** Lookup173: moonbase_runtime::governance::origins::custom_origins::Origin */
  MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin: {
    _enum: ["WhitelistedCaller", "GeneralAdmin", "ReferendumCanceller", "ReferendumKiller"],
  },
  /** Lookup175: sp_core::Void */
  SpCoreVoid: "Null",
  /** Lookup176: pallet_timestamp::pallet::Call<T> */
  PalletTimestampCall: {
    _enum: {
      set: {
        now: "Compact<u64>",
      },
    },
  },
  /** Lookup177: pallet_balances::pallet::Call<T, I> */
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
  /** Lookup178: pallet_sudo::pallet::Call<T> */
  PalletSudoCall: {
    _enum: {
      sudo: {
        call: "Call",
      },
      sudo_unchecked_weight: {
        call: "Call",
        weight: "SpWeightsWeightV2Weight",
      },
      set_key: {
        _alias: {
          new_: "new",
        },
        new_: "AccountId20",
      },
      sudo_as: {
        who: "AccountId20",
        call: "Call",
      },
    },
  },
  /** Lookup179: cumulus_pallet_parachain_system::pallet::Call<T> */
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
  /** Lookup180: cumulus_primitives_parachain_inherent::ParachainInherentData */
  CumulusPrimitivesParachainInherentParachainInherentData: {
    validationData: "PolkadotPrimitivesV2PersistedValidationData",
    relayChainState: "SpTrieStorageProof",
    downwardMessages: "Vec<PolkadotCorePrimitivesInboundDownwardMessage>",
    horizontalMessages: "BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>",
  },
  /** Lookup181: polkadot_primitives::v2::PersistedValidationData<primitive_types::H256, N> */
  PolkadotPrimitivesV2PersistedValidationData: {
    parentHead: "Bytes",
    relayParentNumber: "u32",
    relayParentStorageRoot: "H256",
    maxPovSize: "u32",
  },
  /** Lookup183: sp_trie::storage_proof::StorageProof */
  SpTrieStorageProof: {
    trieNodes: "BTreeSet<Bytes>",
  },
  /** Lookup186: polkadot_core_primitives::InboundDownwardMessage<BlockNumber> */
  PolkadotCorePrimitivesInboundDownwardMessage: {
    sentAt: "u32",
    msg: "Bytes",
  },
  /** Lookup189: polkadot_core_primitives::InboundHrmpMessage<BlockNumber> */
  PolkadotCorePrimitivesInboundHrmpMessage: {
    sentAt: "u32",
    data: "Bytes",
  },
  /** Lookup192: pallet_evm::pallet::Call<T> */
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
    },
  },
  /** Lookup198: pallet_ethereum::pallet::Call<T> */
  PalletEthereumCall: {
    _enum: {
      transact: {
        transaction: "EthereumTransactionTransactionV2",
      },
    },
  },
  /** Lookup199: ethereum::transaction::TransactionV2 */
  EthereumTransactionTransactionV2: {
    _enum: {
      Legacy: "EthereumTransactionLegacyTransaction",
      EIP2930: "EthereumTransactionEip2930Transaction",
      EIP1559: "EthereumTransactionEip1559Transaction",
    },
  },
  /** Lookup200: ethereum::transaction::LegacyTransaction */
  EthereumTransactionLegacyTransaction: {
    nonce: "U256",
    gasPrice: "U256",
    gasLimit: "U256",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    signature: "EthereumTransactionTransactionSignature",
  },
  /** Lookup201: ethereum::transaction::TransactionAction */
  EthereumTransactionTransactionAction: {
    _enum: {
      Call: "H160",
      Create: "Null",
    },
  },
  /** Lookup202: ethereum::transaction::TransactionSignature */
  EthereumTransactionTransactionSignature: {
    v: "u64",
    r: "H256",
    s: "H256",
  },
  /** Lookup204: ethereum::transaction::EIP2930Transaction */
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
  /** Lookup206: ethereum::transaction::AccessListItem */
  EthereumTransactionAccessListItem: {
    address: "H160",
    storageKeys: "Vec<H256>",
  },
  /** Lookup207: ethereum::transaction::EIP1559Transaction */
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
  /** Lookup208: pallet_parachain_staking::pallet::Call<T> */
  PalletParachainStakingCall: {
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
      delegate_with_auto_compound: {
        candidate: "AccountId20",
        amount: "u128",
        autoCompound: "Percent",
        candidateDelegationCount: "u32",
        candidateAutoCompoundingDelegationCount: "u32",
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
      set_auto_compound: {
        candidate: "AccountId20",
        value: "Percent",
        candidateAutoCompoundingDelegationCountHint: "u32",
        delegationCountHint: "u32",
      },
      hotfix_remove_delegation_requests_exited_candidates: {
        candidates: "Vec<AccountId20>",
      },
    },
  },
  /** Lookup212: pallet_scheduler::pallet::Call<T> */
  PalletSchedulerCall: {
    _enum: {
      schedule: {
        when: "u32",
        maybePeriodic: "Option<(u32,u32)>",
        priority: "u8",
        call: "Call",
      },
      cancel: {
        when: "u32",
        index: "u32",
      },
      schedule_named: {
        id: "[u8;32]",
        when: "u32",
        maybePeriodic: "Option<(u32,u32)>",
        priority: "u8",
        call: "Call",
      },
      cancel_named: {
        id: "[u8;32]",
      },
      schedule_after: {
        after: "u32",
        maybePeriodic: "Option<(u32,u32)>",
        priority: "u8",
        call: "Call",
      },
      schedule_named_after: {
        id: "[u8;32]",
        after: "u32",
        maybePeriodic: "Option<(u32,u32)>",
        priority: "u8",
        call: "Call",
      },
    },
  },
  /** Lookup214: pallet_democracy::pallet::Call<T> */
  PalletDemocracyCall: {
    _enum: {
      propose: {
        proposal: "FrameSupportPreimagesBounded",
        value: "Compact<u128>",
      },
      second: {
        proposal: "Compact<u32>",
      },
      vote: {
        refIndex: "Compact<u32>",
        vote: "PalletDemocracyVoteAccountVote",
      },
      emergency_cancel: {
        refIndex: "u32",
      },
      external_propose: {
        proposal: "FrameSupportPreimagesBounded",
      },
      external_propose_majority: {
        proposal: "FrameSupportPreimagesBounded",
      },
      external_propose_default: {
        proposal: "FrameSupportPreimagesBounded",
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
      delegate: {
        to: "AccountId20",
        conviction: "PalletDemocracyConviction",
        balance: "u128",
      },
      undelegate: "Null",
      clear_public_proposals: "Null",
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
      blacklist: {
        proposalHash: "H256",
        maybeRefIndex: "Option<u32>",
      },
      cancel_proposal: {
        propIndex: "Compact<u32>",
      },
    },
  },
  /** Lookup215: pallet_democracy::conviction::Conviction */
  PalletDemocracyConviction: {
    _enum: ["None", "Locked1x", "Locked2x", "Locked3x", "Locked4x", "Locked5x", "Locked6x"],
  },
  /** Lookup217: pallet_collective::pallet::Call<T, I> */
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
      close_old_weight: {
        proposalHash: "H256",
        index: "Compact<u32>",
        proposalWeightBound: "Compact<u64>",
        lengthBound: "Compact<u32>",
      },
      disapprove_proposal: {
        proposalHash: "H256",
      },
      close: {
        proposalHash: "H256",
        index: "Compact<u32>",
        proposalWeightBound: "SpWeightsWeightV2Weight",
        lengthBound: "Compact<u32>",
      },
    },
  },
  /** Lookup221: pallet_treasury::pallet::Call<T, I> */
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
      spend: {
        amount: "Compact<u128>",
        beneficiary: "AccountId20",
      },
      remove_approval: {
        proposalId: "Compact<u32>",
      },
    },
  },
  /** Lookup222: pallet_author_inherent::pallet::Call<T> */
  PalletAuthorInherentCall: {
    _enum: ["kick_off_authorship_validation"],
  },
  /** Lookup223: pallet_author_slot_filter::pallet::Call<T> */
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
  /** Lookup224: pallet_crowdloan_rewards::pallet::Call<T> */
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
  /** Lookup225: sp_runtime::MultiSignature */
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: "SpCoreEd25519Signature",
      Sr25519: "SpCoreSr25519Signature",
      Ecdsa: "SpCoreEcdsaSignature",
    },
  },
  /** Lookup226: sp_core::ed25519::Signature */
  SpCoreEd25519Signature: "[u8;64]",
  /** Lookup228: sp_core::sr25519::Signature */
  SpCoreSr25519Signature: "[u8;64]",
  /** Lookup229: sp_core::ecdsa::Signature */
  SpCoreEcdsaSignature: "[u8;65]",
  /** Lookup235: pallet_author_mapping::pallet::Call<T> */
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
  /** Lookup236: pallet_proxy::pallet::Call<T> */
  PalletProxyCall: {
    _enum: {
      proxy: {
        real: "AccountId20",
        forceProxyType: "Option<MoonbaseRuntimeProxyType>",
        call: "Call",
      },
      add_proxy: {
        delegate: "AccountId20",
        proxyType: "MoonbaseRuntimeProxyType",
        delay: "u32",
      },
      remove_proxy: {
        delegate: "AccountId20",
        proxyType: "MoonbaseRuntimeProxyType",
        delay: "u32",
      },
      remove_proxies: "Null",
      create_pure: {
        proxyType: "MoonbaseRuntimeProxyType",
        delay: "u32",
        index: "u16",
      },
      kill_pure: {
        spawner: "AccountId20",
        proxyType: "MoonbaseRuntimeProxyType",
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
        forceProxyType: "Option<MoonbaseRuntimeProxyType>",
        call: "Call",
      },
    },
  },
  /** Lookup238: pallet_maintenance_mode::pallet::Call<T> */
  PalletMaintenanceModeCall: {
    _enum: ["enter_maintenance_mode", "resume_normal_operation"],
  },
  /** Lookup239: pallet_identity::pallet::Call<T> */
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
        identity: "H256",
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
  /** Lookup240: pallet_identity::types::IdentityInfo<FieldLimit> */
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
  /** Lookup276: pallet_identity::types::BitFlags<pallet_identity::types::IdentityField> */
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
  /** Lookup277: pallet_identity::types::IdentityField */
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
  /** Lookup278: pallet_identity::types::Judgement<Balance> */
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
  /** Lookup279: cumulus_pallet_xcmp_queue::pallet::Call<T> */
  CumulusPalletXcmpQueueCall: {
    _enum: {
      service_overweight: {
        index: "u64",
        weightLimit: "SpWeightsWeightV2Weight",
      },
      suspend_xcm_execution: "Null",
      resume_xcm_execution: "Null",
      update_suspend_threshold: {
        _alias: {
          new_: "new",
        },
        new_: "u32",
      },
      update_drop_threshold: {
        _alias: {
          new_: "new",
        },
        new_: "u32",
      },
      update_resume_threshold: {
        _alias: {
          new_: "new",
        },
        new_: "u32",
      },
      update_threshold_weight: {
        _alias: {
          new_: "new",
        },
        new_: "SpWeightsWeightV2Weight",
      },
      update_weight_restrict_decay: {
        _alias: {
          new_: "new",
        },
        new_: "SpWeightsWeightV2Weight",
      },
      update_xcmp_max_individual_weight: {
        _alias: {
          new_: "new",
        },
        new_: "SpWeightsWeightV2Weight",
      },
    },
  },
  /** Lookup280: cumulus_pallet_dmp_queue::pallet::Call<T> */
  CumulusPalletDmpQueueCall: {
    _enum: {
      service_overweight: {
        index: "u64",
        weightLimit: "SpWeightsWeightV2Weight",
      },
    },
  },
  /** Lookup281: pallet_xcm::pallet::Call<T> */
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
        maxWeight: "SpWeightsWeightV2Weight",
      },
      force_xcm_version: {
        location: "XcmV3MultiLocation",
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
        weightLimit: "XcmV3WeightLimit",
      },
      limited_teleport_assets: {
        dest: "XcmVersionedMultiLocation",
        beneficiary: "XcmVersionedMultiLocation",
        assets: "XcmVersionedMultiAssets",
        feeAssetItem: "u32",
        weightLimit: "XcmV3WeightLimit",
      },
    },
  },
  /** Lookup282: xcm::VersionedXcm<RuntimeCall> */
  XcmVersionedXcm: {
    _enum: {
      __Unused0: "Null",
      __Unused1: "Null",
      V2: "XcmV2Xcm",
      V3: "XcmV3Xcm",
    },
  },
  /** Lookup283: xcm::v2::Xcm<RuntimeCall> */
  XcmV2Xcm: "Vec<XcmV2Instruction>",
  /** Lookup285: xcm::v2::Instruction<RuntimeCall> */
  XcmV2Instruction: {
    _enum: {
      WithdrawAsset: "XcmV2MultiassetMultiAssets",
      ReserveAssetDeposited: "XcmV2MultiassetMultiAssets",
      ReceiveTeleportedAsset: "XcmV2MultiassetMultiAssets",
      QueryResponse: {
        queryId: "Compact<u64>",
        response: "XcmV2Response",
        maxWeight: "Compact<u64>",
      },
      TransferAsset: {
        assets: "XcmV2MultiassetMultiAssets",
        beneficiary: "XcmV2MultiLocation",
      },
      TransferReserveAsset: {
        assets: "XcmV2MultiassetMultiAssets",
        dest: "XcmV2MultiLocation",
        xcm: "XcmV2Xcm",
      },
      Transact: {
        originType: "XcmV2OriginKind",
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
      DescendOrigin: "XcmV2MultilocationJunctions",
      ReportError: {
        queryId: "Compact<u64>",
        dest: "XcmV2MultiLocation",
        maxResponseWeight: "Compact<u64>",
      },
      DepositAsset: {
        assets: "XcmV2MultiassetMultiAssetFilter",
        maxAssets: "Compact<u32>",
        beneficiary: "XcmV2MultiLocation",
      },
      DepositReserveAsset: {
        assets: "XcmV2MultiassetMultiAssetFilter",
        maxAssets: "Compact<u32>",
        dest: "XcmV2MultiLocation",
        xcm: "XcmV2Xcm",
      },
      ExchangeAsset: {
        give: "XcmV2MultiassetMultiAssetFilter",
        receive: "XcmV2MultiassetMultiAssets",
      },
      InitiateReserveWithdraw: {
        assets: "XcmV2MultiassetMultiAssetFilter",
        reserve: "XcmV2MultiLocation",
        xcm: "XcmV2Xcm",
      },
      InitiateTeleport: {
        assets: "XcmV2MultiassetMultiAssetFilter",
        dest: "XcmV2MultiLocation",
        xcm: "XcmV2Xcm",
      },
      QueryHolding: {
        queryId: "Compact<u64>",
        dest: "XcmV2MultiLocation",
        assets: "XcmV2MultiassetMultiAssetFilter",
        maxResponseWeight: "Compact<u64>",
      },
      BuyExecution: {
        fees: "XcmV2MultiAsset",
        weightLimit: "XcmV2WeightLimit",
      },
      RefundSurplus: "Null",
      SetErrorHandler: "XcmV2Xcm",
      SetAppendix: "XcmV2Xcm",
      ClearError: "Null",
      ClaimAsset: {
        assets: "XcmV2MultiassetMultiAssets",
        ticket: "XcmV2MultiLocation",
      },
      Trap: "Compact<u64>",
      SubscribeVersion: {
        queryId: "Compact<u64>",
        maxResponseWeight: "Compact<u64>",
      },
      UnsubscribeVersion: "Null",
    },
  },
  /** Lookup286: xcm::v2::Response */
  XcmV2Response: {
    _enum: {
      Null: "Null",
      Assets: "XcmV2MultiassetMultiAssets",
      ExecutionResult: "Option<(u32,XcmV2TraitsError)>",
      Version: "u32",
    },
  },
  /** Lookup289: xcm::v2::traits::Error */
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
  /** Lookup290: xcm::v2::multiasset::MultiAssetFilter */
  XcmV2MultiassetMultiAssetFilter: {
    _enum: {
      Definite: "XcmV2MultiassetMultiAssets",
      Wild: "XcmV2MultiassetWildMultiAsset",
    },
  },
  /** Lookup291: xcm::v2::multiasset::WildMultiAsset */
  XcmV2MultiassetWildMultiAsset: {
    _enum: {
      All: "Null",
      AllOf: {
        id: "XcmV2MultiassetAssetId",
        fun: "XcmV2MultiassetWildFungibility",
      },
    },
  },
  /** Lookup292: xcm::v2::multiasset::WildFungibility */
  XcmV2MultiassetWildFungibility: {
    _enum: ["Fungible", "NonFungible"],
  },
  /** Lookup293: xcm::v2::WeightLimit */
  XcmV2WeightLimit: {
    _enum: {
      Unlimited: "Null",
      Limited: "Compact<u64>",
    },
  },
  /** Lookup302: pallet_assets::pallet::Call<T, I> */
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
      start_destroy: {
        id: "Compact<u128>",
      },
      destroy_accounts: {
        id: "Compact<u128>",
      },
      destroy_approvals: {
        id: "Compact<u128>",
      },
      finish_destroy: {
        id: "Compact<u128>",
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
  /** Lookup303: orml_xtokens::module::Call<T> */
  OrmlXtokensModuleCall: {
    _enum: {
      transfer: {
        currencyId: "MoonbaseRuntimeXcmConfigCurrencyId",
        amount: "u128",
        dest: "XcmVersionedMultiLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_multiasset: {
        asset: "XcmVersionedMultiAsset",
        dest: "XcmVersionedMultiLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_with_fee: {
        currencyId: "MoonbaseRuntimeXcmConfigCurrencyId",
        amount: "u128",
        fee: "u128",
        dest: "XcmVersionedMultiLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_multiasset_with_fee: {
        asset: "XcmVersionedMultiAsset",
        fee: "XcmVersionedMultiAsset",
        dest: "XcmVersionedMultiLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_multicurrencies: {
        currencies: "Vec<(MoonbaseRuntimeXcmConfigCurrencyId,u128)>",
        feeItem: "u32",
        dest: "XcmVersionedMultiLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_multiassets: {
        assets: "XcmVersionedMultiAssets",
        feeItem: "u32",
        dest: "XcmVersionedMultiLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
    },
  },
  /** Lookup304: moonbase_runtime::xcm_config::CurrencyId */
  MoonbaseRuntimeXcmConfigCurrencyId: {
    _enum: {
      SelfReserve: "Null",
      ForeignAsset: "u128",
      LocalAssetReserve: "u128",
      Erc20: {
        contractAddress: "H160",
      },
    },
  },
  /** Lookup305: xcm::VersionedMultiAsset */
  XcmVersionedMultiAsset: {
    _enum: {
      __Unused0: "Null",
      V2: "XcmV2MultiAsset",
      __Unused2: "Null",
      V3: "XcmV3MultiAsset",
    },
  },
  /** Lookup308: pallet_asset_manager::pallet::Call<T> */
  PalletAssetManagerCall: {
    _enum: {
      register_foreign_asset: {
        asset: "MoonbaseRuntimeXcmConfigAssetType",
        metadata: "MoonbaseRuntimeAssetConfigAssetRegistrarMetadata",
        minAmount: "u128",
        isSufficient: "bool",
      },
      set_asset_units_per_second: {
        assetType: "MoonbaseRuntimeXcmConfigAssetType",
        unitsPerSecond: "u128",
        numAssetsWeightHint: "u32",
      },
      change_existing_asset_type: {
        assetId: "u128",
        newAssetType: "MoonbaseRuntimeXcmConfigAssetType",
        numAssetsWeightHint: "u32",
      },
      remove_supported_asset: {
        assetType: "MoonbaseRuntimeXcmConfigAssetType",
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
        numAssetsWeightHint: "u32",
      },
      destroy_local_asset: {
        assetId: "u128",
      },
    },
  },
  /** Lookup309: pallet_migrations::pallet::Call<T> */
  PalletMigrationsCall: {
    _enum: {
      migrate_democracy_preimage: {
        proposalHash: "H256",
        proposalLenUpperBound: "Compact<u32>",
      },
    },
  },
  /** Lookup310: pallet_xcm_transactor::pallet::Call<T> */
  PalletXcmTransactorCall: {
    _enum: {
      register: {
        who: "AccountId20",
        index: "u16",
      },
      deregister: {
        index: "u16",
      },
      transact_through_derivative: {
        dest: "MoonbaseRuntimeXcmConfigTransactors",
        index: "u16",
        fee: "PalletXcmTransactorCurrencyPayment",
        innerCall: "Bytes",
        weightInfo: "PalletXcmTransactorTransactWeights",
      },
      transact_through_sovereign: {
        dest: "XcmVersionedMultiLocation",
        feePayer: "AccountId20",
        fee: "PalletXcmTransactorCurrencyPayment",
        call: "Bytes",
        originKind: "XcmV2OriginKind",
        weightInfo: "PalletXcmTransactorTransactWeights",
      },
      set_transact_info: {
        location: "XcmVersionedMultiLocation",
        transactExtraWeight: "SpWeightsWeightV2Weight",
        maxWeight: "SpWeightsWeightV2Weight",
        transactExtraWeightSigned: "Option<SpWeightsWeightV2Weight>",
      },
      remove_transact_info: {
        location: "XcmVersionedMultiLocation",
      },
      transact_through_signed: {
        dest: "XcmVersionedMultiLocation",
        fee: "PalletXcmTransactorCurrencyPayment",
        call: "Bytes",
        weightInfo: "PalletXcmTransactorTransactWeights",
      },
      set_fee_per_second: {
        assetLocation: "XcmVersionedMultiLocation",
        feePerSecond: "u128",
      },
      remove_fee_per_second: {
        assetLocation: "XcmVersionedMultiLocation",
      },
      hrmp_manage: {
        action: "PalletXcmTransactorHrmpOperation",
        fee: "PalletXcmTransactorCurrencyPayment",
        weightInfo: "PalletXcmTransactorTransactWeights",
      },
    },
  },
  /** Lookup311: moonbase_runtime::xcm_config::Transactors */
  MoonbaseRuntimeXcmConfigTransactors: {
    _enum: ["Relay"],
  },
  /** Lookup312: pallet_xcm_transactor::pallet::CurrencyPayment<moonbase_runtime::xcm_config::CurrencyId> */
  PalletXcmTransactorCurrencyPayment: {
    currency: "PalletXcmTransactorCurrency",
    feeAmount: "Option<u128>",
  },
  /** Lookup313: pallet_xcm_transactor::pallet::Currency<moonbase_runtime::xcm_config::CurrencyId> */
  PalletXcmTransactorCurrency: {
    _enum: {
      AsCurrencyId: "MoonbaseRuntimeXcmConfigCurrencyId",
      AsMultiLocation: "XcmVersionedMultiLocation",
    },
  },
  /** Lookup315: pallet_xcm_transactor::pallet::TransactWeights */
  PalletXcmTransactorTransactWeights: {
    transactRequiredWeightAtMost: "SpWeightsWeightV2Weight",
    overallWeight: "Option<SpWeightsWeightV2Weight>",
  },
  /** Lookup317: pallet_moonbeam_orbiters::pallet::Call<T> */
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
  /** Lookup318: pallet_ethereum_xcm::pallet::Call<T> */
  PalletEthereumXcmCall: {
    _enum: {
      transact: {
        xcmTransaction: "XcmPrimitivesEthereumXcmEthereumXcmTransaction",
      },
      transact_through_proxy: {
        transactAs: "H160",
        xcmTransaction: "XcmPrimitivesEthereumXcmEthereumXcmTransaction",
      },
      suspend_ethereum_xcm_execution: "Null",
      resume_ethereum_xcm_execution: "Null",
    },
  },
  /** Lookup319: xcm_primitives::ethereum_xcm::EthereumXcmTransaction */
  XcmPrimitivesEthereumXcmEthereumXcmTransaction: {
    _enum: {
      V1: "XcmPrimitivesEthereumXcmEthereumXcmTransactionV1",
      V2: "XcmPrimitivesEthereumXcmEthereumXcmTransactionV2",
    },
  },
  /** Lookup320: xcm_primitives::ethereum_xcm::EthereumXcmTransactionV1 */
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV1: {
    gasLimit: "U256",
    feePayment: "XcmPrimitivesEthereumXcmEthereumXcmFee",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    accessList: "Option<Vec<(H160,Vec<H256>)>>",
  },
  /** Lookup321: xcm_primitives::ethereum_xcm::EthereumXcmFee */
  XcmPrimitivesEthereumXcmEthereumXcmFee: {
    _enum: {
      Manual: "XcmPrimitivesEthereumXcmManualEthereumXcmFee",
      Auto: "Null",
    },
  },
  /** Lookup322: xcm_primitives::ethereum_xcm::ManualEthereumXcmFee */
  XcmPrimitivesEthereumXcmManualEthereumXcmFee: {
    gasPrice: "Option<U256>",
    maxFeePerGas: "Option<U256>",
  },
  /** Lookup325: xcm_primitives::ethereum_xcm::EthereumXcmTransactionV2 */
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV2: {
    gasLimit: "U256",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    accessList: "Option<Vec<(H160,Vec<H256>)>>",
  },
  /** Lookup326: pallet_randomness::pallet::Call<T> */
  PalletRandomnessCall: {
    _enum: ["set_babe_randomness_results"],
  },
  /** Lookup328: pallet_conviction_voting::pallet::Call<T, I> */
  PalletConvictionVotingCall: {
    _enum: {
      vote: {
        pollIndex: "Compact<u32>",
        vote: "PalletConvictionVotingVoteAccountVote",
      },
      delegate: {
        class: "u16",
        to: "AccountId20",
        conviction: "PalletConvictionVotingConviction",
        balance: "u128",
      },
      undelegate: {
        class: "u16",
      },
      unlock: {
        class: "u16",
        target: "AccountId20",
      },
      remove_vote: {
        class: "Option<u16>",
        index: "u32",
      },
      remove_other_vote: {
        target: "AccountId20",
        class: "u16",
        index: "u32",
      },
    },
  },
  /** Lookup329: pallet_conviction_voting::vote::AccountVote<Balance> */
  PalletConvictionVotingVoteAccountVote: {
    _enum: {
      Standard: {
        vote: "Vote",
        balance: "u128",
      },
      Split: {
        aye: "u128",
        nay: "u128",
      },
      SplitAbstain: {
        aye: "u128",
        nay: "u128",
        abstain: "u128",
      },
    },
  },
  /** Lookup331: pallet_conviction_voting::conviction::Conviction */
  PalletConvictionVotingConviction: {
    _enum: ["None", "Locked1x", "Locked2x", "Locked3x", "Locked4x", "Locked5x", "Locked6x"],
  },
  /** Lookup333: pallet_referenda::pallet::Call<T, I> */
  PalletReferendaCall: {
    _enum: {
      submit: {
        proposalOrigin: "MoonbaseRuntimeOriginCaller",
        proposal: "FrameSupportPreimagesBounded",
        enactmentMoment: "FrameSupportScheduleDispatchTime",
      },
      place_decision_deposit: {
        index: "u32",
      },
      refund_decision_deposit: {
        index: "u32",
      },
      cancel: {
        index: "u32",
      },
      kill: {
        index: "u32",
      },
      nudge_referendum: {
        index: "u32",
      },
      one_fewer_deciding: {
        track: "u16",
      },
      refund_submission_deposit: {
        index: "u32",
      },
    },
  },
  /** Lookup334: frame_support::traits::schedule::DispatchTime<BlockNumber> */
  FrameSupportScheduleDispatchTime: {
    _enum: {
      At: "u32",
      After: "u32",
    },
  },
  /** Lookup335: pallet_preimage::pallet::Call<T> */
  PalletPreimageCall: {
    _enum: {
      note_preimage: {
        bytes: "Bytes",
      },
      unnote_preimage: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
      },
      request_preimage: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
      },
      unrequest_preimage: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
      },
    },
  },
  /** Lookup336: pallet_whitelist::pallet::Call<T> */
  PalletWhitelistCall: {
    _enum: {
      whitelist_call: {
        callHash: "H256",
      },
      remove_whitelisted_call: {
        callHash: "H256",
      },
      dispatch_whitelisted_call: {
        callHash: "H256",
        callEncodedLen: "u32",
        callWeightWitness: "SpWeightsWeightV2Weight",
      },
      dispatch_whitelisted_call_with_preimage: {
        call: "Call",
      },
    },
  },
  /** Lookup338: pallet_root_testing::pallet::Call<T> */
  PalletRootTestingCall: {
    _enum: {
      fill_block: {
        ratio: "Perbill",
      },
    },
  },
  /** Lookup340: pallet_conviction_voting::types::Tally<Votes, Total> */
  PalletConvictionVotingTally: {
    ayes: "u128",
    nays: "u128",
    support: "u128",
  },
  /** Lookup341: pallet_preimage::pallet::Event<T> */
  PalletPreimageEvent: {
    _enum: {
      Noted: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
      },
      Requested: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
      },
      Cleared: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
      },
    },
  },
  /** Lookup342: pallet_whitelist::pallet::Event<T> */
  PalletWhitelistEvent: {
    _enum: {
      CallWhitelisted: {
        callHash: "H256",
      },
      WhitelistedCallRemoved: {
        callHash: "H256",
      },
      WhitelistedCallDispatched: {
        callHash: "H256",
        result: "Result<FrameSupportDispatchPostDispatchInfo, SpRuntimeDispatchErrorWithPostInfo>",
      },
    },
  },
  /** Lookup344: frame_support::dispatch::PostDispatchInfo */
  FrameSupportDispatchPostDispatchInfo: {
    actualWeight: "Option<SpWeightsWeightV2Weight>",
    paysFee: "FrameSupportDispatchPays",
  },
  /** Lookup345: sp_runtime::DispatchErrorWithPostInfo<frame_support::dispatch::PostDispatchInfo> */
  SpRuntimeDispatchErrorWithPostInfo: {
    postInfo: "FrameSupportDispatchPostDispatchInfo",
    error: "SpRuntimeDispatchError",
  },
  /** Lookup347: frame_system::Phase */
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: "u32",
      Finalization: "Null",
      Initialization: "Null",
    },
  },
  /** Lookup349: frame_system::LastRuntimeUpgradeInfo */
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: "Compact<u32>",
    specName: "Text",
  },
  /** Lookup350: frame_system::limits::BlockWeights */
  FrameSystemLimitsBlockWeights: {
    baseBlock: "SpWeightsWeightV2Weight",
    maxBlock: "SpWeightsWeightV2Weight",
    perClass: "FrameSupportDispatchPerDispatchClassWeightsPerClass",
  },
  /** Lookup351: frame_support::dispatch::PerDispatchClass<frame_system::limits::WeightsPerClass> */
  FrameSupportDispatchPerDispatchClassWeightsPerClass: {
    normal: "FrameSystemLimitsWeightsPerClass",
    operational: "FrameSystemLimitsWeightsPerClass",
    mandatory: "FrameSystemLimitsWeightsPerClass",
  },
  /** Lookup352: frame_system::limits::WeightsPerClass */
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: "SpWeightsWeightV2Weight",
    maxExtrinsic: "Option<SpWeightsWeightV2Weight>",
    maxTotal: "Option<SpWeightsWeightV2Weight>",
    reserved: "Option<SpWeightsWeightV2Weight>",
  },
  /** Lookup353: frame_system::limits::BlockLength */
  FrameSystemLimitsBlockLength: {
    max: "FrameSupportDispatchPerDispatchClassU32",
  },
  /** Lookup354: frame_support::dispatch::PerDispatchClass<T> */
  FrameSupportDispatchPerDispatchClassU32: {
    normal: "u32",
    operational: "u32",
    mandatory: "u32",
  },
  /** Lookup355: sp_weights::RuntimeDbWeight */
  SpWeightsRuntimeDbWeight: {
    read: "u64",
    write: "u64",
  },
  /** Lookup356: sp_version::RuntimeVersion */
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
  /** Lookup360: frame_system::pallet::Error<T> */
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
  /** Lookup361: pallet_utility::pallet::Error<T> */
  PalletUtilityError: {
    _enum: ["TooManyCalls"],
  },
  /** Lookup363: pallet_balances::BalanceLock<Balance> */
  PalletBalancesBalanceLock: {
    id: "[u8;8]",
    amount: "u128",
    reasons: "PalletBalancesReasons",
  },
  /** Lookup364: pallet_balances::Reasons */
  PalletBalancesReasons: {
    _enum: ["Fee", "Misc", "All"],
  },
  /** Lookup367: pallet_balances::ReserveData<ReserveIdentifier, Balance> */
  PalletBalancesReserveData: {
    id: "[u8;4]",
    amount: "u128",
  },
  /** Lookup369: pallet_balances::pallet::Error<T, I> */
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
  /** Lookup370: pallet_sudo::pallet::Error<T> */
  PalletSudoError: {
    _enum: ["RequireSudo"],
  },
  /** Lookup372: polkadot_primitives::v2::UpgradeRestriction */
  PolkadotPrimitivesV2UpgradeRestriction: {
    _enum: ["Present"],
  },
  /** Lookup373: cumulus_pallet_parachain_system::relay_state_snapshot::MessagingStateSnapshot */
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: {
    dmqMqcHead: "H256",
    relayDispatchQueueSize: "(u32,u32)",
    ingressChannels: "Vec<(u32,PolkadotPrimitivesV2AbridgedHrmpChannel)>",
    egressChannels: "Vec<(u32,PolkadotPrimitivesV2AbridgedHrmpChannel)>",
  },
  /** Lookup376: polkadot_primitives::v2::AbridgedHrmpChannel */
  PolkadotPrimitivesV2AbridgedHrmpChannel: {
    maxCapacity: "u32",
    maxTotalSize: "u32",
    maxMessageSize: "u32",
    msgCount: "u32",
    totalSize: "u32",
    mqcHead: "Option<H256>",
  },
  /** Lookup378: polkadot_primitives::v2::AbridgedHostConfiguration */
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
  /** Lookup384: polkadot_core_primitives::OutboundHrmpMessage<polkadot_parachain::primitives::Id> */
  PolkadotCorePrimitivesOutboundHrmpMessage: {
    recipient: "u32",
    data: "Bytes",
  },
  /** Lookup385: cumulus_pallet_parachain_system::pallet::Error<T> */
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
  /** Lookup387: pallet_transaction_payment::Releases */
  PalletTransactionPaymentReleases: {
    _enum: ["V1Ancient", "V2"],
  },
  /** Lookup389: pallet_evm::pallet::Error<T> */
  PalletEvmError: {
    _enum: [
      "BalanceLow",
      "FeeOverflow",
      "PaymentOverflow",
      "WithdrawFailed",
      "GasPriceTooLow",
      "InvalidNonce",
      "GasLimitTooLow",
      "GasLimitTooHigh",
      "Undefined",
      "Reentrancy",
      "TransactionMustComeFromEOA",
    ],
  },
  /** Lookup392: fp_rpc::TransactionStatus */
  FpRpcTransactionStatus: {
    transactionHash: "H256",
    transactionIndex: "u32",
    from: "H160",
    to: "Option<H160>",
    contractAddress: "Option<H160>",
    logs: "Vec<EthereumLog>",
    logsBloom: "EthbloomBloom",
  },
  /** Lookup395: ethbloom::Bloom */
  EthbloomBloom: "[u8;256]",
  /** Lookup397: ethereum::receipt::ReceiptV3 */
  EthereumReceiptReceiptV3: {
    _enum: {
      Legacy: "EthereumReceiptEip658ReceiptData",
      EIP2930: "EthereumReceiptEip658ReceiptData",
      EIP1559: "EthereumReceiptEip658ReceiptData",
    },
  },
  /** Lookup398: ethereum::receipt::EIP658ReceiptData */
  EthereumReceiptEip658ReceiptData: {
    statusCode: "u8",
    usedGas: "U256",
    logsBloom: "EthbloomBloom",
    logs: "Vec<EthereumLog>",
  },
  /**
   * Lookup399:
   * ethereum::block::Block[ethereum::transaction::TransactionV2](ethereum::transaction::TransactionV2)
   */
  EthereumBlock: {
    header: "EthereumHeader",
    transactions: "Vec<EthereumTransactionTransactionV2>",
    ommers: "Vec<EthereumHeader>",
  },
  /** Lookup400: ethereum::header::Header */
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
  /** Lookup401: ethereum_types::hash::H64 */
  EthereumTypesHashH64: "[u8;8]",
  /** Lookup406: pallet_ethereum::pallet::Error<T> */
  PalletEthereumError: {
    _enum: ["InvalidSignature", "PreLogExists"],
  },
  /**
   * Lookup407:
   * pallet_parachain_staking::types::ParachainBondConfig[account::AccountId20](account::AccountId20)
   */
  PalletParachainStakingParachainBondConfig: {
    account: "AccountId20",
    percent: "Percent",
  },
  /** Lookup408: pallet_parachain_staking::types::RoundInfo<BlockNumber> */
  PalletParachainStakingRoundInfo: {
    current: "u32",
    first: "u32",
    length: "u32",
  },
  /** Lookup409: pallet_parachain_staking::types::Delegator<account::AccountId20, Balance> */
  PalletParachainStakingDelegator: {
    id: "AccountId20",
    delegations: "PalletParachainStakingSetOrderedSet",
    total: "u128",
    lessTotal: "u128",
    status: "PalletParachainStakingDelegatorStatus",
  },
  /**
   * Lookup410:
   * pallet_parachain_staking::set::OrderedSet<pallet_parachain_staking::types::Bond<account::AccountId20,
   * Balance>>
   */
  PalletParachainStakingSetOrderedSet: "Vec<PalletParachainStakingBond>",
  /** Lookup411: pallet_parachain_staking::types::Bond<account::AccountId20, Balance> */
  PalletParachainStakingBond: {
    owner: "AccountId20",
    amount: "u128",
  },
  /** Lookup413: pallet_parachain_staking::types::DelegatorStatus */
  PalletParachainStakingDelegatorStatus: {
    _enum: {
      Active: "Null",
      Leaving: "u32",
    },
  },
  /** Lookup414: pallet_parachain_staking::types::CandidateMetadata<Balance> */
  PalletParachainStakingCandidateMetadata: {
    bond: "u128",
    delegationCount: "u32",
    totalCounted: "u128",
    lowestTopDelegationAmount: "u128",
    highestBottomDelegationAmount: "u128",
    lowestBottomDelegationAmount: "u128",
    topCapacity: "PalletParachainStakingCapacityStatus",
    bottomCapacity: "PalletParachainStakingCapacityStatus",
    request: "Option<PalletParachainStakingCandidateBondLessRequest>",
    status: "PalletParachainStakingCollatorStatus",
  },
  /** Lookup415: pallet_parachain_staking::types::CapacityStatus */
  PalletParachainStakingCapacityStatus: {
    _enum: ["Full", "Empty", "Partial"],
  },
  /** Lookup417: pallet_parachain_staking::types::CandidateBondLessRequest<Balance> */
  PalletParachainStakingCandidateBondLessRequest: {
    amount: "u128",
    whenExecutable: "u32",
  },
  /** Lookup418: pallet_parachain_staking::types::CollatorStatus */
  PalletParachainStakingCollatorStatus: {
    _enum: {
      Active: "Null",
      Idle: "Null",
      Leaving: "u32",
    },
  },
  /** Lookup420: pallet_parachain_staking::delegation_requests::ScheduledRequest<account::AccountId20, Balance> */
  PalletParachainStakingDelegationRequestsScheduledRequest: {
    delegator: "AccountId20",
    whenExecutable: "u32",
    action: "PalletParachainStakingDelegationRequestsDelegationAction",
  },
  /**
   * Lookup422:
   * pallet_parachain_staking::auto_compound::AutoCompoundConfig[account::AccountId20](account::AccountId20)
   */
  PalletParachainStakingAutoCompoundAutoCompoundConfig: {
    delegator: "AccountId20",
    value: "Percent",
  },
  /** Lookup423: pallet_parachain_staking::types::Delegations<account::AccountId20, Balance> */
  PalletParachainStakingDelegations: {
    delegations: "Vec<PalletParachainStakingBond>",
    total: "u128",
  },
  /** Lookup425: pallet_parachain_staking::types::CollatorSnapshot<account::AccountId20, Balance> */
  PalletParachainStakingCollatorSnapshot: {
    bond: "u128",
    delegations: "Vec<PalletParachainStakingBondWithAutoCompound>",
    total: "u128",
  },
  /** Lookup427: pallet_parachain_staking::types::BondWithAutoCompound<account::AccountId20, Balance> */
  PalletParachainStakingBondWithAutoCompound: {
    owner: "AccountId20",
    amount: "u128",
    autoCompound: "Percent",
  },
  /** Lookup428: pallet_parachain_staking::types::DelayedPayout<Balance> */
  PalletParachainStakingDelayedPayout: {
    roundIssuance: "u128",
    totalStakingReward: "u128",
    collatorCommission: "Perbill",
  },
  /** Lookup429: pallet_parachain_staking::inflation::InflationInfo<Balance> */
  PalletParachainStakingInflationInflationInfo: {
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
  /** Lookup430: pallet_parachain_staking::pallet::Error<T> */
  PalletParachainStakingError: {
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
      "RoundLengthMustBeGreaterThanTotalSelectedCollators",
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
      "TooLowDelegationCountToAutoCompound",
      "TooLowCandidateAutoCompoundingDelegationCountToAutoCompound",
      "TooLowCandidateAutoCompoundingDelegationCountToDelegate",
    ],
  },
  /**
   * Lookup433: pallet_scheduler::Scheduled<Name,
   * frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall>, BlockNumber,
   * moonbase_runtime::OriginCaller, account::AccountId20>
   */
  PalletSchedulerScheduled: {
    maybeId: "Option<[u8;32]>",
    priority: "u8",
    call: "FrameSupportPreimagesBounded",
    maybePeriodic: "Option<(u32,u32)>",
    origin: "MoonbaseRuntimeOriginCaller",
  },
  /** Lookup435: pallet_scheduler::pallet::Error<T> */
  PalletSchedulerError: {
    _enum: [
      "FailedToSchedule",
      "NotFound",
      "TargetBlockNumberInPast",
      "RescheduleNoChange",
      "Named",
    ],
  },
  /**
   * Lookup441: pallet_democracy::types::ReferendumInfo<BlockNumber,
   * frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall>, Balance>
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
   * Lookup442: pallet_democracy::types::ReferendumStatus<BlockNumber,
   * frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall>, Balance>
   */
  PalletDemocracyReferendumStatus: {
    end: "u32",
    proposal: "FrameSupportPreimagesBounded",
    threshold: "PalletDemocracyVoteThreshold",
    delay: "u32",
    tally: "PalletDemocracyTally",
  },
  /** Lookup443: pallet_democracy::types::Tally<Balance> */
  PalletDemocracyTally: {
    ayes: "u128",
    nays: "u128",
    turnout: "u128",
  },
  /** Lookup444: pallet_democracy::vote::Voting<Balance, account::AccountId20, BlockNumber, MaxVotes> */
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
  /** Lookup448: pallet_democracy::types::Delegations<Balance> */
  PalletDemocracyDelegations: {
    votes: "u128",
    capital: "u128",
  },
  /** Lookup449: pallet_democracy::vote::PriorLock<BlockNumber, Balance> */
  PalletDemocracyVotePriorLock: "(u32,u128)",
  /** Lookup452: pallet_democracy::pallet::Error<T> */
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
      "ReferendumInvalid",
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
      "TooMany",
      "VotingPeriodLow",
    ],
  },
  /** Lookup454: pallet_collective::Votes<account::AccountId20, BlockNumber> */
  PalletCollectiveVotes: {
    index: "u32",
    threshold: "u32",
    ayes: "Vec<AccountId20>",
    nays: "Vec<AccountId20>",
    end: "u32",
  },
  /** Lookup455: pallet_collective::pallet::Error<T, I> */
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
  /** Lookup457: pallet_treasury::Proposal<account::AccountId20, Balance> */
  PalletTreasuryProposal: {
    proposer: "AccountId20",
    value: "u128",
    beneficiary: "AccountId20",
    bond: "u128",
  },
  /** Lookup461: frame_support::PalletId */
  FrameSupportPalletId: "[u8;8]",
  /** Lookup462: pallet_treasury::pallet::Error<T, I> */
  PalletTreasuryError: {
    _enum: [
      "InsufficientProposersBalance",
      "InvalidIndex",
      "TooManyApprovals",
      "InsufficientPermission",
      "ProposalNotApproved",
    ],
  },
  /** Lookup463: pallet_author_inherent::pallet::Error<T> */
  PalletAuthorInherentError: {
    _enum: ["AuthorAlreadySet", "NoAccountId", "CannotBeAuthor"],
  },
  /** Lookup464: pallet_crowdloan_rewards::pallet::RewardInfo<T> */
  PalletCrowdloanRewardsRewardInfo: {
    totalReward: "u128",
    claimedReward: "u128",
    contributedRelayAddresses: "Vec<[u8;32]>",
  },
  /** Lookup466: pallet_crowdloan_rewards::pallet::Error<T> */
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
  /** Lookup467: pallet_author_mapping::pallet::RegistrationInfo<T> */
  PalletAuthorMappingRegistrationInfo: {
    _alias: {
      keys_: "keys",
    },
    account: "AccountId20",
    deposit: "u128",
    keys_: "SessionKeysPrimitivesVrfVrfCryptoPublic",
  },
  /** Lookup468: pallet_author_mapping::pallet::Error<T> */
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
  /** Lookup471: pallet_proxy::ProxyDefinition<account::AccountId20, moonbase_runtime::ProxyType, BlockNumber> */
  PalletProxyProxyDefinition: {
    delegate: "AccountId20",
    proxyType: "MoonbaseRuntimeProxyType",
    delay: "u32",
  },
  /** Lookup475: pallet_proxy::Announcement<account::AccountId20, primitive_types::H256, BlockNumber> */
  PalletProxyAnnouncement: {
    real: "AccountId20",
    callHash: "H256",
    height: "u32",
  },
  /** Lookup477: pallet_proxy::pallet::Error<T> */
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
  /** Lookup478: pallet_maintenance_mode::pallet::Error<T> */
  PalletMaintenanceModeError: {
    _enum: ["AlreadyInMaintenanceMode", "NotInMaintenanceMode"],
  },
  /** Lookup479: pallet_identity::types::Registration<Balance, MaxJudgements, MaxAdditionalFields> */
  PalletIdentityRegistration: {
    judgements: "Vec<(u32,PalletIdentityJudgement)>",
    deposit: "u128",
    info: "PalletIdentityIdentityInfo",
  },
  /** Lookup486: pallet_identity::types::RegistrarInfo<Balance, account::AccountId20> */
  PalletIdentityRegistrarInfo: {
    account: "AccountId20",
    fee: "u128",
    fields: "PalletIdentityBitFlags",
  },
  /** Lookup488: pallet_identity::pallet::Error<T> */
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
      "JudgementForDifferentIdentity",
      "JudgementPaymentFailed",
    ],
  },
  /** Lookup490: cumulus_pallet_xcmp_queue::InboundChannelDetails */
  CumulusPalletXcmpQueueInboundChannelDetails: {
    sender: "u32",
    state: "CumulusPalletXcmpQueueInboundState",
    messageMetadata: "Vec<(u32,PolkadotParachainPrimitivesXcmpMessageFormat)>",
  },
  /** Lookup491: cumulus_pallet_xcmp_queue::InboundState */
  CumulusPalletXcmpQueueInboundState: {
    _enum: ["Ok", "Suspended"],
  },
  /** Lookup494: polkadot_parachain::primitives::XcmpMessageFormat */
  PolkadotParachainPrimitivesXcmpMessageFormat: {
    _enum: ["ConcatenatedVersionedXcm", "ConcatenatedEncodedBlob", "Signals"],
  },
  /** Lookup497: cumulus_pallet_xcmp_queue::OutboundChannelDetails */
  CumulusPalletXcmpQueueOutboundChannelDetails: {
    recipient: "u32",
    state: "CumulusPalletXcmpQueueOutboundState",
    signalsExist: "bool",
    firstIndex: "u16",
    lastIndex: "u16",
  },
  /** Lookup498: cumulus_pallet_xcmp_queue::OutboundState */
  CumulusPalletXcmpQueueOutboundState: {
    _enum: ["Ok", "Suspended"],
  },
  /** Lookup500: cumulus_pallet_xcmp_queue::QueueConfigData */
  CumulusPalletXcmpQueueQueueConfigData: {
    suspendThreshold: "u32",
    dropThreshold: "u32",
    resumeThreshold: "u32",
    thresholdWeight: "SpWeightsWeightV2Weight",
    weightRestrictDecay: "SpWeightsWeightV2Weight",
    xcmpMaxIndividualWeight: "SpWeightsWeightV2Weight",
  },
  /** Lookup502: cumulus_pallet_xcmp_queue::pallet::Error<T> */
  CumulusPalletXcmpQueueError: {
    _enum: ["FailedToSend", "BadXcmOrigin", "BadXcm", "BadOverweightIndex", "WeightOverLimit"],
  },
  /** Lookup503: cumulus_pallet_xcm::pallet::Error<T> */
  CumulusPalletXcmError: "Null",
  /** Lookup504: cumulus_pallet_dmp_queue::ConfigData */
  CumulusPalletDmpQueueConfigData: {
    maxIndividual: "SpWeightsWeightV2Weight",
  },
  /** Lookup505: cumulus_pallet_dmp_queue::PageIndexData */
  CumulusPalletDmpQueuePageIndexData: {
    beginUsed: "u32",
    endUsed: "u32",
    overweightCount: "u64",
  },
  /** Lookup508: cumulus_pallet_dmp_queue::pallet::Error<T> */
  CumulusPalletDmpQueueError: {
    _enum: ["Unknown", "OverLimit"],
  },
  /** Lookup509: pallet_xcm::pallet::QueryStatus<BlockNumber> */
  PalletXcmQueryStatus: {
    _enum: {
      Pending: {
        responder: "XcmVersionedMultiLocation",
        maybeMatchQuerier: "Option<XcmVersionedMultiLocation>",
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
  /** Lookup513: xcm::VersionedResponse */
  XcmVersionedResponse: {
    _enum: {
      __Unused0: "Null",
      __Unused1: "Null",
      V2: "XcmV2Response",
      V3: "XcmV3Response",
    },
  },
  /** Lookup519: pallet_xcm::pallet::VersionMigrationStage */
  PalletXcmVersionMigrationStage: {
    _enum: {
      MigrateSupportedVersion: "Null",
      MigrateVersionNotifiers: "Null",
      NotifyCurrentTargets: "Option<Bytes>",
      MigrateAndNotifyOldTargets: "Null",
    },
  },
  /** Lookup522: xcm::VersionedAssetId */
  XcmVersionedAssetId: {
    _enum: {
      __Unused0: "Null",
      __Unused1: "Null",
      __Unused2: "Null",
      V3: "XcmV3MultiassetAssetId",
    },
  },
  /** Lookup523: pallet_xcm::pallet::RemoteLockedFungibleRecord */
  PalletXcmRemoteLockedFungibleRecord: {
    amount: "u128",
    owner: "XcmVersionedMultiLocation",
    locker: "XcmVersionedMultiLocation",
    users: "u32",
  },
  /** Lookup527: pallet_xcm::pallet::Error<T> */
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
      "InvalidAsset",
      "LowBalance",
      "TooManyLocks",
      "AccountNotSovereign",
      "FeesNotMet",
      "LockNotFound",
      "InUse",
    ],
  },
  /** Lookup528: pallet_assets::types::AssetDetails<Balance, account::AccountId20, DepositBalance> */
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
    status: "PalletAssetsAssetStatus",
  },
  /** Lookup529: pallet_assets::types::AssetStatus */
  PalletAssetsAssetStatus: {
    _enum: ["Live", "Frozen", "Destroying"],
  },
  /** Lookup531: pallet_assets::types::AssetAccount<Balance, DepositBalance, Extra> */
  PalletAssetsAssetAccount: {
    balance: "u128",
    isFrozen: "bool",
    reason: "PalletAssetsExistenceReason",
    extra: "Null",
  },
  /** Lookup532: pallet_assets::types::ExistenceReason<Balance> */
  PalletAssetsExistenceReason: {
    _enum: {
      Consumer: "Null",
      Sufficient: "Null",
      DepositHeld: "u128",
      DepositRefunded: "Null",
    },
  },
  /** Lookup534: pallet_assets::types::Approval<Balance, DepositBalance> */
  PalletAssetsApproval: {
    amount: "u128",
    deposit: "u128",
  },
  /**
   * Lookup535: pallet_assets::types::AssetMetadata<DepositBalance,
   * sp_core::bounded::bounded_vec::BoundedVec<T, S>>
   */
  PalletAssetsAssetMetadata: {
    deposit: "u128",
    name: "Bytes",
    symbol: "Bytes",
    decimals: "u8",
    isFrozen: "bool",
  },
  /** Lookup537: pallet_assets::pallet::Error<T, I> */
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
      "LiveAsset",
      "AssetNotLive",
      "IncorrectStatus",
      "NotFrozen",
    ],
  },
  /** Lookup538: orml_xtokens::module::Error<T> */
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
      "MinXcmFeeNotDefined",
    ],
  },
  /** Lookup539: pallet_asset_manager::pallet::AssetInfo<T> */
  PalletAssetManagerAssetInfo: {
    creator: "AccountId20",
    deposit: "u128",
  },
  /** Lookup541: pallet_asset_manager::pallet::Error<T> */
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
  /** Lookup542: pallet_migrations::pallet::Error<T> */
  PalletMigrationsError: {
    _enum: ["PreimageMissing", "WrongUpperBound", "PreimageIsTooBig", "PreimageAlreadyExists"],
  },
  /** Lookup543: pallet_xcm_transactor::pallet::Error<T> */
  PalletXcmTransactorError: {
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
      "ErrorDelivering",
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
      "HrmpHandlerNotImplemented",
      "TooMuchFeeUsed",
      "ErrorValidating",
    ],
  },
  /** Lookup545: pallet_moonbeam_orbiters::types::CollatorPoolInfo[account::AccountId20](account::AccountId20) */
  PalletMoonbeamOrbitersCollatorPoolInfo: {
    orbiters: "Vec<AccountId20>",
    maybeCurrentOrbiter: "Option<PalletMoonbeamOrbitersCurrentOrbiter>",
    nextOrbiter: "u32",
  },
  /** Lookup547: pallet_moonbeam_orbiters::types::CurrentOrbiter[account::AccountId20](account::AccountId20) */
  PalletMoonbeamOrbitersCurrentOrbiter: {
    accountId: "AccountId20",
    removed: "bool",
  },
  /** Lookup548: pallet_moonbeam_orbiters::pallet::Error<T> */
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
  /** Lookup549: pallet_ethereum_xcm::pallet::Error<T> */
  PalletEthereumXcmError: {
    _enum: ["EthereumXcmExecutionSuspended"],
  },
  /** Lookup550: pallet_randomness::types::RequestState<T> */
  PalletRandomnessRequestState: {
    request: "PalletRandomnessRequest",
    deposit: "u128",
  },
  /** Lookup551: pallet_randomness::types::Request<Balance, pallet_randomness::types::RequestInfo<T>> */
  PalletRandomnessRequest: {
    refundAddress: "H160",
    contractAddress: "H160",
    fee: "u128",
    gasLimit: "u64",
    numWords: "u8",
    salt: "H256",
    info: "PalletRandomnessRequestInfo",
  },
  /** Lookup552: pallet_randomness::types::RequestInfo<T> */
  PalletRandomnessRequestInfo: {
    _enum: {
      BabeEpoch: "(u64,u64)",
      Local: "(u32,u32)",
    },
  },
  /** Lookup553: pallet_randomness::types::RequestType<T> */
  PalletRandomnessRequestType: {
    _enum: {
      BabeEpoch: "u64",
      Local: "u32",
    },
  },
  /** Lookup554: pallet_randomness::types::RandomnessResult<primitive_types::H256> */
  PalletRandomnessRandomnessResult: {
    randomness: "Option<H256>",
    requestCount: "u64",
  },
  /** Lookup555: pallet_randomness::pallet::Error<T> */
  PalletRandomnessError: {
    _enum: [
      "RequestCounterOverflowed",
      "RequestFeeOverflowed",
      "MustRequestAtLeastOneWord",
      "CannotRequestMoreWordsThanMax",
      "CannotRequestRandomnessAfterMaxDelay",
      "CannotRequestRandomnessBeforeMinDelay",
      "RequestDNE",
      "RequestCannotYetBeFulfilled",
      "OnlyRequesterCanIncreaseFee",
      "RequestHasNotExpired",
      "RandomnessResultDNE",
      "RandomnessResultNotFilled",
    ],
  },
  /**
   * Lookup559: pallet_conviction_voting::vote::Voting<Balance, account::AccountId20, BlockNumber,
   * PollIndex, MaxVotes>
   */
  PalletConvictionVotingVoteVoting: {
    _enum: {
      Casting: "PalletConvictionVotingVoteCasting",
      Delegating: "PalletConvictionVotingVoteDelegating",
    },
  },
  /** Lookup560: pallet_conviction_voting::vote::Casting<Balance, BlockNumber, PollIndex, MaxVotes> */
  PalletConvictionVotingVoteCasting: {
    votes: "Vec<(u32,PalletConvictionVotingVoteAccountVote)>",
    delegations: "PalletConvictionVotingDelegations",
    prior: "PalletConvictionVotingVotePriorLock",
  },
  /** Lookup564: pallet_conviction_voting::types::Delegations<Balance> */
  PalletConvictionVotingDelegations: {
    votes: "u128",
    capital: "u128",
  },
  /** Lookup565: pallet_conviction_voting::vote::PriorLock<BlockNumber, Balance> */
  PalletConvictionVotingVotePriorLock: "(u32,u128)",
  /** Lookup566: pallet_conviction_voting::vote::Delegating<Balance, account::AccountId20, BlockNumber> */
  PalletConvictionVotingVoteDelegating: {
    balance: "u128",
    target: "AccountId20",
    conviction: "PalletConvictionVotingConviction",
    delegations: "PalletConvictionVotingDelegations",
    prior: "PalletConvictionVotingVotePriorLock",
  },
  /** Lookup570: pallet_conviction_voting::pallet::Error<T, I> */
  PalletConvictionVotingError: {
    _enum: [
      "NotOngoing",
      "NotVoter",
      "NoPermission",
      "NoPermissionYet",
      "AlreadyDelegating",
      "AlreadyVoting",
      "InsufficientFunds",
      "NotDelegating",
      "Nonsense",
      "MaxVotesReached",
      "ClassNeeded",
      "BadClass",
    ],
  },
  /**
   * Lookup571: pallet_referenda::types::ReferendumInfo<TrackId, moonbase_runtime::OriginCaller,
   * Moment, frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall>, Balance,
   * pallet_conviction_voting::types::Tally<Votes, Total>, account::AccountId20, ScheduleAddress>
   */
  PalletReferendaReferendumInfo: {
    _enum: {
      Ongoing: "PalletReferendaReferendumStatus",
      Approved: "(u32,Option<PalletReferendaDeposit>,Option<PalletReferendaDeposit>)",
      Rejected: "(u32,Option<PalletReferendaDeposit>,Option<PalletReferendaDeposit>)",
      Cancelled: "(u32,Option<PalletReferendaDeposit>,Option<PalletReferendaDeposit>)",
      TimedOut: "(u32,Option<PalletReferendaDeposit>,Option<PalletReferendaDeposit>)",
      Killed: "u32",
    },
  },
  /**
   * Lookup572: pallet_referenda::types::ReferendumStatus<TrackId, moonbase_runtime::OriginCaller,
   * Moment, frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall>, Balance,
   * pallet_conviction_voting::types::Tally<Votes, Total>, account::AccountId20, ScheduleAddress>
   */
  PalletReferendaReferendumStatus: {
    track: "u16",
    origin: "MoonbaseRuntimeOriginCaller",
    proposal: "FrameSupportPreimagesBounded",
    enactment: "FrameSupportScheduleDispatchTime",
    submitted: "u32",
    submissionDeposit: "PalletReferendaDeposit",
    decisionDeposit: "Option<PalletReferendaDeposit>",
    deciding: "Option<PalletReferendaDecidingStatus>",
    tally: "PalletConvictionVotingTally",
    inQueue: "bool",
    alarm: "Option<(u32,(u32,u32))>",
  },
  /** Lookup573: pallet_referenda::types::Deposit<account::AccountId20, Balance> */
  PalletReferendaDeposit: {
    who: "AccountId20",
    amount: "u128",
  },
  /** Lookup576: pallet_referenda::types::DecidingStatus<BlockNumber> */
  PalletReferendaDecidingStatus: {
    since: "u32",
    confirming: "Option<u32>",
  },
  /** Lookup584: pallet_referenda::types::TrackInfo<Balance, Moment> */
  PalletReferendaTrackInfo: {
    name: "Text",
    maxDeciding: "u32",
    decisionDeposit: "u128",
    preparePeriod: "u32",
    decisionPeriod: "u32",
    confirmPeriod: "u32",
    minEnactmentPeriod: "u32",
    minApproval: "PalletReferendaCurve",
    minSupport: "PalletReferendaCurve",
  },
  /** Lookup585: pallet_referenda::types::Curve */
  PalletReferendaCurve: {
    _enum: {
      LinearDecreasing: {
        length: "Perbill",
        floor: "Perbill",
        ceil: "Perbill",
      },
      SteppedDecreasing: {
        begin: "Perbill",
        end: "Perbill",
        step: "Perbill",
        period: "Perbill",
      },
      Reciprocal: {
        factor: "i64",
        xOffset: "i64",
        yOffset: "i64",
      },
    },
  },
  /** Lookup588: pallet_referenda::pallet::Error<T, I> */
  PalletReferendaError: {
    _enum: [
      "NotOngoing",
      "HasDeposit",
      "BadTrack",
      "Full",
      "QueueEmpty",
      "BadReferendum",
      "NothingToDo",
      "NoTrack",
      "Unfinished",
      "NoPermission",
      "NoDeposit",
      "BadStatus",
    ],
  },
  /** Lookup589: pallet_preimage::RequestStatus<account::AccountId20, Balance> */
  PalletPreimageRequestStatus: {
    _enum: {
      Unrequested: {
        deposit: "(AccountId20,u128)",
        len: "u32",
      },
      Requested: {
        deposit: "Option<(AccountId20,u128)>",
        count: "u32",
        len: "Option<u32>",
      },
    },
  },
  /** Lookup594: pallet_preimage::pallet::Error<T> */
  PalletPreimageError: {
    _enum: ["TooBig", "AlreadyNoted", "NotAuthorized", "NotNoted", "Requested", "NotRequested"],
  },
  /** Lookup595: pallet_whitelist::pallet::Error<T> */
  PalletWhitelistError: {
    _enum: [
      "UnavailablePreImage",
      "UndecodableCall",
      "InvalidCallWeightWitness",
      "CallIsNotWhitelisted",
      "CallAlreadyWhitelisted",
    ],
  },
  /** Lookup598: account::EthereumSignature */
  AccountEthereumSignature: "SpCoreEcdsaSignature",
  /** Lookup600: frame_system::extensions::check_non_zero_sender::CheckNonZeroSender<T> */
  FrameSystemExtensionsCheckNonZeroSender: "Null",
  /** Lookup601: frame_system::extensions::check_spec_version::CheckSpecVersion<T> */
  FrameSystemExtensionsCheckSpecVersion: "Null",
  /** Lookup602: frame_system::extensions::check_tx_version::CheckTxVersion<T> */
  FrameSystemExtensionsCheckTxVersion: "Null",
  /** Lookup603: frame_system::extensions::check_genesis::CheckGenesis<T> */
  FrameSystemExtensionsCheckGenesis: "Null",
  /** Lookup606: frame_system::extensions::check_nonce::CheckNonce<T> */
  FrameSystemExtensionsCheckNonce: "Compact<u32>",
  /** Lookup607: frame_system::extensions::check_weight::CheckWeight<T> */
  FrameSystemExtensionsCheckWeight: "Null",
  /** Lookup608: pallet_transaction_payment::ChargeTransactionPayment<T> */
  PalletTransactionPaymentChargeTransactionPayment: "Compact<u128>",
  /** Lookup610: moonbase_runtime::Runtime */
  MoonbaseRuntimeRuntime: "Null",
};
