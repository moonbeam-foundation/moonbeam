// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

/* eslint-disable sort-keys */

export default {
  /**
   * Lookup3: frame_system::AccountInfo<Index, pallet_balances::AccountData<Balance>>
   **/
  FrameSystemAccountInfo: {
    nonce: 'u32',
    consumers: 'u32',
    providers: 'u32',
    sufficients: 'u32',
    data: 'PalletBalancesAccountData'
  },
  /**
   * Lookup5: pallet_balances::AccountData<Balance>
   **/
  PalletBalancesAccountData: {
    free: 'u128',
    reserved: 'u128',
    miscFrozen: 'u128',
    feeFrozen: 'u128'
  },
  /**
   * Lookup7: frame_support::weights::PerDispatchClass<T>
   **/
  FrameSupportWeightsPerDispatchClassU64: {
    normal: 'u64',
    operational: 'u64',
    mandatory: 'u64'
  },
  /**
   * Lookup12: sp_runtime::generic::digest::Digest<primitive_types::H256>
   **/
  SpRuntimeDigest: {
    logs: 'Vec<SpRuntimeDigestDigestItem>'
  },
  /**
   * Lookup14: sp_runtime::generic::digest::DigestItem<primitive_types::H256>
   **/
  SpRuntimeDigestDigestItem: {
    _enum: {
      Other: 'Bytes',
      __Unused1: 'Null',
      ChangesTrieRoot: 'H256',
      __Unused3: 'Null',
      Consensus: '([u8;4],Bytes)',
      Seal: '([u8;4],Bytes)',
      PreRuntime: '([u8;4],Bytes)',
      ChangesTrieSignal: 'SpRuntimeDigestChangesTrieSignal',
      RuntimeEnvironmentUpdated: 'Null'
    }
  },
  /**
   * Lookup16: sp_runtime::generic::digest::ChangesTrieSignal
   **/
  SpRuntimeDigestChangesTrieSignal: {
    _enum: {
      NewConfiguration: 'Option<SpCoreChangesTrieChangesTrieConfiguration>'
    }
  },
  /**
   * Lookup18: sp_core::changes_trie::ChangesTrieConfiguration
   **/
  SpCoreChangesTrieChangesTrieConfiguration: {
    digestInterval: 'u32',
    digestLevels: 'u32'
  },
  /**
   * Lookup20: frame_system::EventRecord<moonbeam_runtime::Event, primitive_types::H256>
   **/
  FrameSystemEventRecord: {
    phase: 'FrameSystemPhase',
    event: 'Event',
    topics: 'Vec<H256>'
  },
  /**
   * Lookup22: frame_system::pallet::Event<T>
   **/
  FrameSystemEvent: {
    _enum: {
      ExtrinsicSuccess: 'FrameSupportWeightsDispatchInfo',
      ExtrinsicFailed: '(SpRuntimeDispatchError,FrameSupportWeightsDispatchInfo)',
      CodeUpdated: 'Null',
      NewAccount: 'AccountId20',
      KilledAccount: 'AccountId20',
      Remarked: '(AccountId20,H256)'
    }
  },
  /**
   * Lookup23: frame_support::weights::DispatchInfo
   **/
  FrameSupportWeightsDispatchInfo: {
    weight: 'u64',
    class: 'FrameSupportWeightsDispatchClass',
    paysFee: 'FrameSupportWeightsPays'
  },
  /**
   * Lookup24: frame_support::weights::DispatchClass
   **/
  FrameSupportWeightsDispatchClass: {
    _enum: ['Normal', 'Operational', 'Mandatory']
  },
  /**
   * Lookup25: frame_support::weights::Pays
   **/
  FrameSupportWeightsPays: {
    _enum: ['Yes', 'No']
  },
  /**
   * Lookup26: sp_runtime::DispatchError
   **/
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
      Arithmetic: 'SpRuntimeArithmeticError'
    }
  },
  /**
   * Lookup27: sp_runtime::TokenError
   **/
  SpRuntimeTokenError: {
    _enum: ['NoFunds', 'WouldDie', 'BelowMinimum', 'CannotCreate', 'UnknownAsset', 'Frozen', 'Unsupported']
  },
  /**
   * Lookup28: sp_runtime::ArithmeticError
   **/
  SpRuntimeArithmeticError: {
    _enum: ['Underflow', 'Overflow', 'DivisionByZero']
  },
  /**
   * Lookup29: cumulus_pallet_parachain_system::pallet::Event<T>
   **/
  CumulusPalletParachainSystemEvent: {
    _enum: {
      ValidationFunctionStored: 'Null',
      ValidationFunctionApplied: 'u32',
      ValidationFunctionDiscarded: 'Null',
      UpgradeAuthorized: 'H256',
      DownwardMessagesReceived: 'u32',
      DownwardMessagesProcessed: '(u64,H256)'
    }
  },
  /**
   * Lookup30: pallet_balances::pallet::Event<T, I>
   **/
  PalletBalancesEvent: {
    _enum: {
      Endowed: '(AccountId20,u128)',
      DustLost: '(AccountId20,u128)',
      Transfer: '(AccountId20,AccountId20,u128)',
      BalanceSet: '(AccountId20,u128,u128)',
      Reserved: '(AccountId20,u128)',
      Unreserved: '(AccountId20,u128)',
      ReserveRepatriated: '(AccountId20,AccountId20,u128,FrameSupportTokensMiscBalanceStatus)',
      Deposit: '(AccountId20,u128)',
      Withdraw: '(AccountId20,u128)',
      Slashed: '(AccountId20,u128)'
    }
  },
  /**
   * Lookup31: frame_support::traits::tokens::misc::BalanceStatus
   **/
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ['Free', 'Reserved']
  },
  /**
   * Lookup32: parachain_staking::pallet::Event<T>
   **/
  ParachainStakingEvent: {
    _enum: {
      NewRound: '(u32,u32,u32,u128)',
      JoinedCollatorCandidates: '(AccountId20,u128,u128)',
      CollatorChosen: '(u32,AccountId20,u128)',
      CandidateBondLessRequested: '(AccountId20,u128,u32)',
      CandidateBondedMore: '(AccountId20,u128,u128)',
      CandidateBondedLess: '(AccountId20,u128,u128)',
      CandidateWentOffline: '(u32,AccountId20)',
      CandidateBackOnline: '(u32,AccountId20)',
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
      BlocksPerRoundSet: '(u32,u32,u32,u32,Perbill,Perbill,Perbill)'
    }
  },
  /**
   * Lookup34: parachain_staking::pallet::DelegationRequest<account::AccountId20, Balance>
   **/
  ParachainStakingDelegationRequest: {
    collator: 'AccountId20',
    amount: 'u128',
    whenExecutable: 'u32',
    action: 'ParachainStakingDelegationChange'
  },
  /**
   * Lookup35: parachain_staking::pallet::DelegationChange
   **/
  ParachainStakingDelegationChange: {
    _enum: ['Revoke', 'Decrease']
  },
  /**
   * Lookup36: parachain_staking::pallet::DelegatorAdded<B>
   **/
  ParachainStakingDelegatorAdded: {
    _enum: {
      AddedToTop: {
        newTotal: 'u128',
      },
      AddedToBottom: 'Null'
    }
  },
  /**
   * Lookup39: pallet_author_slot_filter::pallet::Event
   **/
  PalletAuthorSlotFilterEvent: {
    _enum: {
      EligibleUpdated: 'Percent'
    }
  },
  /**
   * Lookup40: pallet_author_mapping::pallet::Event<T>
   **/
  PalletAuthorMappingEvent: {
    _enum: {
      AuthorRegistered: '(NimbusPrimitivesNimbusCryptoPublic,AccountId20)',
      AuthorDeRegistered: 'NimbusPrimitivesNimbusCryptoPublic',
      AuthorRotated: '(NimbusPrimitivesNimbusCryptoPublic,AccountId20)',
      DefunctAuthorBusted: '(NimbusPrimitivesNimbusCryptoPublic,AccountId20)'
    }
  },
  /**
   * Lookup41: nimbus_primitives::nimbus_crypto::Public
   **/
  NimbusPrimitivesNimbusCryptoPublic: 'SpCoreSr25519Public',
  /**
   * Lookup42: sp_core::sr25519::Public
   **/
  SpCoreSr25519Public: '[u8;32]',
  /**
   * Lookup43: pallet_utility::pallet::Event
   **/
  PalletUtilityEvent: {
    _enum: {
      BatchInterrupted: '(u32,SpRuntimeDispatchError)',
      BatchCompleted: 'Null',
      ItemCompleted: 'Null'
    }
  },
  /**
   * Lookup44: pallet_proxy::pallet::Event<T>
   **/
  PalletProxyEvent: {
    _enum: {
      ProxyExecuted: 'Result<Null, SpRuntimeDispatchError>',
      AnonymousCreated: '(AccountId20,AccountId20,MoonbeamRuntimeProxyType,u16)',
      Announced: '(AccountId20,AccountId20,H256)',
      ProxyAdded: '(AccountId20,AccountId20,MoonbeamRuntimeProxyType,u32)'
    }
  },
  /**
   * Lookup47: moonbeam_runtime::ProxyType
   **/
  MoonbeamRuntimeProxyType: {
    _enum: ['Any', 'NonTransfer', 'Governance', 'Staking', 'CancelProxy', 'Balances', 'AuthorMapping']
  },
  /**
   * Lookup49: pallet_maintenance_mode::pallet::Event
   **/
  PalletMaintenanceModeEvent: {
    _enum: ['EnteredMaintenanceMode', 'NormalOperationResumed']
  },
  /**
   * Lookup50: pallet_identity::pallet::Event<T>
   **/
  PalletIdentityEvent: {
    _enum: {
      IdentitySet: 'AccountId20',
      IdentityCleared: '(AccountId20,u128)',
      IdentityKilled: '(AccountId20,u128)',
      JudgementRequested: '(AccountId20,u32)',
      JudgementUnrequested: '(AccountId20,u32)',
      JudgementGiven: '(AccountId20,u32)',
      RegistrarAdded: 'u32',
      SubIdentityAdded: '(AccountId20,AccountId20,u128)',
      SubIdentityRemoved: '(AccountId20,AccountId20,u128)',
      SubIdentityRevoked: '(AccountId20,AccountId20,u128)'
    }
  },
  /**
   * Lookup51: pallet_migrations::pallet::Event<T>
   **/
  PalletMigrationsEvent: {
    _enum: {
      RuntimeUpgradeStarted: 'Null',
      RuntimeUpgradeCompleted: 'u64',
      MigrationStarted: 'Bytes',
      MigrationCompleted: '(Bytes,u64)'
    }
  },
  /**
   * Lookup52: pallet_evm::pallet::Event<T>
   **/
  PalletEvmEvent: {
    _enum: {
      Log: 'EthereumLog',
      Created: 'H160',
      CreatedFailed: 'H160',
      Executed: 'H160',
      ExecutedFailed: 'H160',
      BalanceDeposit: '(AccountId20,H160,U256)',
      BalanceWithdraw: '(AccountId20,H160,U256)'
    }
  },
  /**
   * Lookup53: ethereum::log::Log
   **/
  EthereumLog: {
    address: 'H160',
    topics: 'Vec<H256>',
    data: 'Bytes'
  },
  /**
   * Lookup58: pallet_ethereum::pallet::Event
   **/
  PalletEthereumEvent: {
    _enum: {
      Executed: '(H160,H160,H256,EvmCoreErrorExitReason)'
    }
  },
  /**
   * Lookup59: evm_core::error::ExitReason
   **/
  EvmCoreErrorExitReason: {
    _enum: {
      Succeed: 'EvmCoreErrorExitSucceed',
      Error: 'EvmCoreErrorExitError',
      Revert: 'EvmCoreErrorExitRevert',
      Fatal: 'EvmCoreErrorExitFatal'
    }
  },
  /**
   * Lookup60: evm_core::error::ExitSucceed
   **/
  EvmCoreErrorExitSucceed: {
    _enum: ['Stopped', 'Returned', 'Suicided']
  },
  /**
   * Lookup61: evm_core::error::ExitError
   **/
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
      Other: 'Text'
    }
  },
  /**
   * Lookup64: evm_core::error::ExitRevert
   **/
  EvmCoreErrorExitRevert: {
    _enum: ['Reverted']
  },
  /**
   * Lookup65: evm_core::error::ExitFatal
   **/
  EvmCoreErrorExitFatal: {
    _enum: {
      NotSupported: 'Null',
      UnhandledInterrupt: 'Null',
      CallErrorAsFatal: 'EvmCoreErrorExitError',
      Other: 'Text'
    }
  },
  /**
   * Lookup66: pallet_scheduler::pallet::Event<T>
   **/
  PalletSchedulerEvent: {
    _enum: {
      Scheduled: '(u32,u32)',
      Canceled: '(u32,u32)',
      Dispatched: '((u32,u32),Option<Bytes>,Result<Null, SpRuntimeDispatchError>)'
    }
  },
  /**
   * Lookup69: pallet_democracy::pallet::Event<T>
   **/
  PalletDemocracyEvent: {
    _enum: {
      Proposed: '(u32,u128)',
      Tabled: '(u32,u128,Vec<AccountId20>)',
      ExternalTabled: 'Null',
      Started: '(u32,PalletDemocracyVoteThreshold)',
      Passed: 'u32',
      NotPassed: 'u32',
      Cancelled: 'u32',
      Executed: '(u32,Result<Null, SpRuntimeDispatchError>)',
      Delegated: '(AccountId20,AccountId20)',
      Undelegated: 'AccountId20',
      Vetoed: '(AccountId20,H256,u32)',
      PreimageNoted: '(H256,AccountId20,u128)',
      PreimageUsed: '(H256,AccountId20,u128)',
      PreimageInvalid: '(H256,u32)',
      PreimageMissing: '(H256,u32)',
      PreimageReaped: '(H256,AccountId20,u128,AccountId20)',
      Blacklisted: 'H256',
      Voted: '(AccountId20,u32,PalletDemocracyVoteAccountVote)',
      Seconded: '(AccountId20,u32)'
    }
  },
  /**
   * Lookup71: pallet_democracy::vote_threshold::VoteThreshold
   **/
  PalletDemocracyVoteThreshold: {
    _enum: ['SuperMajorityApprove', 'SuperMajorityAgainst', 'SimpleMajority']
  },
  /**
   * Lookup72: pallet_democracy::vote::AccountVote<Balance>
   **/
  PalletDemocracyVoteAccountVote: {
    _enum: {
      Standard: {
        vote: 'Vote',
        balance: 'u128',
      },
      Split: {
        aye: 'u128',
        nay: 'u128'
      }
    }
  },
  /**
   * Lookup74: pallet_collective::pallet::Event<T, I>
   **/
  PalletCollectiveEvent: {
    _enum: {
      Proposed: '(AccountId20,u32,H256,u32)',
      Voted: '(AccountId20,H256,bool,u32,u32)',
      Approved: 'H256',
      Disapproved: 'H256',
      Executed: '(H256,Result<Null, SpRuntimeDispatchError>)',
      MemberExecuted: '(H256,Result<Null, SpRuntimeDispatchError>)',
      Closed: '(H256,u32,u32)'
    }
  },
  /**
   * Lookup76: pallet_treasury::pallet::Event<T, I>
   **/
  PalletTreasuryEvent: {
    _enum: {
      Proposed: 'u32',
      Spending: 'u128',
      Awarded: '(u32,u128,AccountId20)',
      Rejected: '(u32,u128)',
      Burnt: 'u128',
      Rollover: 'u128',
      Deposit: 'u128'
    }
  },
  /**
   * Lookup77: pallet_crowdloan_rewards::pallet::Event<T>
   **/
  PalletCrowdloanRewardsEvent: {
    _enum: {
      InitialPaymentMade: '(AccountId20,u128)',
      NativeIdentityAssociated: '([u8;32],AccountId20,u128)',
      RewardsPaid: '(AccountId20,u128)',
      RewardAddressUpdated: '(AccountId20,AccountId20)',
      InitializedAlreadyInitializedAccount: '([u8;32],Option<AccountId20>,u128)',
      InitializedAccountWithNotEnoughContribution: '([u8;32],Option<AccountId20>,u128)'
    }
  },
  /**
   * Lookup79: frame_system::Phase
   **/
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: 'u32',
      Finalization: 'Null',
      Initialization: 'Null'
    }
  },
  /**
   * Lookup81: frame_system::LastRuntimeUpgradeInfo
   **/
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: 'Compact<u32>',
    specName: 'Text'
  },
  /**
   * Lookup83: frame_system::pallet::Call<T>
   **/
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
      set_changes_trie_config: {
        changesTrieConfig: 'Option<SpCoreChangesTrieChangesTrieConfiguration>',
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
        remark: 'Bytes'
      }
    }
  },
  /**
   * Lookup87: frame_system::limits::BlockWeights
   **/
  FrameSystemLimitsBlockWeights: {
    baseBlock: 'u64',
    maxBlock: 'u64',
    perClass: 'FrameSupportWeightsPerDispatchClassWeightsPerClass'
  },
  /**
   * Lookup88: frame_support::weights::PerDispatchClass<frame_system::limits::WeightsPerClass>
   **/
  FrameSupportWeightsPerDispatchClassWeightsPerClass: {
    normal: 'FrameSystemLimitsWeightsPerClass',
    operational: 'FrameSystemLimitsWeightsPerClass',
    mandatory: 'FrameSystemLimitsWeightsPerClass'
  },
  /**
   * Lookup89: frame_system::limits::WeightsPerClass
   **/
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: 'u64',
    maxExtrinsic: 'Option<u64>',
    maxTotal: 'Option<u64>',
    reserved: 'Option<u64>'
  },
  /**
   * Lookup91: frame_system::limits::BlockLength
   **/
  FrameSystemLimitsBlockLength: {
    max: 'FrameSupportWeightsPerDispatchClassU32'
  },
  /**
   * Lookup92: frame_support::weights::PerDispatchClass<T>
   **/
  FrameSupportWeightsPerDispatchClassU32: {
    normal: 'u32',
    operational: 'u32',
    mandatory: 'u32'
  },
  /**
   * Lookup93: frame_support::weights::RuntimeDbWeight
   **/
  FrameSupportWeightsRuntimeDbWeight: {
    read: 'u64',
    write: 'u64'
  },
  /**
   * Lookup94: sp_version::RuntimeVersion
   **/
  SpVersionRuntimeVersion: {
    specName: 'Text',
    implName: 'Text',
    authoringVersion: 'u32',
    specVersion: 'u32',
    implVersion: 'u32',
    apis: 'Vec<([u8;8],u32)>',
    transactionVersion: 'u32'
  },
  /**
   * Lookup99: frame_system::pallet::Error<T>
   **/
  FrameSystemError: {
    _enum: ['InvalidSpecName', 'SpecVersionNeedsToIncrease', 'FailedToExtractRuntimeVersion', 'NonDefaultComposite', 'NonZeroRefCount']
  },
  /**
   * Lookup100: polkadot_primitives::v1::PersistedValidationData<primitive_types::H256, N>
   **/
  PolkadotPrimitivesV1PersistedValidationData: {
    parentHead: 'Bytes',
    relayParentNumber: 'u32',
    relayParentStorageRoot: 'H256',
    maxPovSize: 'u32'
  },
  /**
   * Lookup103: polkadot_primitives::v1::UpgradeRestriction
   **/
  PolkadotPrimitivesV1UpgradeRestriction: {
    _enum: ['Present']
  },
  /**
   * Lookup104: cumulus_pallet_parachain_system::relay_state_snapshot::MessagingStateSnapshot
   **/
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: {
    dmqMqcHead: 'H256',
    relayDispatchQueueSize: '(u32,u32)',
    ingressChannels: 'Vec<(u32,PolkadotPrimitivesV1AbridgedHrmpChannel)>',
    egressChannels: 'Vec<(u32,PolkadotPrimitivesV1AbridgedHrmpChannel)>'
  },
  /**
   * Lookup108: polkadot_primitives::v1::AbridgedHrmpChannel
   **/
  PolkadotPrimitivesV1AbridgedHrmpChannel: {
    maxCapacity: 'u32',
    maxTotalSize: 'u32',
    maxMessageSize: 'u32',
    msgCount: 'u32',
    totalSize: 'u32',
    mqcHead: 'Option<H256>'
  },
  /**
   * Lookup110: polkadot_primitives::v1::AbridgedHostConfiguration
   **/
  PolkadotPrimitivesV1AbridgedHostConfiguration: {
    maxCodeSize: 'u32',
    maxHeadDataSize: 'u32',
    maxUpwardQueueCount: 'u32',
    maxUpwardQueueSize: 'u32',
    maxUpwardMessageSize: 'u32',
    maxUpwardMessageNumPerCandidate: 'u32',
    hrmpMaxMessageNumPerCandidate: 'u32',
    validationUpgradeFrequency: 'u32',
    validationUpgradeDelay: 'u32'
  },
  /**
   * Lookup116: polkadot_core_primitives::OutboundHrmpMessage<polkadot_parachain::primitives::Id>
   **/
  PolkadotCorePrimitivesOutboundHrmpMessage: {
    recipient: 'u32',
    data: 'Bytes'
  },
  /**
   * Lookup117: cumulus_pallet_parachain_system::pallet::Call<T>
   **/
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
        code: 'Bytes'
      }
    }
  },
  /**
   * Lookup118: cumulus_primitives_parachain_inherent::ParachainInherentData
   **/
  CumulusPrimitivesParachainInherentParachainInherentData: {
    validationData: 'PolkadotPrimitivesV1PersistedValidationData',
    relayChainState: 'SpTrieStorageProof',
    downwardMessages: 'Vec<PolkadotCorePrimitivesInboundDownwardMessage>',
    horizontalMessages: 'BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>'
  },
  /**
   * Lookup119: sp_trie::storage_proof::StorageProof
   **/
  SpTrieStorageProof: {
    trieNodes: 'Vec<Bytes>'
  },
  /**
   * Lookup121: polkadot_core_primitives::InboundDownwardMessage<BlockNumber>
   **/
  PolkadotCorePrimitivesInboundDownwardMessage: {
    sentAt: 'u32',
    msg: 'Bytes'
  },
  /**
   * Lookup124: polkadot_core_primitives::InboundHrmpMessage<BlockNumber>
   **/
  PolkadotCorePrimitivesInboundHrmpMessage: {
    sentAt: 'u32',
    data: 'Bytes'
  },
  /**
   * Lookup127: cumulus_pallet_parachain_system::pallet::Error<T>
   **/
  CumulusPalletParachainSystemError: {
    _enum: ['OverlappingUpgrades', 'ProhibitedByPolkadot', 'TooBig', 'ValidationDataNotAvailable', 'HostConfigurationNotAvailable', 'NotScheduled', 'NothingAuthorized', 'Unauthorized']
  },
  /**
   * Lookup128: pallet_timestamp::pallet::Call<T>
   **/
  PalletTimestampCall: {
    _enum: {
      set: {
        now: 'Compact<u64>'
      }
    }
  },
  /**
   * Lookup131: pallet_balances::BalanceLock<Balance>
   **/
  PalletBalancesBalanceLock: {
    id: '[u8;8]',
    amount: 'u128',
    reasons: 'PalletBalancesReasons'
  },
  /**
   * Lookup132: pallet_balances::Reasons
   **/
  PalletBalancesReasons: {
    _enum: ['Fee', 'Misc', 'All']
  },
  /**
   * Lookup135: pallet_balances::ReserveData<ReserveIdentifier, Balance>
   **/
  PalletBalancesReserveData: {
    id: '[u8;4]',
    amount: 'u128'
  },
  /**
   * Lookup137: pallet_balances::Releases
   **/
  PalletBalancesReleases: {
    _enum: ['V1_0_0', 'V2_0_0']
  },
  /**
   * Lookup138: pallet_balances::pallet::Call<T, I>
   **/
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
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup140: pallet_balances::pallet::Error<T, I>
   **/
  PalletBalancesError: {
    _enum: ['VestingBalance', 'LiquidityRestrictions', 'InsufficientBalance', 'ExistentialDeposit', 'KeepAlive', 'ExistingVestingSchedule', 'DeadAccount', 'TooManyReserves']
  },
  /**
   * Lookup142: pallet_transaction_payment::Releases
   **/
  PalletTransactionPaymentReleases: {
    _enum: ['V1Ancient', 'V2']
  },
  /**
   * Lookup144: frame_support::weights::WeightToFeeCoefficient<Balance>
   **/
  FrameSupportWeightsWeightToFeeCoefficient: {
    coeffInteger: 'u128',
    coeffFrac: 'Perbill',
    negative: 'bool',
    degree: 'u8'
  },
  /**
   * Lookup145: parachain_staking::pallet::ParachainBondConfig<account::AccountId20>
   **/
  ParachainStakingParachainBondConfig: {
    account: 'AccountId20',
    percent: 'Percent'
  },
  /**
   * Lookup146: parachain_staking::pallet::RoundInfo<BlockNumber>
   **/
  ParachainStakingRoundInfo: {
    current: 'u32',
    first: 'u32',
    length: 'u32'
  },
  /**
   * Lookup147: parachain_staking::pallet::Nominator2<account::AccountId20, Balance>
   **/
  ParachainStakingNominator2: {
    delegations: 'ParachainStakingSetOrderedSetBond',
    revocations: 'ParachainStakingSetOrderedSetAccountId20',
    total: 'u128',
    scheduledRevocationsCount: 'u32',
    scheduledRevocationsTotal: 'u128',
    status: 'ParachainStakingDelegatorStatus'
  },
  /**
   * Lookup148: parachain_staking::set::OrderedSet<parachain_staking::pallet::Bond<account::AccountId20, Balance>>
   **/
  ParachainStakingSetOrderedSetBond: 'Vec<ParachainStakingBond>',
  /**
   * Lookup149: parachain_staking::pallet::Bond<account::AccountId20, Balance>
   **/
  ParachainStakingBond: {
    owner: 'AccountId20',
    amount: 'u128'
  },
  /**
   * Lookup151: parachain_staking::set::OrderedSet<account::AccountId20>
   **/
  ParachainStakingSetOrderedSetAccountId20: 'Vec<AccountId20>',
  /**
   * Lookup152: parachain_staking::pallet::DelegatorStatus
   **/
  ParachainStakingDelegatorStatus: {
    _enum: {
      Active: 'Null',
      Leaving: 'u32'
    }
  },
  /**
   * Lookup153: parachain_staking::pallet::Delegator<account::AccountId20, Balance>
   **/
  ParachainStakingDelegator: {
    id: 'AccountId20',
    delegations: 'ParachainStakingSetOrderedSetBond',
    total: 'u128',
    requests: 'ParachainStakingPendingDelegationRequests',
    status: 'ParachainStakingDelegatorStatus'
  },
  /**
   * Lookup154: parachain_staking::pallet::PendingDelegationRequests<account::AccountId20, Balance>
   **/
  ParachainStakingPendingDelegationRequests: {
    revocationsCount: 'u32',
    requests: 'BTreeMap<AccountId20, ParachainStakingDelegationRequest>',
    lessTotal: 'u128'
  },
  /**
   * Lookup158: parachain_staking::pallet::CollatorCandidate<account::AccountId20, Balance>
   **/
  ParachainStakingCollatorCandidate: {
    id: 'AccountId20',
    bond: 'u128',
    delegators: 'ParachainStakingSetOrderedSetAccountId20',
    topDelegations: 'Vec<ParachainStakingBond>',
    bottomDelegations: 'Vec<ParachainStakingBond>',
    totalCounted: 'u128',
    totalBacking: 'u128',
    request: 'Option<ParachainStakingCandidateBondLessRequest>',
    state: 'ParachainStakingCollatorStatus'
  },
  /**
   * Lookup160: parachain_staking::pallet::CandidateBondLessRequest<Balance>
   **/
  ParachainStakingCandidateBondLessRequest: {
    amount: 'u128',
    whenExecutable: 'u32'
  },
  /**
   * Lookup161: parachain_staking::pallet::CollatorStatus
   **/
  ParachainStakingCollatorStatus: {
    _enum: {
      Active: 'Null',
      Idle: 'Null',
      Leaving: 'u32'
    }
  },
  /**
   * Lookup162: parachain_staking::pallet::Collator2<account::AccountId20, Balance>
   **/
  ParachainStakingCollator2: {
    id: 'AccountId20',
    bond: 'u128',
    nominators: 'ParachainStakingSetOrderedSetAccountId20',
    topNominators: 'Vec<ParachainStakingBond>',
    bottomNominators: 'Vec<ParachainStakingBond>',
    totalCounted: 'u128',
    totalBacking: 'u128',
    state: 'ParachainStakingCollatorStatus'
  },
  /**
   * Lookup163: parachain_staking::pallet::ExitQ<account::AccountId20>
   **/
  ParachainStakingExitQ: {
    candidates: 'ParachainStakingSetOrderedSetAccountId20',
    nominatorsLeaving: 'ParachainStakingSetOrderedSetAccountId20',
    candidateSchedule: 'Vec<(AccountId20,u32)>',
    nominatorSchedule: 'Vec<(AccountId20,Option<AccountId20>,u32)>'
  },
  /**
   * Lookup169: parachain_staking::pallet::CollatorSnapshot<account::AccountId20, Balance>
   **/
  ParachainStakingCollatorSnapshot: {
    bond: 'u128',
    delegations: 'Vec<ParachainStakingBond>',
    total: 'u128'
  },
  /**
   * Lookup170: parachain_staking::pallet::DelayedPayout<Balance>
   **/
  ParachainStakingDelayedPayout: {
    roundIssuance: 'u128',
    totalStakingReward: 'u128',
    collatorCommission: 'Perbill'
  },
  /**
   * Lookup171: parachain_staking::inflation::InflationInfo<Balance>
   **/
  ParachainStakingInflationInflationInfo: {
    expect: 'ParachainStakingInflationRangeU128',
    annual: 'ParachainStakingInflationRangePerbill',
    round: 'ParachainStakingInflationRangePerbill'
  },
  /**
   * Lookup172: parachain_staking::inflation::Range<T>
   **/
  ParachainStakingInflationRangeU128: {
    min: 'u128',
    ideal: 'u128',
    max: 'u128'
  },
  /**
   * Lookup173: parachain_staking::inflation::Range<sp_arithmetic::per_things::Perbill>
   **/
  ParachainStakingInflationRangePerbill: {
    min: 'Perbill',
    ideal: 'Perbill',
    max: 'Perbill'
  },
  /**
   * Lookup174: parachain_staking::pallet::Call<T>
   **/
  ParachainStakingCall: {
    _enum: {
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
        collator: 'AccountId20',
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
        candidate: 'AccountId20'
      }
    }
  },
  /**
   * Lookup175: parachain_staking::pallet::Error<T>
   **/
  ParachainStakingError: {
    _enum: ['DelegatorDNE', 'DelegatorDNEinTopNorBottom', 'DelegatorDNEInDelegatorSet', 'CandidateDNE', 'DelegationDNE', 'DelegatorExists', 'CandidateExists', 'CandidateBondBelowMin', 'InsufficientBalance', 'DelegatorBondBelowMin', 'DelegationBelowMin', 'AlreadyOffline', 'AlreadyActive', 'DelegatorAlreadyLeaving', 'DelegatorNotLeaving', 'DelegatorCannotLeaveYet', 'CannotDelegateIfLeaving', 'CandidateAlreadyLeaving', 'CandidateNotLeaving', 'CandidateCannotLeaveYet', 'CannotGoOnlineIfLeaving', 'ExceedMaxDelegationsPerDelegator', 'AlreadyDelegatedCandidate', 'InvalidSchedule', 'CannotSetBelowMin', 'RoundLengthMustBeAtLeastTotalSelectedCollators', 'NoWritingSameValue', 'TooLowCandidateCountWeightHintJoinCandidates', 'TooLowCandidateCountWeightHintCancelLeaveCandidates', 'TooLowCandidateCountToLeaveCandidates', 'TooLowDelegationCountToDelegate', 'TooLowCandidateDelegationCountToDelegate', 'TooLowDelegationCountToLeaveDelegators', 'PendingCandidateRequestsDNE', 'PendingCandidateRequestAlreadyExists', 'PendingCandidateRequestNotDueYet', 'PendingDelegationRequestDNE', 'PendingDelegationRequestAlreadyExists', 'PendingDelegationRequestNotDueYet']
  },
  /**
   * Lookup176: pallet_author_inherent::pallet::Call<T>
   **/
  PalletAuthorInherentCall: {
    _enum: ['kick_off_authorship_validation']
  },
  /**
   * Lookup177: pallet_author_inherent::pallet::Error<T>
   **/
  PalletAuthorInherentError: {
    _enum: ['AuthorAlreadySet', 'NoAccountId', 'CannotBeAuthor']
  },
  /**
   * Lookup178: pallet_author_slot_filter::pallet::Call<T>
   **/
  PalletAuthorSlotFilterCall: {
    _enum: {
      set_eligible: {
        _alias: {
          new_: 'new',
        },
        new_: 'Percent'
      }
    }
  },
  /**
   * Lookup179: pallet_author_mapping::pallet::RegistrationInfo<account::AccountId20, Balance>
   **/
  PalletAuthorMappingRegistrationInfo: {
    account: 'AccountId20',
    deposit: 'u128'
  },
  /**
   * Lookup180: pallet_author_mapping::pallet::Call<T>
   **/
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
        authorId: 'NimbusPrimitivesNimbusCryptoPublic'
      }
    }
  },
  /**
   * Lookup181: pallet_author_mapping::pallet::Error<T>
   **/
  PalletAuthorMappingError: {
    _enum: ['AssociationNotFound', 'NotYourAssociation', 'CannotAffordSecurityDeposit', 'AlreadyAssociated']
  },
  /**
   * Lookup182: pallet_utility::pallet::Call<T>
   **/
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
        calls: 'Vec<Call>'
      }
    }
  },
  /**
   * Lookup185: pallet_proxy::pallet::Call<T>
   **/
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
        call: 'Call'
      }
    }
  },
  /**
   * Lookup187: pallet_maintenance_mode::pallet::Call<T>
   **/
  PalletMaintenanceModeCall: {
    _enum: ['enter_maintenance_mode', 'resume_normal_operation']
  },
  /**
   * Lookup188: pallet_identity::pallet::Call<T>
   **/
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
      quit_sub: 'Null'
    }
  },
  /**
   * Lookup189: pallet_identity::types::IdentityInfo<FieldLimit>
   **/
  PalletIdentityIdentityInfo: {
    additional: 'Vec<(Data,Data)>',
    display: 'Data',
    legal: 'Data',
    web: 'Data',
    riot: 'Data',
    email: 'Data',
    pgpFingerprint: 'Option<[u8;20]>',
    image: 'Data',
    twitter: 'Data'
  },
  /**
   * Lookup226: pallet_identity::types::BitFlags<pallet_identity::types::IdentityField>
   **/
  PalletIdentityBitFlags: {
    _bitLength: 64,
    Display: 1,
    Legal: 2,
    Web: 4,
    Riot: 8,
    Email: 16,
    PgpFingerprint: 32,
    Image: 64,
    Twitter: 128
  },
  /**
   * Lookup227: pallet_identity::types::IdentityField
   **/
  PalletIdentityIdentityField: {
    _enum: ['__Unused0', 'Display', 'Legal', '__Unused3', 'Web', '__Unused5', '__Unused6', '__Unused7', 'Riot', '__Unused9', '__Unused10', '__Unused11', '__Unused12', '__Unused13', '__Unused14', '__Unused15', 'Email', '__Unused17', '__Unused18', '__Unused19', '__Unused20', '__Unused21', '__Unused22', '__Unused23', '__Unused24', '__Unused25', '__Unused26', '__Unused27', '__Unused28', '__Unused29', '__Unused30', '__Unused31', 'PgpFingerprint', '__Unused33', '__Unused34', '__Unused35', '__Unused36', '__Unused37', '__Unused38', '__Unused39', '__Unused40', '__Unused41', '__Unused42', '__Unused43', '__Unused44', '__Unused45', '__Unused46', '__Unused47', '__Unused48', '__Unused49', '__Unused50', '__Unused51', '__Unused52', '__Unused53', '__Unused54', '__Unused55', '__Unused56', '__Unused57', '__Unused58', '__Unused59', '__Unused60', '__Unused61', '__Unused62', '__Unused63', 'Image', '__Unused65', '__Unused66', '__Unused67', '__Unused68', '__Unused69', '__Unused70', '__Unused71', '__Unused72', '__Unused73', '__Unused74', '__Unused75', '__Unused76', '__Unused77', '__Unused78', '__Unused79', '__Unused80', '__Unused81', '__Unused82', '__Unused83', '__Unused84', '__Unused85', '__Unused86', '__Unused87', '__Unused88', '__Unused89', '__Unused90', '__Unused91', '__Unused92', '__Unused93', '__Unused94', '__Unused95', '__Unused96', '__Unused97', '__Unused98', '__Unused99', '__Unused100', '__Unused101', '__Unused102', '__Unused103', '__Unused104', '__Unused105', '__Unused106', '__Unused107', '__Unused108', '__Unused109', '__Unused110', '__Unused111', '__Unused112', '__Unused113', '__Unused114', '__Unused115', '__Unused116', '__Unused117', '__Unused118', '__Unused119', '__Unused120', '__Unused121', '__Unused122', '__Unused123', '__Unused124', '__Unused125', '__Unused126', '__Unused127', 'Twitter']
  },
  /**
   * Lookup228: pallet_identity::types::Judgement<Balance>
   **/
  PalletIdentityJudgement: {
    _enum: {
      Unknown: 'Null',
      FeePaid: 'u128',
      Reasonable: 'Null',
      KnownGood: 'Null',
      OutOfDate: 'Null',
      LowQuality: 'Null',
      Erroneous: 'Null'
    }
  },
  /**
   * Lookup229: pallet_evm::pallet::Call<T>
   **/
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
        gasPrice: 'U256',
        nonce: 'Option<U256>',
      },
      create: {
        source: 'H160',
        init: 'Bytes',
        value: 'U256',
        gasLimit: 'u64',
        gasPrice: 'U256',
        nonce: 'Option<U256>',
      },
      create2: {
        source: 'H160',
        init: 'Bytes',
        salt: 'H256',
        value: 'U256',
        gasLimit: 'u64',
        gasPrice: 'U256',
        nonce: 'Option<U256>'
      }
    }
  },
  /**
   * Lookup231: pallet_ethereum::pallet::Call<T>
   **/
  PalletEthereumCall: {
    _enum: {
      transact: {
        transaction: 'EthereumTransactionLegacyTransaction'
      }
    }
  },
  /**
   * Lookup232: ethereum::transaction::LegacyTransaction
   **/
  EthereumTransactionLegacyTransaction: {
    nonce: 'U256',
    gasPrice: 'U256',
    gasLimit: 'U256',
    action: 'EthereumTransactionTransactionAction',
    value: 'U256',
    input: 'Bytes',
    signature: 'EthereumTransactionTransactionSignature'
  },
  /**
   * Lookup233: ethereum::transaction::TransactionAction
   **/
  EthereumTransactionTransactionAction: {
    _enum: {
      Call: 'H160',
      Create: 'Null'
    }
  },
  /**
   * Lookup234: ethereum::transaction::TransactionSignature
   **/
  EthereumTransactionTransactionSignature: {
    v: 'u64',
    r: 'H256',
    s: 'H256'
  },
  /**
   * Lookup236: pallet_scheduler::pallet::Call<T>
   **/
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
        call: 'Call'
      }
    }
  },
  /**
   * Lookup238: pallet_democracy::pallet::Call<T>
   **/
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
        propIndex: 'Compact<u32>'
      }
    }
  },
  /**
   * Lookup239: pallet_democracy::conviction::Conviction
   **/
  PalletDemocracyConviction: {
    _enum: ['None', 'Locked1x', 'Locked2x', 'Locked3x', 'Locked4x', 'Locked5x', 'Locked6x']
  },
  /**
   * Lookup241: pallet_collective::pallet::Call<T, I>
   **/
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
        proposalHash: 'H256'
      }
    }
  },
  /**
   * Lookup243: pallet_treasury::pallet::Call<T, I>
   **/
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
        proposalId: 'Compact<u32>'
      }
    }
  },
  /**
   * Lookup244: pallet_crowdloan_rewards::pallet::Call<T>
   **/
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
        rewards: 'Vec<([u8;32],Option<AccountId20>,u128)>'
      }
    }
  },
  /**
   * Lookup245: sp_runtime::MultiSignature
   **/
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: 'SpCoreEd25519Signature',
      Sr25519: 'SpCoreSr25519Signature',
      Ecdsa: 'SpCoreEcdsaSignature'
    }
  },
  /**
   * Lookup246: sp_core::ed25519::Signature
   **/
  SpCoreEd25519Signature: '[u8;64]',
  /**
   * Lookup248: sp_core::sr25519::Signature
   **/
  SpCoreSr25519Signature: '[u8;64]',
  /**
   * Lookup249: sp_core::ecdsa::Signature
   **/
  SpCoreEcdsaSignature: '[u8;65]',
  /**
   * Lookup255: pallet_utility::pallet::Error<T>
   **/
  PalletUtilityError: {
    _enum: ['TooManyCalls']
  },
  /**
   * Lookup258: pallet_proxy::ProxyDefinition<account::AccountId20, moonbeam_runtime::ProxyType, BlockNumber>
   **/
  PalletProxyProxyDefinition: {
    delegate: 'AccountId20',
    proxyType: 'MoonbeamRuntimeProxyType',
    delay: 'u32'
  },
  /**
   * Lookup262: pallet_proxy::Announcement<account::AccountId20, primitive_types::H256, BlockNumber>
   **/
  PalletProxyAnnouncement: {
    real: 'AccountId20',
    callHash: 'H256',
    height: 'u32'
  },
  /**
   * Lookup264: pallet_proxy::pallet::Error<T>
   **/
  PalletProxyError: {
    _enum: ['TooMany', 'NotFound', 'NotProxy', 'Unproxyable', 'Duplicate', 'NoPermission', 'Unannounced', 'NoSelfProxy']
  },
  /**
   * Lookup265: pallet_maintenance_mode::pallet::Error<T>
   **/
  PalletMaintenanceModeError: {
    _enum: ['AlreadyInMaintenanceMode', 'NotInMaintenanceMode']
  },
  /**
   * Lookup266: pallet_identity::types::Registration<Balance, MaxJudgements, MaxAdditionalFields>
   **/
  PalletIdentityRegistration: {
    judgements: 'Vec<(u32,PalletIdentityJudgement)>',
    deposit: 'u128',
    info: 'PalletIdentityIdentityInfo'
  },
  /**
   * Lookup274: pallet_identity::types::RegistrarInfo<Balance, account::AccountId20>
   **/
  PalletIdentityRegistrarInfo: {
    account: 'AccountId20',
    fee: 'u128',
    fields: 'PalletIdentityBitFlags'
  },
  /**
   * Lookup276: pallet_identity::pallet::Error<T>
   **/
  PalletIdentityError: {
    _enum: ['TooManySubAccounts', 'NotFound', 'NotNamed', 'EmptyIndex', 'FeeChanged', 'NoIdentity', 'StickyJudgement', 'JudgementGiven', 'InvalidJudgement', 'InvalidIndex', 'InvalidTarget', 'TooManyFields', 'TooManyRegistrars', 'AlreadyClaimed', 'NotSub', 'NotOwned']
  },
  /**
   * Lookup278: pallet_evm::pallet::Error<T>
   **/
  PalletEvmError: {
    _enum: ['BalanceLow', 'FeeOverflow', 'PaymentOverflow', 'WithdrawFailed', 'GasPriceTooLow', 'InvalidNonce']
  },
  /**
   * Lookup281: fp_rpc::TransactionStatus
   **/
  FpRpcTransactionStatus: {
    transactionHash: 'H256',
    transactionIndex: 'u32',
    from: 'H160',
    to: 'Option<H160>',
    contractAddress: 'Option<H160>',
    logs: 'Vec<EthereumLog>',
    logsBloom: 'EthbloomBloom'
  },
  /**
   * Lookup284: ethbloom::Bloom
   **/
  EthbloomBloom: '[u8;256]',
  /**
   * Lookup286: ethereum::receipt::Receipt
   **/
  EthereumReceipt: {
    stateRoot: 'H256',
    usedGas: 'U256',
    logsBloom: 'EthbloomBloom',
    logs: 'Vec<EthereumLog>'
  },
  /**
   * Lookup287: ethereum::block::Block<ethereum::transaction::LegacyTransaction>
   **/
  EthereumBlock: {
    header: 'EthereumHeader',
    transactions: 'Vec<EthereumTransactionLegacyTransaction>',
    ommers: 'Vec<EthereumHeader>'
  },
  /**
   * Lookup288: ethereum::header::Header
   **/
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
    nonce: 'EthereumTypesHashH64'
  },
  /**
   * Lookup289: ethereum_types::hash::H64
   **/
  EthereumTypesHashH64: '[u8;8]',
  /**
   * Lookup294: pallet_ethereum::pallet::Error<T>
   **/
  PalletEthereumError: {
    _enum: ['InvalidSignature', 'PreLogExists']
  },
  /**
   * Lookup297: pallet_scheduler::ScheduledV2<moonbeam_runtime::Call, BlockNumber, moonbeam_runtime::OriginCaller, account::AccountId20>
   **/
  PalletSchedulerScheduledV2: {
    maybeId: 'Option<Bytes>',
    priority: 'u8',
    call: 'Call',
    maybePeriodic: 'Option<(u32,u32)>',
    origin: 'MoonbeamRuntimeOriginCaller'
  },
  /**
   * Lookup298: moonbeam_runtime::OriginCaller
   **/
  MoonbeamRuntimeOriginCaller: {
    _enum: {
      system: 'FrameSystemRawOrigin',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      Void: 'SpCoreVoid',
      __Unused5: 'Null',
      __Unused6: 'Null',
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
      TechCommitteeCollective: 'PalletCollectiveRawOrigin'
    }
  },
  /**
   * Lookup299: frame_system::RawOrigin<account::AccountId20>
   **/
  FrameSystemRawOrigin: {
    _enum: {
      Root: 'Null',
      Signed: 'AccountId20',
      None: 'Null'
    }
  },
  /**
   * Lookup300: pallet_ethereum::RawOrigin
   **/
  PalletEthereumRawOrigin: {
    _enum: {
      EthereumTransaction: 'H160'
    }
  },
  /**
   * Lookup301: pallet_collective::RawOrigin<account::AccountId20, I>
   **/
  PalletCollectiveRawOrigin: {
    _enum: {
      Members: '(u32,u32)',
      Member: 'AccountId20',
      _Phantom: 'Null'
    }
  },
  /**
   * Lookup303: sp_core::Void
   **/
  SpCoreVoid: 'Null',
  /**
   * Lookup304: pallet_scheduler::Releases
   **/
  PalletSchedulerReleases: {
    _enum: ['V1', 'V2']
  },
  /**
   * Lookup305: pallet_scheduler::pallet::Error<T>
   **/
  PalletSchedulerError: {
    _enum: ['FailedToSchedule', 'NotFound', 'TargetBlockNumberInPast', 'RescheduleNoChange']
  },
  /**
   * Lookup309: pallet_democracy::PreimageStatus<account::AccountId20, Balance, BlockNumber>
   **/
  PalletDemocracyPreimageStatus: {
    _enum: {
      Missing: 'u32',
      Available: {
        data: 'Bytes',
        provider: 'AccountId20',
        deposit: 'u128',
        since: 'u32',
        expiry: 'Option<u32>'
      }
    }
  },
  /**
   * Lookup310: pallet_democracy::types::ReferendumInfo<BlockNumber, primitive_types::H256, Balance>
   **/
  PalletDemocracyReferendumInfo: {
    _enum: {
      Ongoing: 'PalletDemocracyReferendumStatus',
      Finished: {
        approved: 'bool',
        end: 'u32'
      }
    }
  },
  /**
   * Lookup311: pallet_democracy::types::ReferendumStatus<BlockNumber, primitive_types::H256, Balance>
   **/
  PalletDemocracyReferendumStatus: {
    end: 'u32',
    proposalHash: 'H256',
    threshold: 'PalletDemocracyVoteThreshold',
    delay: 'u32',
    tally: 'PalletDemocracyTally'
  },
  /**
   * Lookup312: pallet_democracy::types::Tally<Balance>
   **/
  PalletDemocracyTally: {
    ayes: 'u128',
    nays: 'u128',
    turnout: 'u128'
  },
  /**
   * Lookup313: pallet_democracy::vote::Voting<Balance, account::AccountId20, BlockNumber>
   **/
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
        prior: 'PalletDemocracyVotePriorLock'
      }
    }
  },
  /**
   * Lookup316: pallet_democracy::types::Delegations<Balance>
   **/
  PalletDemocracyDelegations: {
    votes: 'u128',
    capital: 'u128'
  },
  /**
   * Lookup317: pallet_democracy::vote::PriorLock<BlockNumber, Balance>
   **/
  PalletDemocracyVotePriorLock: '(u32,u128)',
  /**
   * Lookup320: pallet_democracy::Releases
   **/
  PalletDemocracyReleases: {
    _enum: ['V1']
  },
  /**
   * Lookup321: pallet_democracy::pallet::Error<T>
   **/
  PalletDemocracyError: {
    _enum: ['ValueLow', 'ProposalMissing', 'AlreadyCanceled', 'DuplicateProposal', 'ProposalBlacklisted', 'NotSimpleMajority', 'InvalidHash', 'NoProposal', 'AlreadyVetoed', 'DuplicatePreimage', 'NotImminent', 'TooEarly', 'Imminent', 'PreimageMissing', 'ReferendumInvalid', 'PreimageInvalid', 'NoneWaiting', 'NotVoter', 'NoPermission', 'AlreadyDelegating', 'InsufficientFunds', 'NotDelegating', 'VotesExist', 'InstantNotAllowed', 'Nonsense', 'WrongUpperBound', 'MaxVotesReached', 'TooManyProposals']
  },
  /**
   * Lookup323: pallet_collective::Votes<account::AccountId20, BlockNumber>
   **/
  PalletCollectiveVotes: {
    index: 'u32',
    threshold: 'u32',
    ayes: 'Vec<AccountId20>',
    nays: 'Vec<AccountId20>',
    end: 'u32'
  },
  /**
   * Lookup324: pallet_collective::pallet::Error<T, I>
   **/
  PalletCollectiveError: {
    _enum: ['NotMember', 'DuplicateProposal', 'ProposalMissing', 'WrongIndex', 'DuplicateVote', 'AlreadyInitialized', 'TooEarly', 'TooManyProposals', 'WrongProposalWeight', 'WrongProposalLength']
  },
  /**
   * Lookup327: pallet_treasury::Proposal<account::AccountId20, Balance>
   **/
  PalletTreasuryProposal: {
    proposer: 'AccountId20',
    value: 'u128',
    beneficiary: 'AccountId20',
    bond: 'u128'
  },
  /**
   * Lookup331: frame_support::PalletId
   **/
  FrameSupportPalletId: '[u8;8]',
  /**
   * Lookup332: pallet_treasury::pallet::Error<T, I>
   **/
  PalletTreasuryError: {
    _enum: ['InsufficientProposersBalance', 'InvalidIndex', 'TooManyApprovals']
  },
  /**
   * Lookup333: pallet_crowdloan_rewards::pallet::RewardInfo<T>
   **/
  PalletCrowdloanRewardsRewardInfo: {
    totalReward: 'u128',
    claimedReward: 'u128',
    contributedRelayAddresses: 'Vec<[u8;32]>'
  },
  /**
   * Lookup335: pallet_crowdloan_rewards::pallet::Error<T>
   **/
  PalletCrowdloanRewardsError: {
    _enum: ['AlreadyAssociated', 'BatchBeyondFundPot', 'FirstClaimAlreadyDone', 'RewardNotHighEnough', 'InvalidClaimSignature', 'InvalidFreeClaimSignature', 'NoAssociatedClaim', 'RewardsAlreadyClaimed', 'RewardVecAlreadyInitialized', 'RewardVecNotFullyInitializedYet', 'RewardsDoNotMatchFund', 'TooManyContributors', 'VestingPeriodNonValid', 'NonContributedAddressProvided', 'InsufficientNumberOfValidProofs']
  },
  /**
   * Lookup337: account::EthereumSignature
   **/
  AccountEthereumSignature: 'SpCoreEcdsaSignature',
  /**
   * Lookup339: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
   **/
  FrameSystemExtensionsCheckSpecVersion: 'Null',
  /**
   * Lookup340: frame_system::extensions::check_tx_version::CheckTxVersion<T>
   **/
  FrameSystemExtensionsCheckTxVersion: 'Null',
  /**
   * Lookup341: frame_system::extensions::check_genesis::CheckGenesis<T>
   **/
  FrameSystemExtensionsCheckGenesis: 'Null',
  /**
   * Lookup344: frame_system::extensions::check_nonce::CheckNonce<T>
   **/
  FrameSystemExtensionsCheckNonce: 'Compact<u32>',
  /**
   * Lookup345: frame_system::extensions::check_weight::CheckWeight<T>
   **/
  FrameSystemExtensionsCheckWeight: 'Null',
  /**
   * Lookup346: pallet_transaction_payment::ChargeTransactionPayment<T>
   **/
  PalletTransactionPaymentChargeTransactionPayment: 'Compact<u128>',
  /**
   * Lookup348: moonbeam_runtime::Runtime
   **/
  MoonbeamRuntimeRuntime: 'Null'
};
