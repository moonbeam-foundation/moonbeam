// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

/* eslint-disable sort-keys */

export default {
  /** Lookup3: frame_system::AccountInfo<Nonce, pallet_balances::types::AccountData<Balance>> */
  FrameSystemAccountInfo: {
    nonce: "u32",
    consumers: "u32",
    providers: "u32",
    sufficients: "u32",
    data: "PalletBalancesAccountData",
  },
  /** Lookup5: pallet_balances::types::AccountData<Balance> */
  PalletBalancesAccountData: {
    free: "u128",
    reserved: "u128",
    frozen: "u128",
    flags: "u128",
  },
  /** Lookup8: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight> */
  FrameSupportDispatchPerDispatchClassWeight: {
    normal: "SpWeightsWeightV2Weight",
    operational: "SpWeightsWeightV2Weight",
    mandatory: "SpWeightsWeightV2Weight",
  },
  /** Lookup9: sp_weights::weight_v2::Weight */
  SpWeightsWeightV2Weight: {
    refTime: "Compact<u64>",
    proofSize: "Compact<u64>",
  },
  /** Lookup15: sp_runtime::generic::digest::Digest */
  SpRuntimeDigest: {
    logs: "Vec<SpRuntimeDigestDigestItem>",
  },
  /** Lookup17: sp_runtime::generic::digest::DigestItem */
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
  /** Lookup20: frame_system::EventRecord<moonbase_runtime::RuntimeEvent, primitive_types::H256> */
  FrameSystemEventRecord: {
    phase: "FrameSystemPhase",
    event: "Event",
    topics: "Vec<H256>",
  },
  /** Lookup22: frame_system::pallet::Event<T> */
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
      UpgradeAuthorized: {
        codeHash: "H256",
        checkVersion: "bool",
      },
    },
  },
  /** Lookup23: frame_support::dispatch::DispatchInfo */
  FrameSupportDispatchDispatchInfo: {
    weight: "SpWeightsWeightV2Weight",
    class: "FrameSupportDispatchDispatchClass",
    paysFee: "FrameSupportDispatchPays",
  },
  /** Lookup24: frame_support::dispatch::DispatchClass */
  FrameSupportDispatchDispatchClass: {
    _enum: ["Normal", "Operational", "Mandatory"],
  },
  /** Lookup25: frame_support::dispatch::Pays */
  FrameSupportDispatchPays: {
    _enum: ["Yes", "No"],
  },
  /** Lookup26: sp_runtime::DispatchError */
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
      RootNotAllowed: "Null",
    },
  },
  /** Lookup27: sp_runtime::ModuleError */
  SpRuntimeModuleError: {
    index: "u8",
    error: "[u8;4]",
  },
  /** Lookup28: sp_runtime::TokenError */
  SpRuntimeTokenError: {
    _enum: [
      "FundsUnavailable",
      "OnlyProvider",
      "BelowMinimum",
      "CannotCreate",
      "UnknownAsset",
      "Frozen",
      "Unsupported",
      "CannotCreateHold",
      "NotExpendable",
      "Blocked",
    ],
  },
  /** Lookup29: sp_arithmetic::ArithmeticError */
  SpArithmeticArithmeticError: {
    _enum: ["Underflow", "Overflow", "DivisionByZero"],
  },
  /** Lookup30: sp_runtime::TransactionalError */
  SpRuntimeTransactionalError: {
    _enum: ["LimitReached", "NoLayer"],
  },
  /** Lookup32: pallet_utility::pallet::Event */
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
  /** Lookup35: pallet_balances::pallet::Event<T, I> */
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
      Minted: {
        who: "AccountId20",
        amount: "u128",
      },
      Burned: {
        who: "AccountId20",
        amount: "u128",
      },
      Suspended: {
        who: "AccountId20",
        amount: "u128",
      },
      Restored: {
        who: "AccountId20",
        amount: "u128",
      },
      Upgraded: {
        who: "AccountId20",
      },
      Issued: {
        amount: "u128",
      },
      Rescinded: {
        amount: "u128",
      },
      Locked: {
        who: "AccountId20",
        amount: "u128",
      },
      Unlocked: {
        who: "AccountId20",
        amount: "u128",
      },
      Frozen: {
        who: "AccountId20",
        amount: "u128",
      },
      Thawed: {
        who: "AccountId20",
        amount: "u128",
      },
      TotalIssuanceForced: {
        _alias: {
          new_: "new",
        },
        old: "u128",
        new_: "u128",
      },
    },
  },
  /** Lookup36: frame_support::traits::tokens::misc::BalanceStatus */
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ["Free", "Reserved"],
  },
  /** Lookup37: pallet_sudo::pallet::Event<T> */
  PalletSudoEvent: {
    _enum: {
      Sudid: {
        sudoResult: "Result<Null, SpRuntimeDispatchError>",
      },
      KeyChanged: {
        _alias: {
          new_: "new",
        },
        old: "Option<AccountId20>",
        new_: "AccountId20",
      },
      KeyRemoved: "Null",
      SudoAsDone: {
        sudoResult: "Result<Null, SpRuntimeDispatchError>",
      },
    },
  },
  /** Lookup39: cumulus_pallet_parachain_system::pallet::Event<T> */
  CumulusPalletParachainSystemEvent: {
    _enum: {
      ValidationFunctionStored: "Null",
      ValidationFunctionApplied: {
        relayChainBlockNum: "u32",
      },
      ValidationFunctionDiscarded: "Null",
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
  /** Lookup41: pallet_transaction_payment::pallet::Event<T> */
  PalletTransactionPaymentEvent: {
    _enum: {
      TransactionFeePaid: {
        who: "AccountId20",
        actualFee: "u128",
        tip: "u128",
      },
    },
  },
  /** Lookup42: pallet_evm::pallet::Event<T> */
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
  /** Lookup43: ethereum::log::Log */
  EthereumLog: {
    address: "H160",
    topics: "Vec<H256>",
    data: "Bytes",
  },
  /** Lookup46: pallet_ethereum::pallet::Event */
  PalletEthereumEvent: {
    _enum: {
      Executed: {
        from: "H160",
        to: "H160",
        transactionHash: "H256",
        exitReason: "EvmCoreErrorExitReason",
        extraData: "Bytes",
      },
    },
  },
  /** Lookup47: evm_core::error::ExitReason */
  EvmCoreErrorExitReason: {
    _enum: {
      Succeed: "EvmCoreErrorExitSucceed",
      Error: "EvmCoreErrorExitError",
      Revert: "EvmCoreErrorExitRevert",
      Fatal: "EvmCoreErrorExitFatal",
    },
  },
  /** Lookup48: evm_core::error::ExitSucceed */
  EvmCoreErrorExitSucceed: {
    _enum: ["Stopped", "Returned", "Suicided"],
  },
  /** Lookup49: evm_core::error::ExitError */
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
      MaxNonce: "Null",
      InvalidCode: "u8",
    },
  },
  /** Lookup53: evm_core::error::ExitRevert */
  EvmCoreErrorExitRevert: {
    _enum: ["Reverted"],
  },
  /** Lookup54: evm_core::error::ExitFatal */
  EvmCoreErrorExitFatal: {
    _enum: {
      NotSupported: "Null",
      UnhandledInterrupt: "Null",
      CallErrorAsFatal: "EvmCoreErrorExitError",
      Other: "Text",
    },
  },
  /** Lookup55: pallet_parachain_staking::pallet::Event<T> */
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
  /** Lookup56: pallet_parachain_staking::delegation_requests::CancelledScheduledRequest<Balance> */
  PalletParachainStakingDelegationRequestsCancelledScheduledRequest: {
    whenExecutable: "u32",
    action: "PalletParachainStakingDelegationRequestsDelegationAction",
  },
  /** Lookup57: pallet_parachain_staking::delegation_requests::DelegationAction<Balance> */
  PalletParachainStakingDelegationRequestsDelegationAction: {
    _enum: {
      Revoke: "u128",
      Decrease: "u128",
    },
  },
  /** Lookup58: pallet_parachain_staking::types::DelegatorAdded<B> */
  PalletParachainStakingDelegatorAdded: {
    _enum: {
      AddedToTop: {
        newTotal: "u128",
      },
      AddedToBottom: "Null",
    },
  },
  /** Lookup61: pallet_scheduler::pallet::Event<T> */
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
  /** Lookup63: pallet_treasury::pallet::Event<T, I> */
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
      AssetSpendApproved: {
        index: "u32",
        assetKind: "Null",
        amount: "u128",
        beneficiary: "AccountId20",
        validFrom: "u32",
        expireAt: "u32",
      },
      AssetSpendVoided: {
        index: "u32",
      },
      Paid: {
        index: "u32",
        paymentId: "Null",
      },
      PaymentFailed: {
        index: "u32",
        paymentId: "Null",
      },
      SpendProcessed: {
        index: "u32",
      },
    },
  },
  /** Lookup64: pallet_author_slot_filter::pallet::Event */
  PalletAuthorSlotFilterEvent: {
    _enum: {
      EligibleUpdated: "u32",
    },
  },
  /** Lookup66: pallet_crowdloan_rewards::pallet::Event<T> */
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
  /** Lookup67: pallet_author_mapping::pallet::Event<T> */
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
  /** Lookup68: nimbus_primitives::nimbus_crypto::Public */
  NimbusPrimitivesNimbusCryptoPublic: "SpCoreSr25519Public",
  /** Lookup69: sp_core::sr25519::Public */
  SpCoreSr25519Public: "[u8;32]",
  /** Lookup70: session_keys_primitives::vrf::vrf_crypto::Public */
  SessionKeysPrimitivesVrfVrfCryptoPublic: "SpCoreSr25519Public",
  /** Lookup71: pallet_proxy::pallet::Event<T> */
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
  /** Lookup72: moonbase_runtime::ProxyType */
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
  /** Lookup74: pallet_maintenance_mode::pallet::Event */
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
  /** Lookup75: pallet_identity::pallet::Event<T> */
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
      AuthorityAdded: {
        authority: "AccountId20",
      },
      AuthorityRemoved: {
        authority: "AccountId20",
      },
      UsernameSet: {
        who: "AccountId20",
        username: "Bytes",
      },
      UsernameQueued: {
        who: "AccountId20",
        username: "Bytes",
        expiration: "u32",
      },
      PreapprovalExpired: {
        whose: "AccountId20",
      },
      PrimaryUsernameSet: {
        who: "AccountId20",
        username: "Bytes",
      },
      DanglingUsernameRemoved: {
        who: "AccountId20",
        username: "Bytes",
      },
    },
  },
  /** Lookup77: cumulus_pallet_xcmp_queue::pallet::Event<T> */
  CumulusPalletXcmpQueueEvent: {
    _enum: {
      XcmpMessageSent: {
        messageHash: "[u8;32]",
      },
    },
  },
  /** Lookup78: cumulus_pallet_xcm::pallet::Event<T> */
  CumulusPalletXcmEvent: {
    _enum: {
      InvalidFormat: "[u8;32]",
      UnsupportedVersion: "[u8;32]",
      ExecutedDownward: "([u8;32],StagingXcmV4TraitsOutcome)",
    },
  },
  /** Lookup79: staging_xcm::v4::traits::Outcome */
  StagingXcmV4TraitsOutcome: {
    _enum: {
      Complete: {
        used: "SpWeightsWeightV2Weight",
      },
      Incomplete: {
        used: "SpWeightsWeightV2Weight",
        error: "XcmV3TraitsError",
      },
      Error: {
        error: "XcmV3TraitsError",
      },
    },
  },
  /** Lookup80: xcm::v3::traits::Error */
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
  /** Lookup81: cumulus_pallet_dmp_queue::pallet::Event<T> */
  CumulusPalletDmpQueueEvent: {
    _enum: {
      StartedExport: "Null",
      Exported: {
        page: "u32",
      },
      ExportFailed: {
        page: "u32",
      },
      CompletedExport: "Null",
      StartedOverweightExport: "Null",
      ExportedOverweight: {
        index: "u64",
      },
      ExportOverweightFailed: {
        index: "u64",
      },
      CompletedOverweightExport: "Null",
      StartedCleanup: "Null",
      CleanedSome: {
        keysRemoved: "u32",
      },
      Completed: {
        error: "bool",
      },
    },
  },
  /** Lookup82: pallet_xcm::pallet::Event<T> */
  PalletXcmEvent: {
    _enum: {
      Attempted: {
        outcome: "StagingXcmV4TraitsOutcome",
      },
      Sent: {
        origin: "StagingXcmV4Location",
        destination: "StagingXcmV4Location",
        message: "StagingXcmV4Xcm",
        messageId: "[u8;32]",
      },
      UnexpectedResponse: {
        origin: "StagingXcmV4Location",
        queryId: "u64",
      },
      ResponseReady: {
        queryId: "u64",
        response: "StagingXcmV4Response",
      },
      Notified: {
        queryId: "u64",
        palletIndex: "u8",
        callIndex: "u8",
      },
      NotifyOverweight: {
        queryId: "u64",
        palletIndex: "u8",
        callIndex: "u8",
        actualWeight: "SpWeightsWeightV2Weight",
        maxBudgetedWeight: "SpWeightsWeightV2Weight",
      },
      NotifyDispatchError: {
        queryId: "u64",
        palletIndex: "u8",
        callIndex: "u8",
      },
      NotifyDecodeFailed: {
        queryId: "u64",
        palletIndex: "u8",
        callIndex: "u8",
      },
      InvalidResponder: {
        origin: "StagingXcmV4Location",
        queryId: "u64",
        expectedLocation: "Option<StagingXcmV4Location>",
      },
      InvalidResponderVersion: {
        origin: "StagingXcmV4Location",
        queryId: "u64",
      },
      ResponseTaken: {
        queryId: "u64",
      },
      AssetsTrapped: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
        origin: "StagingXcmV4Location",
        assets: "XcmVersionedAssets",
      },
      VersionChangeNotified: {
        destination: "StagingXcmV4Location",
        result: "u32",
        cost: "StagingXcmV4AssetAssets",
        messageId: "[u8;32]",
      },
      SupportedVersionChanged: {
        location: "StagingXcmV4Location",
        version: "u32",
      },
      NotifyTargetSendFail: {
        location: "StagingXcmV4Location",
        queryId: "u64",
        error: "XcmV3TraitsError",
      },
      NotifyTargetMigrationFail: {
        location: "XcmVersionedLocation",
        queryId: "u64",
      },
      InvalidQuerierVersion: {
        origin: "StagingXcmV4Location",
        queryId: "u64",
      },
      InvalidQuerier: {
        origin: "StagingXcmV4Location",
        queryId: "u64",
        expectedQuerier: "StagingXcmV4Location",
        maybeActualQuerier: "Option<StagingXcmV4Location>",
      },
      VersionNotifyStarted: {
        destination: "StagingXcmV4Location",
        cost: "StagingXcmV4AssetAssets",
        messageId: "[u8;32]",
      },
      VersionNotifyRequested: {
        destination: "StagingXcmV4Location",
        cost: "StagingXcmV4AssetAssets",
        messageId: "[u8;32]",
      },
      VersionNotifyUnrequested: {
        destination: "StagingXcmV4Location",
        cost: "StagingXcmV4AssetAssets",
        messageId: "[u8;32]",
      },
      FeesPaid: {
        paying: "StagingXcmV4Location",
        fees: "StagingXcmV4AssetAssets",
      },
      AssetsClaimed: {
        _alias: {
          hash_: "hash",
        },
        hash_: "H256",
        origin: "StagingXcmV4Location",
        assets: "XcmVersionedAssets",
      },
      VersionMigrationFinished: {
        version: "u32",
      },
    },
  },
  /** Lookup83: staging_xcm::v4::location::Location */
  StagingXcmV4Location: {
    parents: "u8",
    interior: "StagingXcmV4Junctions",
  },
  /** Lookup84: staging_xcm::v4::junctions::Junctions */
  StagingXcmV4Junctions: {
    _enum: {
      Here: "Null",
      X1: "[Lookup86;1]",
      X2: "[Lookup86;2]",
      X3: "[Lookup86;3]",
      X4: "[Lookup86;4]",
      X5: "[Lookup86;5]",
      X6: "[Lookup86;6]",
      X7: "[Lookup86;7]",
      X8: "[Lookup86;8]",
    },
  },
  /** Lookup86: staging_xcm::v4::junction::Junction */
  StagingXcmV4Junction: {
    _enum: {
      Parachain: "Compact<u32>",
      AccountId32: {
        network: "Option<StagingXcmV4JunctionNetworkId>",
        id: "[u8;32]",
      },
      AccountIndex64: {
        network: "Option<StagingXcmV4JunctionNetworkId>",
        index: "Compact<u64>",
      },
      AccountKey20: {
        network: "Option<StagingXcmV4JunctionNetworkId>",
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
      GlobalConsensus: "StagingXcmV4JunctionNetworkId",
    },
  },
  /** Lookup89: staging_xcm::v4::junction::NetworkId */
  StagingXcmV4JunctionNetworkId: {
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
      PolkadotBulletin: "Null",
    },
  },
  /** Lookup91: xcm::v3::junction::BodyId */
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
  /** Lookup92: xcm::v3::junction::BodyPart */
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
  /** Lookup100: staging_xcm::v4::Xcm<Call> */
  StagingXcmV4Xcm: "Vec<StagingXcmV4Instruction>",
  /** Lookup102: staging_xcm::v4::Instruction<Call> */
  StagingXcmV4Instruction: {
    _enum: {
      WithdrawAsset: "StagingXcmV4AssetAssets",
      ReserveAssetDeposited: "StagingXcmV4AssetAssets",
      ReceiveTeleportedAsset: "StagingXcmV4AssetAssets",
      QueryResponse: {
        queryId: "Compact<u64>",
        response: "StagingXcmV4Response",
        maxWeight: "SpWeightsWeightV2Weight",
        querier: "Option<StagingXcmV4Location>",
      },
      TransferAsset: {
        assets: "StagingXcmV4AssetAssets",
        beneficiary: "StagingXcmV4Location",
      },
      TransferReserveAsset: {
        assets: "StagingXcmV4AssetAssets",
        dest: "StagingXcmV4Location",
        xcm: "StagingXcmV4Xcm",
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
      DescendOrigin: "StagingXcmV4Junctions",
      ReportError: "StagingXcmV4QueryResponseInfo",
      DepositAsset: {
        assets: "StagingXcmV4AssetAssetFilter",
        beneficiary: "StagingXcmV4Location",
      },
      DepositReserveAsset: {
        assets: "StagingXcmV4AssetAssetFilter",
        dest: "StagingXcmV4Location",
        xcm: "StagingXcmV4Xcm",
      },
      ExchangeAsset: {
        give: "StagingXcmV4AssetAssetFilter",
        want: "StagingXcmV4AssetAssets",
        maximal: "bool",
      },
      InitiateReserveWithdraw: {
        assets: "StagingXcmV4AssetAssetFilter",
        reserve: "StagingXcmV4Location",
        xcm: "StagingXcmV4Xcm",
      },
      InitiateTeleport: {
        assets: "StagingXcmV4AssetAssetFilter",
        dest: "StagingXcmV4Location",
        xcm: "StagingXcmV4Xcm",
      },
      ReportHolding: {
        responseInfo: "StagingXcmV4QueryResponseInfo",
        assets: "StagingXcmV4AssetAssetFilter",
      },
      BuyExecution: {
        fees: "StagingXcmV4Asset",
        weightLimit: "XcmV3WeightLimit",
      },
      RefundSurplus: "Null",
      SetErrorHandler: "StagingXcmV4Xcm",
      SetAppendix: "StagingXcmV4Xcm",
      ClearError: "Null",
      ClaimAsset: {
        assets: "StagingXcmV4AssetAssets",
        ticket: "StagingXcmV4Location",
      },
      Trap: "Compact<u64>",
      SubscribeVersion: {
        queryId: "Compact<u64>",
        maxResponseWeight: "SpWeightsWeightV2Weight",
      },
      UnsubscribeVersion: "Null",
      BurnAsset: "StagingXcmV4AssetAssets",
      ExpectAsset: "StagingXcmV4AssetAssets",
      ExpectOrigin: "Option<StagingXcmV4Location>",
      ExpectError: "Option<(u32,XcmV3TraitsError)>",
      ExpectTransactStatus: "XcmV3MaybeErrorCode",
      QueryPallet: {
        moduleName: "Bytes",
        responseInfo: "StagingXcmV4QueryResponseInfo",
      },
      ExpectPallet: {
        index: "Compact<u32>",
        name: "Bytes",
        moduleName: "Bytes",
        crateMajor: "Compact<u32>",
        minCrateMinor: "Compact<u32>",
      },
      ReportTransactStatus: "StagingXcmV4QueryResponseInfo",
      ClearTransactStatus: "Null",
      UniversalOrigin: "StagingXcmV4Junction",
      ExportMessage: {
        network: "StagingXcmV4JunctionNetworkId",
        destination: "StagingXcmV4Junctions",
        xcm: "StagingXcmV4Xcm",
      },
      LockAsset: {
        asset: "StagingXcmV4Asset",
        unlocker: "StagingXcmV4Location",
      },
      UnlockAsset: {
        asset: "StagingXcmV4Asset",
        target: "StagingXcmV4Location",
      },
      NoteUnlockable: {
        asset: "StagingXcmV4Asset",
        owner: "StagingXcmV4Location",
      },
      RequestUnlock: {
        asset: "StagingXcmV4Asset",
        locker: "StagingXcmV4Location",
      },
      SetFeesMode: {
        jitWithdraw: "bool",
      },
      SetTopic: "[u8;32]",
      ClearTopic: "Null",
      AliasOrigin: "StagingXcmV4Location",
      UnpaidExecution: {
        weightLimit: "XcmV3WeightLimit",
        checkOrigin: "Option<StagingXcmV4Location>",
      },
    },
  },
  /** Lookup103: staging_xcm::v4::asset::Assets */
  StagingXcmV4AssetAssets: "Vec<StagingXcmV4Asset>",
  /** Lookup105: staging_xcm::v4::asset::Asset */
  StagingXcmV4Asset: {
    id: "StagingXcmV4AssetAssetId",
    fun: "StagingXcmV4AssetFungibility",
  },
  /** Lookup106: staging_xcm::v4::asset::AssetId */
  StagingXcmV4AssetAssetId: "StagingXcmV4Location",
  /** Lookup107: staging_xcm::v4::asset::Fungibility */
  StagingXcmV4AssetFungibility: {
    _enum: {
      Fungible: "Compact<u128>",
      NonFungible: "StagingXcmV4AssetAssetInstance",
    },
  },
  /** Lookup108: staging_xcm::v4::asset::AssetInstance */
  StagingXcmV4AssetAssetInstance: {
    _enum: {
      Undefined: "Null",
      Index: "Compact<u128>",
      Array4: "[u8;4]",
      Array8: "[u8;8]",
      Array16: "[u8;16]",
      Array32: "[u8;32]",
    },
  },
  /** Lookup111: staging_xcm::v4::Response */
  StagingXcmV4Response: {
    _enum: {
      Null: "Null",
      Assets: "StagingXcmV4AssetAssets",
      ExecutionResult: "Option<(u32,XcmV3TraitsError)>",
      Version: "u32",
      PalletsInfo: "Vec<StagingXcmV4PalletInfo>",
      DispatchResult: "XcmV3MaybeErrorCode",
    },
  },
  /** Lookup115: staging_xcm::v4::PalletInfo */
  StagingXcmV4PalletInfo: {
    index: "Compact<u32>",
    name: "Bytes",
    moduleName: "Bytes",
    major: "Compact<u32>",
    minor: "Compact<u32>",
    patch: "Compact<u32>",
  },
  /** Lookup118: xcm::v3::MaybeErrorCode */
  XcmV3MaybeErrorCode: {
    _enum: {
      Success: "Null",
      Error: "Bytes",
      TruncatedError: "Bytes",
    },
  },
  /** Lookup121: xcm::v2::OriginKind */
  XcmV2OriginKind: {
    _enum: ["Native", "SovereignAccount", "Superuser", "Xcm"],
  },
  /** Lookup122: xcm::double_encoded::DoubleEncoded<T> */
  XcmDoubleEncoded: {
    encoded: "Bytes",
  },
  /** Lookup123: staging_xcm::v4::QueryResponseInfo */
  StagingXcmV4QueryResponseInfo: {
    destination: "StagingXcmV4Location",
    queryId: "Compact<u64>",
    maxWeight: "SpWeightsWeightV2Weight",
  },
  /** Lookup124: staging_xcm::v4::asset::AssetFilter */
  StagingXcmV4AssetAssetFilter: {
    _enum: {
      Definite: "StagingXcmV4AssetAssets",
      Wild: "StagingXcmV4AssetWildAsset",
    },
  },
  /** Lookup125: staging_xcm::v4::asset::WildAsset */
  StagingXcmV4AssetWildAsset: {
    _enum: {
      All: "Null",
      AllOf: {
        id: "StagingXcmV4AssetAssetId",
        fun: "StagingXcmV4AssetWildFungibility",
      },
      AllCounted: "Compact<u32>",
      AllOfCounted: {
        id: "StagingXcmV4AssetAssetId",
        fun: "StagingXcmV4AssetWildFungibility",
        count: "Compact<u32>",
      },
    },
  },
  /** Lookup126: staging_xcm::v4::asset::WildFungibility */
  StagingXcmV4AssetWildFungibility: {
    _enum: ["Fungible", "NonFungible"],
  },
  /** Lookup127: xcm::v3::WeightLimit */
  XcmV3WeightLimit: {
    _enum: {
      Unlimited: "Null",
      Limited: "SpWeightsWeightV2Weight",
    },
  },
  /** Lookup128: xcm::VersionedAssets */
  XcmVersionedAssets: {
    _enum: {
      __Unused0: "Null",
      V2: "XcmV2MultiassetMultiAssets",
      __Unused2: "Null",
      V3: "XcmV3MultiassetMultiAssets",
      V4: "StagingXcmV4AssetAssets",
    },
  },
  /** Lookup129: xcm::v2::multiasset::MultiAssets */
  XcmV2MultiassetMultiAssets: "Vec<XcmV2MultiAsset>",
  /** Lookup131: xcm::v2::multiasset::MultiAsset */
  XcmV2MultiAsset: {
    id: "XcmV2MultiassetAssetId",
    fun: "XcmV2MultiassetFungibility",
  },
  /** Lookup132: xcm::v2::multiasset::AssetId */
  XcmV2MultiassetAssetId: {
    _enum: {
      Concrete: "XcmV2MultiLocation",
      Abstract: "Bytes",
    },
  },
  /** Lookup133: xcm::v2::multilocation::MultiLocation */
  XcmV2MultiLocation: {
    parents: "u8",
    interior: "XcmV2MultilocationJunctions",
  },
  /** Lookup134: xcm::v2::multilocation::Junctions */
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
  /** Lookup135: xcm::v2::junction::Junction */
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
  /** Lookup136: xcm::v2::NetworkId */
  XcmV2NetworkId: {
    _enum: {
      Any: "Null",
      Named: "Bytes",
      Polkadot: "Null",
      Kusama: "Null",
    },
  },
  /** Lookup138: xcm::v2::BodyId */
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
  /** Lookup139: xcm::v2::BodyPart */
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
  /** Lookup140: xcm::v2::multiasset::Fungibility */
  XcmV2MultiassetFungibility: {
    _enum: {
      Fungible: "Compact<u128>",
      NonFungible: "XcmV2MultiassetAssetInstance",
    },
  },
  /** Lookup141: xcm::v2::multiasset::AssetInstance */
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
  /** Lookup142: xcm::v3::multiasset::MultiAssets */
  XcmV3MultiassetMultiAssets: "Vec<XcmV3MultiAsset>",
  /** Lookup144: xcm::v3::multiasset::MultiAsset */
  XcmV3MultiAsset: {
    id: "XcmV3MultiassetAssetId",
    fun: "XcmV3MultiassetFungibility",
  },
  /** Lookup145: xcm::v3::multiasset::AssetId */
  XcmV3MultiassetAssetId: {
    _enum: {
      Concrete: "StagingXcmV3MultiLocation",
      Abstract: "[u8;32]",
    },
  },
  /** Lookup146: staging_xcm::v3::multilocation::MultiLocation */
  StagingXcmV3MultiLocation: {
    parents: "u8",
    interior: "XcmV3Junctions",
  },
  /** Lookup147: xcm::v3::junctions::Junctions */
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
  /** Lookup148: xcm::v3::junction::Junction */
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
  /** Lookup150: xcm::v3::junction::NetworkId */
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
      PolkadotBulletin: "Null",
    },
  },
  /** Lookup151: xcm::v3::multiasset::Fungibility */
  XcmV3MultiassetFungibility: {
    _enum: {
      Fungible: "Compact<u128>",
      NonFungible: "XcmV3MultiassetAssetInstance",
    },
  },
  /** Lookup152: xcm::v3::multiasset::AssetInstance */
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
  /** Lookup153: xcm::VersionedLocation */
  XcmVersionedLocation: {
    _enum: {
      __Unused0: "Null",
      V2: "XcmV2MultiLocation",
      __Unused2: "Null",
      V3: "StagingXcmV3MultiLocation",
      V4: "StagingXcmV4Location",
    },
  },
  /** Lookup154: pallet_assets::pallet::Event<T, I> */
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
      AssetMinBalanceChanged: {
        assetId: "u128",
        newMinBalance: "u128",
      },
      Touched: {
        assetId: "u128",
        who: "AccountId20",
        depositor: "AccountId20",
      },
      Blocked: {
        assetId: "u128",
        who: "AccountId20",
      },
    },
  },
  /** Lookup155: orml_xtokens::module::Event<T> */
  OrmlXtokensModuleEvent: {
    _enum: {
      TransferredAssets: {
        sender: "AccountId20",
        assets: "StagingXcmV4AssetAssets",
        fee: "StagingXcmV4Asset",
        dest: "StagingXcmV4Location",
      },
    },
  },
  /** Lookup156: pallet_asset_manager::pallet::Event<T> */
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
      ForeignAssetDestroyed: {
        assetId: "u128",
        assetType: "MoonbaseRuntimeXcmConfigAssetType",
      },
      LocalAssetDestroyed: {
        assetId: "u128",
      },
    },
  },
  /** Lookup157: moonbase_runtime::xcm_config::AssetType */
  MoonbaseRuntimeXcmConfigAssetType: {
    _enum: {
      Xcm: "StagingXcmV3MultiLocation",
    },
  },
  /** Lookup158: moonbase_runtime::asset_config::AssetRegistrarMetadata */
  MoonbaseRuntimeAssetConfigAssetRegistrarMetadata: {
    name: "Bytes",
    symbol: "Bytes",
    decimals: "u8",
    isFrozen: "bool",
  },
  /** Lookup159: pallet_migrations::pallet::Event<T> */
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
  /** Lookup160: pallet_xcm_transactor::pallet::Event<T> */
  PalletXcmTransactorEvent: {
    _enum: {
      TransactedDerivative: {
        accountId: "AccountId20",
        dest: "StagingXcmV4Location",
        call: "Bytes",
        index: "u16",
      },
      TransactedSovereign: {
        feePayer: "Option<AccountId20>",
        dest: "StagingXcmV4Location",
        call: "Bytes",
      },
      TransactedSigned: {
        feePayer: "AccountId20",
        dest: "StagingXcmV4Location",
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
        location: "StagingXcmV4Location",
        remoteInfo: "PalletXcmTransactorRemoteTransactInfoWithMaxWeight",
      },
      TransactInfoRemoved: {
        location: "StagingXcmV4Location",
      },
      DestFeePerSecondChanged: {
        location: "StagingXcmV4Location",
        feePerSecond: "u128",
      },
      DestFeePerSecondRemoved: {
        location: "StagingXcmV4Location",
      },
      HrmpManagementSent: {
        action: "PalletXcmTransactorHrmpOperation",
      },
    },
  },
  /** Lookup161: pallet_xcm_transactor::pallet::RemoteTransactInfoWithMaxWeight */
  PalletXcmTransactorRemoteTransactInfoWithMaxWeight: {
    transactExtraWeight: "SpWeightsWeightV2Weight",
    maxWeight: "SpWeightsWeightV2Weight",
    transactExtraWeightSigned: "Option<SpWeightsWeightV2Weight>",
  },
  /** Lookup163: pallet_xcm_transactor::pallet::HrmpOperation */
  PalletXcmTransactorHrmpOperation: {
    _enum: {
      InitOpen: "PalletXcmTransactorHrmpInitParams",
      Accept: {
        paraId: "u32",
      },
      Close: "PolkadotParachainPrimitivesPrimitivesHrmpChannelId",
      Cancel: {
        channelId: "PolkadotParachainPrimitivesPrimitivesHrmpChannelId",
        openRequests: "u32",
      },
    },
  },
  /** Lookup164: pallet_xcm_transactor::pallet::HrmpInitParams */
  PalletXcmTransactorHrmpInitParams: {
    paraId: "u32",
    proposedMaxCapacity: "u32",
    proposedMaxMessageSize: "u32",
  },
  /** Lookup166: polkadot_parachain_primitives::primitives::HrmpChannelId */
  PolkadotParachainPrimitivesPrimitivesHrmpChannelId: {
    sender: "u32",
    recipient: "u32",
  },
  /** Lookup167: pallet_moonbeam_orbiters::pallet::Event<T> */
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
  /** Lookup168: pallet_randomness::pallet::Event<T> */
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
  /** Lookup169: pallet_collective::pallet::Event<T, I> */
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
  /** Lookup170: pallet_conviction_voting::pallet::Event<T, I> */
  PalletConvictionVotingEvent: {
    _enum: {
      Delegated: "(AccountId20,AccountId20)",
      Undelegated: "AccountId20",
    },
  },
  /** Lookup171: pallet_referenda::pallet::Event<T, I> */
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
      MetadataSet: {
        _alias: {
          hash_: "hash",
        },
        index: "u32",
        hash_: "H256",
      },
      MetadataCleared: {
        _alias: {
          hash_: "hash",
        },
        index: "u32",
        hash_: "H256",
      },
    },
  },
  /**
   * Lookup172: frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall,
   * sp_runtime::traits::BlakeTwo256>
   */
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
  /** Lookup174: frame_system::pallet::Call<T> */
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
      __Unused8: "Null",
      authorize_upgrade: {
        codeHash: "H256",
      },
      authorize_upgrade_without_checks: {
        codeHash: "H256",
      },
      apply_authorized_upgrade: {
        code: "Bytes",
      },
    },
  },
  /** Lookup178: pallet_utility::pallet::Call<T> */
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
  /** Lookup180: moonbase_runtime::OriginCaller */
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
      Void: "SpCoreVoid",
      __Unused9: "Null",
      __Unused10: "Null",
      Ethereum: "PalletEthereumRawOrigin",
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
  /** Lookup181: frame_support::dispatch::RawOrigin[account::AccountId20](account::AccountId20) */
  FrameSupportDispatchRawOrigin: {
    _enum: {
      Root: "Null",
      Signed: "AccountId20",
      None: "Null",
    },
  },
  /** Lookup182: pallet_ethereum::RawOrigin */
  PalletEthereumRawOrigin: {
    _enum: {
      EthereumTransaction: "H160",
    },
  },
  /** Lookup183: cumulus_pallet_xcm::pallet::Origin */
  CumulusPalletXcmOrigin: {
    _enum: {
      Relay: "Null",
      SiblingParachain: "u32",
    },
  },
  /** Lookup184: pallet_xcm::pallet::Origin */
  PalletXcmOrigin: {
    _enum: {
      Xcm: "StagingXcmV4Location",
      Response: "StagingXcmV4Location",
    },
  },
  /** Lookup185: pallet_ethereum_xcm::RawOrigin */
  PalletEthereumXcmRawOrigin: {
    _enum: {
      XcmEthereumTransaction: "H160",
    },
  },
  /** Lookup186: pallet_collective::RawOrigin<account::AccountId20, I> */
  PalletCollectiveRawOrigin: {
    _enum: {
      Members: "(u32,u32)",
      Member: "AccountId20",
      _Phantom: "Null",
    },
  },
  /** Lookup187: moonbase_runtime::governance::origins::custom_origins::Origin */
  MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin: {
    _enum: [
      "WhitelistedCaller",
      "GeneralAdmin",
      "ReferendumCanceller",
      "ReferendumKiller",
      "FastGeneralAdmin",
    ],
  },
  /** Lookup189: sp_core::Void */
  SpCoreVoid: "Null",
  /** Lookup190: pallet_timestamp::pallet::Call<T> */
  PalletTimestampCall: {
    _enum: {
      set: {
        now: "Compact<u64>",
      },
    },
  },
  /** Lookup191: pallet_balances::pallet::Call<T, I> */
  PalletBalancesCall: {
    _enum: {
      transfer_allow_death: {
        dest: "AccountId20",
        value: "Compact<u128>",
      },
      __Unused1: "Null",
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
      upgrade_accounts: {
        who: "Vec<AccountId20>",
      },
      __Unused7: "Null",
      force_set_balance: {
        who: "AccountId20",
        newFree: "Compact<u128>",
      },
      force_adjust_total_issuance: {
        direction: "PalletBalancesAdjustmentDirection",
        delta: "Compact<u128>",
      },
    },
  },
  /** Lookup193: pallet_balances::types::AdjustmentDirection */
  PalletBalancesAdjustmentDirection: {
    _enum: ["Increase", "Decrease"],
  },
  /** Lookup194: pallet_sudo::pallet::Call<T> */
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
      remove_key: "Null",
    },
  },
  /** Lookup195: cumulus_pallet_parachain_system::pallet::Call<T> */
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
        checkVersion: "bool",
      },
      enact_authorized_upgrade: {
        code: "Bytes",
      },
    },
  },
  /** Lookup196: cumulus_primitives_parachain_inherent::ParachainInherentData */
  CumulusPrimitivesParachainInherentParachainInherentData: {
    validationData: "PolkadotPrimitivesV6PersistedValidationData",
    relayChainState: "SpTrieStorageProof",
    downwardMessages: "Vec<PolkadotCorePrimitivesInboundDownwardMessage>",
    horizontalMessages: "BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>",
  },
  /** Lookup197: polkadot_primitives::v6::PersistedValidationData<primitive_types::H256, N> */
  PolkadotPrimitivesV6PersistedValidationData: {
    parentHead: "Bytes",
    relayParentNumber: "u32",
    relayParentStorageRoot: "H256",
    maxPovSize: "u32",
  },
  /** Lookup199: sp_trie::storage_proof::StorageProof */
  SpTrieStorageProof: {
    trieNodes: "BTreeSet<Bytes>",
  },
  /** Lookup202: polkadot_core_primitives::InboundDownwardMessage<BlockNumber> */
  PolkadotCorePrimitivesInboundDownwardMessage: {
    sentAt: "u32",
    msg: "Bytes",
  },
  /** Lookup205: polkadot_core_primitives::InboundHrmpMessage<BlockNumber> */
  PolkadotCorePrimitivesInboundHrmpMessage: {
    sentAt: "u32",
    data: "Bytes",
  },
  /** Lookup208: pallet_evm::pallet::Call<T> */
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
  /** Lookup214: pallet_ethereum::pallet::Call<T> */
  PalletEthereumCall: {
    _enum: {
      transact: {
        transaction: "EthereumTransactionTransactionV2",
      },
    },
  },
  /** Lookup215: ethereum::transaction::TransactionV2 */
  EthereumTransactionTransactionV2: {
    _enum: {
      Legacy: "EthereumTransactionLegacyTransaction",
      EIP2930: "EthereumTransactionEip2930Transaction",
      EIP1559: "EthereumTransactionEip1559Transaction",
    },
  },
  /** Lookup216: ethereum::transaction::LegacyTransaction */
  EthereumTransactionLegacyTransaction: {
    nonce: "U256",
    gasPrice: "U256",
    gasLimit: "U256",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    signature: "EthereumTransactionTransactionSignature",
  },
  /** Lookup217: ethereum::transaction::TransactionAction */
  EthereumTransactionTransactionAction: {
    _enum: {
      Call: "H160",
      Create: "Null",
    },
  },
  /** Lookup218: ethereum::transaction::TransactionSignature */
  EthereumTransactionTransactionSignature: {
    v: "u64",
    r: "H256",
    s: "H256",
  },
  /** Lookup220: ethereum::transaction::EIP2930Transaction */
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
  /** Lookup222: ethereum::transaction::AccessListItem */
  EthereumTransactionAccessListItem: {
    address: "H160",
    storageKeys: "Vec<H256>",
  },
  /** Lookup223: ethereum::transaction::EIP1559Transaction */
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
  /** Lookup224: pallet_parachain_staking::pallet::Call<T> */
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
      removed_call_19: "Null",
      removed_call_20: "Null",
      removed_call_21: "Null",
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
      notify_inactive_collator: {
        collator: "AccountId20",
      },
      enable_marking_offline: {
        value: "bool",
      },
      force_join_candidates: {
        account: "AccountId20",
        bond: "u128",
        candidateCount: "u32",
      },
    },
  },
  /** Lookup227: pallet_scheduler::pallet::Call<T> */
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
  /** Lookup229: pallet_treasury::pallet::Call<T, I> */
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
      spend_local: {
        amount: "Compact<u128>",
        beneficiary: "AccountId20",
      },
      remove_approval: {
        proposalId: "Compact<u32>",
      },
      spend: {
        assetKind: "Null",
        amount: "Compact<u128>",
        beneficiary: "AccountId20",
        validFrom: "Option<u32>",
      },
      payout: {
        index: "u32",
      },
      check_status: {
        index: "u32",
      },
      void_spend: {
        index: "u32",
      },
    },
  },
  /** Lookup231: pallet_author_inherent::pallet::Call<T> */
  PalletAuthorInherentCall: {
    _enum: ["kick_off_authorship_validation"],
  },
  /** Lookup232: pallet_author_slot_filter::pallet::Call<T> */
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
  /** Lookup233: pallet_crowdloan_rewards::pallet::Call<T> */
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
  /** Lookup234: sp_runtime::MultiSignature */
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: "SpCoreEd25519Signature",
      Sr25519: "SpCoreSr25519Signature",
      Ecdsa: "SpCoreEcdsaSignature",
    },
  },
  /** Lookup235: sp_core::ed25519::Signature */
  SpCoreEd25519Signature: "[u8;64]",
  /** Lookup237: sp_core::sr25519::Signature */
  SpCoreSr25519Signature: "[u8;64]",
  /** Lookup238: sp_core::ecdsa::Signature */
  SpCoreEcdsaSignature: "[u8;65]",
  /** Lookup244: pallet_author_mapping::pallet::Call<T> */
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
  /** Lookup245: pallet_proxy::pallet::Call<T> */
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
  /** Lookup247: pallet_maintenance_mode::pallet::Call<T> */
  PalletMaintenanceModeCall: {
    _enum: ["enter_maintenance_mode", "resume_normal_operation"],
  },
  /** Lookup248: pallet_identity::pallet::Call<T> */
  PalletIdentityCall: {
    _enum: {
      add_registrar: {
        account: "AccountId20",
      },
      set_identity: {
        info: "PalletIdentityLegacyIdentityInfo",
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
        fields: "u64",
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
      add_username_authority: {
        authority: "AccountId20",
        suffix: "Bytes",
        allocation: "u32",
      },
      remove_username_authority: {
        authority: "AccountId20",
      },
      set_username_for: {
        who: "AccountId20",
        username: "Bytes",
        signature: "Option<AccountEthereumSignature>",
      },
      accept_username: {
        username: "Bytes",
      },
      remove_expired_approval: {
        username: "Bytes",
      },
      set_primary_username: {
        username: "Bytes",
      },
      remove_dangling_username: {
        username: "Bytes",
      },
    },
  },
  /** Lookup249: pallet_identity::legacy::IdentityInfo<FieldLimit> */
  PalletIdentityLegacyIdentityInfo: {
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
  /** Lookup285: pallet_identity::types::Judgement<Balance> */
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
  /** Lookup287: account::EthereumSignature */
  AccountEthereumSignature: "SpCoreEcdsaSignature",
  /** Lookup288: cumulus_pallet_xcmp_queue::pallet::Call<T> */
  CumulusPalletXcmpQueueCall: {
    _enum: {
      __Unused0: "Null",
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
    },
  },
  /** Lookup289: cumulus_pallet_dmp_queue::pallet::Call<T> */
  CumulusPalletDmpQueueCall: "Null",
  /** Lookup290: pallet_xcm::pallet::Call<T> */
  PalletXcmCall: {
    _enum: {
      send: {
        dest: "XcmVersionedLocation",
        message: "XcmVersionedXcm",
      },
      teleport_assets: {
        dest: "XcmVersionedLocation",
        beneficiary: "XcmVersionedLocation",
        assets: "XcmVersionedAssets",
        feeAssetItem: "u32",
      },
      reserve_transfer_assets: {
        dest: "XcmVersionedLocation",
        beneficiary: "XcmVersionedLocation",
        assets: "XcmVersionedAssets",
        feeAssetItem: "u32",
      },
      execute: {
        message: "XcmVersionedXcm",
        maxWeight: "SpWeightsWeightV2Weight",
      },
      force_xcm_version: {
        location: "StagingXcmV4Location",
        version: "u32",
      },
      force_default_xcm_version: {
        maybeXcmVersion: "Option<u32>",
      },
      force_subscribe_version_notify: {
        location: "XcmVersionedLocation",
      },
      force_unsubscribe_version_notify: {
        location: "XcmVersionedLocation",
      },
      limited_reserve_transfer_assets: {
        dest: "XcmVersionedLocation",
        beneficiary: "XcmVersionedLocation",
        assets: "XcmVersionedAssets",
        feeAssetItem: "u32",
        weightLimit: "XcmV3WeightLimit",
      },
      limited_teleport_assets: {
        dest: "XcmVersionedLocation",
        beneficiary: "XcmVersionedLocation",
        assets: "XcmVersionedAssets",
        feeAssetItem: "u32",
        weightLimit: "XcmV3WeightLimit",
      },
      force_suspension: {
        suspended: "bool",
      },
      transfer_assets: {
        dest: "XcmVersionedLocation",
        beneficiary: "XcmVersionedLocation",
        assets: "XcmVersionedAssets",
        feeAssetItem: "u32",
        weightLimit: "XcmV3WeightLimit",
      },
    },
  },
  /** Lookup291: xcm::VersionedXcm<RuntimeCall> */
  XcmVersionedXcm: {
    _enum: {
      __Unused0: "Null",
      __Unused1: "Null",
      V2: "XcmV2Xcm",
      V3: "XcmV3Xcm",
      V4: "StagingXcmV4Xcm",
    },
  },
  /** Lookup292: xcm::v2::Xcm<RuntimeCall> */
  XcmV2Xcm: "Vec<XcmV2Instruction>",
  /** Lookup294: xcm::v2::Instruction<RuntimeCall> */
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
  /** Lookup295: xcm::v2::Response */
  XcmV2Response: {
    _enum: {
      Null: "Null",
      Assets: "XcmV2MultiassetMultiAssets",
      ExecutionResult: "Option<(u32,XcmV2TraitsError)>",
      Version: "u32",
    },
  },
  /** Lookup298: xcm::v2::traits::Error */
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
  /** Lookup299: xcm::v2::multiasset::MultiAssetFilter */
  XcmV2MultiassetMultiAssetFilter: {
    _enum: {
      Definite: "XcmV2MultiassetMultiAssets",
      Wild: "XcmV2MultiassetWildMultiAsset",
    },
  },
  /** Lookup300: xcm::v2::multiasset::WildMultiAsset */
  XcmV2MultiassetWildMultiAsset: {
    _enum: {
      All: "Null",
      AllOf: {
        id: "XcmV2MultiassetAssetId",
        fun: "XcmV2MultiassetWildFungibility",
      },
    },
  },
  /** Lookup301: xcm::v2::multiasset::WildFungibility */
  XcmV2MultiassetWildFungibility: {
    _enum: ["Fungible", "NonFungible"],
  },
  /** Lookup302: xcm::v2::WeightLimit */
  XcmV2WeightLimit: {
    _enum: {
      Unlimited: "Null",
      Limited: "Compact<u64>",
    },
  },
  /** Lookup303: xcm::v3::Xcm<Call> */
  XcmV3Xcm: "Vec<XcmV3Instruction>",
  /** Lookup305: xcm::v3::Instruction<Call> */
  XcmV3Instruction: {
    _enum: {
      WithdrawAsset: "XcmV3MultiassetMultiAssets",
      ReserveAssetDeposited: "XcmV3MultiassetMultiAssets",
      ReceiveTeleportedAsset: "XcmV3MultiassetMultiAssets",
      QueryResponse: {
        queryId: "Compact<u64>",
        response: "XcmV3Response",
        maxWeight: "SpWeightsWeightV2Weight",
        querier: "Option<StagingXcmV3MultiLocation>",
      },
      TransferAsset: {
        assets: "XcmV3MultiassetMultiAssets",
        beneficiary: "StagingXcmV3MultiLocation",
      },
      TransferReserveAsset: {
        assets: "XcmV3MultiassetMultiAssets",
        dest: "StagingXcmV3MultiLocation",
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
        beneficiary: "StagingXcmV3MultiLocation",
      },
      DepositReserveAsset: {
        assets: "XcmV3MultiassetMultiAssetFilter",
        dest: "StagingXcmV3MultiLocation",
        xcm: "XcmV3Xcm",
      },
      ExchangeAsset: {
        give: "XcmV3MultiassetMultiAssetFilter",
        want: "XcmV3MultiassetMultiAssets",
        maximal: "bool",
      },
      InitiateReserveWithdraw: {
        assets: "XcmV3MultiassetMultiAssetFilter",
        reserve: "StagingXcmV3MultiLocation",
        xcm: "XcmV3Xcm",
      },
      InitiateTeleport: {
        assets: "XcmV3MultiassetMultiAssetFilter",
        dest: "StagingXcmV3MultiLocation",
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
        ticket: "StagingXcmV3MultiLocation",
      },
      Trap: "Compact<u64>",
      SubscribeVersion: {
        queryId: "Compact<u64>",
        maxResponseWeight: "SpWeightsWeightV2Weight",
      },
      UnsubscribeVersion: "Null",
      BurnAsset: "XcmV3MultiassetMultiAssets",
      ExpectAsset: "XcmV3MultiassetMultiAssets",
      ExpectOrigin: "Option<StagingXcmV3MultiLocation>",
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
        unlocker: "StagingXcmV3MultiLocation",
      },
      UnlockAsset: {
        asset: "XcmV3MultiAsset",
        target: "StagingXcmV3MultiLocation",
      },
      NoteUnlockable: {
        asset: "XcmV3MultiAsset",
        owner: "StagingXcmV3MultiLocation",
      },
      RequestUnlock: {
        asset: "XcmV3MultiAsset",
        locker: "StagingXcmV3MultiLocation",
      },
      SetFeesMode: {
        jitWithdraw: "bool",
      },
      SetTopic: "[u8;32]",
      ClearTopic: "Null",
      AliasOrigin: "StagingXcmV3MultiLocation",
      UnpaidExecution: {
        weightLimit: "XcmV3WeightLimit",
        checkOrigin: "Option<StagingXcmV3MultiLocation>",
      },
    },
  },
  /** Lookup306: xcm::v3::Response */
  XcmV3Response: {
    _enum: {
      Null: "Null",
      Assets: "XcmV3MultiassetMultiAssets",
      ExecutionResult: "Option<(u32,XcmV3TraitsError)>",
      Version: "u32",
      PalletsInfo: "Vec<XcmV3PalletInfo>",
      DispatchResult: "XcmV3MaybeErrorCode",
    },
  },
  /** Lookup308: xcm::v3::PalletInfo */
  XcmV3PalletInfo: {
    index: "Compact<u32>",
    name: "Bytes",
    moduleName: "Bytes",
    major: "Compact<u32>",
    minor: "Compact<u32>",
    patch: "Compact<u32>",
  },
  /** Lookup312: xcm::v3::QueryResponseInfo */
  XcmV3QueryResponseInfo: {
    destination: "StagingXcmV3MultiLocation",
    queryId: "Compact<u64>",
    maxWeight: "SpWeightsWeightV2Weight",
  },
  /** Lookup313: xcm::v3::multiasset::MultiAssetFilter */
  XcmV3MultiassetMultiAssetFilter: {
    _enum: {
      Definite: "XcmV3MultiassetMultiAssets",
      Wild: "XcmV3MultiassetWildMultiAsset",
    },
  },
  /** Lookup314: xcm::v3::multiasset::WildMultiAsset */
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
  /** Lookup315: xcm::v3::multiasset::WildFungibility */
  XcmV3MultiassetWildFungibility: {
    _enum: ["Fungible", "NonFungible"],
  },
  /** Lookup327: pallet_assets::pallet::Call<T, I> */
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
      set_min_balance: {
        id: "Compact<u128>",
        minBalance: "u128",
      },
      touch_other: {
        id: "Compact<u128>",
        who: "AccountId20",
      },
      refund_other: {
        id: "Compact<u128>",
        who: "AccountId20",
      },
      block: {
        id: "Compact<u128>",
        who: "AccountId20",
      },
    },
  },
  /** Lookup328: orml_xtokens::module::Call<T> */
  OrmlXtokensModuleCall: {
    _enum: {
      transfer: {
        currencyId: "MoonbaseRuntimeXcmConfigCurrencyId",
        amount: "u128",
        dest: "XcmVersionedLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_multiasset: {
        asset: "XcmVersionedAsset",
        dest: "XcmVersionedLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_with_fee: {
        currencyId: "MoonbaseRuntimeXcmConfigCurrencyId",
        amount: "u128",
        fee: "u128",
        dest: "XcmVersionedLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_multiasset_with_fee: {
        asset: "XcmVersionedAsset",
        fee: "XcmVersionedAsset",
        dest: "XcmVersionedLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_multicurrencies: {
        currencies: "Vec<(MoonbaseRuntimeXcmConfigCurrencyId,u128)>",
        feeItem: "u32",
        dest: "XcmVersionedLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
      transfer_multiassets: {
        assets: "XcmVersionedAssets",
        feeItem: "u32",
        dest: "XcmVersionedLocation",
        destWeightLimit: "XcmV3WeightLimit",
      },
    },
  },
  /** Lookup329: moonbase_runtime::xcm_config::CurrencyId */
  MoonbaseRuntimeXcmConfigCurrencyId: {
    _enum: {
      SelfReserve: "Null",
      ForeignAsset: "u128",
      Erc20: {
        contractAddress: "H160",
      },
    },
  },
  /** Lookup330: xcm::VersionedAsset */
  XcmVersionedAsset: {
    _enum: {
      __Unused0: "Null",
      V2: "XcmV2MultiAsset",
      __Unused2: "Null",
      V3: "XcmV3MultiAsset",
      V4: "StagingXcmV4Asset",
    },
  },
  /** Lookup333: pallet_asset_manager::pallet::Call<T> */
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
      __Unused5: "Null",
      destroy_foreign_asset: {
        assetId: "u128",
        numAssetsWeightHint: "u32",
      },
    },
  },
  /** Lookup334: pallet_xcm_transactor::pallet::Call<T> */
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
        refund: "bool",
      },
      transact_through_sovereign: {
        dest: "XcmVersionedLocation",
        feePayer: "Option<AccountId20>",
        fee: "PalletXcmTransactorCurrencyPayment",
        call: "Bytes",
        originKind: "XcmV2OriginKind",
        weightInfo: "PalletXcmTransactorTransactWeights",
        refund: "bool",
      },
      set_transact_info: {
        location: "XcmVersionedLocation",
        transactExtraWeight: "SpWeightsWeightV2Weight",
        maxWeight: "SpWeightsWeightV2Weight",
        transactExtraWeightSigned: "Option<SpWeightsWeightV2Weight>",
      },
      remove_transact_info: {
        location: "XcmVersionedLocation",
      },
      transact_through_signed: {
        dest: "XcmVersionedLocation",
        fee: "PalletXcmTransactorCurrencyPayment",
        call: "Bytes",
        weightInfo: "PalletXcmTransactorTransactWeights",
        refund: "bool",
      },
      set_fee_per_second: {
        assetLocation: "XcmVersionedLocation",
        feePerSecond: "u128",
      },
      remove_fee_per_second: {
        assetLocation: "XcmVersionedLocation",
      },
      hrmp_manage: {
        action: "PalletXcmTransactorHrmpOperation",
        fee: "PalletXcmTransactorCurrencyPayment",
        weightInfo: "PalletXcmTransactorTransactWeights",
      },
    },
  },
  /** Lookup335: moonbase_runtime::xcm_config::Transactors */
  MoonbaseRuntimeXcmConfigTransactors: {
    _enum: ["Relay"],
  },
  /** Lookup336: pallet_xcm_transactor::pallet::CurrencyPayment<moonbase_runtime::xcm_config::CurrencyId> */
  PalletXcmTransactorCurrencyPayment: {
    currency: "PalletXcmTransactorCurrency",
    feeAmount: "Option<u128>",
  },
  /** Lookup337: pallet_xcm_transactor::pallet::Currency<moonbase_runtime::xcm_config::CurrencyId> */
  PalletXcmTransactorCurrency: {
    _enum: {
      AsCurrencyId: "MoonbaseRuntimeXcmConfigCurrencyId",
      AsMultiLocation: "XcmVersionedLocation",
    },
  },
  /** Lookup339: pallet_xcm_transactor::pallet::TransactWeights */
  PalletXcmTransactorTransactWeights: {
    transactRequiredWeightAtMost: "SpWeightsWeightV2Weight",
    overallWeight: "Option<XcmV3WeightLimit>",
  },
  /** Lookup341: pallet_moonbeam_orbiters::pallet::Call<T> */
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
  /** Lookup342: pallet_ethereum_xcm::pallet::Call<T> */
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
  /** Lookup343: xcm_primitives::ethereum_xcm::EthereumXcmTransaction */
  XcmPrimitivesEthereumXcmEthereumXcmTransaction: {
    _enum: {
      V1: "XcmPrimitivesEthereumXcmEthereumXcmTransactionV1",
      V2: "XcmPrimitivesEthereumXcmEthereumXcmTransactionV2",
    },
  },
  /** Lookup344: xcm_primitives::ethereum_xcm::EthereumXcmTransactionV1 */
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV1: {
    gasLimit: "U256",
    feePayment: "XcmPrimitivesEthereumXcmEthereumXcmFee",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    accessList: "Option<Vec<(H160,Vec<H256>)>>",
  },
  /** Lookup345: xcm_primitives::ethereum_xcm::EthereumXcmFee */
  XcmPrimitivesEthereumXcmEthereumXcmFee: {
    _enum: {
      Manual: "XcmPrimitivesEthereumXcmManualEthereumXcmFee",
      Auto: "Null",
    },
  },
  /** Lookup346: xcm_primitives::ethereum_xcm::ManualEthereumXcmFee */
  XcmPrimitivesEthereumXcmManualEthereumXcmFee: {
    gasPrice: "Option<U256>",
    maxFeePerGas: "Option<U256>",
  },
  /** Lookup349: xcm_primitives::ethereum_xcm::EthereumXcmTransactionV2 */
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV2: {
    gasLimit: "U256",
    action: "EthereumTransactionTransactionAction",
    value: "U256",
    input: "Bytes",
    accessList: "Option<Vec<(H160,Vec<H256>)>>",
  },
  /** Lookup350: pallet_randomness::pallet::Call<T> */
  PalletRandomnessCall: {
    _enum: ["set_babe_randomness_results"],
  },
  /** Lookup351: pallet_collective::pallet::Call<T, I> */
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
      __Unused4: "Null",
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
  /** Lookup352: pallet_conviction_voting::pallet::Call<T, I> */
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
  /** Lookup353: pallet_conviction_voting::vote::AccountVote<Balance> */
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
  /** Lookup355: pallet_conviction_voting::conviction::Conviction */
  PalletConvictionVotingConviction: {
    _enum: ["None", "Locked1x", "Locked2x", "Locked3x", "Locked4x", "Locked5x", "Locked6x"],
  },
  /** Lookup357: pallet_referenda::pallet::Call<T, I> */
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
      set_metadata: {
        index: "u32",
        maybeHash: "Option<H256>",
      },
    },
  },
  /** Lookup358: frame_support::traits::schedule::DispatchTime<BlockNumber> */
  FrameSupportScheduleDispatchTime: {
    _enum: {
      At: "u32",
      After: "u32",
    },
  },
  /** Lookup360: pallet_preimage::pallet::Call<T> */
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
      ensure_updated: {
        hashes: "Vec<H256>",
      },
    },
  },
  /** Lookup361: pallet_whitelist::pallet::Call<T> */
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
  /** Lookup363: pallet_root_testing::pallet::Call<T> */
  PalletRootTestingCall: {
    _enum: {
      fill_block: {
        ratio: "Perbill",
      },
      trigger_defensive: "Null",
    },
  },
  /** Lookup364: pallet_multisig::pallet::Call<T> */
  PalletMultisigCall: {
    _enum: {
      as_multi_threshold_1: {
        otherSignatories: "Vec<AccountId20>",
        call: "Call",
      },
      as_multi: {
        threshold: "u16",
        otherSignatories: "Vec<AccountId20>",
        maybeTimepoint: "Option<PalletMultisigTimepoint>",
        call: "Call",
        maxWeight: "SpWeightsWeightV2Weight",
      },
      approve_as_multi: {
        threshold: "u16",
        otherSignatories: "Vec<AccountId20>",
        maybeTimepoint: "Option<PalletMultisigTimepoint>",
        callHash: "[u8;32]",
        maxWeight: "SpWeightsWeightV2Weight",
      },
      cancel_as_multi: {
        threshold: "u16",
        otherSignatories: "Vec<AccountId20>",
        timepoint: "PalletMultisigTimepoint",
        callHash: "[u8;32]",
      },
    },
  },
  /** Lookup366: pallet_multisig::Timepoint<BlockNumber> */
  PalletMultisigTimepoint: {
    height: "u32",
    index: "u32",
  },
  /** Lookup367: pallet_moonbeam_lazy_migrations::pallet::Call<T> */
  PalletMoonbeamLazyMigrationsCall: {
    _enum: {
      __Unused0: "Null",
      clear_suicided_storage: {
        addresses: "Vec<H160>",
        limit: "u32",
      },
    },
  },
  /** Lookup370: pallet_message_queue::pallet::Call<T> */
  PalletMessageQueueCall: {
    _enum: {
      reap_page: {
        messageOrigin: "CumulusPrimitivesCoreAggregateMessageOrigin",
        pageIndex: "u32",
      },
      execute_overweight: {
        messageOrigin: "CumulusPrimitivesCoreAggregateMessageOrigin",
        page: "u32",
        index: "u32",
        weightLimit: "SpWeightsWeightV2Weight",
      },
    },
  },
  /** Lookup371: cumulus_primitives_core::AggregateMessageOrigin */
  CumulusPrimitivesCoreAggregateMessageOrigin: {
    _enum: {
      Here: "Null",
      Parent: "Null",
      Sibling: "u32",
    },
  },
  /** Lookup372: pallet_emergency_para_xcm::pallet::Call<T> */
  PalletEmergencyParaXcmCall: {
    _enum: {
      paused_to_normal: "Null",
      fast_authorize_upgrade: {
        codeHash: "H256",
      },
    },
  },
  /** Lookup373: sp_runtime::traits::BlakeTwo256 */
  SpRuntimeBlakeTwo256: "Null",
  /** Lookup375: pallet_conviction_voting::types::Tally<Votes, Total> */
  PalletConvictionVotingTally: {
    ayes: "u128",
    nays: "u128",
    support: "u128",
  },
  /** Lookup376: pallet_preimage::pallet::Event<T> */
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
  /** Lookup377: pallet_whitelist::pallet::Event<T> */
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
  /** Lookup379: frame_support::dispatch::PostDispatchInfo */
  FrameSupportDispatchPostDispatchInfo: {
    actualWeight: "Option<SpWeightsWeightV2Weight>",
    paysFee: "FrameSupportDispatchPays",
  },
  /** Lookup380: sp_runtime::DispatchErrorWithPostInfo<frame_support::dispatch::PostDispatchInfo> */
  SpRuntimeDispatchErrorWithPostInfo: {
    postInfo: "FrameSupportDispatchPostDispatchInfo",
    error: "SpRuntimeDispatchError",
  },
  /** Lookup382: pallet_root_testing::pallet::Event<T> */
  PalletRootTestingEvent: {
    _enum: ["DefensiveTestCall"],
  },
  /** Lookup383: pallet_multisig::pallet::Event<T> */
  PalletMultisigEvent: {
    _enum: {
      NewMultisig: {
        approving: "AccountId20",
        multisig: "AccountId20",
        callHash: "[u8;32]",
      },
      MultisigApproval: {
        approving: "AccountId20",
        timepoint: "PalletMultisigTimepoint",
        multisig: "AccountId20",
        callHash: "[u8;32]",
      },
      MultisigExecuted: {
        approving: "AccountId20",
        timepoint: "PalletMultisigTimepoint",
        multisig: "AccountId20",
        callHash: "[u8;32]",
        result: "Result<Null, SpRuntimeDispatchError>",
      },
      MultisigCancelled: {
        cancelling: "AccountId20",
        timepoint: "PalletMultisigTimepoint",
        multisig: "AccountId20",
        callHash: "[u8;32]",
      },
    },
  },
  /** Lookup384: pallet_message_queue::pallet::Event<T> */
  PalletMessageQueueEvent: {
    _enum: {
      ProcessingFailed: {
        id: "H256",
        origin: "CumulusPrimitivesCoreAggregateMessageOrigin",
        error: "FrameSupportMessagesProcessMessageError",
      },
      Processed: {
        id: "H256",
        origin: "CumulusPrimitivesCoreAggregateMessageOrigin",
        weightUsed: "SpWeightsWeightV2Weight",
        success: "bool",
      },
      OverweightEnqueued: {
        id: "[u8;32]",
        origin: "CumulusPrimitivesCoreAggregateMessageOrigin",
        pageIndex: "u32",
        messageIndex: "u32",
      },
      PageReaped: {
        origin: "CumulusPrimitivesCoreAggregateMessageOrigin",
        index: "u32",
      },
    },
  },
  /** Lookup385: frame_support::traits::messages::ProcessMessageError */
  FrameSupportMessagesProcessMessageError: {
    _enum: {
      BadFormat: "Null",
      Corrupt: "Null",
      Unsupported: "Null",
      Overweight: "SpWeightsWeightV2Weight",
      Yield: "Null",
    },
  },
  /** Lookup386: pallet_emergency_para_xcm::pallet::Event */
  PalletEmergencyParaXcmEvent: {
    _enum: ["EnteredPausedXcmMode", "NormalXcmOperationResumed"],
  },
  /** Lookup387: frame_system::Phase */
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: "u32",
      Finalization: "Null",
      Initialization: "Null",
    },
  },
  /** Lookup389: frame_system::LastRuntimeUpgradeInfo */
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: "Compact<u32>",
    specName: "Text",
  },
  /** Lookup390: frame_system::CodeUpgradeAuthorization<T> */
  FrameSystemCodeUpgradeAuthorization: {
    codeHash: "H256",
    checkVersion: "bool",
  },
  /** Lookup391: frame_system::limits::BlockWeights */
  FrameSystemLimitsBlockWeights: {
    baseBlock: "SpWeightsWeightV2Weight",
    maxBlock: "SpWeightsWeightV2Weight",
    perClass: "FrameSupportDispatchPerDispatchClassWeightsPerClass",
  },
  /** Lookup392: frame_support::dispatch::PerDispatchClass<frame_system::limits::WeightsPerClass> */
  FrameSupportDispatchPerDispatchClassWeightsPerClass: {
    normal: "FrameSystemLimitsWeightsPerClass",
    operational: "FrameSystemLimitsWeightsPerClass",
    mandatory: "FrameSystemLimitsWeightsPerClass",
  },
  /** Lookup393: frame_system::limits::WeightsPerClass */
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: "SpWeightsWeightV2Weight",
    maxExtrinsic: "Option<SpWeightsWeightV2Weight>",
    maxTotal: "Option<SpWeightsWeightV2Weight>",
    reserved: "Option<SpWeightsWeightV2Weight>",
  },
  /** Lookup394: frame_system::limits::BlockLength */
  FrameSystemLimitsBlockLength: {
    max: "FrameSupportDispatchPerDispatchClassU32",
  },
  /** Lookup395: frame_support::dispatch::PerDispatchClass<T> */
  FrameSupportDispatchPerDispatchClassU32: {
    normal: "u32",
    operational: "u32",
    mandatory: "u32",
  },
  /** Lookup396: sp_weights::RuntimeDbWeight */
  SpWeightsRuntimeDbWeight: {
    read: "u64",
    write: "u64",
  },
  /** Lookup397: sp_version::RuntimeVersion */
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
  /** Lookup401: frame_system::pallet::Error<T> */
  FrameSystemError: {
    _enum: [
      "InvalidSpecName",
      "SpecVersionNeedsToIncrease",
      "FailedToExtractRuntimeVersion",
      "NonDefaultComposite",
      "NonZeroRefCount",
      "CallFiltered",
      "NothingAuthorized",
      "Unauthorized",
    ],
  },
  /** Lookup402: pallet_utility::pallet::Error<T> */
  PalletUtilityError: {
    _enum: ["TooManyCalls"],
  },
  /** Lookup404: pallet_balances::types::BalanceLock<Balance> */
  PalletBalancesBalanceLock: {
    id: "[u8;8]",
    amount: "u128",
    reasons: "PalletBalancesReasons",
  },
  /** Lookup405: pallet_balances::types::Reasons */
  PalletBalancesReasons: {
    _enum: ["Fee", "Misc", "All"],
  },
  /** Lookup408: pallet_balances::types::ReserveData<ReserveIdentifier, Balance> */
  PalletBalancesReserveData: {
    id: "[u8;4]",
    amount: "u128",
  },
  /** Lookup412: moonbase_runtime::RuntimeHoldReason */
  MoonbaseRuntimeRuntimeHoldReason: {
    _enum: {
      __Unused0: "Null",
      __Unused1: "Null",
      __Unused2: "Null",
      __Unused3: "Null",
      __Unused4: "Null",
      __Unused5: "Null",
      __Unused6: "Null",
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
      Preimage: "PalletPreimageHoldReason",
    },
  },
  /** Lookup413: pallet_preimage::pallet::HoldReason */
  PalletPreimageHoldReason: {
    _enum: ["Preimage"],
  },
  /** Lookup416: pallet_balances::types::IdAmount<Id, Balance> */
  PalletBalancesIdAmount: {
    id: "Null",
    amount: "u128",
  },
  /** Lookup418: pallet_balances::pallet::Error<T, I> */
  PalletBalancesError: {
    _enum: [
      "VestingBalance",
      "LiquidityRestrictions",
      "InsufficientBalance",
      "ExistentialDeposit",
      "Expendability",
      "ExistingVestingSchedule",
      "DeadAccount",
      "TooManyReserves",
      "TooManyHolds",
      "TooManyFreezes",
      "IssuanceDeactivated",
      "DeltaZero",
    ],
  },
  /** Lookup419: pallet_sudo::pallet::Error<T> */
  PalletSudoError: {
    _enum: ["RequireSudo"],
  },
  /** Lookup421: cumulus_pallet_parachain_system::unincluded_segment::Ancestor<primitive_types::H256> */
  CumulusPalletParachainSystemUnincludedSegmentAncestor: {
    usedBandwidth: "CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth",
    paraHeadHash: "Option<H256>",
    consumedGoAheadSignal: "Option<PolkadotPrimitivesV6UpgradeGoAhead>",
  },
  /** Lookup422: cumulus_pallet_parachain_system::unincluded_segment::UsedBandwidth */
  CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth: {
    umpMsgCount: "u32",
    umpTotalBytes: "u32",
    hrmpOutgoing: "BTreeMap<u32, CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate>",
  },
  /** Lookup424: cumulus_pallet_parachain_system::unincluded_segment::HrmpChannelUpdate */
  CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate: {
    msgCount: "u32",
    totalBytes: "u32",
  },
  /** Lookup428: polkadot_primitives::v6::UpgradeGoAhead */
  PolkadotPrimitivesV6UpgradeGoAhead: {
    _enum: ["Abort", "GoAhead"],
  },
  /** Lookup429: cumulus_pallet_parachain_system::unincluded_segment::SegmentTracker<primitive_types::H256> */
  CumulusPalletParachainSystemUnincludedSegmentSegmentTracker: {
    usedBandwidth: "CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth",
    hrmpWatermark: "Option<u32>",
    consumedGoAheadSignal: "Option<PolkadotPrimitivesV6UpgradeGoAhead>",
  },
  /** Lookup431: polkadot_primitives::v6::UpgradeRestriction */
  PolkadotPrimitivesV6UpgradeRestriction: {
    _enum: ["Present"],
  },
  /** Lookup432: cumulus_pallet_parachain_system::relay_state_snapshot::MessagingStateSnapshot */
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: {
    dmqMqcHead: "H256",
    relayDispatchQueueRemainingCapacity:
      "CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity",
    ingressChannels: "Vec<(u32,PolkadotPrimitivesV6AbridgedHrmpChannel)>",
    egressChannels: "Vec<(u32,PolkadotPrimitivesV6AbridgedHrmpChannel)>",
  },
  /** Lookup433: cumulus_pallet_parachain_system::relay_state_snapshot::RelayDispatchQueueRemainingCapacity */
  CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity: {
    remainingCount: "u32",
    remainingSize: "u32",
  },
  /** Lookup436: polkadot_primitives::v6::AbridgedHrmpChannel */
  PolkadotPrimitivesV6AbridgedHrmpChannel: {
    maxCapacity: "u32",
    maxTotalSize: "u32",
    maxMessageSize: "u32",
    msgCount: "u32",
    totalSize: "u32",
    mqcHead: "Option<H256>",
  },
  /** Lookup437: polkadot_primitives::v6::AbridgedHostConfiguration */
  PolkadotPrimitivesV6AbridgedHostConfiguration: {
    maxCodeSize: "u32",
    maxHeadDataSize: "u32",
    maxUpwardQueueCount: "u32",
    maxUpwardQueueSize: "u32",
    maxUpwardMessageSize: "u32",
    maxUpwardMessageNumPerCandidate: "u32",
    hrmpMaxMessageNumPerCandidate: "u32",
    validationUpgradeCooldown: "u32",
    validationUpgradeDelay: "u32",
    asyncBackingParams: "PolkadotPrimitivesV6AsyncBackingAsyncBackingParams",
  },
  /** Lookup438: polkadot_primitives::v6::async_backing::AsyncBackingParams */
  PolkadotPrimitivesV6AsyncBackingAsyncBackingParams: {
    maxCandidateDepth: "u32",
    allowedAncestryLen: "u32",
  },
  /** Lookup444: polkadot_core_primitives::OutboundHrmpMessage<polkadot_parachain_primitives::primitives::Id> */
  PolkadotCorePrimitivesOutboundHrmpMessage: {
    recipient: "u32",
    data: "Bytes",
  },
  /** Lookup446: cumulus_pallet_parachain_system::pallet::Error<T> */
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
  /** Lookup447: pallet_transaction_payment::Releases */
  PalletTransactionPaymentReleases: {
    _enum: ["V1Ancient", "V2"],
  },
  /** Lookup448: pallet_evm::CodeMetadata */
  PalletEvmCodeMetadata: {
    _alias: {
      size_: "size",
      hash_: "hash",
    },
    size_: "u64",
    hash_: "H256",
  },
  /** Lookup450: pallet_evm::pallet::Error<T> */
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
      "InvalidChainId",
      "InvalidSignature",
      "Reentrancy",
      "TransactionMustComeFromEOA",
      "InvalidTransaction",
      "Undefined",
    ],
  },
  /** Lookup453: fp_rpc::TransactionStatus */
  FpRpcTransactionStatus: {
    transactionHash: "H256",
    transactionIndex: "u32",
    from: "H160",
    to: "Option<H160>",
    contractAddress: "Option<H160>",
    logs: "Vec<EthereumLog>",
    logsBloom: "EthbloomBloom",
  },
  /** Lookup456: ethbloom::Bloom */
  EthbloomBloom: "[u8;256]",
  /** Lookup458: ethereum::receipt::ReceiptV3 */
  EthereumReceiptReceiptV3: {
    _enum: {
      Legacy: "EthereumReceiptEip658ReceiptData",
      EIP2930: "EthereumReceiptEip658ReceiptData",
      EIP1559: "EthereumReceiptEip658ReceiptData",
    },
  },
  /** Lookup459: ethereum::receipt::EIP658ReceiptData */
  EthereumReceiptEip658ReceiptData: {
    statusCode: "u8",
    usedGas: "U256",
    logsBloom: "EthbloomBloom",
    logs: "Vec<EthereumLog>",
  },
  /**
   * Lookup460:
   * ethereum::block::Block[ethereum::transaction::TransactionV2](ethereum::transaction::TransactionV2)
   */
  EthereumBlock: {
    header: "EthereumHeader",
    transactions: "Vec<EthereumTransactionTransactionV2>",
    ommers: "Vec<EthereumHeader>",
  },
  /** Lookup461: ethereum::header::Header */
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
  /** Lookup462: ethereum_types::hash::H64 */
  EthereumTypesHashH64: "[u8;8]",
  /** Lookup467: pallet_ethereum::pallet::Error<T> */
  PalletEthereumError: {
    _enum: ["InvalidSignature", "PreLogExists"],
  },
  /**
   * Lookup468:
   * pallet_parachain_staking::types::ParachainBondConfig[account::AccountId20](account::AccountId20)
   */
  PalletParachainStakingParachainBondConfig: {
    account: "AccountId20",
    percent: "Percent",
  },
  /** Lookup469: pallet_parachain_staking::types::RoundInfo<BlockNumber> */
  PalletParachainStakingRoundInfo: {
    current: "u32",
    first: "u32",
    length: "u32",
    firstSlot: "u64",
  },
  /** Lookup470: pallet_parachain_staking::types::Delegator<account::AccountId20, Balance> */
  PalletParachainStakingDelegator: {
    id: "AccountId20",
    delegations: "PalletParachainStakingSetOrderedSet",
    total: "u128",
    lessTotal: "u128",
    status: "PalletParachainStakingDelegatorStatus",
  },
  /**
   * Lookup471:
   * pallet_parachain_staking::set::OrderedSet<pallet_parachain_staking::types::Bond<account::AccountId20,
   * Balance>>
   */
  PalletParachainStakingSetOrderedSet: "Vec<PalletParachainStakingBond>",
  /** Lookup472: pallet_parachain_staking::types::Bond<account::AccountId20, Balance> */
  PalletParachainStakingBond: {
    owner: "AccountId20",
    amount: "u128",
  },
  /** Lookup474: pallet_parachain_staking::types::DelegatorStatus */
  PalletParachainStakingDelegatorStatus: {
    _enum: {
      Active: "Null",
      Leaving: "u32",
    },
  },
  /** Lookup475: pallet_parachain_staking::types::CandidateMetadata<Balance> */
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
  /** Lookup476: pallet_parachain_staking::types::CapacityStatus */
  PalletParachainStakingCapacityStatus: {
    _enum: ["Full", "Empty", "Partial"],
  },
  /** Lookup478: pallet_parachain_staking::types::CandidateBondLessRequest<Balance> */
  PalletParachainStakingCandidateBondLessRequest: {
    amount: "u128",
    whenExecutable: "u32",
  },
  /** Lookup479: pallet_parachain_staking::types::CollatorStatus */
  PalletParachainStakingCollatorStatus: {
    _enum: {
      Active: "Null",
      Idle: "Null",
      Leaving: "u32",
    },
  },
  /** Lookup481: pallet_parachain_staking::delegation_requests::ScheduledRequest<account::AccountId20, Balance> */
  PalletParachainStakingDelegationRequestsScheduledRequest: {
    delegator: "AccountId20",
    whenExecutable: "u32",
    action: "PalletParachainStakingDelegationRequestsDelegationAction",
  },
  /**
   * Lookup484:
   * pallet_parachain_staking::auto_compound::AutoCompoundConfig[account::AccountId20](account::AccountId20)
   */
  PalletParachainStakingAutoCompoundAutoCompoundConfig: {
    delegator: "AccountId20",
    value: "Percent",
  },
  /** Lookup486: pallet_parachain_staking::types::Delegations<account::AccountId20, Balance> */
  PalletParachainStakingDelegations: {
    delegations: "Vec<PalletParachainStakingBond>",
    total: "u128",
  },
  /**
   * Lookup488:
   * pallet_parachain_staking::set::BoundedOrderedSet<pallet_parachain_staking::types::Bond<account::AccountId20,
   * Balance>, S>
   */
  PalletParachainStakingSetBoundedOrderedSet: "Vec<PalletParachainStakingBond>",
  /** Lookup491: pallet_parachain_staking::types::CollatorSnapshot<account::AccountId20, Balance> */
  PalletParachainStakingCollatorSnapshot: {
    bond: "u128",
    delegations: "Vec<PalletParachainStakingBondWithAutoCompound>",
    total: "u128",
  },
  /** Lookup493: pallet_parachain_staking::types::BondWithAutoCompound<account::AccountId20, Balance> */
  PalletParachainStakingBondWithAutoCompound: {
    owner: "AccountId20",
    amount: "u128",
    autoCompound: "Percent",
  },
  /** Lookup494: pallet_parachain_staking::types::DelayedPayout<Balance> */
  PalletParachainStakingDelayedPayout: {
    roundIssuance: "u128",
    totalStakingReward: "u128",
    collatorCommission: "Perbill",
  },
  /** Lookup495: pallet_parachain_staking::inflation::InflationInfo<Balance> */
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
  /** Lookup496: pallet_parachain_staking::pallet::Error<T> */
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
      "TooLowCollatorCountToNotifyAsInactive",
      "CannotBeNotifiedAsInactive",
      "TooLowCandidateAutoCompoundingDelegationCountToLeaveCandidates",
      "TooLowCandidateCountWeightHint",
      "TooLowCandidateCountWeightHintGoOffline",
      "CandidateLimitReached",
      "CannotSetAboveMaxCandidates",
      "RemovedCall",
      "MarkingOfflineNotEnabled",
      "CurrentRoundTooLow",
    ],
  },
  /**
   * Lookup499: pallet_scheduler::Scheduled<Name,
   * frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall,
   * sp_runtime::traits::BlakeTwo256>, BlockNumber, moonbase_runtime::OriginCaller, account::AccountId20>
   */
  PalletSchedulerScheduled: {
    maybeId: "Option<[u8;32]>",
    priority: "u8",
    call: "FrameSupportPreimagesBounded",
    maybePeriodic: "Option<(u32,u32)>",
    origin: "MoonbaseRuntimeOriginCaller",
  },
  /** Lookup501: pallet_scheduler::pallet::Error<T> */
  PalletSchedulerError: {
    _enum: [
      "FailedToSchedule",
      "NotFound",
      "TargetBlockNumberInPast",
      "RescheduleNoChange",
      "Named",
    ],
  },
  /** Lookup502: pallet_treasury::Proposal<account::AccountId20, Balance> */
  PalletTreasuryProposal: {
    proposer: "AccountId20",
    value: "u128",
    beneficiary: "AccountId20",
    bond: "u128",
  },
  /**
   * Lookup505: pallet_treasury::SpendStatus<AssetKind, AssetBalance, account::AccountId20,
   * BlockNumber, PaymentId>
   */
  PalletTreasurySpendStatus: {
    assetKind: "Null",
    amount: "u128",
    beneficiary: "AccountId20",
    validFrom: "u32",
    expireAt: "u32",
    status: "PalletTreasuryPaymentState",
  },
  /** Lookup506: pallet_treasury::PaymentState<Id> */
  PalletTreasuryPaymentState: {
    _enum: {
      Pending: "Null",
      Attempted: {
        id: "Null",
      },
      Failed: "Null",
    },
  },
  /** Lookup508: frame_support::PalletId */
  FrameSupportPalletId: "[u8;8]",
  /** Lookup509: pallet_treasury::pallet::Error<T, I> */
  PalletTreasuryError: {
    _enum: [
      "InsufficientProposersBalance",
      "InvalidIndex",
      "TooManyApprovals",
      "InsufficientPermission",
      "ProposalNotApproved",
      "FailedToConvertBalance",
      "SpendExpired",
      "EarlyPayout",
      "AlreadyAttempted",
      "PayoutError",
      "NotAttempted",
      "Inconclusive",
    ],
  },
  /** Lookup510: pallet_author_inherent::pallet::Error<T> */
  PalletAuthorInherentError: {
    _enum: ["AuthorAlreadySet", "NoAccountId", "CannotBeAuthor"],
  },
  /** Lookup511: pallet_crowdloan_rewards::pallet::RewardInfo<T> */
  PalletCrowdloanRewardsRewardInfo: {
    totalReward: "u128",
    claimedReward: "u128",
    contributedRelayAddresses: "Vec<[u8;32]>",
  },
  /** Lookup513: pallet_crowdloan_rewards::pallet::Error<T> */
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
  /** Lookup514: pallet_author_mapping::pallet::RegistrationInfo<T> */
  PalletAuthorMappingRegistrationInfo: {
    _alias: {
      keys_: "keys",
    },
    account: "AccountId20",
    deposit: "u128",
    keys_: "SessionKeysPrimitivesVrfVrfCryptoPublic",
  },
  /** Lookup515: pallet_author_mapping::pallet::Error<T> */
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
  /** Lookup518: pallet_proxy::ProxyDefinition<account::AccountId20, moonbase_runtime::ProxyType, BlockNumber> */
  PalletProxyProxyDefinition: {
    delegate: "AccountId20",
    proxyType: "MoonbaseRuntimeProxyType",
    delay: "u32",
  },
  /** Lookup522: pallet_proxy::Announcement<account::AccountId20, primitive_types::H256, BlockNumber> */
  PalletProxyAnnouncement: {
    real: "AccountId20",
    callHash: "H256",
    height: "u32",
  },
  /** Lookup524: pallet_proxy::pallet::Error<T> */
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
  /** Lookup525: pallet_maintenance_mode::pallet::Error<T> */
  PalletMaintenanceModeError: {
    _enum: ["AlreadyInMaintenanceMode", "NotInMaintenanceMode"],
  },
  /**
   * Lookup527: pallet_identity::types::Registration<Balance, MaxJudgements,
   * pallet_identity::legacy::IdentityInfo<FieldLimit>>
   */
  PalletIdentityRegistration: {
    judgements: "Vec<(u32,PalletIdentityJudgement)>",
    deposit: "u128",
    info: "PalletIdentityLegacyIdentityInfo",
  },
  /** Lookup536: pallet_identity::types::RegistrarInfo<Balance, account::AccountId20, IdField> */
  PalletIdentityRegistrarInfo: {
    account: "AccountId20",
    fee: "u128",
    fields: "u64",
  },
  /**
   * Lookup538:
   * pallet_identity::types::AuthorityProperties<bounded_collections::bounded_vec::BoundedVec<T, S>>
   */
  PalletIdentityAuthorityProperties: {
    suffix: "Bytes",
    allocation: "u32",
  },
  /** Lookup541: pallet_identity::pallet::Error<T> */
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
      "TooManyRegistrars",
      "AlreadyClaimed",
      "NotSub",
      "NotOwned",
      "JudgementForDifferentIdentity",
      "JudgementPaymentFailed",
      "InvalidSuffix",
      "NotUsernameAuthority",
      "NoAllocation",
      "InvalidSignature",
      "RequiresSignature",
      "InvalidUsername",
      "UsernameTaken",
      "NoUsername",
      "NotExpired",
    ],
  },
  /** Lookup546: cumulus_pallet_xcmp_queue::OutboundChannelDetails */
  CumulusPalletXcmpQueueOutboundChannelDetails: {
    recipient: "u32",
    state: "CumulusPalletXcmpQueueOutboundState",
    signalsExist: "bool",
    firstIndex: "u16",
    lastIndex: "u16",
  },
  /** Lookup547: cumulus_pallet_xcmp_queue::OutboundState */
  CumulusPalletXcmpQueueOutboundState: {
    _enum: ["Ok", "Suspended"],
  },
  /** Lookup549: cumulus_pallet_xcmp_queue::QueueConfigData */
  CumulusPalletXcmpQueueQueueConfigData: {
    suspendThreshold: "u32",
    dropThreshold: "u32",
    resumeThreshold: "u32",
  },
  /** Lookup550: cumulus_pallet_xcmp_queue::pallet::Error<T> */
  CumulusPalletXcmpQueueError: {
    _enum: ["BadQueueConfig", "AlreadySuspended", "AlreadyResumed"],
  },
  /** Lookup551: cumulus_pallet_dmp_queue::pallet::MigrationState */
  CumulusPalletDmpQueueMigrationState: {
    _enum: {
      NotStarted: "Null",
      StartedExport: {
        nextBeginUsed: "u32",
      },
      CompletedExport: "Null",
      StartedOverweightExport: {
        nextOverweightIndex: "u64",
      },
      CompletedOverweightExport: "Null",
      StartedCleanup: {
        cursor: "Option<Bytes>",
      },
      Completed: "Null",
    },
  },
  /** Lookup554: pallet_xcm::pallet::QueryStatus<BlockNumber> */
  PalletXcmQueryStatus: {
    _enum: {
      Pending: {
        responder: "XcmVersionedLocation",
        maybeMatchQuerier: "Option<XcmVersionedLocation>",
        maybeNotify: "Option<(u8,u8)>",
        timeout: "u32",
      },
      VersionNotifier: {
        origin: "XcmVersionedLocation",
        isActive: "bool",
      },
      Ready: {
        response: "XcmVersionedResponse",
        at: "u32",
      },
    },
  },
  /** Lookup558: xcm::VersionedResponse */
  XcmVersionedResponse: {
    _enum: {
      __Unused0: "Null",
      __Unused1: "Null",
      V2: "XcmV2Response",
      V3: "XcmV3Response",
      V4: "StagingXcmV4Response",
    },
  },
  /** Lookup564: pallet_xcm::pallet::VersionMigrationStage */
  PalletXcmVersionMigrationStage: {
    _enum: {
      MigrateSupportedVersion: "Null",
      MigrateVersionNotifiers: "Null",
      NotifyCurrentTargets: "Option<Bytes>",
      MigrateAndNotifyOldTargets: "Null",
    },
  },
  /** Lookup567: xcm::VersionedAssetId */
  XcmVersionedAssetId: {
    _enum: {
      __Unused0: "Null",
      __Unused1: "Null",
      __Unused2: "Null",
      V3: "XcmV3MultiassetAssetId",
      V4: "StagingXcmV4AssetAssetId",
    },
  },
  /** Lookup568: pallet_xcm::pallet::RemoteLockedFungibleRecord<ConsumerIdentifier, MaxConsumers> */
  PalletXcmRemoteLockedFungibleRecord: {
    amount: "u128",
    owner: "XcmVersionedLocation",
    locker: "XcmVersionedLocation",
    consumers: "Vec<(Null,u128)>",
  },
  /** Lookup575: pallet_xcm::pallet::Error<T> */
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
      "CannotCheckOutTeleport",
      "LowBalance",
      "TooManyLocks",
      "AccountNotSovereign",
      "FeesNotMet",
      "LockNotFound",
      "InUse",
      "InvalidAssetNotConcrete",
      "InvalidAssetUnknownReserve",
      "InvalidAssetUnsupportedReserve",
      "TooManyReserves",
      "LocalExecutionIncomplete",
    ],
  },
  /** Lookup576: pallet_assets::types::AssetDetails<Balance, account::AccountId20, DepositBalance> */
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
  /** Lookup577: pallet_assets::types::AssetStatus */
  PalletAssetsAssetStatus: {
    _enum: ["Live", "Frozen", "Destroying"],
  },
  /** Lookup579: pallet_assets::types::AssetAccount<Balance, DepositBalance, Extra, account::AccountId20> */
  PalletAssetsAssetAccount: {
    balance: "u128",
    status: "PalletAssetsAccountStatus",
    reason: "PalletAssetsExistenceReason",
    extra: "Null",
  },
  /** Lookup580: pallet_assets::types::AccountStatus */
  PalletAssetsAccountStatus: {
    _enum: ["Liquid", "Frozen", "Blocked"],
  },
  /** Lookup581: pallet_assets::types::ExistenceReason<Balance, account::AccountId20> */
  PalletAssetsExistenceReason: {
    _enum: {
      Consumer: "Null",
      Sufficient: "Null",
      DepositHeld: "u128",
      DepositRefunded: "Null",
      DepositFrom: "(AccountId20,u128)",
    },
  },
  /** Lookup583: pallet_assets::types::Approval<Balance, DepositBalance> */
  PalletAssetsApproval: {
    amount: "u128",
    deposit: "u128",
  },
  /**
   * Lookup584: pallet_assets::types::AssetMetadata<DepositBalance,
   * bounded_collections::bounded_vec::BoundedVec<T, S>>
   */
  PalletAssetsAssetMetadata: {
    deposit: "u128",
    name: "Bytes",
    symbol: "Bytes",
    decimals: "u8",
    isFrozen: "bool",
  },
  /** Lookup586: pallet_assets::pallet::Error<T, I> */
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
      "UnavailableConsumer",
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
      "CallbackFailed",
    ],
  },
  /** Lookup587: orml_xtokens::module::Error<T> */
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
      "NotSupportedLocation",
      "MinXcmFeeNotDefined",
    ],
  },
  /** Lookup589: pallet_asset_manager::pallet::Error<T> */
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
  /** Lookup590: pallet_migrations::pallet::Error<T> */
  PalletMigrationsError: {
    _enum: ["PreimageMissing", "WrongUpperBound", "PreimageIsTooBig", "PreimageAlreadyExists"],
  },
  /** Lookup591: pallet_xcm_transactor::relay_indices::RelayChainIndices */
  PalletXcmTransactorRelayIndicesRelayChainIndices: {
    staking: "u8",
    utility: "u8",
    hrmp: "u8",
    bond: "u8",
    bondExtra: "u8",
    unbond: "u8",
    withdrawUnbonded: "u8",
    validate: "u8",
    nominate: "u8",
    chill: "u8",
    setPayee: "u8",
    setController: "u8",
    rebond: "u8",
    asDerivative: "u8",
    initOpenChannel: "u8",
    acceptOpenChannel: "u8",
    closeChannel: "u8",
    cancelOpenRequest: "u8",
  },
  /** Lookup592: pallet_xcm_transactor::pallet::Error<T> */
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
      "RefundNotSupportedWithTransactInfo",
    ],
  },
  /** Lookup593: pallet_moonbeam_orbiters::types::CollatorPoolInfo[account::AccountId20](account::AccountId20) */
  PalletMoonbeamOrbitersCollatorPoolInfo: {
    orbiters: "Vec<AccountId20>",
    maybeCurrentOrbiter: "Option<PalletMoonbeamOrbitersCurrentOrbiter>",
    nextOrbiter: "u32",
  },
  /** Lookup595: pallet_moonbeam_orbiters::types::CurrentOrbiter[account::AccountId20](account::AccountId20) */
  PalletMoonbeamOrbitersCurrentOrbiter: {
    accountId: "AccountId20",
    removed: "bool",
  },
  /** Lookup596: pallet_moonbeam_orbiters::pallet::Error<T> */
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
  /** Lookup597: pallet_ethereum_xcm::pallet::Error<T> */
  PalletEthereumXcmError: {
    _enum: ["EthereumXcmExecutionSuspended"],
  },
  /** Lookup598: pallet_randomness::types::RequestState<T> */
  PalletRandomnessRequestState: {
    request: "PalletRandomnessRequest",
    deposit: "u128",
  },
  /** Lookup599: pallet_randomness::types::Request<Balance, pallet_randomness::types::RequestInfo<T>> */
  PalletRandomnessRequest: {
    refundAddress: "H160",
    contractAddress: "H160",
    fee: "u128",
    gasLimit: "u64",
    numWords: "u8",
    salt: "H256",
    info: "PalletRandomnessRequestInfo",
  },
  /** Lookup600: pallet_randomness::types::RequestInfo<T> */
  PalletRandomnessRequestInfo: {
    _enum: {
      BabeEpoch: "(u64,u64)",
      Local: "(u32,u32)",
    },
  },
  /** Lookup601: pallet_randomness::types::RequestType<T> */
  PalletRandomnessRequestType: {
    _enum: {
      BabeEpoch: "u64",
      Local: "u32",
    },
  },
  /** Lookup602: pallet_randomness::types::RandomnessResult<primitive_types::H256> */
  PalletRandomnessRandomnessResult: {
    randomness: "Option<H256>",
    requestCount: "u64",
  },
  /** Lookup603: pallet_randomness::pallet::Error<T> */
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
  /** Lookup605: pallet_collective::Votes<account::AccountId20, BlockNumber> */
  PalletCollectiveVotes: {
    index: "u32",
    threshold: "u32",
    ayes: "Vec<AccountId20>",
    nays: "Vec<AccountId20>",
    end: "u32",
  },
  /** Lookup606: pallet_collective::pallet::Error<T, I> */
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
      "PrimeAccountNotMember",
    ],
  },
  /**
   * Lookup608: pallet_conviction_voting::vote::Voting<Balance, account::AccountId20, BlockNumber,
   * PollIndex, MaxVotes>
   */
  PalletConvictionVotingVoteVoting: {
    _enum: {
      Casting: "PalletConvictionVotingVoteCasting",
      Delegating: "PalletConvictionVotingVoteDelegating",
    },
  },
  /** Lookup609: pallet_conviction_voting::vote::Casting<Balance, BlockNumber, PollIndex, MaxVotes> */
  PalletConvictionVotingVoteCasting: {
    votes: "Vec<(u32,PalletConvictionVotingVoteAccountVote)>",
    delegations: "PalletConvictionVotingDelegations",
    prior: "PalletConvictionVotingVotePriorLock",
  },
  /** Lookup613: pallet_conviction_voting::types::Delegations<Balance> */
  PalletConvictionVotingDelegations: {
    votes: "u128",
    capital: "u128",
  },
  /** Lookup614: pallet_conviction_voting::vote::PriorLock<BlockNumber, Balance> */
  PalletConvictionVotingVotePriorLock: "(u32,u128)",
  /** Lookup615: pallet_conviction_voting::vote::Delegating<Balance, account::AccountId20, BlockNumber> */
  PalletConvictionVotingVoteDelegating: {
    balance: "u128",
    target: "AccountId20",
    conviction: "PalletConvictionVotingConviction",
    delegations: "PalletConvictionVotingDelegations",
    prior: "PalletConvictionVotingVotePriorLock",
  },
  /** Lookup619: pallet_conviction_voting::pallet::Error<T, I> */
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
   * Lookup620: pallet_referenda::types::ReferendumInfo<TrackId, moonbase_runtime::OriginCaller,
   * Moment, frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall,
   * sp_runtime::traits::BlakeTwo256>, Balance, pallet_conviction_voting::types::Tally<Votes,
   * Total>, account::AccountId20, ScheduleAddress>
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
   * Lookup621: pallet_referenda::types::ReferendumStatus<TrackId, moonbase_runtime::OriginCaller,
   * Moment, frame_support::traits::preimages::Bounded<moonbase_runtime::RuntimeCall,
   * sp_runtime::traits::BlakeTwo256>, Balance, pallet_conviction_voting::types::Tally<Votes,
   * Total>, account::AccountId20, ScheduleAddress>
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
  /** Lookup622: pallet_referenda::types::Deposit<account::AccountId20, Balance> */
  PalletReferendaDeposit: {
    who: "AccountId20",
    amount: "u128",
  },
  /** Lookup625: pallet_referenda::types::DecidingStatus<BlockNumber> */
  PalletReferendaDecidingStatus: {
    since: "u32",
    confirming: "Option<u32>",
  },
  /** Lookup633: pallet_referenda::types::TrackInfo<Balance, Moment> */
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
  /** Lookup634: pallet_referenda::types::Curve */
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
  /** Lookup637: pallet_referenda::pallet::Error<T, I> */
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
      "PreimageNotExist",
    ],
  },
  /** Lookup638: pallet_preimage::OldRequestStatus<account::AccountId20, Balance> */
  PalletPreimageOldRequestStatus: {
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
  /**
   * Lookup641: pallet_preimage::RequestStatus<account::AccountId20,
   * frame_support::traits::tokens::fungible::HoldConsideration<A, F, R, D>>
   */
  PalletPreimageRequestStatus: {
    _enum: {
      Unrequested: {
        ticket: "(AccountId20,u128)",
        len: "u32",
      },
      Requested: {
        maybeTicket: "Option<(AccountId20,u128)>",
        count: "u32",
        maybeLen: "Option<u32>",
      },
    },
  },
  /** Lookup647: pallet_preimage::pallet::Error<T> */
  PalletPreimageError: {
    _enum: [
      "TooBig",
      "AlreadyNoted",
      "NotAuthorized",
      "NotNoted",
      "Requested",
      "NotRequested",
      "TooMany",
      "TooFew",
    ],
  },
  /** Lookup648: pallet_whitelist::pallet::Error<T> */
  PalletWhitelistError: {
    _enum: [
      "UnavailablePreImage",
      "UndecodableCall",
      "InvalidCallWeightWitness",
      "CallIsNotWhitelisted",
      "CallAlreadyWhitelisted",
    ],
  },
  /** Lookup652: pallet_multisig::Multisig<BlockNumber, Balance, account::AccountId20, MaxApprovals> */
  PalletMultisigMultisig: {
    when: "PalletMultisigTimepoint",
    deposit: "u128",
    depositor: "AccountId20",
    approvals: "Vec<AccountId20>",
  },
  /** Lookup654: pallet_multisig::pallet::Error<T> */
  PalletMultisigError: {
    _enum: [
      "MinimumThreshold",
      "AlreadyApproved",
      "NoApprovalsNeeded",
      "TooFewSignatories",
      "TooManySignatories",
      "SignatoriesOutOfOrder",
      "SenderInSignatories",
      "NotFound",
      "NotOwner",
      "NoTimepoint",
      "WrongTimepoint",
      "UnexpectedTimepoint",
      "MaxWeightTooLow",
      "AlreadyStored",
    ],
  },
  /** Lookup657: pallet_moonbeam_lazy_migrations::pallet::Error<T> */
  PalletMoonbeamLazyMigrationsError: {
    _enum: ["LimitCannotBeZero", "AddressesLengthCannotBeZero", "ContractNotCorrupted"],
  },
  /** Lookup659: pallet_precompile_benchmarks::pallet::Error<T> */
  PalletPrecompileBenchmarksError: {
    _enum: ["BenchmarkError"],
  },
  /** Lookup660: pallet_message_queue::BookState<cumulus_primitives_core::AggregateMessageOrigin> */
  PalletMessageQueueBookState: {
    _alias: {
      size_: "size",
    },
    begin: "u32",
    end: "u32",
    count: "u32",
    readyNeighbours: "Option<PalletMessageQueueNeighbours>",
    messageCount: "u64",
    size_: "u64",
  },
  /** Lookup662: pallet_message_queue::Neighbours<cumulus_primitives_core::AggregateMessageOrigin> */
  PalletMessageQueueNeighbours: {
    prev: "CumulusPrimitivesCoreAggregateMessageOrigin",
    next: "CumulusPrimitivesCoreAggregateMessageOrigin",
  },
  /** Lookup664: pallet_message_queue::Page<Size, HeapSize> */
  PalletMessageQueuePage: {
    remaining: "u32",
    remainingSize: "u32",
    firstIndex: "u32",
    first: "u32",
    last: "u32",
    heap: "Bytes",
  },
  /** Lookup666: pallet_message_queue::pallet::Error<T> */
  PalletMessageQueueError: {
    _enum: [
      "NotReapable",
      "NoPage",
      "NoMessage",
      "AlreadyProcessed",
      "Queued",
      "InsufficientWeight",
      "TemporarilyUnprocessable",
      "QueuePaused",
      "RecursiveDisallowed",
    ],
  },
  /** Lookup667: pallet_emergency_para_xcm::XcmMode */
  PalletEmergencyParaXcmXcmMode: {
    _enum: ["Normal", "Paused"],
  },
  /** Lookup668: pallet_emergency_para_xcm::pallet::Error<T> */
  PalletEmergencyParaXcmError: {
    _enum: ["NotInPausedMode"],
  },
  /** Lookup671: frame_system::extensions::check_non_zero_sender::CheckNonZeroSender<T> */
  FrameSystemExtensionsCheckNonZeroSender: "Null",
  /** Lookup672: frame_system::extensions::check_spec_version::CheckSpecVersion<T> */
  FrameSystemExtensionsCheckSpecVersion: "Null",
  /** Lookup673: frame_system::extensions::check_tx_version::CheckTxVersion<T> */
  FrameSystemExtensionsCheckTxVersion: "Null",
  /** Lookup674: frame_system::extensions::check_genesis::CheckGenesis<T> */
  FrameSystemExtensionsCheckGenesis: "Null",
  /** Lookup677: frame_system::extensions::check_nonce::CheckNonce<T> */
  FrameSystemExtensionsCheckNonce: "Compact<u32>",
  /** Lookup678: frame_system::extensions::check_weight::CheckWeight<T> */
  FrameSystemExtensionsCheckWeight: "Null",
  /** Lookup679: pallet_transaction_payment::ChargeTransactionPayment<T> */
  PalletTransactionPaymentChargeTransactionPayment: "Compact<u128>",
  /** Lookup681: moonbase_runtime::Runtime */
  MoonbaseRuntimeRuntime: "Null",
};
