declare const _default: {
    /**
     * Lookup3: frame_system::AccountInfo<Nonce, pallet_balances::types::AccountData<Balance>>
     **/
    FrameSystemAccountInfo: {
        nonce: string;
        consumers: string;
        providers: string;
        sufficients: string;
        data: string;
    };
    /**
     * Lookup5: pallet_balances::types::AccountData<Balance>
     **/
    PalletBalancesAccountData: {
        free: string;
        reserved: string;
        frozen: string;
        flags: string;
    };
    /**
     * Lookup9: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight>
     **/
    FrameSupportDispatchPerDispatchClassWeight: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup10: sp_weights::weight_v2::Weight
     **/
    SpWeightsWeightV2Weight: {
        refTime: string;
        proofSize: string;
    };
    /**
     * Lookup16: sp_runtime::generic::digest::Digest
     **/
    SpRuntimeDigest: {
        logs: string;
    };
    /**
     * Lookup18: sp_runtime::generic::digest::DigestItem
     **/
    SpRuntimeDigestDigestItem: {
        _enum: {
            Other: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            Consensus: string;
            Seal: string;
            PreRuntime: string;
            __Unused7: string;
            RuntimeEnvironmentUpdated: string;
        };
    };
    /**
     * Lookup21: frame_system::EventRecord<moonriver_runtime::RuntimeEvent, primitive_types::H256>
     **/
    FrameSystemEventRecord: {
        phase: string;
        event: string;
        topics: string;
    };
    /**
     * Lookup23: frame_system::pallet::Event<T>
     **/
    FrameSystemEvent: {
        _enum: {
            ExtrinsicSuccess: {
                dispatchInfo: string;
            };
            ExtrinsicFailed: {
                dispatchError: string;
                dispatchInfo: string;
            };
            CodeUpdated: string;
            NewAccount: {
                account: string;
            };
            KilledAccount: {
                account: string;
            };
            Remarked: {
                _alias: {
                    hash_: string;
                };
                sender: string;
                hash_: string;
            };
            UpgradeAuthorized: {
                codeHash: string;
                checkVersion: string;
            };
        };
    };
    /**
     * Lookup24: frame_support::dispatch::DispatchInfo
     **/
    FrameSupportDispatchDispatchInfo: {
        weight: string;
        class: string;
        paysFee: string;
    };
    /**
     * Lookup25: frame_support::dispatch::DispatchClass
     **/
    FrameSupportDispatchDispatchClass: {
        _enum: string[];
    };
    /**
     * Lookup26: frame_support::dispatch::Pays
     **/
    FrameSupportDispatchPays: {
        _enum: string[];
    };
    /**
     * Lookup27: sp_runtime::DispatchError
     **/
    SpRuntimeDispatchError: {
        _enum: {
            Other: string;
            CannotLookup: string;
            BadOrigin: string;
            Module: string;
            ConsumerRemaining: string;
            NoProviders: string;
            TooManyConsumers: string;
            Token: string;
            Arithmetic: string;
            Transactional: string;
            Exhausted: string;
            Corruption: string;
            Unavailable: string;
            RootNotAllowed: string;
        };
    };
    /**
     * Lookup28: sp_runtime::ModuleError
     **/
    SpRuntimeModuleError: {
        index: string;
        error: string;
    };
    /**
     * Lookup29: sp_runtime::TokenError
     **/
    SpRuntimeTokenError: {
        _enum: string[];
    };
    /**
     * Lookup30: sp_arithmetic::ArithmeticError
     **/
    SpArithmeticArithmeticError: {
        _enum: string[];
    };
    /**
     * Lookup31: sp_runtime::TransactionalError
     **/
    SpRuntimeTransactionalError: {
        _enum: string[];
    };
    /**
     * Lookup32: cumulus_pallet_parachain_system::pallet::Event<T>
     **/
    CumulusPalletParachainSystemEvent: {
        _enum: {
            ValidationFunctionStored: string;
            ValidationFunctionApplied: {
                relayChainBlockNum: string;
            };
            ValidationFunctionDiscarded: string;
            DownwardMessagesReceived: {
                count: string;
            };
            DownwardMessagesProcessed: {
                weightUsed: string;
                dmqHead: string;
            };
            UpwardMessageSent: {
                messageHash: string;
            };
        };
    };
    /**
     * Lookup34: pallet_root_testing::pallet::Event<T>
     **/
    PalletRootTestingEvent: {
        _enum: string[];
    };
    /**
     * Lookup35: pallet_balances::pallet::Event<T, I>
     **/
    PalletBalancesEvent: {
        _enum: {
            Endowed: {
                account: string;
                freeBalance: string;
            };
            DustLost: {
                account: string;
                amount: string;
            };
            Transfer: {
                from: string;
                to: string;
                amount: string;
            };
            BalanceSet: {
                who: string;
                free: string;
            };
            Reserved: {
                who: string;
                amount: string;
            };
            Unreserved: {
                who: string;
                amount: string;
            };
            ReserveRepatriated: {
                from: string;
                to: string;
                amount: string;
                destinationStatus: string;
            };
            Deposit: {
                who: string;
                amount: string;
            };
            Withdraw: {
                who: string;
                amount: string;
            };
            Slashed: {
                who: string;
                amount: string;
            };
            Minted: {
                who: string;
                amount: string;
            };
            Burned: {
                who: string;
                amount: string;
            };
            Suspended: {
                who: string;
                amount: string;
            };
            Restored: {
                who: string;
                amount: string;
            };
            Upgraded: {
                who: string;
            };
            Issued: {
                amount: string;
            };
            Rescinded: {
                amount: string;
            };
            Locked: {
                who: string;
                amount: string;
            };
            Unlocked: {
                who: string;
                amount: string;
            };
            Frozen: {
                who: string;
                amount: string;
            };
            Thawed: {
                who: string;
                amount: string;
            };
            TotalIssuanceForced: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
        };
    };
    /**
     * Lookup36: frame_support::traits::tokens::misc::BalanceStatus
     **/
    FrameSupportTokensMiscBalanceStatus: {
        _enum: string[];
    };
    /**
     * Lookup37: pallet_transaction_payment::pallet::Event<T>
     **/
    PalletTransactionPaymentEvent: {
        _enum: {
            TransactionFeePaid: {
                who: string;
                actualFee: string;
                tip: string;
            };
        };
    };
    /**
     * Lookup38: pallet_parachain_staking::pallet::Event<T>
     **/
    PalletParachainStakingEvent: {
        _enum: {
            NewRound: {
                startingBlock: string;
                round: string;
                selectedCollatorsNumber: string;
                totalBalance: string;
            };
            JoinedCollatorCandidates: {
                account: string;
                amountLocked: string;
                newTotalAmtLocked: string;
            };
            CollatorChosen: {
                round: string;
                collatorAccount: string;
                totalExposedAmount: string;
            };
            CandidateBondLessRequested: {
                candidate: string;
                amountToDecrease: string;
                executeRound: string;
            };
            CandidateBondedMore: {
                candidate: string;
                amount: string;
                newTotalBond: string;
            };
            CandidateBondedLess: {
                candidate: string;
                amount: string;
                newBond: string;
            };
            CandidateWentOffline: {
                candidate: string;
            };
            CandidateBackOnline: {
                candidate: string;
            };
            CandidateScheduledExit: {
                exitAllowedRound: string;
                candidate: string;
                scheduledExit: string;
            };
            CancelledCandidateExit: {
                candidate: string;
            };
            CancelledCandidateBondLess: {
                candidate: string;
                amount: string;
                executeRound: string;
            };
            CandidateLeft: {
                exCandidate: string;
                unlockedAmount: string;
                newTotalAmtLocked: string;
            };
            DelegationDecreaseScheduled: {
                delegator: string;
                candidate: string;
                amountToDecrease: string;
                executeRound: string;
            };
            DelegationIncreased: {
                delegator: string;
                candidate: string;
                amount: string;
                inTop: string;
            };
            DelegationDecreased: {
                delegator: string;
                candidate: string;
                amount: string;
                inTop: string;
            };
            DelegatorExitScheduled: {
                round: string;
                delegator: string;
                scheduledExit: string;
            };
            DelegationRevocationScheduled: {
                round: string;
                delegator: string;
                candidate: string;
                scheduledExit: string;
            };
            DelegatorLeft: {
                delegator: string;
                unstakedAmount: string;
            };
            DelegationRevoked: {
                delegator: string;
                candidate: string;
                unstakedAmount: string;
            };
            DelegationKicked: {
                delegator: string;
                candidate: string;
                unstakedAmount: string;
            };
            DelegatorExitCancelled: {
                delegator: string;
            };
            CancelledDelegationRequest: {
                delegator: string;
                cancelledRequest: string;
                collator: string;
            };
            Delegation: {
                delegator: string;
                lockedAmount: string;
                candidate: string;
                delegatorPosition: string;
                autoCompound: string;
            };
            DelegatorLeftCandidate: {
                delegator: string;
                candidate: string;
                unstakedAmount: string;
                totalCandidateStaked: string;
            };
            Rewarded: {
                account: string;
                rewards: string;
            };
            InflationDistributed: {
                index: string;
                account: string;
                value: string;
            };
            InflationDistributionConfigUpdated: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
            InflationSet: {
                annualMin: string;
                annualIdeal: string;
                annualMax: string;
                roundMin: string;
                roundIdeal: string;
                roundMax: string;
            };
            StakeExpectationsSet: {
                expectMin: string;
                expectIdeal: string;
                expectMax: string;
            };
            TotalSelectedSet: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
            CollatorCommissionSet: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
            BlocksPerRoundSet: {
                _alias: {
                    new_: string;
                };
                currentRound: string;
                firstBlock: string;
                old: string;
                new_: string;
                newPerRoundInflationMin: string;
                newPerRoundInflationIdeal: string;
                newPerRoundInflationMax: string;
            };
            AutoCompoundSet: {
                candidate: string;
                delegator: string;
                value: string;
            };
            Compounded: {
                candidate: string;
                delegator: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup39: pallet_parachain_staking::delegation_requests::CancelledScheduledRequest<Balance>
     **/
    PalletParachainStakingDelegationRequestsCancelledScheduledRequest: {
        whenExecutable: string;
        action: string;
    };
    /**
     * Lookup40: pallet_parachain_staking::delegation_requests::DelegationAction<Balance>
     **/
    PalletParachainStakingDelegationRequestsDelegationAction: {
        _enum: {
            Revoke: string;
            Decrease: string;
        };
    };
    /**
     * Lookup41: pallet_parachain_staking::types::DelegatorAdded<B>
     **/
    PalletParachainStakingDelegatorAdded: {
        _enum: {
            AddedToTop: {
                newTotal: string;
            };
            AddedToBottom: string;
        };
    };
    /**
     * Lookup43: pallet_parachain_staking::types::InflationDistributionConfig<account::AccountId20>
     **/
    PalletParachainStakingInflationDistributionConfig: string;
    /**
     * Lookup45: pallet_parachain_staking::types::InflationDistributionAccount<account::AccountId20>
     **/
    PalletParachainStakingInflationDistributionAccount: {
        account: string;
        percent: string;
    };
    /**
     * Lookup47: pallet_author_slot_filter::pallet::Event
     **/
    PalletAuthorSlotFilterEvent: {
        _enum: {
            EligibleUpdated: string;
        };
    };
    /**
     * Lookup49: pallet_author_mapping::pallet::Event<T>
     **/
    PalletAuthorMappingEvent: {
        _enum: {
            KeysRegistered: {
                _alias: {
                    keys_: string;
                };
                nimbusId: string;
                accountId: string;
                keys_: string;
            };
            KeysRemoved: {
                _alias: {
                    keys_: string;
                };
                nimbusId: string;
                accountId: string;
                keys_: string;
            };
            KeysRotated: {
                newNimbusId: string;
                accountId: string;
                newKeys: string;
            };
        };
    };
    /**
     * Lookup50: nimbus_primitives::nimbus_crypto::Public
     **/
    NimbusPrimitivesNimbusCryptoPublic: string;
    /**
     * Lookup51: session_keys_primitives::vrf::vrf_crypto::Public
     **/
    SessionKeysPrimitivesVrfVrfCryptoPublic: string;
    /**
     * Lookup52: pallet_moonbeam_orbiters::pallet::Event<T>
     **/
    PalletMoonbeamOrbitersEvent: {
        _enum: {
            OrbiterJoinCollatorPool: {
                collator: string;
                orbiter: string;
            };
            OrbiterLeaveCollatorPool: {
                collator: string;
                orbiter: string;
            };
            OrbiterRewarded: {
                account: string;
                rewards: string;
            };
            OrbiterRotation: {
                collator: string;
                oldOrbiter: string;
                newOrbiter: string;
            };
            OrbiterRegistered: {
                account: string;
                deposit: string;
            };
            OrbiterUnregistered: {
                account: string;
            };
        };
    };
    /**
     * Lookup54: pallet_utility::pallet::Event
     **/
    PalletUtilityEvent: {
        _enum: {
            BatchInterrupted: {
                index: string;
                error: string;
            };
            BatchCompleted: string;
            BatchCompletedWithErrors: string;
            ItemCompleted: string;
            ItemFailed: {
                error: string;
            };
            DispatchedAs: {
                result: string;
            };
        };
    };
    /**
     * Lookup57: pallet_proxy::pallet::Event<T>
     **/
    PalletProxyEvent: {
        _enum: {
            ProxyExecuted: {
                result: string;
            };
            PureCreated: {
                pure: string;
                who: string;
                proxyType: string;
                disambiguationIndex: string;
            };
            Announced: {
                real: string;
                proxy: string;
                callHash: string;
            };
            ProxyAdded: {
                delegator: string;
                delegatee: string;
                proxyType: string;
                delay: string;
            };
            ProxyRemoved: {
                delegator: string;
                delegatee: string;
                proxyType: string;
                delay: string;
            };
        };
    };
    /**
     * Lookup58: moonriver_runtime::ProxyType
     **/
    MoonriverRuntimeProxyType: {
        _enum: string[];
    };
    /**
     * Lookup60: pallet_maintenance_mode::pallet::Event
     **/
    PalletMaintenanceModeEvent: {
        _enum: {
            EnteredMaintenanceMode: string;
            NormalOperationResumed: string;
            FailedToSuspendIdleXcmExecution: {
                error: string;
            };
            FailedToResumeIdleXcmExecution: {
                error: string;
            };
        };
    };
    /**
     * Lookup61: pallet_identity::pallet::Event<T>
     **/
    PalletIdentityEvent: {
        _enum: {
            IdentitySet: {
                who: string;
            };
            IdentityCleared: {
                who: string;
                deposit: string;
            };
            IdentityKilled: {
                who: string;
                deposit: string;
            };
            JudgementRequested: {
                who: string;
                registrarIndex: string;
            };
            JudgementUnrequested: {
                who: string;
                registrarIndex: string;
            };
            JudgementGiven: {
                target: string;
                registrarIndex: string;
            };
            RegistrarAdded: {
                registrarIndex: string;
            };
            SubIdentityAdded: {
                sub: string;
                main: string;
                deposit: string;
            };
            SubIdentityRemoved: {
                sub: string;
                main: string;
                deposit: string;
            };
            SubIdentityRevoked: {
                sub: string;
                main: string;
                deposit: string;
            };
            AuthorityAdded: {
                authority: string;
            };
            AuthorityRemoved: {
                authority: string;
            };
            UsernameSet: {
                who: string;
                username: string;
            };
            UsernameQueued: {
                who: string;
                username: string;
                expiration: string;
            };
            PreapprovalExpired: {
                whose: string;
            };
            PrimaryUsernameSet: {
                who: string;
                username: string;
            };
            DanglingUsernameRemoved: {
                who: string;
                username: string;
            };
        };
    };
    /**
     * Lookup63: pallet_migrations::pallet::Event<T>
     **/
    PalletMigrationsEvent: {
        _enum: {
            RuntimeUpgradeStarted: string;
            RuntimeUpgradeCompleted: {
                weight: string;
            };
            MigrationStarted: {
                migrationName: string;
            };
            MigrationCompleted: {
                migrationName: string;
                consumedWeight: string;
            };
            FailedToSuspendIdleXcmExecution: {
                error: string;
            };
            FailedToResumeIdleXcmExecution: {
                error: string;
            };
        };
    };
    /**
     * Lookup64: pallet_multisig::pallet::Event<T>
     **/
    PalletMultisigEvent: {
        _enum: {
            NewMultisig: {
                approving: string;
                multisig: string;
                callHash: string;
            };
            MultisigApproval: {
                approving: string;
                timepoint: string;
                multisig: string;
                callHash: string;
            };
            MultisigExecuted: {
                approving: string;
                timepoint: string;
                multisig: string;
                callHash: string;
                result: string;
            };
            MultisigCancelled: {
                cancelling: string;
                timepoint: string;
                multisig: string;
                callHash: string;
            };
        };
    };
    /**
     * Lookup65: pallet_multisig::Timepoint<BlockNumber>
     **/
    PalletMultisigTimepoint: {
        height: string;
        index: string;
    };
    /**
     * Lookup66: pallet_parameters::pallet::Event<T>
     **/
    PalletParametersEvent: {
        _enum: {
            Updated: {
                key: string;
                oldValue: string;
                newValue: string;
            };
        };
    };
    /**
     * Lookup67: moonriver_runtime::runtime_params::RuntimeParametersKey
     **/
    MoonriverRuntimeRuntimeParamsRuntimeParametersKey: {
        _enum: {
            RuntimeConfig: string;
            PalletRandomness: string;
        };
    };
    /**
     * Lookup68: moonriver_runtime::runtime_params::dynamic_params::runtime_config::ParametersKey
     **/
    MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersKey: {
        _enum: string[];
    };
    /**
     * Lookup69: moonriver_runtime::runtime_params::dynamic_params::runtime_config::FeesTreasuryProportion
     **/
    MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigFeesTreasuryProportion: string;
    /**
     * Lookup70: moonriver_runtime::runtime_params::dynamic_params::pallet_randomness::ParametersKey
     **/
    MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersKey: {
        _enum: string[];
    };
    /**
     * Lookup71: moonriver_runtime::runtime_params::dynamic_params::pallet_randomness::Deposit
     **/
    MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessDeposit: string;
    /**
     * Lookup73: moonriver_runtime::runtime_params::RuntimeParametersValue
     **/
    MoonriverRuntimeRuntimeParamsRuntimeParametersValue: {
        _enum: {
            RuntimeConfig: string;
            PalletRandomness: string;
        };
    };
    /**
     * Lookup74: moonriver_runtime::runtime_params::dynamic_params::runtime_config::ParametersValue
     **/
    MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersValue: {
        _enum: {
            FeesTreasuryProportion: string;
        };
    };
    /**
     * Lookup75: moonriver_runtime::runtime_params::dynamic_params::pallet_randomness::ParametersValue
     **/
    MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersValue: {
        _enum: {
            Deposit: string;
        };
    };
    /**
     * Lookup77: pallet_evm::pallet::Event<T>
     **/
    PalletEvmEvent: {
        _enum: {
            Log: {
                log: string;
            };
            Created: {
                address: string;
            };
            CreatedFailed: {
                address: string;
            };
            Executed: {
                address: string;
            };
            ExecutedFailed: {
                address: string;
            };
        };
    };
    /**
     * Lookup78: ethereum::log::Log
     **/
    EthereumLog: {
        address: string;
        topics: string;
        data: string;
    };
    /**
     * Lookup81: pallet_ethereum::pallet::Event
     **/
    PalletEthereumEvent: {
        _enum: {
            Executed: {
                from: string;
                to: string;
                transactionHash: string;
                exitReason: string;
                extraData: string;
            };
        };
    };
    /**
     * Lookup82: evm_core::error::ExitReason
     **/
    EvmCoreErrorExitReason: {
        _enum: {
            Succeed: string;
            Error: string;
            Revert: string;
            Fatal: string;
        };
    };
    /**
     * Lookup83: evm_core::error::ExitSucceed
     **/
    EvmCoreErrorExitSucceed: {
        _enum: string[];
    };
    /**
     * Lookup84: evm_core::error::ExitError
     **/
    EvmCoreErrorExitError: {
        _enum: {
            StackUnderflow: string;
            StackOverflow: string;
            InvalidJump: string;
            InvalidRange: string;
            DesignatedInvalid: string;
            CallTooDeep: string;
            CreateCollision: string;
            CreateContractLimit: string;
            OutOfOffset: string;
            OutOfGas: string;
            OutOfFund: string;
            PCUnderflow: string;
            CreateEmpty: string;
            Other: string;
            MaxNonce: string;
            InvalidCode: string;
        };
    };
    /**
     * Lookup88: evm_core::error::ExitRevert
     **/
    EvmCoreErrorExitRevert: {
        _enum: string[];
    };
    /**
     * Lookup89: evm_core::error::ExitFatal
     **/
    EvmCoreErrorExitFatal: {
        _enum: {
            NotSupported: string;
            UnhandledInterrupt: string;
            CallErrorAsFatal: string;
            Other: string;
        };
    };
    /**
     * Lookup90: pallet_scheduler::pallet::Event<T>
     **/
    PalletSchedulerEvent: {
        _enum: {
            Scheduled: {
                when: string;
                index: string;
            };
            Canceled: {
                when: string;
                index: string;
            };
            Dispatched: {
                task: string;
                id: string;
                result: string;
            };
            RetrySet: {
                task: string;
                id: string;
                period: string;
                retries: string;
            };
            RetryCancelled: {
                task: string;
                id: string;
            };
            CallUnavailable: {
                task: string;
                id: string;
            };
            PeriodicFailed: {
                task: string;
                id: string;
            };
            RetryFailed: {
                task: string;
                id: string;
            };
            PermanentlyOverweight: {
                task: string;
                id: string;
            };
        };
    };
    /**
     * Lookup92: pallet_preimage::pallet::Event<T>
     **/
    PalletPreimageEvent: {
        _enum: {
            Noted: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Requested: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Cleared: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
        };
    };
    /**
     * Lookup93: pallet_conviction_voting::pallet::Event<T, I>
     **/
    PalletConvictionVotingEvent: {
        _enum: {
            Delegated: string;
            Undelegated: string;
        };
    };
    /**
     * Lookup94: pallet_referenda::pallet::Event<T, I>
     **/
    PalletReferendaEvent: {
        _enum: {
            Submitted: {
                index: string;
                track: string;
                proposal: string;
            };
            DecisionDepositPlaced: {
                index: string;
                who: string;
                amount: string;
            };
            DecisionDepositRefunded: {
                index: string;
                who: string;
                amount: string;
            };
            DepositSlashed: {
                who: string;
                amount: string;
            };
            DecisionStarted: {
                index: string;
                track: string;
                proposal: string;
                tally: string;
            };
            ConfirmStarted: {
                index: string;
            };
            ConfirmAborted: {
                index: string;
            };
            Confirmed: {
                index: string;
                tally: string;
            };
            Approved: {
                index: string;
            };
            Rejected: {
                index: string;
                tally: string;
            };
            TimedOut: {
                index: string;
                tally: string;
            };
            Cancelled: {
                index: string;
                tally: string;
            };
            Killed: {
                index: string;
                tally: string;
            };
            SubmissionDepositRefunded: {
                index: string;
                who: string;
                amount: string;
            };
            MetadataSet: {
                _alias: {
                    hash_: string;
                };
                index: string;
                hash_: string;
            };
            MetadataCleared: {
                _alias: {
                    hash_: string;
                };
                index: string;
                hash_: string;
            };
        };
    };
    /**
     * Lookup95: frame_support::traits::preimages::Bounded<moonriver_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>
     **/
    FrameSupportPreimagesBounded: {
        _enum: {
            Legacy: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Inline: string;
            Lookup: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
                len: string;
            };
        };
    };
    /**
     * Lookup97: frame_system::pallet::Call<T>
     **/
    FrameSystemCall: {
        _enum: {
            remark: {
                remark: string;
            };
            set_heap_pages: {
                pages: string;
            };
            set_code: {
                code: string;
            };
            set_code_without_checks: {
                code: string;
            };
            set_storage: {
                items: string;
            };
            kill_storage: {
                _alias: {
                    keys_: string;
                };
                keys_: string;
            };
            kill_prefix: {
                prefix: string;
                subkeys: string;
            };
            remark_with_event: {
                remark: string;
            };
            __Unused8: string;
            authorize_upgrade: {
                codeHash: string;
            };
            authorize_upgrade_without_checks: {
                codeHash: string;
            };
            apply_authorized_upgrade: {
                code: string;
            };
        };
    };
    /**
     * Lookup101: cumulus_pallet_parachain_system::pallet::Call<T>
     **/
    CumulusPalletParachainSystemCall: {
        _enum: {
            set_validation_data: {
                data: string;
            };
            sudo_send_upward_message: {
                message: string;
            };
            authorize_upgrade: {
                codeHash: string;
                checkVersion: string;
            };
            enact_authorized_upgrade: {
                code: string;
            };
        };
    };
    /**
     * Lookup102: cumulus_primitives_parachain_inherent::ParachainInherentData
     **/
    CumulusPrimitivesParachainInherentParachainInherentData: {
        validationData: string;
        relayChainState: string;
        downwardMessages: string;
        horizontalMessages: string;
    };
    /**
     * Lookup103: polkadot_primitives::v7::PersistedValidationData<primitive_types::H256, N>
     **/
    PolkadotPrimitivesV7PersistedValidationData: {
        parentHead: string;
        relayParentNumber: string;
        relayParentStorageRoot: string;
        maxPovSize: string;
    };
    /**
     * Lookup105: sp_trie::storage_proof::StorageProof
     **/
    SpTrieStorageProof: {
        trieNodes: string;
    };
    /**
     * Lookup108: polkadot_core_primitives::InboundDownwardMessage<BlockNumber>
     **/
    PolkadotCorePrimitivesInboundDownwardMessage: {
        sentAt: string;
        msg: string;
    };
    /**
     * Lookup112: polkadot_core_primitives::InboundHrmpMessage<BlockNumber>
     **/
    PolkadotCorePrimitivesInboundHrmpMessage: {
        sentAt: string;
        data: string;
    };
    /**
     * Lookup115: pallet_timestamp::pallet::Call<T>
     **/
    PalletTimestampCall: {
        _enum: {
            set: {
                now: string;
            };
        };
    };
    /**
     * Lookup116: pallet_root_testing::pallet::Call<T>
     **/
    PalletRootTestingCall: {
        _enum: {
            fill_block: {
                ratio: string;
            };
            trigger_defensive: string;
        };
    };
    /**
     * Lookup117: pallet_balances::pallet::Call<T, I>
     **/
    PalletBalancesCall: {
        _enum: {
            transfer_allow_death: {
                dest: string;
                value: string;
            };
            __Unused1: string;
            force_transfer: {
                source: string;
                dest: string;
                value: string;
            };
            transfer_keep_alive: {
                dest: string;
                value: string;
            };
            transfer_all: {
                dest: string;
                keepAlive: string;
            };
            force_unreserve: {
                who: string;
                amount: string;
            };
            upgrade_accounts: {
                who: string;
            };
            __Unused7: string;
            force_set_balance: {
                who: string;
                newFree: string;
            };
            force_adjust_total_issuance: {
                direction: string;
                delta: string;
            };
            burn: {
                value: string;
                keepAlive: string;
            };
        };
    };
    /**
     * Lookup120: pallet_balances::types::AdjustmentDirection
     **/
    PalletBalancesAdjustmentDirection: {
        _enum: string[];
    };
    /**
     * Lookup121: pallet_parachain_staking::pallet::Call<T>
     **/
    PalletParachainStakingCall: {
        _enum: {
            set_staking_expectations: {
                expectations: {
                    min: string;
                    ideal: string;
                    max: string;
                };
            };
            set_inflation: {
                schedule: {
                    min: string;
                    ideal: string;
                    max: string;
                };
            };
            set_parachain_bond_account: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_parachain_bond_reserve_percent: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_total_selected: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_collator_commission: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_blocks_per_round: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            join_candidates: {
                bond: string;
                candidateCount: string;
            };
            schedule_leave_candidates: {
                candidateCount: string;
            };
            execute_leave_candidates: {
                candidate: string;
                candidateDelegationCount: string;
            };
            cancel_leave_candidates: {
                candidateCount: string;
            };
            go_offline: string;
            go_online: string;
            candidate_bond_more: {
                more: string;
            };
            schedule_candidate_bond_less: {
                less: string;
            };
            execute_candidate_bond_less: {
                candidate: string;
            };
            cancel_candidate_bond_less: string;
            delegate: {
                candidate: string;
                amount: string;
                candidateDelegationCount: string;
                delegationCount: string;
            };
            delegate_with_auto_compound: {
                candidate: string;
                amount: string;
                autoCompound: string;
                candidateDelegationCount: string;
                candidateAutoCompoundingDelegationCount: string;
                delegationCount: string;
            };
            removed_call_19: string;
            removed_call_20: string;
            removed_call_21: string;
            schedule_revoke_delegation: {
                collator: string;
            };
            delegator_bond_more: {
                candidate: string;
                more: string;
            };
            schedule_delegator_bond_less: {
                candidate: string;
                less: string;
            };
            execute_delegation_request: {
                delegator: string;
                candidate: string;
            };
            cancel_delegation_request: {
                candidate: string;
            };
            set_auto_compound: {
                candidate: string;
                value: string;
                candidateAutoCompoundingDelegationCountHint: string;
                delegationCountHint: string;
            };
            hotfix_remove_delegation_requests_exited_candidates: {
                candidates: string;
            };
            notify_inactive_collator: {
                collator: string;
            };
            enable_marking_offline: {
                value: string;
            };
            force_join_candidates: {
                account: string;
                bond: string;
                candidateCount: string;
            };
            set_inflation_distribution_config: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
        };
    };
    /**
     * Lookup124: pallet_author_inherent::pallet::Call<T>
     **/
    PalletAuthorInherentCall: {
        _enum: string[];
    };
    /**
     * Lookup125: pallet_author_slot_filter::pallet::Call<T>
     **/
    PalletAuthorSlotFilterCall: {
        _enum: {
            set_eligible: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
        };
    };
    /**
     * Lookup126: pallet_author_mapping::pallet::Call<T>
     **/
    PalletAuthorMappingCall: {
        _enum: {
            add_association: {
                nimbusId: string;
            };
            update_association: {
                oldNimbusId: string;
                newNimbusId: string;
            };
            clear_association: {
                nimbusId: string;
            };
            remove_keys: string;
            set_keys: {
                _alias: {
                    keys_: string;
                };
                keys_: string;
            };
        };
    };
    /**
     * Lookup127: pallet_moonbeam_orbiters::pallet::Call<T>
     **/
    PalletMoonbeamOrbitersCall: {
        _enum: {
            collator_add_orbiter: {
                orbiter: string;
            };
            collator_remove_orbiter: {
                orbiter: string;
            };
            orbiter_leave_collator_pool: {
                collator: string;
            };
            orbiter_register: string;
            orbiter_unregister: {
                collatorsPoolCount: string;
            };
            add_collator: {
                collator: string;
            };
            remove_collator: {
                collator: string;
            };
        };
    };
    /**
     * Lookup128: pallet_utility::pallet::Call<T>
     **/
    PalletUtilityCall: {
        _enum: {
            batch: {
                calls: string;
            };
            as_derivative: {
                index: string;
                call: string;
            };
            batch_all: {
                calls: string;
            };
            dispatch_as: {
                asOrigin: string;
                call: string;
            };
            force_batch: {
                calls: string;
            };
            with_weight: {
                call: string;
                weight: string;
            };
        };
    };
    /**
     * Lookup130: moonriver_runtime::OriginCaller
     **/
    MoonriverRuntimeOriginCaller: {
        _enum: {
            system: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            __Unused7: string;
            Void: string;
            __Unused9: string;
            __Unused10: string;
            __Unused11: string;
            __Unused12: string;
            __Unused13: string;
            __Unused14: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            __Unused21: string;
            __Unused22: string;
            __Unused23: string;
            __Unused24: string;
            __Unused25: string;
            __Unused26: string;
            __Unused27: string;
            __Unused28: string;
            __Unused29: string;
            __Unused30: string;
            __Unused31: string;
            __Unused32: string;
            __Unused33: string;
            __Unused34: string;
            __Unused35: string;
            __Unused36: string;
            __Unused37: string;
            __Unused38: string;
            __Unused39: string;
            __Unused40: string;
            __Unused41: string;
            __Unused42: string;
            __Unused43: string;
            __Unused44: string;
            __Unused45: string;
            __Unused46: string;
            __Unused47: string;
            __Unused48: string;
            __Unused49: string;
            __Unused50: string;
            __Unused51: string;
            Ethereum: string;
            __Unused53: string;
            __Unused54: string;
            __Unused55: string;
            __Unused56: string;
            __Unused57: string;
            __Unused58: string;
            __Unused59: string;
            __Unused60: string;
            __Unused61: string;
            __Unused62: string;
            __Unused63: string;
            __Unused64: string;
            Origins: string;
            __Unused66: string;
            __Unused67: string;
            __Unused68: string;
            __Unused69: string;
            __Unused70: string;
            __Unused71: string;
            TreasuryCouncilCollective: string;
            OpenTechCommitteeCollective: string;
            __Unused74: string;
            __Unused75: string;
            __Unused76: string;
            __Unused77: string;
            __Unused78: string;
            __Unused79: string;
            __Unused80: string;
            __Unused81: string;
            __Unused82: string;
            __Unused83: string;
            __Unused84: string;
            __Unused85: string;
            __Unused86: string;
            __Unused87: string;
            __Unused88: string;
            __Unused89: string;
            __Unused90: string;
            __Unused91: string;
            __Unused92: string;
            __Unused93: string;
            __Unused94: string;
            __Unused95: string;
            __Unused96: string;
            __Unused97: string;
            __Unused98: string;
            __Unused99: string;
            __Unused100: string;
            CumulusXcm: string;
            __Unused102: string;
            PolkadotXcm: string;
            __Unused104: string;
            __Unused105: string;
            __Unused106: string;
            __Unused107: string;
            __Unused108: string;
            EthereumXcm: string;
        };
    };
    /**
     * Lookup131: frame_support::dispatch::RawOrigin<account::AccountId20>
     **/
    FrameSupportDispatchRawOrigin: {
        _enum: {
            Root: string;
            Signed: string;
            None: string;
        };
    };
    /**
     * Lookup132: pallet_ethereum::RawOrigin
     **/
    PalletEthereumRawOrigin: {
        _enum: {
            EthereumTransaction: string;
        };
    };
    /**
     * Lookup133: moonriver_runtime::governance::origins::custom_origins::Origin
     **/
    MoonriverRuntimeGovernanceOriginsCustomOriginsOrigin: {
        _enum: string[];
    };
    /**
     * Lookup134: pallet_collective::RawOrigin<account::AccountId20, I>
     **/
    PalletCollectiveRawOrigin: {
        _enum: {
            Members: string;
            Member: string;
            _Phantom: string;
        };
    };
    /**
     * Lookup136: cumulus_pallet_xcm::pallet::Origin
     **/
    CumulusPalletXcmOrigin: {
        _enum: {
            Relay: string;
            SiblingParachain: string;
        };
    };
    /**
     * Lookup137: pallet_xcm::pallet::Origin
     **/
    PalletXcmOrigin: {
        _enum: {
            Xcm: string;
            Response: string;
        };
    };
    /**
     * Lookup138: staging_xcm::v4::location::Location
     **/
    StagingXcmV4Location: {
        parents: string;
        interior: string;
    };
    /**
     * Lookup139: staging_xcm::v4::junctions::Junctions
     **/
    StagingXcmV4Junctions: {
        _enum: {
            Here: string;
            X1: string;
            X2: string;
            X3: string;
            X4: string;
            X5: string;
            X6: string;
            X7: string;
            X8: string;
        };
    };
    /**
     * Lookup141: staging_xcm::v4::junction::Junction
     **/
    StagingXcmV4Junction: {
        _enum: {
            Parachain: string;
            AccountId32: {
                network: string;
                id: string;
            };
            AccountIndex64: {
                network: string;
                index: string;
            };
            AccountKey20: {
                network: string;
                key: string;
            };
            PalletInstance: string;
            GeneralIndex: string;
            GeneralKey: {
                length: string;
                data: string;
            };
            OnlyChild: string;
            Plurality: {
                id: string;
                part: string;
            };
            GlobalConsensus: string;
        };
    };
    /**
     * Lookup144: staging_xcm::v4::junction::NetworkId
     **/
    StagingXcmV4JunctionNetworkId: {
        _enum: {
            ByGenesis: string;
            ByFork: {
                blockNumber: string;
                blockHash: string;
            };
            Polkadot: string;
            Kusama: string;
            Westend: string;
            Rococo: string;
            Wococo: string;
            Ethereum: {
                chainId: string;
            };
            BitcoinCore: string;
            BitcoinCash: string;
            PolkadotBulletin: string;
        };
    };
    /**
     * Lookup145: xcm::v3::junction::BodyId
     **/
    XcmV3JunctionBodyId: {
        _enum: {
            Unit: string;
            Moniker: string;
            Index: string;
            Executive: string;
            Technical: string;
            Legislative: string;
            Judicial: string;
            Defense: string;
            Administration: string;
            Treasury: string;
        };
    };
    /**
     * Lookup146: xcm::v3::junction::BodyPart
     **/
    XcmV3JunctionBodyPart: {
        _enum: {
            Voice: string;
            Members: {
                count: string;
            };
            Fraction: {
                nom: string;
                denom: string;
            };
            AtLeastProportion: {
                nom: string;
                denom: string;
            };
            MoreThanProportion: {
                nom: string;
                denom: string;
            };
        };
    };
    /**
     * Lookup154: pallet_ethereum_xcm::RawOrigin
     **/
    PalletEthereumXcmRawOrigin: {
        _enum: {
            XcmEthereumTransaction: string;
        };
    };
    /**
     * Lookup155: sp_core::Void
     **/
    SpCoreVoid: string;
    /**
     * Lookup156: pallet_proxy::pallet::Call<T>
     **/
    PalletProxyCall: {
        _enum: {
            proxy: {
                real: string;
                forceProxyType: string;
                call: string;
            };
            add_proxy: {
                delegate: string;
                proxyType: string;
                delay: string;
            };
            remove_proxy: {
                delegate: string;
                proxyType: string;
                delay: string;
            };
            remove_proxies: string;
            create_pure: {
                proxyType: string;
                delay: string;
                index: string;
            };
            kill_pure: {
                spawner: string;
                proxyType: string;
                index: string;
                height: string;
                extIndex: string;
            };
            announce: {
                real: string;
                callHash: string;
            };
            remove_announcement: {
                real: string;
                callHash: string;
            };
            reject_announcement: {
                delegate: string;
                callHash: string;
            };
            proxy_announced: {
                delegate: string;
                real: string;
                forceProxyType: string;
                call: string;
            };
        };
    };
    /**
     * Lookup158: pallet_maintenance_mode::pallet::Call<T>
     **/
    PalletMaintenanceModeCall: {
        _enum: string[];
    };
    /**
     * Lookup159: pallet_identity::pallet::Call<T>
     **/
    PalletIdentityCall: {
        _enum: {
            add_registrar: {
                account: string;
            };
            set_identity: {
                info: string;
            };
            set_subs: {
                subs: string;
            };
            clear_identity: string;
            request_judgement: {
                regIndex: string;
                maxFee: string;
            };
            cancel_request: {
                regIndex: string;
            };
            set_fee: {
                index: string;
                fee: string;
            };
            set_account_id: {
                _alias: {
                    new_: string;
                };
                index: string;
                new_: string;
            };
            set_fields: {
                index: string;
                fields: string;
            };
            provide_judgement: {
                regIndex: string;
                target: string;
                judgement: string;
                identity: string;
            };
            kill_identity: {
                target: string;
            };
            add_sub: {
                sub: string;
                data: string;
            };
            rename_sub: {
                sub: string;
                data: string;
            };
            remove_sub: {
                sub: string;
            };
            quit_sub: string;
            add_username_authority: {
                authority: string;
                suffix: string;
                allocation: string;
            };
            remove_username_authority: {
                authority: string;
            };
            set_username_for: {
                who: string;
                username: string;
                signature: string;
            };
            accept_username: {
                username: string;
            };
            remove_expired_approval: {
                username: string;
            };
            set_primary_username: {
                username: string;
            };
            remove_dangling_username: {
                username: string;
            };
        };
    };
    /**
     * Lookup160: pallet_identity::legacy::IdentityInfo<FieldLimit>
     **/
    PalletIdentityLegacyIdentityInfo: {
        additional: string;
        display: string;
        legal: string;
        web: string;
        riot: string;
        email: string;
        pgpFingerprint: string;
        image: string;
        twitter: string;
    };
    /**
     * Lookup198: pallet_identity::types::Judgement<Balance>
     **/
    PalletIdentityJudgement: {
        _enum: {
            Unknown: string;
            FeePaid: string;
            Reasonable: string;
            KnownGood: string;
            OutOfDate: string;
            LowQuality: string;
            Erroneous: string;
        };
    };
    /**
     * Lookup200: account::EthereumSignature
     **/
    AccountEthereumSignature: string;
    /**
     * Lookup202: pallet_multisig::pallet::Call<T>
     **/
    PalletMultisigCall: {
        _enum: {
            as_multi_threshold_1: {
                otherSignatories: string;
                call: string;
            };
            as_multi: {
                threshold: string;
                otherSignatories: string;
                maybeTimepoint: string;
                call: string;
                maxWeight: string;
            };
            approve_as_multi: {
                threshold: string;
                otherSignatories: string;
                maybeTimepoint: string;
                callHash: string;
                maxWeight: string;
            };
            cancel_as_multi: {
                threshold: string;
                otherSignatories: string;
                timepoint: string;
                callHash: string;
            };
        };
    };
    /**
     * Lookup204: pallet_moonbeam_lazy_migrations::pallet::Call<T>
     **/
    PalletMoonbeamLazyMigrationsCall: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            create_contract_metadata: {
                address: string;
            };
            approve_assets_to_migrate: {
                assets: string;
            };
            start_foreign_assets_migration: {
                assetId: string;
            };
            migrate_foreign_asset_balances: {
                limit: string;
            };
            migrate_foreign_asset_approvals: {
                limit: string;
            };
            finish_foreign_assets_migration: string;
        };
    };
    /**
     * Lookup207: pallet_parameters::pallet::Call<T>
     **/
    PalletParametersCall: {
        _enum: {
            set_parameter: {
                keyValue: string;
            };
        };
    };
    /**
     * Lookup208: moonriver_runtime::runtime_params::RuntimeParameters
     **/
    MoonriverRuntimeRuntimeParamsRuntimeParameters: {
        _enum: {
            RuntimeConfig: string;
            PalletRandomness: string;
        };
    };
    /**
     * Lookup209: moonriver_runtime::runtime_params::dynamic_params::runtime_config::Parameters
     **/
    MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParameters: {
        _enum: {
            FeesTreasuryProportion: string;
        };
    };
    /**
     * Lookup211: moonriver_runtime::runtime_params::dynamic_params::pallet_randomness::Parameters
     **/
    MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParameters: {
        _enum: {
            Deposit: string;
        };
    };
    /**
     * Lookup213: pallet_evm::pallet::Call<T>
     **/
    PalletEvmCall: {
        _enum: {
            withdraw: {
                address: string;
                value: string;
            };
            call: {
                source: string;
                target: string;
                input: string;
                value: string;
                gasLimit: string;
                maxFeePerGas: string;
                maxPriorityFeePerGas: string;
                nonce: string;
                accessList: string;
            };
            create: {
                source: string;
                init: string;
                value: string;
                gasLimit: string;
                maxFeePerGas: string;
                maxPriorityFeePerGas: string;
                nonce: string;
                accessList: string;
            };
            create2: {
                source: string;
                init: string;
                salt: string;
                value: string;
                gasLimit: string;
                maxFeePerGas: string;
                maxPriorityFeePerGas: string;
                nonce: string;
                accessList: string;
            };
        };
    };
    /**
     * Lookup219: pallet_ethereum::pallet::Call<T>
     **/
    PalletEthereumCall: {
        _enum: {
            transact: {
                transaction: string;
            };
        };
    };
    /**
     * Lookup220: ethereum::transaction::TransactionV2
     **/
    EthereumTransactionTransactionV2: {
        _enum: {
            Legacy: string;
            EIP2930: string;
            EIP1559: string;
        };
    };
    /**
     * Lookup221: ethereum::transaction::LegacyTransaction
     **/
    EthereumTransactionLegacyTransaction: {
        nonce: string;
        gasPrice: string;
        gasLimit: string;
        action: string;
        value: string;
        input: string;
        signature: string;
    };
    /**
     * Lookup222: ethereum::transaction::TransactionAction
     **/
    EthereumTransactionTransactionAction: {
        _enum: {
            Call: string;
            Create: string;
        };
    };
    /**
     * Lookup223: ethereum::transaction::TransactionSignature
     **/
    EthereumTransactionTransactionSignature: {
        v: string;
        r: string;
        s: string;
    };
    /**
     * Lookup225: ethereum::transaction::EIP2930Transaction
     **/
    EthereumTransactionEip2930Transaction: {
        chainId: string;
        nonce: string;
        gasPrice: string;
        gasLimit: string;
        action: string;
        value: string;
        input: string;
        accessList: string;
        oddYParity: string;
        r: string;
        s: string;
    };
    /**
     * Lookup227: ethereum::transaction::AccessListItem
     **/
    EthereumTransactionAccessListItem: {
        address: string;
        storageKeys: string;
    };
    /**
     * Lookup228: ethereum::transaction::EIP1559Transaction
     **/
    EthereumTransactionEip1559Transaction: {
        chainId: string;
        nonce: string;
        maxPriorityFeePerGas: string;
        maxFeePerGas: string;
        gasLimit: string;
        action: string;
        value: string;
        input: string;
        accessList: string;
        oddYParity: string;
        r: string;
        s: string;
    };
    /**
     * Lookup229: pallet_scheduler::pallet::Call<T>
     **/
    PalletSchedulerCall: {
        _enum: {
            schedule: {
                when: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            cancel: {
                when: string;
                index: string;
            };
            schedule_named: {
                id: string;
                when: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            cancel_named: {
                id: string;
            };
            schedule_after: {
                after: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            schedule_named_after: {
                id: string;
                after: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            set_retry: {
                task: string;
                retries: string;
                period: string;
            };
            set_retry_named: {
                id: string;
                retries: string;
                period: string;
            };
            cancel_retry: {
                task: string;
            };
            cancel_retry_named: {
                id: string;
            };
        };
    };
    /**
     * Lookup231: pallet_preimage::pallet::Call<T>
     **/
    PalletPreimageCall: {
        _enum: {
            note_preimage: {
                bytes: string;
            };
            unnote_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            request_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            unrequest_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            ensure_updated: {
                hashes: string;
            };
        };
    };
    /**
     * Lookup232: pallet_conviction_voting::pallet::Call<T, I>
     **/
    PalletConvictionVotingCall: {
        _enum: {
            vote: {
                pollIndex: string;
                vote: string;
            };
            delegate: {
                class: string;
                to: string;
                conviction: string;
                balance: string;
            };
            undelegate: {
                class: string;
            };
            unlock: {
                class: string;
                target: string;
            };
            remove_vote: {
                class: string;
                index: string;
            };
            remove_other_vote: {
                target: string;
                class: string;
                index: string;
            };
        };
    };
    /**
     * Lookup233: pallet_conviction_voting::vote::AccountVote<Balance>
     **/
    PalletConvictionVotingVoteAccountVote: {
        _enum: {
            Standard: {
                vote: string;
                balance: string;
            };
            Split: {
                aye: string;
                nay: string;
            };
            SplitAbstain: {
                aye: string;
                nay: string;
                abstain: string;
            };
        };
    };
    /**
     * Lookup235: pallet_conviction_voting::conviction::Conviction
     **/
    PalletConvictionVotingConviction: {
        _enum: string[];
    };
    /**
     * Lookup237: pallet_referenda::pallet::Call<T, I>
     **/
    PalletReferendaCall: {
        _enum: {
            submit: {
                proposalOrigin: string;
                proposal: string;
                enactmentMoment: string;
            };
            place_decision_deposit: {
                index: string;
            };
            refund_decision_deposit: {
                index: string;
            };
            cancel: {
                index: string;
            };
            kill: {
                index: string;
            };
            nudge_referendum: {
                index: string;
            };
            one_fewer_deciding: {
                track: string;
            };
            refund_submission_deposit: {
                index: string;
            };
            set_metadata: {
                index: string;
                maybeHash: string;
            };
        };
    };
    /**
     * Lookup238: frame_support::traits::schedule::DispatchTime<BlockNumber>
     **/
    FrameSupportScheduleDispatchTime: {
        _enum: {
            At: string;
            After: string;
        };
    };
    /**
     * Lookup240: pallet_whitelist::pallet::Call<T>
     **/
    PalletWhitelistCall: {
        _enum: {
            whitelist_call: {
                callHash: string;
            };
            remove_whitelisted_call: {
                callHash: string;
            };
            dispatch_whitelisted_call: {
                callHash: string;
                callEncodedLen: string;
                callWeightWitness: string;
            };
            dispatch_whitelisted_call_with_preimage: {
                call: string;
            };
        };
    };
    /**
     * Lookup241: pallet_collective::pallet::Call<T, I>
     **/
    PalletCollectiveCall: {
        _enum: {
            set_members: {
                newMembers: string;
                prime: string;
                oldCount: string;
            };
            execute: {
                proposal: string;
                lengthBound: string;
            };
            propose: {
                threshold: string;
                proposal: string;
                lengthBound: string;
            };
            vote: {
                proposal: string;
                index: string;
                approve: string;
            };
            __Unused4: string;
            disapprove_proposal: {
                proposalHash: string;
            };
            close: {
                proposalHash: string;
                index: string;
                proposalWeightBound: string;
                lengthBound: string;
            };
        };
    };
    /**
     * Lookup243: pallet_treasury::pallet::Call<T, I>
     **/
    PalletTreasuryCall: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            spend_local: {
                amount: string;
                beneficiary: string;
            };
            remove_approval: {
                proposalId: string;
            };
            spend: {
                assetKind: string;
                amount: string;
                beneficiary: string;
                validFrom: string;
            };
            payout: {
                index: string;
            };
            check_status: {
                index: string;
            };
            void_spend: {
                index: string;
            };
        };
    };
    /**
     * Lookup245: pallet_crowdloan_rewards::pallet::Call<T>
     **/
    PalletCrowdloanRewardsCall: {
        _enum: {
            associate_native_identity: {
                rewardAccount: string;
                relayAccount: string;
                proof: string;
            };
            change_association_with_relay_keys: {
                rewardAccount: string;
                previousAccount: string;
                proofs: string;
            };
            claim: string;
            update_reward_address: {
                newRewardAccount: string;
            };
            complete_initialization: {
                leaseEndingBlock: string;
            };
            initialize_reward_vec: {
                rewards: string;
            };
        };
    };
    /**
     * Lookup246: sp_runtime::MultiSignature
     **/
    SpRuntimeMultiSignature: {
        _enum: {
            Ed25519: string;
            Sr25519: string;
            Ecdsa: string;
        };
    };
    /**
     * Lookup252: pallet_xcm::pallet::Call<T>
     **/
    PalletXcmCall: {
        _enum: {
            send: {
                dest: string;
                message: string;
            };
            teleport_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
            };
            reserve_transfer_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
            };
            execute: {
                message: string;
                maxWeight: string;
            };
            force_xcm_version: {
                location: string;
                version: string;
            };
            force_default_xcm_version: {
                maybeXcmVersion: string;
            };
            force_subscribe_version_notify: {
                location: string;
            };
            force_unsubscribe_version_notify: {
                location: string;
            };
            limited_reserve_transfer_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
                weightLimit: string;
            };
            limited_teleport_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
                weightLimit: string;
            };
            force_suspension: {
                suspended: string;
            };
            transfer_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
                weightLimit: string;
            };
            claim_assets: {
                assets: string;
                beneficiary: string;
            };
            transfer_assets_using_type_and_then: {
                dest: string;
                assets: string;
                assetsTransferType: string;
                remoteFeesId: string;
                feesTransferType: string;
                customXcmOnDest: string;
                weightLimit: string;
            };
        };
    };
    /**
     * Lookup253: xcm::VersionedLocation
     **/
    XcmVersionedLocation: {
        _enum: {
            __Unused0: string;
            V2: string;
            __Unused2: string;
            V3: string;
            V4: string;
        };
    };
    /**
     * Lookup254: xcm::v2::multilocation::MultiLocation
     **/
    XcmV2MultiLocation: {
        parents: string;
        interior: string;
    };
    /**
     * Lookup255: xcm::v2::multilocation::Junctions
     **/
    XcmV2MultilocationJunctions: {
        _enum: {
            Here: string;
            X1: string;
            X2: string;
            X3: string;
            X4: string;
            X5: string;
            X6: string;
            X7: string;
            X8: string;
        };
    };
    /**
     * Lookup256: xcm::v2::junction::Junction
     **/
    XcmV2Junction: {
        _enum: {
            Parachain: string;
            AccountId32: {
                network: string;
                id: string;
            };
            AccountIndex64: {
                network: string;
                index: string;
            };
            AccountKey20: {
                network: string;
                key: string;
            };
            PalletInstance: string;
            GeneralIndex: string;
            GeneralKey: string;
            OnlyChild: string;
            Plurality: {
                id: string;
                part: string;
            };
        };
    };
    /**
     * Lookup257: xcm::v2::NetworkId
     **/
    XcmV2NetworkId: {
        _enum: {
            Any: string;
            Named: string;
            Polkadot: string;
            Kusama: string;
        };
    };
    /**
     * Lookup259: xcm::v2::BodyId
     **/
    XcmV2BodyId: {
        _enum: {
            Unit: string;
            Named: string;
            Index: string;
            Executive: string;
            Technical: string;
            Legislative: string;
            Judicial: string;
            Defense: string;
            Administration: string;
            Treasury: string;
        };
    };
    /**
     * Lookup260: xcm::v2::BodyPart
     **/
    XcmV2BodyPart: {
        _enum: {
            Voice: string;
            Members: {
                count: string;
            };
            Fraction: {
                nom: string;
                denom: string;
            };
            AtLeastProportion: {
                nom: string;
                denom: string;
            };
            MoreThanProportion: {
                nom: string;
                denom: string;
            };
        };
    };
    /**
     * Lookup261: staging_xcm::v3::multilocation::MultiLocation
     **/
    StagingXcmV3MultiLocation: {
        parents: string;
        interior: string;
    };
    /**
     * Lookup262: xcm::v3::junctions::Junctions
     **/
    XcmV3Junctions: {
        _enum: {
            Here: string;
            X1: string;
            X2: string;
            X3: string;
            X4: string;
            X5: string;
            X6: string;
            X7: string;
            X8: string;
        };
    };
    /**
     * Lookup263: xcm::v3::junction::Junction
     **/
    XcmV3Junction: {
        _enum: {
            Parachain: string;
            AccountId32: {
                network: string;
                id: string;
            };
            AccountIndex64: {
                network: string;
                index: string;
            };
            AccountKey20: {
                network: string;
                key: string;
            };
            PalletInstance: string;
            GeneralIndex: string;
            GeneralKey: {
                length: string;
                data: string;
            };
            OnlyChild: string;
            Plurality: {
                id: string;
                part: string;
            };
            GlobalConsensus: string;
        };
    };
    /**
     * Lookup265: xcm::v3::junction::NetworkId
     **/
    XcmV3JunctionNetworkId: {
        _enum: {
            ByGenesis: string;
            ByFork: {
                blockNumber: string;
                blockHash: string;
            };
            Polkadot: string;
            Kusama: string;
            Westend: string;
            Rococo: string;
            Wococo: string;
            Ethereum: {
                chainId: string;
            };
            BitcoinCore: string;
            BitcoinCash: string;
            PolkadotBulletin: string;
        };
    };
    /**
     * Lookup266: xcm::VersionedXcm<RuntimeCall>
     **/
    XcmVersionedXcm: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            V2: string;
            V3: string;
            V4: string;
        };
    };
    /**
     * Lookup267: xcm::v2::Xcm<RuntimeCall>
     **/
    XcmV2Xcm: string;
    /**
     * Lookup269: xcm::v2::Instruction<RuntimeCall>
     **/
    XcmV2Instruction: {
        _enum: {
            WithdrawAsset: string;
            ReserveAssetDeposited: string;
            ReceiveTeleportedAsset: string;
            QueryResponse: {
                queryId: string;
                response: string;
                maxWeight: string;
            };
            TransferAsset: {
                assets: string;
                beneficiary: string;
            };
            TransferReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            Transact: {
                originType: string;
                requireWeightAtMost: string;
                call: string;
            };
            HrmpNewChannelOpenRequest: {
                sender: string;
                maxMessageSize: string;
                maxCapacity: string;
            };
            HrmpChannelAccepted: {
                recipient: string;
            };
            HrmpChannelClosing: {
                initiator: string;
                sender: string;
                recipient: string;
            };
            ClearOrigin: string;
            DescendOrigin: string;
            ReportError: {
                queryId: string;
                dest: string;
                maxResponseWeight: string;
            };
            DepositAsset: {
                assets: string;
                maxAssets: string;
                beneficiary: string;
            };
            DepositReserveAsset: {
                assets: string;
                maxAssets: string;
                dest: string;
                xcm: string;
            };
            ExchangeAsset: {
                give: string;
                receive: string;
            };
            InitiateReserveWithdraw: {
                assets: string;
                reserve: string;
                xcm: string;
            };
            InitiateTeleport: {
                assets: string;
                dest: string;
                xcm: string;
            };
            QueryHolding: {
                queryId: string;
                dest: string;
                assets: string;
                maxResponseWeight: string;
            };
            BuyExecution: {
                fees: string;
                weightLimit: string;
            };
            RefundSurplus: string;
            SetErrorHandler: string;
            SetAppendix: string;
            ClearError: string;
            ClaimAsset: {
                assets: string;
                ticket: string;
            };
            Trap: string;
            SubscribeVersion: {
                queryId: string;
                maxResponseWeight: string;
            };
            UnsubscribeVersion: string;
        };
    };
    /**
     * Lookup270: xcm::v2::multiasset::MultiAssets
     **/
    XcmV2MultiassetMultiAssets: string;
    /**
     * Lookup272: xcm::v2::multiasset::MultiAsset
     **/
    XcmV2MultiAsset: {
        id: string;
        fun: string;
    };
    /**
     * Lookup273: xcm::v2::multiasset::AssetId
     **/
    XcmV2MultiassetAssetId: {
        _enum: {
            Concrete: string;
            Abstract: string;
        };
    };
    /**
     * Lookup274: xcm::v2::multiasset::Fungibility
     **/
    XcmV2MultiassetFungibility: {
        _enum: {
            Fungible: string;
            NonFungible: string;
        };
    };
    /**
     * Lookup275: xcm::v2::multiasset::AssetInstance
     **/
    XcmV2MultiassetAssetInstance: {
        _enum: {
            Undefined: string;
            Index: string;
            Array4: string;
            Array8: string;
            Array16: string;
            Array32: string;
            Blob: string;
        };
    };
    /**
     * Lookup276: xcm::v2::Response
     **/
    XcmV2Response: {
        _enum: {
            Null: string;
            Assets: string;
            ExecutionResult: string;
            Version: string;
        };
    };
    /**
     * Lookup279: xcm::v2::traits::Error
     **/
    XcmV2TraitsError: {
        _enum: {
            Overflow: string;
            Unimplemented: string;
            UntrustedReserveLocation: string;
            UntrustedTeleportLocation: string;
            MultiLocationFull: string;
            MultiLocationNotInvertible: string;
            BadOrigin: string;
            InvalidLocation: string;
            AssetNotFound: string;
            FailedToTransactAsset: string;
            NotWithdrawable: string;
            LocationCannotHold: string;
            ExceedsMaxMessageSize: string;
            DestinationUnsupported: string;
            Transport: string;
            Unroutable: string;
            UnknownClaim: string;
            FailedToDecode: string;
            MaxWeightInvalid: string;
            NotHoldingFees: string;
            TooExpensive: string;
            Trap: string;
            UnhandledXcmVersion: string;
            WeightLimitReached: string;
            Barrier: string;
            WeightNotComputable: string;
        };
    };
    /**
     * Lookup280: xcm::v2::OriginKind
     **/
    XcmV2OriginKind: {
        _enum: string[];
    };
    /**
     * Lookup281: xcm::double_encoded::DoubleEncoded<T>
     **/
    XcmDoubleEncoded: {
        encoded: string;
    };
    /**
     * Lookup282: xcm::v2::multiasset::MultiAssetFilter
     **/
    XcmV2MultiassetMultiAssetFilter: {
        _enum: {
            Definite: string;
            Wild: string;
        };
    };
    /**
     * Lookup283: xcm::v2::multiasset::WildMultiAsset
     **/
    XcmV2MultiassetWildMultiAsset: {
        _enum: {
            All: string;
            AllOf: {
                id: string;
                fun: string;
            };
        };
    };
    /**
     * Lookup284: xcm::v2::multiasset::WildFungibility
     **/
    XcmV2MultiassetWildFungibility: {
        _enum: string[];
    };
    /**
     * Lookup285: xcm::v2::WeightLimit
     **/
    XcmV2WeightLimit: {
        _enum: {
            Unlimited: string;
            Limited: string;
        };
    };
    /**
     * Lookup286: xcm::v3::Xcm<Call>
     **/
    XcmV3Xcm: string;
    /**
     * Lookup288: xcm::v3::Instruction<Call>
     **/
    XcmV3Instruction: {
        _enum: {
            WithdrawAsset: string;
            ReserveAssetDeposited: string;
            ReceiveTeleportedAsset: string;
            QueryResponse: {
                queryId: string;
                response: string;
                maxWeight: string;
                querier: string;
            };
            TransferAsset: {
                assets: string;
                beneficiary: string;
            };
            TransferReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            Transact: {
                originKind: string;
                requireWeightAtMost: string;
                call: string;
            };
            HrmpNewChannelOpenRequest: {
                sender: string;
                maxMessageSize: string;
                maxCapacity: string;
            };
            HrmpChannelAccepted: {
                recipient: string;
            };
            HrmpChannelClosing: {
                initiator: string;
                sender: string;
                recipient: string;
            };
            ClearOrigin: string;
            DescendOrigin: string;
            ReportError: string;
            DepositAsset: {
                assets: string;
                beneficiary: string;
            };
            DepositReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ExchangeAsset: {
                give: string;
                want: string;
                maximal: string;
            };
            InitiateReserveWithdraw: {
                assets: string;
                reserve: string;
                xcm: string;
            };
            InitiateTeleport: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ReportHolding: {
                responseInfo: string;
                assets: string;
            };
            BuyExecution: {
                fees: string;
                weightLimit: string;
            };
            RefundSurplus: string;
            SetErrorHandler: string;
            SetAppendix: string;
            ClearError: string;
            ClaimAsset: {
                assets: string;
                ticket: string;
            };
            Trap: string;
            SubscribeVersion: {
                queryId: string;
                maxResponseWeight: string;
            };
            UnsubscribeVersion: string;
            BurnAsset: string;
            ExpectAsset: string;
            ExpectOrigin: string;
            ExpectError: string;
            ExpectTransactStatus: string;
            QueryPallet: {
                moduleName: string;
                responseInfo: string;
            };
            ExpectPallet: {
                index: string;
                name: string;
                moduleName: string;
                crateMajor: string;
                minCrateMinor: string;
            };
            ReportTransactStatus: string;
            ClearTransactStatus: string;
            UniversalOrigin: string;
            ExportMessage: {
                network: string;
                destination: string;
                xcm: string;
            };
            LockAsset: {
                asset: string;
                unlocker: string;
            };
            UnlockAsset: {
                asset: string;
                target: string;
            };
            NoteUnlockable: {
                asset: string;
                owner: string;
            };
            RequestUnlock: {
                asset: string;
                locker: string;
            };
            SetFeesMode: {
                jitWithdraw: string;
            };
            SetTopic: string;
            ClearTopic: string;
            AliasOrigin: string;
            UnpaidExecution: {
                weightLimit: string;
                checkOrigin: string;
            };
        };
    };
    /**
     * Lookup289: xcm::v3::multiasset::MultiAssets
     **/
    XcmV3MultiassetMultiAssets: string;
    /**
     * Lookup291: xcm::v3::multiasset::MultiAsset
     **/
    XcmV3MultiAsset: {
        id: string;
        fun: string;
    };
    /**
     * Lookup292: xcm::v3::multiasset::AssetId
     **/
    XcmV3MultiassetAssetId: {
        _enum: {
            Concrete: string;
            Abstract: string;
        };
    };
    /**
     * Lookup293: xcm::v3::multiasset::Fungibility
     **/
    XcmV3MultiassetFungibility: {
        _enum: {
            Fungible: string;
            NonFungible: string;
        };
    };
    /**
     * Lookup294: xcm::v3::multiasset::AssetInstance
     **/
    XcmV3MultiassetAssetInstance: {
        _enum: {
            Undefined: string;
            Index: string;
            Array4: string;
            Array8: string;
            Array16: string;
            Array32: string;
        };
    };
    /**
     * Lookup295: xcm::v3::Response
     **/
    XcmV3Response: {
        _enum: {
            Null: string;
            Assets: string;
            ExecutionResult: string;
            Version: string;
            PalletsInfo: string;
            DispatchResult: string;
        };
    };
    /**
     * Lookup298: xcm::v3::traits::Error
     **/
    XcmV3TraitsError: {
        _enum: {
            Overflow: string;
            Unimplemented: string;
            UntrustedReserveLocation: string;
            UntrustedTeleportLocation: string;
            LocationFull: string;
            LocationNotInvertible: string;
            BadOrigin: string;
            InvalidLocation: string;
            AssetNotFound: string;
            FailedToTransactAsset: string;
            NotWithdrawable: string;
            LocationCannotHold: string;
            ExceedsMaxMessageSize: string;
            DestinationUnsupported: string;
            Transport: string;
            Unroutable: string;
            UnknownClaim: string;
            FailedToDecode: string;
            MaxWeightInvalid: string;
            NotHoldingFees: string;
            TooExpensive: string;
            Trap: string;
            ExpectationFalse: string;
            PalletNotFound: string;
            NameMismatch: string;
            VersionIncompatible: string;
            HoldingWouldOverflow: string;
            ExportError: string;
            ReanchorFailed: string;
            NoDeal: string;
            FeesNotMet: string;
            LockError: string;
            NoPermission: string;
            Unanchored: string;
            NotDepositable: string;
            UnhandledXcmVersion: string;
            WeightLimitReached: string;
            Barrier: string;
            WeightNotComputable: string;
            ExceedsStackLimit: string;
        };
    };
    /**
     * Lookup300: xcm::v3::PalletInfo
     **/
    XcmV3PalletInfo: {
        index: string;
        name: string;
        moduleName: string;
        major: string;
        minor: string;
        patch: string;
    };
    /**
     * Lookup303: xcm::v3::MaybeErrorCode
     **/
    XcmV3MaybeErrorCode: {
        _enum: {
            Success: string;
            Error: string;
            TruncatedError: string;
        };
    };
    /**
     * Lookup306: xcm::v3::OriginKind
     **/
    XcmV3OriginKind: {
        _enum: string[];
    };
    /**
     * Lookup307: xcm::v3::QueryResponseInfo
     **/
    XcmV3QueryResponseInfo: {
        destination: string;
        queryId: string;
        maxWeight: string;
    };
    /**
     * Lookup308: xcm::v3::multiasset::MultiAssetFilter
     **/
    XcmV3MultiassetMultiAssetFilter: {
        _enum: {
            Definite: string;
            Wild: string;
        };
    };
    /**
     * Lookup309: xcm::v3::multiasset::WildMultiAsset
     **/
    XcmV3MultiassetWildMultiAsset: {
        _enum: {
            All: string;
            AllOf: {
                id: string;
                fun: string;
            };
            AllCounted: string;
            AllOfCounted: {
                id: string;
                fun: string;
                count: string;
            };
        };
    };
    /**
     * Lookup310: xcm::v3::multiasset::WildFungibility
     **/
    XcmV3MultiassetWildFungibility: {
        _enum: string[];
    };
    /**
     * Lookup311: xcm::v3::WeightLimit
     **/
    XcmV3WeightLimit: {
        _enum: {
            Unlimited: string;
            Limited: string;
        };
    };
    /**
     * Lookup312: staging_xcm::v4::Xcm<Call>
     **/
    StagingXcmV4Xcm: string;
    /**
     * Lookup314: staging_xcm::v4::Instruction<Call>
     **/
    StagingXcmV4Instruction: {
        _enum: {
            WithdrawAsset: string;
            ReserveAssetDeposited: string;
            ReceiveTeleportedAsset: string;
            QueryResponse: {
                queryId: string;
                response: string;
                maxWeight: string;
                querier: string;
            };
            TransferAsset: {
                assets: string;
                beneficiary: string;
            };
            TransferReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            Transact: {
                originKind: string;
                requireWeightAtMost: string;
                call: string;
            };
            HrmpNewChannelOpenRequest: {
                sender: string;
                maxMessageSize: string;
                maxCapacity: string;
            };
            HrmpChannelAccepted: {
                recipient: string;
            };
            HrmpChannelClosing: {
                initiator: string;
                sender: string;
                recipient: string;
            };
            ClearOrigin: string;
            DescendOrigin: string;
            ReportError: string;
            DepositAsset: {
                assets: string;
                beneficiary: string;
            };
            DepositReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ExchangeAsset: {
                give: string;
                want: string;
                maximal: string;
            };
            InitiateReserveWithdraw: {
                assets: string;
                reserve: string;
                xcm: string;
            };
            InitiateTeleport: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ReportHolding: {
                responseInfo: string;
                assets: string;
            };
            BuyExecution: {
                fees: string;
                weightLimit: string;
            };
            RefundSurplus: string;
            SetErrorHandler: string;
            SetAppendix: string;
            ClearError: string;
            ClaimAsset: {
                assets: string;
                ticket: string;
            };
            Trap: string;
            SubscribeVersion: {
                queryId: string;
                maxResponseWeight: string;
            };
            UnsubscribeVersion: string;
            BurnAsset: string;
            ExpectAsset: string;
            ExpectOrigin: string;
            ExpectError: string;
            ExpectTransactStatus: string;
            QueryPallet: {
                moduleName: string;
                responseInfo: string;
            };
            ExpectPallet: {
                index: string;
                name: string;
                moduleName: string;
                crateMajor: string;
                minCrateMinor: string;
            };
            ReportTransactStatus: string;
            ClearTransactStatus: string;
            UniversalOrigin: string;
            ExportMessage: {
                network: string;
                destination: string;
                xcm: string;
            };
            LockAsset: {
                asset: string;
                unlocker: string;
            };
            UnlockAsset: {
                asset: string;
                target: string;
            };
            NoteUnlockable: {
                asset: string;
                owner: string;
            };
            RequestUnlock: {
                asset: string;
                locker: string;
            };
            SetFeesMode: {
                jitWithdraw: string;
            };
            SetTopic: string;
            ClearTopic: string;
            AliasOrigin: string;
            UnpaidExecution: {
                weightLimit: string;
                checkOrigin: string;
            };
        };
    };
    /**
     * Lookup315: staging_xcm::v4::asset::Assets
     **/
    StagingXcmV4AssetAssets: string;
    /**
     * Lookup317: staging_xcm::v4::asset::Asset
     **/
    StagingXcmV4Asset: {
        id: string;
        fun: string;
    };
    /**
     * Lookup318: staging_xcm::v4::asset::AssetId
     **/
    StagingXcmV4AssetAssetId: string;
    /**
     * Lookup319: staging_xcm::v4::asset::Fungibility
     **/
    StagingXcmV4AssetFungibility: {
        _enum: {
            Fungible: string;
            NonFungible: string;
        };
    };
    /**
     * Lookup320: staging_xcm::v4::asset::AssetInstance
     **/
    StagingXcmV4AssetAssetInstance: {
        _enum: {
            Undefined: string;
            Index: string;
            Array4: string;
            Array8: string;
            Array16: string;
            Array32: string;
        };
    };
    /**
     * Lookup321: staging_xcm::v4::Response
     **/
    StagingXcmV4Response: {
        _enum: {
            Null: string;
            Assets: string;
            ExecutionResult: string;
            Version: string;
            PalletsInfo: string;
            DispatchResult: string;
        };
    };
    /**
     * Lookup323: staging_xcm::v4::PalletInfo
     **/
    StagingXcmV4PalletInfo: {
        index: string;
        name: string;
        moduleName: string;
        major: string;
        minor: string;
        patch: string;
    };
    /**
     * Lookup327: staging_xcm::v4::QueryResponseInfo
     **/
    StagingXcmV4QueryResponseInfo: {
        destination: string;
        queryId: string;
        maxWeight: string;
    };
    /**
     * Lookup328: staging_xcm::v4::asset::AssetFilter
     **/
    StagingXcmV4AssetAssetFilter: {
        _enum: {
            Definite: string;
            Wild: string;
        };
    };
    /**
     * Lookup329: staging_xcm::v4::asset::WildAsset
     **/
    StagingXcmV4AssetWildAsset: {
        _enum: {
            All: string;
            AllOf: {
                id: string;
                fun: string;
            };
            AllCounted: string;
            AllOfCounted: {
                id: string;
                fun: string;
                count: string;
            };
        };
    };
    /**
     * Lookup330: staging_xcm::v4::asset::WildFungibility
     **/
    StagingXcmV4AssetWildFungibility: {
        _enum: string[];
    };
    /**
     * Lookup331: xcm::VersionedAssets
     **/
    XcmVersionedAssets: {
        _enum: {
            __Unused0: string;
            V2: string;
            __Unused2: string;
            V3: string;
            V4: string;
        };
    };
    /**
     * Lookup343: staging_xcm_executor::traits::asset_transfer::TransferType
     **/
    StagingXcmExecutorAssetTransferTransferType: {
        _enum: {
            Teleport: string;
            LocalReserve: string;
            DestinationReserve: string;
            RemoteReserve: string;
        };
    };
    /**
     * Lookup344: xcm::VersionedAssetId
     **/
    XcmVersionedAssetId: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            V3: string;
            V4: string;
        };
    };
    /**
     * Lookup345: pallet_assets::pallet::Call<T, I>
     **/
    PalletAssetsCall: {
        _enum: {
            create: {
                id: string;
                admin: string;
                minBalance: string;
            };
            force_create: {
                id: string;
                owner: string;
                isSufficient: string;
                minBalance: string;
            };
            start_destroy: {
                id: string;
            };
            destroy_accounts: {
                id: string;
            };
            destroy_approvals: {
                id: string;
            };
            finish_destroy: {
                id: string;
            };
            mint: {
                id: string;
                beneficiary: string;
                amount: string;
            };
            burn: {
                id: string;
                who: string;
                amount: string;
            };
            transfer: {
                id: string;
                target: string;
                amount: string;
            };
            transfer_keep_alive: {
                id: string;
                target: string;
                amount: string;
            };
            force_transfer: {
                id: string;
                source: string;
                dest: string;
                amount: string;
            };
            freeze: {
                id: string;
                who: string;
            };
            thaw: {
                id: string;
                who: string;
            };
            freeze_asset: {
                id: string;
            };
            thaw_asset: {
                id: string;
            };
            transfer_ownership: {
                id: string;
                owner: string;
            };
            set_team: {
                id: string;
                issuer: string;
                admin: string;
                freezer: string;
            };
            set_metadata: {
                id: string;
                name: string;
                symbol: string;
                decimals: string;
            };
            clear_metadata: {
                id: string;
            };
            force_set_metadata: {
                id: string;
                name: string;
                symbol: string;
                decimals: string;
                isFrozen: string;
            };
            force_clear_metadata: {
                id: string;
            };
            force_asset_status: {
                id: string;
                owner: string;
                issuer: string;
                admin: string;
                freezer: string;
                minBalance: string;
                isSufficient: string;
                isFrozen: string;
            };
            approve_transfer: {
                id: string;
                delegate: string;
                amount: string;
            };
            cancel_approval: {
                id: string;
                delegate: string;
            };
            force_cancel_approval: {
                id: string;
                owner: string;
                delegate: string;
            };
            transfer_approved: {
                id: string;
                owner: string;
                destination: string;
                amount: string;
            };
            touch: {
                id: string;
            };
            refund: {
                id: string;
                allowBurn: string;
            };
            set_min_balance: {
                id: string;
                minBalance: string;
            };
            touch_other: {
                id: string;
                who: string;
            };
            refund_other: {
                id: string;
                who: string;
            };
            block: {
                id: string;
                who: string;
            };
        };
    };
    /**
     * Lookup346: pallet_asset_manager::pallet::Call<T>
     **/
    PalletAssetManagerCall: {
        _enum: {
            register_foreign_asset: {
                asset: string;
                metadata: string;
                minAmount: string;
                isSufficient: string;
            };
            __Unused1: string;
            change_existing_asset_type: {
                assetId: string;
                newAssetType: string;
                numAssetsWeightHint: string;
            };
            __Unused3: string;
            remove_existing_asset_type: {
                assetId: string;
                numAssetsWeightHint: string;
            };
            __Unused5: string;
            destroy_foreign_asset: {
                assetId: string;
                numAssetsWeightHint: string;
            };
        };
    };
    /**
     * Lookup347: moonriver_runtime::xcm_config::AssetType
     **/
    MoonriverRuntimeXcmConfigAssetType: {
        _enum: {
            Xcm: string;
        };
    };
    /**
     * Lookup348: moonriver_runtime::asset_config::AssetRegistrarMetadata
     **/
    MoonriverRuntimeAssetConfigAssetRegistrarMetadata: {
        name: string;
        symbol: string;
        decimals: string;
        isFrozen: string;
    };
    /**
     * Lookup349: pallet_xcm_transactor::pallet::Call<T>
     **/
    PalletXcmTransactorCall: {
        _enum: {
            register: {
                who: string;
                index: string;
            };
            deregister: {
                index: string;
            };
            transact_through_derivative: {
                dest: string;
                index: string;
                fee: string;
                innerCall: string;
                weightInfo: string;
                refund: string;
            };
            transact_through_sovereign: {
                dest: string;
                feePayer: string;
                fee: string;
                call: string;
                originKind: string;
                weightInfo: string;
                refund: string;
            };
            set_transact_info: {
                location: string;
                transactExtraWeight: string;
                maxWeight: string;
                transactExtraWeightSigned: string;
            };
            remove_transact_info: {
                location: string;
            };
            transact_through_signed: {
                dest: string;
                fee: string;
                call: string;
                weightInfo: string;
                refund: string;
            };
            set_fee_per_second: {
                assetLocation: string;
                feePerSecond: string;
            };
            remove_fee_per_second: {
                assetLocation: string;
            };
            hrmp_manage: {
                action: string;
                fee: string;
                weightInfo: string;
            };
        };
    };
    /**
     * Lookup350: moonriver_runtime::xcm_config::Transactors
     **/
    MoonriverRuntimeXcmConfigTransactors: {
        _enum: string[];
    };
    /**
     * Lookup351: pallet_xcm_transactor::pallet::CurrencyPayment<moonriver_runtime::xcm_config::CurrencyId>
     **/
    PalletXcmTransactorCurrencyPayment: {
        currency: string;
        feeAmount: string;
    };
    /**
     * Lookup352: moonriver_runtime::xcm_config::CurrencyId
     **/
    MoonriverRuntimeXcmConfigCurrencyId: {
        _enum: {
            SelfReserve: string;
            ForeignAsset: string;
            Erc20: {
                contractAddress: string;
            };
        };
    };
    /**
     * Lookup353: pallet_xcm_transactor::pallet::Currency<moonriver_runtime::xcm_config::CurrencyId>
     **/
    PalletXcmTransactorCurrency: {
        _enum: {
            AsCurrencyId: string;
            AsMultiLocation: string;
        };
    };
    /**
     * Lookup355: pallet_xcm_transactor::pallet::TransactWeights
     **/
    PalletXcmTransactorTransactWeights: {
        transactRequiredWeightAtMost: string;
        overallWeight: string;
    };
    /**
     * Lookup358: pallet_xcm_transactor::pallet::HrmpOperation
     **/
    PalletXcmTransactorHrmpOperation: {
        _enum: {
            InitOpen: string;
            Accept: {
                paraId: string;
            };
            Close: string;
            Cancel: {
                channelId: string;
                openRequests: string;
            };
        };
    };
    /**
     * Lookup359: pallet_xcm_transactor::pallet::HrmpInitParams
     **/
    PalletXcmTransactorHrmpInitParams: {
        paraId: string;
        proposedMaxCapacity: string;
        proposedMaxMessageSize: string;
    };
    /**
     * Lookup360: polkadot_parachain_primitives::primitives::HrmpChannelId
     **/
    PolkadotParachainPrimitivesPrimitivesHrmpChannelId: {
        sender: string;
        recipient: string;
    };
    /**
     * Lookup361: pallet_ethereum_xcm::pallet::Call<T>
     **/
    PalletEthereumXcmCall: {
        _enum: {
            transact: {
                xcmTransaction: string;
            };
            transact_through_proxy: {
                transactAs: string;
                xcmTransaction: string;
            };
            suspend_ethereum_xcm_execution: string;
            resume_ethereum_xcm_execution: string;
            force_transact_as: {
                transactAs: string;
                xcmTransaction: string;
                forceCreateAddress: string;
            };
        };
    };
    /**
     * Lookup362: xcm_primitives::ethereum_xcm::EthereumXcmTransaction
     **/
    XcmPrimitivesEthereumXcmEthereumXcmTransaction: {
        _enum: {
            V1: string;
            V2: string;
        };
    };
    /**
     * Lookup363: xcm_primitives::ethereum_xcm::EthereumXcmTransactionV1
     **/
    XcmPrimitivesEthereumXcmEthereumXcmTransactionV1: {
        gasLimit: string;
        feePayment: string;
        action: string;
        value: string;
        input: string;
        accessList: string;
    };
    /**
     * Lookup364: xcm_primitives::ethereum_xcm::EthereumXcmFee
     **/
    XcmPrimitivesEthereumXcmEthereumXcmFee: {
        _enum: {
            Manual: string;
            Auto: string;
        };
    };
    /**
     * Lookup365: xcm_primitives::ethereum_xcm::ManualEthereumXcmFee
     **/
    XcmPrimitivesEthereumXcmManualEthereumXcmFee: {
        gasPrice: string;
        maxFeePerGas: string;
    };
    /**
     * Lookup368: xcm_primitives::ethereum_xcm::EthereumXcmTransactionV2
     **/
    XcmPrimitivesEthereumXcmEthereumXcmTransactionV2: {
        gasLimit: string;
        action: string;
        value: string;
        input: string;
        accessList: string;
    };
    /**
     * Lookup370: pallet_message_queue::pallet::Call<T>
     **/
    PalletMessageQueueCall: {
        _enum: {
            reap_page: {
                messageOrigin: string;
                pageIndex: string;
            };
            execute_overweight: {
                messageOrigin: string;
                page: string;
                index: string;
                weightLimit: string;
            };
        };
    };
    /**
     * Lookup371: cumulus_primitives_core::AggregateMessageOrigin
     **/
    CumulusPrimitivesCoreAggregateMessageOrigin: {
        _enum: {
            Here: string;
            Parent: string;
            Sibling: string;
        };
    };
    /**
     * Lookup372: pallet_moonbeam_foreign_assets::pallet::Call<T>
     **/
    PalletMoonbeamForeignAssetsCall: {
        _enum: {
            create_foreign_asset: {
                assetId: string;
                xcmLocation: string;
                decimals: string;
                symbol: string;
                name: string;
            };
            change_xcm_location: {
                assetId: string;
                newXcmLocation: string;
            };
            freeze_foreign_asset: {
                assetId: string;
                allowXcmDeposit: string;
            };
            unfreeze_foreign_asset: {
                assetId: string;
            };
        };
    };
    /**
     * Lookup374: pallet_xcm_weight_trader::pallet::Call<T>
     **/
    PalletXcmWeightTraderCall: {
        _enum: {
            add_asset: {
                location: string;
                relativePrice: string;
            };
            edit_asset: {
                location: string;
                relativePrice: string;
            };
            pause_asset_support: {
                location: string;
            };
            resume_asset_support: {
                location: string;
            };
            remove_asset: {
                location: string;
            };
        };
    };
    /**
     * Lookup375: pallet_emergency_para_xcm::pallet::Call<T>
     **/
    PalletEmergencyParaXcmCall: {
        _enum: {
            paused_to_normal: string;
            fast_authorize_upgrade: {
                codeHash: string;
            };
        };
    };
    /**
     * Lookup376: pallet_randomness::pallet::Call<T>
     **/
    PalletRandomnessCall: {
        _enum: string[];
    };
    /**
     * Lookup377: sp_runtime::traits::BlakeTwo256
     **/
    SpRuntimeBlakeTwo256: string;
    /**
     * Lookup379: pallet_conviction_voting::types::Tally<Votes, Total>
     **/
    PalletConvictionVotingTally: {
        ayes: string;
        nays: string;
        support: string;
    };
    /**
     * Lookup380: pallet_whitelist::pallet::Event<T>
     **/
    PalletWhitelistEvent: {
        _enum: {
            CallWhitelisted: {
                callHash: string;
            };
            WhitelistedCallRemoved: {
                callHash: string;
            };
            WhitelistedCallDispatched: {
                callHash: string;
                result: string;
            };
        };
    };
    /**
     * Lookup382: frame_support::dispatch::PostDispatchInfo
     **/
    FrameSupportDispatchPostDispatchInfo: {
        actualWeight: string;
        paysFee: string;
    };
    /**
     * Lookup383: sp_runtime::DispatchErrorWithPostInfo<frame_support::dispatch::PostDispatchInfo>
     **/
    SpRuntimeDispatchErrorWithPostInfo: {
        postInfo: string;
        error: string;
    };
    /**
     * Lookup384: pallet_collective::pallet::Event<T, I>
     **/
    PalletCollectiveEvent: {
        _enum: {
            Proposed: {
                account: string;
                proposalIndex: string;
                proposalHash: string;
                threshold: string;
            };
            Voted: {
                account: string;
                proposalHash: string;
                voted: string;
                yes: string;
                no: string;
            };
            Approved: {
                proposalHash: string;
            };
            Disapproved: {
                proposalHash: string;
            };
            Executed: {
                proposalHash: string;
                result: string;
            };
            MemberExecuted: {
                proposalHash: string;
                result: string;
            };
            Closed: {
                proposalHash: string;
                yes: string;
                no: string;
            };
        };
    };
    /**
     * Lookup386: pallet_treasury::pallet::Event<T, I>
     **/
    PalletTreasuryEvent: {
        _enum: {
            Spending: {
                budgetRemaining: string;
            };
            Awarded: {
                proposalIndex: string;
                award: string;
                account: string;
            };
            Burnt: {
                burntFunds: string;
            };
            Rollover: {
                rolloverBalance: string;
            };
            Deposit: {
                value: string;
            };
            SpendApproved: {
                proposalIndex: string;
                amount: string;
                beneficiary: string;
            };
            UpdatedInactive: {
                reactivated: string;
                deactivated: string;
            };
            AssetSpendApproved: {
                index: string;
                assetKind: string;
                amount: string;
                beneficiary: string;
                validFrom: string;
                expireAt: string;
            };
            AssetSpendVoided: {
                index: string;
            };
            Paid: {
                index: string;
                paymentId: string;
            };
            PaymentFailed: {
                index: string;
                paymentId: string;
            };
            SpendProcessed: {
                index: string;
            };
        };
    };
    /**
     * Lookup387: pallet_crowdloan_rewards::pallet::Event<T>
     **/
    PalletCrowdloanRewardsEvent: {
        _enum: {
            InitialPaymentMade: string;
            NativeIdentityAssociated: string;
            RewardsPaid: string;
            RewardAddressUpdated: string;
            InitializedAlreadyInitializedAccount: string;
            InitializedAccountWithNotEnoughContribution: string;
        };
    };
    /**
     * Lookup388: cumulus_pallet_xcmp_queue::pallet::Event<T>
     **/
    CumulusPalletXcmpQueueEvent: {
        _enum: {
            XcmpMessageSent: {
                messageHash: string;
            };
        };
    };
    /**
     * Lookup389: cumulus_pallet_xcm::pallet::Event<T>
     **/
    CumulusPalletXcmEvent: {
        _enum: {
            InvalidFormat: string;
            UnsupportedVersion: string;
            ExecutedDownward: string;
        };
    };
    /**
     * Lookup390: staging_xcm::v4::traits::Outcome
     **/
    StagingXcmV4TraitsOutcome: {
        _enum: {
            Complete: {
                used: string;
            };
            Incomplete: {
                used: string;
                error: string;
            };
            Error: {
                error: string;
            };
        };
    };
    /**
     * Lookup391: pallet_xcm::pallet::Event<T>
     **/
    PalletXcmEvent: {
        _enum: {
            Attempted: {
                outcome: string;
            };
            Sent: {
                origin: string;
                destination: string;
                message: string;
                messageId: string;
            };
            UnexpectedResponse: {
                origin: string;
                queryId: string;
            };
            ResponseReady: {
                queryId: string;
                response: string;
            };
            Notified: {
                queryId: string;
                palletIndex: string;
                callIndex: string;
            };
            NotifyOverweight: {
                queryId: string;
                palletIndex: string;
                callIndex: string;
                actualWeight: string;
                maxBudgetedWeight: string;
            };
            NotifyDispatchError: {
                queryId: string;
                palletIndex: string;
                callIndex: string;
            };
            NotifyDecodeFailed: {
                queryId: string;
                palletIndex: string;
                callIndex: string;
            };
            InvalidResponder: {
                origin: string;
                queryId: string;
                expectedLocation: string;
            };
            InvalidResponderVersion: {
                origin: string;
                queryId: string;
            };
            ResponseTaken: {
                queryId: string;
            };
            AssetsTrapped: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
                origin: string;
                assets: string;
            };
            VersionChangeNotified: {
                destination: string;
                result: string;
                cost: string;
                messageId: string;
            };
            SupportedVersionChanged: {
                location: string;
                version: string;
            };
            NotifyTargetSendFail: {
                location: string;
                queryId: string;
                error: string;
            };
            NotifyTargetMigrationFail: {
                location: string;
                queryId: string;
            };
            InvalidQuerierVersion: {
                origin: string;
                queryId: string;
            };
            InvalidQuerier: {
                origin: string;
                queryId: string;
                expectedQuerier: string;
                maybeActualQuerier: string;
            };
            VersionNotifyStarted: {
                destination: string;
                cost: string;
                messageId: string;
            };
            VersionNotifyRequested: {
                destination: string;
                cost: string;
                messageId: string;
            };
            VersionNotifyUnrequested: {
                destination: string;
                cost: string;
                messageId: string;
            };
            FeesPaid: {
                paying: string;
                fees: string;
            };
            AssetsClaimed: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
                origin: string;
                assets: string;
            };
            VersionMigrationFinished: {
                version: string;
            };
        };
    };
    /**
     * Lookup392: pallet_assets::pallet::Event<T, I>
     **/
    PalletAssetsEvent: {
        _enum: {
            Created: {
                assetId: string;
                creator: string;
                owner: string;
            };
            Issued: {
                assetId: string;
                owner: string;
                amount: string;
            };
            Transferred: {
                assetId: string;
                from: string;
                to: string;
                amount: string;
            };
            Burned: {
                assetId: string;
                owner: string;
                balance: string;
            };
            TeamChanged: {
                assetId: string;
                issuer: string;
                admin: string;
                freezer: string;
            };
            OwnerChanged: {
                assetId: string;
                owner: string;
            };
            Frozen: {
                assetId: string;
                who: string;
            };
            Thawed: {
                assetId: string;
                who: string;
            };
            AssetFrozen: {
                assetId: string;
            };
            AssetThawed: {
                assetId: string;
            };
            AccountsDestroyed: {
                assetId: string;
                accountsDestroyed: string;
                accountsRemaining: string;
            };
            ApprovalsDestroyed: {
                assetId: string;
                approvalsDestroyed: string;
                approvalsRemaining: string;
            };
            DestructionStarted: {
                assetId: string;
            };
            Destroyed: {
                assetId: string;
            };
            ForceCreated: {
                assetId: string;
                owner: string;
            };
            MetadataSet: {
                assetId: string;
                name: string;
                symbol: string;
                decimals: string;
                isFrozen: string;
            };
            MetadataCleared: {
                assetId: string;
            };
            ApprovedTransfer: {
                assetId: string;
                source: string;
                delegate: string;
                amount: string;
            };
            ApprovalCancelled: {
                assetId: string;
                owner: string;
                delegate: string;
            };
            TransferredApproved: {
                assetId: string;
                owner: string;
                delegate: string;
                destination: string;
                amount: string;
            };
            AssetStatusChanged: {
                assetId: string;
            };
            AssetMinBalanceChanged: {
                assetId: string;
                newMinBalance: string;
            };
            Touched: {
                assetId: string;
                who: string;
                depositor: string;
            };
            Blocked: {
                assetId: string;
                who: string;
            };
            Deposited: {
                assetId: string;
                who: string;
                amount: string;
            };
            Withdrawn: {
                assetId: string;
                who: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup393: pallet_asset_manager::pallet::Event<T>
     **/
    PalletAssetManagerEvent: {
        _enum: {
            ForeignAssetRegistered: {
                assetId: string;
                asset: string;
                metadata: string;
            };
            UnitsPerSecondChanged: string;
            ForeignAssetXcmLocationChanged: {
                assetId: string;
                newAssetType: string;
            };
            ForeignAssetRemoved: {
                assetId: string;
                assetType: string;
            };
            SupportedAssetRemoved: {
                assetType: string;
            };
            ForeignAssetDestroyed: {
                assetId: string;
                assetType: string;
            };
            LocalAssetDestroyed: {
                assetId: string;
            };
        };
    };
    /**
     * Lookup394: pallet_xcm_transactor::pallet::Event<T>
     **/
    PalletXcmTransactorEvent: {
        _enum: {
            TransactedDerivative: {
                accountId: string;
                dest: string;
                call: string;
                index: string;
            };
            TransactedSovereign: {
                feePayer: string;
                dest: string;
                call: string;
            };
            TransactedSigned: {
                feePayer: string;
                dest: string;
                call: string;
            };
            RegisteredDerivative: {
                accountId: string;
                index: string;
            };
            DeRegisteredDerivative: {
                index: string;
            };
            TransactFailed: {
                error: string;
            };
            TransactInfoChanged: {
                location: string;
                remoteInfo: string;
            };
            TransactInfoRemoved: {
                location: string;
            };
            DestFeePerSecondChanged: {
                location: string;
                feePerSecond: string;
            };
            DestFeePerSecondRemoved: {
                location: string;
            };
            HrmpManagementSent: {
                action: string;
            };
        };
    };
    /**
     * Lookup395: pallet_xcm_transactor::pallet::RemoteTransactInfoWithMaxWeight
     **/
    PalletXcmTransactorRemoteTransactInfoWithMaxWeight: {
        transactExtraWeight: string;
        maxWeight: string;
        transactExtraWeightSigned: string;
    };
    /**
     * Lookup396: pallet_ethereum_xcm::pallet::Event<T>
     **/
    PalletEthereumXcmEvent: {
        _enum: {
            ExecutedFromXcm: {
                xcmMsgHash: string;
                ethTxHash: string;
            };
        };
    };
    /**
     * Lookup397: pallet_message_queue::pallet::Event<T>
     **/
    PalletMessageQueueEvent: {
        _enum: {
            ProcessingFailed: {
                id: string;
                origin: string;
                error: string;
            };
            Processed: {
                id: string;
                origin: string;
                weightUsed: string;
                success: string;
            };
            OverweightEnqueued: {
                id: string;
                origin: string;
                pageIndex: string;
                messageIndex: string;
            };
            PageReaped: {
                origin: string;
                index: string;
            };
        };
    };
    /**
     * Lookup398: frame_support::traits::messages::ProcessMessageError
     **/
    FrameSupportMessagesProcessMessageError: {
        _enum: {
            BadFormat: string;
            Corrupt: string;
            Unsupported: string;
            Overweight: string;
            Yield: string;
            StackLimitReached: string;
        };
    };
    /**
     * Lookup399: pallet_moonbeam_foreign_assets::pallet::Event<T>
     **/
    PalletMoonbeamForeignAssetsEvent: {
        _enum: {
            ForeignAssetCreated: {
                contractAddress: string;
                assetId: string;
                xcmLocation: string;
            };
            ForeignAssetXcmLocationChanged: {
                assetId: string;
                newXcmLocation: string;
            };
            ForeignAssetFrozen: {
                assetId: string;
                xcmLocation: string;
            };
            ForeignAssetUnfrozen: {
                assetId: string;
                xcmLocation: string;
            };
        };
    };
    /**
     * Lookup400: pallet_xcm_weight_trader::pallet::Event<T>
     **/
    PalletXcmWeightTraderEvent: {
        _enum: {
            SupportedAssetAdded: {
                location: string;
                relativePrice: string;
            };
            SupportedAssetEdited: {
                location: string;
                relativePrice: string;
            };
            PauseAssetSupport: {
                location: string;
            };
            ResumeAssetSupport: {
                location: string;
            };
            SupportedAssetRemoved: {
                location: string;
            };
        };
    };
    /**
     * Lookup401: pallet_emergency_para_xcm::pallet::Event
     **/
    PalletEmergencyParaXcmEvent: {
        _enum: string[];
    };
    /**
     * Lookup402: pallet_randomness::pallet::Event<T>
     **/
    PalletRandomnessEvent: {
        _enum: {
            RandomnessRequestedBabeEpoch: {
                id: string;
                refundAddress: string;
                contractAddress: string;
                fee: string;
                gasLimit: string;
                numWords: string;
                salt: string;
                earliestEpoch: string;
            };
            RandomnessRequestedLocal: {
                id: string;
                refundAddress: string;
                contractAddress: string;
                fee: string;
                gasLimit: string;
                numWords: string;
                salt: string;
                earliestBlock: string;
            };
            RequestFulfilled: {
                id: string;
            };
            RequestFeeIncreased: {
                id: string;
                newFee: string;
            };
            RequestExpirationExecuted: {
                id: string;
            };
        };
    };
    /**
     * Lookup403: frame_system::Phase
     **/
    FrameSystemPhase: {
        _enum: {
            ApplyExtrinsic: string;
            Finalization: string;
            Initialization: string;
        };
    };
    /**
     * Lookup405: frame_system::LastRuntimeUpgradeInfo
     **/
    FrameSystemLastRuntimeUpgradeInfo: {
        specVersion: string;
        specName: string;
    };
    /**
     * Lookup406: frame_system::CodeUpgradeAuthorization<T>
     **/
    FrameSystemCodeUpgradeAuthorization: {
        codeHash: string;
        checkVersion: string;
    };
    /**
     * Lookup407: frame_system::limits::BlockWeights
     **/
    FrameSystemLimitsBlockWeights: {
        baseBlock: string;
        maxBlock: string;
        perClass: string;
    };
    /**
     * Lookup408: frame_support::dispatch::PerDispatchClass<frame_system::limits::WeightsPerClass>
     **/
    FrameSupportDispatchPerDispatchClassWeightsPerClass: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup409: frame_system::limits::WeightsPerClass
     **/
    FrameSystemLimitsWeightsPerClass: {
        baseExtrinsic: string;
        maxExtrinsic: string;
        maxTotal: string;
        reserved: string;
    };
    /**
     * Lookup410: frame_system::limits::BlockLength
     **/
    FrameSystemLimitsBlockLength: {
        max: string;
    };
    /**
     * Lookup411: frame_support::dispatch::PerDispatchClass<T>
     **/
    FrameSupportDispatchPerDispatchClassU32: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup412: sp_weights::RuntimeDbWeight
     **/
    SpWeightsRuntimeDbWeight: {
        read: string;
        write: string;
    };
    /**
     * Lookup413: sp_version::RuntimeVersion
     **/
    SpVersionRuntimeVersion: {
        specName: string;
        implName: string;
        authoringVersion: string;
        specVersion: string;
        implVersion: string;
        apis: string;
        transactionVersion: string;
        stateVersion: string;
    };
    /**
     * Lookup417: frame_system::pallet::Error<T>
     **/
    FrameSystemError: {
        _enum: string[];
    };
    /**
     * Lookup419: cumulus_pallet_parachain_system::unincluded_segment::Ancestor<primitive_types::H256>
     **/
    CumulusPalletParachainSystemUnincludedSegmentAncestor: {
        usedBandwidth: string;
        paraHeadHash: string;
        consumedGoAheadSignal: string;
    };
    /**
     * Lookup420: cumulus_pallet_parachain_system::unincluded_segment::UsedBandwidth
     **/
    CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth: {
        umpMsgCount: string;
        umpTotalBytes: string;
        hrmpOutgoing: string;
    };
    /**
     * Lookup422: cumulus_pallet_parachain_system::unincluded_segment::HrmpChannelUpdate
     **/
    CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate: {
        msgCount: string;
        totalBytes: string;
    };
    /**
     * Lookup426: polkadot_primitives::v7::UpgradeGoAhead
     **/
    PolkadotPrimitivesV7UpgradeGoAhead: {
        _enum: string[];
    };
    /**
     * Lookup427: cumulus_pallet_parachain_system::unincluded_segment::SegmentTracker<primitive_types::H256>
     **/
    CumulusPalletParachainSystemUnincludedSegmentSegmentTracker: {
        usedBandwidth: string;
        hrmpWatermark: string;
        consumedGoAheadSignal: string;
    };
    /**
     * Lookup429: polkadot_primitives::v7::UpgradeRestriction
     **/
    PolkadotPrimitivesV7UpgradeRestriction: {
        _enum: string[];
    };
    /**
     * Lookup430: cumulus_pallet_parachain_system::relay_state_snapshot::MessagingStateSnapshot
     **/
    CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: {
        dmqMqcHead: string;
        relayDispatchQueueRemainingCapacity: string;
        ingressChannels: string;
        egressChannels: string;
    };
    /**
     * Lookup431: cumulus_pallet_parachain_system::relay_state_snapshot::RelayDispatchQueueRemainingCapacity
     **/
    CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity: {
        remainingCount: string;
        remainingSize: string;
    };
    /**
     * Lookup434: polkadot_primitives::v7::AbridgedHrmpChannel
     **/
    PolkadotPrimitivesV7AbridgedHrmpChannel: {
        maxCapacity: string;
        maxTotalSize: string;
        maxMessageSize: string;
        msgCount: string;
        totalSize: string;
        mqcHead: string;
    };
    /**
     * Lookup435: polkadot_primitives::v7::AbridgedHostConfiguration
     **/
    PolkadotPrimitivesV7AbridgedHostConfiguration: {
        maxCodeSize: string;
        maxHeadDataSize: string;
        maxUpwardQueueCount: string;
        maxUpwardQueueSize: string;
        maxUpwardMessageSize: string;
        maxUpwardMessageNumPerCandidate: string;
        hrmpMaxMessageNumPerCandidate: string;
        validationUpgradeCooldown: string;
        validationUpgradeDelay: string;
        asyncBackingParams: string;
    };
    /**
     * Lookup436: polkadot_primitives::v7::async_backing::AsyncBackingParams
     **/
    PolkadotPrimitivesV7AsyncBackingAsyncBackingParams: {
        maxCandidateDepth: string;
        allowedAncestryLen: string;
    };
    /**
     * Lookup442: polkadot_core_primitives::OutboundHrmpMessage<polkadot_parachain_primitives::primitives::Id>
     **/
    PolkadotCorePrimitivesOutboundHrmpMessage: {
        recipient: string;
        data: string;
    };
    /**
     * Lookup444: cumulus_pallet_parachain_system::pallet::Error<T>
     **/
    CumulusPalletParachainSystemError: {
        _enum: string[];
    };
    /**
     * Lookup446: pallet_balances::types::BalanceLock<Balance>
     **/
    PalletBalancesBalanceLock: {
        id: string;
        amount: string;
        reasons: string;
    };
    /**
     * Lookup447: pallet_balances::types::Reasons
     **/
    PalletBalancesReasons: {
        _enum: string[];
    };
    /**
     * Lookup450: pallet_balances::types::ReserveData<ReserveIdentifier, Balance>
     **/
    PalletBalancesReserveData: {
        id: string;
        amount: string;
    };
    /**
     * Lookup454: moonriver_runtime::RuntimeHoldReason
     **/
    MoonriverRuntimeRuntimeHoldReason: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            __Unused7: string;
            __Unused8: string;
            __Unused9: string;
            __Unused10: string;
            __Unused11: string;
            __Unused12: string;
            __Unused13: string;
            __Unused14: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            __Unused21: string;
            __Unused22: string;
            __Unused23: string;
            __Unused24: string;
            __Unused25: string;
            __Unused26: string;
            __Unused27: string;
            __Unused28: string;
            __Unused29: string;
            __Unused30: string;
            __Unused31: string;
            __Unused32: string;
            __Unused33: string;
            __Unused34: string;
            __Unused35: string;
            __Unused36: string;
            __Unused37: string;
            __Unused38: string;
            __Unused39: string;
            __Unused40: string;
            __Unused41: string;
            __Unused42: string;
            __Unused43: string;
            __Unused44: string;
            __Unused45: string;
            __Unused46: string;
            __Unused47: string;
            __Unused48: string;
            __Unused49: string;
            __Unused50: string;
            __Unused51: string;
            __Unused52: string;
            __Unused53: string;
            __Unused54: string;
            __Unused55: string;
            __Unused56: string;
            __Unused57: string;
            __Unused58: string;
            __Unused59: string;
            __Unused60: string;
            __Unused61: string;
            Preimage: string;
        };
    };
    /**
     * Lookup455: pallet_preimage::pallet::HoldReason
     **/
    PalletPreimageHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup458: frame_support::traits::tokens::misc::IdAmount<Id, Balance>
     **/
    FrameSupportTokensMiscIdAmount: {
        id: string;
        amount: string;
    };
    /**
     * Lookup460: pallet_balances::pallet::Error<T, I>
     **/
    PalletBalancesError: {
        _enum: string[];
    };
    /**
     * Lookup461: pallet_transaction_payment::Releases
     **/
    PalletTransactionPaymentReleases: {
        _enum: string[];
    };
    /**
     * Lookup462: pallet_parachain_staking::types::RoundInfo<BlockNumber>
     **/
    PalletParachainStakingRoundInfo: {
        current: string;
        first: string;
        length: string;
        firstSlot: string;
    };
    /**
     * Lookup463: pallet_parachain_staking::types::Delegator<account::AccountId20, Balance>
     **/
    PalletParachainStakingDelegator: {
        id: string;
        delegations: string;
        total: string;
        lessTotal: string;
        status: string;
    };
    /**
     * Lookup464: pallet_parachain_staking::set::OrderedSet<pallet_parachain_staking::types::Bond<account::AccountId20, Balance>>
     **/
    PalletParachainStakingSetOrderedSet: string;
    /**
     * Lookup465: pallet_parachain_staking::types::Bond<account::AccountId20, Balance>
     **/
    PalletParachainStakingBond: {
        owner: string;
        amount: string;
    };
    /**
     * Lookup467: pallet_parachain_staking::types::DelegatorStatus
     **/
    PalletParachainStakingDelegatorStatus: {
        _enum: {
            Active: string;
            Leaving: string;
        };
    };
    /**
     * Lookup468: pallet_parachain_staking::types::CandidateMetadata<Balance>
     **/
    PalletParachainStakingCandidateMetadata: {
        bond: string;
        delegationCount: string;
        totalCounted: string;
        lowestTopDelegationAmount: string;
        highestBottomDelegationAmount: string;
        lowestBottomDelegationAmount: string;
        topCapacity: string;
        bottomCapacity: string;
        request: string;
        status: string;
    };
    /**
     * Lookup469: pallet_parachain_staking::types::CapacityStatus
     **/
    PalletParachainStakingCapacityStatus: {
        _enum: string[];
    };
    /**
     * Lookup471: pallet_parachain_staking::types::CandidateBondLessRequest<Balance>
     **/
    PalletParachainStakingCandidateBondLessRequest: {
        amount: string;
        whenExecutable: string;
    };
    /**
     * Lookup472: pallet_parachain_staking::types::CollatorStatus
     **/
    PalletParachainStakingCollatorStatus: {
        _enum: {
            Active: string;
            Idle: string;
            Leaving: string;
        };
    };
    /**
     * Lookup474: pallet_parachain_staking::delegation_requests::ScheduledRequest<account::AccountId20, Balance>
     **/
    PalletParachainStakingDelegationRequestsScheduledRequest: {
        delegator: string;
        whenExecutable: string;
        action: string;
    };
    /**
     * Lookup477: pallet_parachain_staking::auto_compound::AutoCompoundConfig<account::AccountId20>
     **/
    PalletParachainStakingAutoCompoundAutoCompoundConfig: {
        delegator: string;
        value: string;
    };
    /**
     * Lookup479: pallet_parachain_staking::types::Delegations<account::AccountId20, Balance>
     **/
    PalletParachainStakingDelegations: {
        delegations: string;
        total: string;
    };
    /**
     * Lookup481: pallet_parachain_staking::set::BoundedOrderedSet<pallet_parachain_staking::types::Bond<account::AccountId20, Balance>, S>
     **/
    PalletParachainStakingSetBoundedOrderedSet: string;
    /**
     * Lookup484: pallet_parachain_staking::types::CollatorSnapshot<account::AccountId20, Balance>
     **/
    PalletParachainStakingCollatorSnapshot: {
        bond: string;
        delegations: string;
        total: string;
    };
    /**
     * Lookup486: pallet_parachain_staking::types::BondWithAutoCompound<account::AccountId20, Balance>
     **/
    PalletParachainStakingBondWithAutoCompound: {
        owner: string;
        amount: string;
        autoCompound: string;
    };
    /**
     * Lookup487: pallet_parachain_staking::types::DelayedPayout<Balance>
     **/
    PalletParachainStakingDelayedPayout: {
        roundIssuance: string;
        totalStakingReward: string;
        collatorCommission: string;
    };
    /**
     * Lookup488: pallet_parachain_staking::inflation::InflationInfo<Balance>
     **/
    PalletParachainStakingInflationInflationInfo: {
        expect: {
            min: string;
            ideal: string;
            max: string;
        };
        annual: {
            min: string;
            ideal: string;
            max: string;
        };
        round: {
            min: string;
            ideal: string;
            max: string;
        };
    };
    /**
     * Lookup489: pallet_parachain_staking::pallet::Error<T>
     **/
    PalletParachainStakingError: {
        _enum: string[];
    };
    /**
     * Lookup490: pallet_author_inherent::pallet::Error<T>
     **/
    PalletAuthorInherentError: {
        _enum: string[];
    };
    /**
     * Lookup491: pallet_author_mapping::pallet::RegistrationInfo<T>
     **/
    PalletAuthorMappingRegistrationInfo: {
        _alias: {
            keys_: string;
        };
        account: string;
        deposit: string;
        keys_: string;
    };
    /**
     * Lookup492: pallet_author_mapping::pallet::Error<T>
     **/
    PalletAuthorMappingError: {
        _enum: string[];
    };
    /**
     * Lookup493: pallet_moonbeam_orbiters::types::CollatorPoolInfo<account::AccountId20>
     **/
    PalletMoonbeamOrbitersCollatorPoolInfo: {
        orbiters: string;
        maybeCurrentOrbiter: string;
        nextOrbiter: string;
    };
    /**
     * Lookup495: pallet_moonbeam_orbiters::types::CurrentOrbiter<account::AccountId20>
     **/
    PalletMoonbeamOrbitersCurrentOrbiter: {
        accountId: string;
        removed: string;
    };
    /**
     * Lookup496: pallet_moonbeam_orbiters::pallet::Error<T>
     **/
    PalletMoonbeamOrbitersError: {
        _enum: string[];
    };
    /**
     * Lookup499: pallet_utility::pallet::Error<T>
     **/
    PalletUtilityError: {
        _enum: string[];
    };
    /**
     * Lookup502: pallet_proxy::ProxyDefinition<account::AccountId20, moonriver_runtime::ProxyType, BlockNumber>
     **/
    PalletProxyProxyDefinition: {
        delegate: string;
        proxyType: string;
        delay: string;
    };
    /**
     * Lookup506: pallet_proxy::Announcement<account::AccountId20, primitive_types::H256, BlockNumber>
     **/
    PalletProxyAnnouncement: {
        real: string;
        callHash: string;
        height: string;
    };
    /**
     * Lookup508: pallet_proxy::pallet::Error<T>
     **/
    PalletProxyError: {
        _enum: string[];
    };
    /**
     * Lookup509: pallet_maintenance_mode::pallet::Error<T>
     **/
    PalletMaintenanceModeError: {
        _enum: string[];
    };
    /**
     * Lookup511: pallet_identity::types::Registration<Balance, MaxJudgements, pallet_identity::legacy::IdentityInfo<FieldLimit>>
     **/
    PalletIdentityRegistration: {
        judgements: string;
        deposit: string;
        info: string;
    };
    /**
     * Lookup520: pallet_identity::types::RegistrarInfo<Balance, account::AccountId20, IdField>
     **/
    PalletIdentityRegistrarInfo: {
        account: string;
        fee: string;
        fields: string;
    };
    /**
     * Lookup522: pallet_identity::types::AuthorityProperties<bounded_collections::bounded_vec::BoundedVec<T, S>>
     **/
    PalletIdentityAuthorityProperties: {
        suffix: string;
        allocation: string;
    };
    /**
     * Lookup525: pallet_identity::pallet::Error<T>
     **/
    PalletIdentityError: {
        _enum: string[];
    };
    /**
     * Lookup526: pallet_migrations::pallet::Error<T>
     **/
    PalletMigrationsError: {
        _enum: string[];
    };
    /**
     * Lookup528: pallet_multisig::Multisig<BlockNumber, Balance, account::AccountId20, MaxApprovals>
     **/
    PalletMultisigMultisig: {
        when: string;
        deposit: string;
        depositor: string;
        approvals: string;
    };
    /**
     * Lookup530: pallet_multisig::pallet::Error<T>
     **/
    PalletMultisigError: {
        _enum: string[];
    };
    /**
     * Lookup532: pallet_moonbeam_lazy_migrations::pallet::StateMigrationStatus
     **/
    PalletMoonbeamLazyMigrationsStateMigrationStatus: {
        _enum: {
            NotStarted: string;
            Started: string;
            Error: string;
            Complete: string;
        };
    };
    /**
     * Lookup534: pallet_moonbeam_lazy_migrations::foreign_asset::ForeignAssetMigrationStatus
     **/
    PalletMoonbeamLazyMigrationsForeignAssetForeignAssetMigrationStatus: {
        _enum: {
            Idle: string;
            Migrating: string;
        };
    };
    /**
     * Lookup535: pallet_moonbeam_lazy_migrations::foreign_asset::ForeignAssetMigrationInfo
     **/
    PalletMoonbeamLazyMigrationsForeignAssetForeignAssetMigrationInfo: {
        assetId: string;
        remainingBalances: string;
        remainingApprovals: string;
    };
    /**
     * Lookup536: pallet_moonbeam_lazy_migrations::pallet::Error<T>
     **/
    PalletMoonbeamLazyMigrationsError: {
        _enum: string[];
    };
    /**
     * Lookup537: pallet_evm::CodeMetadata
     **/
    PalletEvmCodeMetadata: {
        _alias: {
            size_: string;
            hash_: string;
        };
        size_: string;
        hash_: string;
    };
    /**
     * Lookup539: pallet_evm::pallet::Error<T>
     **/
    PalletEvmError: {
        _enum: string[];
    };
    /**
     * Lookup542: fp_rpc::TransactionStatus
     **/
    FpRpcTransactionStatus: {
        transactionHash: string;
        transactionIndex: string;
        from: string;
        to: string;
        contractAddress: string;
        logs: string;
        logsBloom: string;
    };
    /**
     * Lookup544: ethbloom::Bloom
     **/
    EthbloomBloom: string;
    /**
     * Lookup546: ethereum::receipt::ReceiptV3
     **/
    EthereumReceiptReceiptV3: {
        _enum: {
            Legacy: string;
            EIP2930: string;
            EIP1559: string;
        };
    };
    /**
     * Lookup547: ethereum::receipt::EIP658ReceiptData
     **/
    EthereumReceiptEip658ReceiptData: {
        statusCode: string;
        usedGas: string;
        logsBloom: string;
        logs: string;
    };
    /**
     * Lookup548: ethereum::block::Block<ethereum::transaction::TransactionV2>
     **/
    EthereumBlock: {
        header: string;
        transactions: string;
        ommers: string;
    };
    /**
     * Lookup549: ethereum::header::Header
     **/
    EthereumHeader: {
        parentHash: string;
        ommersHash: string;
        beneficiary: string;
        stateRoot: string;
        transactionsRoot: string;
        receiptsRoot: string;
        logsBloom: string;
        difficulty: string;
        number: string;
        gasLimit: string;
        gasUsed: string;
        timestamp: string;
        extraData: string;
        mixHash: string;
        nonce: string;
    };
    /**
     * Lookup550: ethereum_types::hash::H64
     **/
    EthereumTypesHashH64: string;
    /**
     * Lookup555: pallet_ethereum::pallet::Error<T>
     **/
    PalletEthereumError: {
        _enum: string[];
    };
    /**
     * Lookup558: pallet_scheduler::Scheduled<Name, frame_support::traits::preimages::Bounded<moonriver_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, BlockNumber, moonriver_runtime::OriginCaller, account::AccountId20>
     **/
    PalletSchedulerScheduled: {
        maybeId: string;
        priority: string;
        call: string;
        maybePeriodic: string;
        origin: string;
    };
    /**
     * Lookup560: pallet_scheduler::RetryConfig<Period>
     **/
    PalletSchedulerRetryConfig: {
        totalRetries: string;
        remaining: string;
        period: string;
    };
    /**
     * Lookup561: pallet_scheduler::pallet::Error<T>
     **/
    PalletSchedulerError: {
        _enum: string[];
    };
    /**
     * Lookup562: pallet_preimage::OldRequestStatus<account::AccountId20, Balance>
     **/
    PalletPreimageOldRequestStatus: {
        _enum: {
            Unrequested: {
                deposit: string;
                len: string;
            };
            Requested: {
                deposit: string;
                count: string;
                len: string;
            };
        };
    };
    /**
     * Lookup565: pallet_preimage::RequestStatus<account::AccountId20, frame_support::traits::tokens::fungible::HoldConsideration<A, F, R, D, Fp>>
     **/
    PalletPreimageRequestStatus: {
        _enum: {
            Unrequested: {
                ticket: string;
                len: string;
            };
            Requested: {
                maybeTicket: string;
                count: string;
                maybeLen: string;
            };
        };
    };
    /**
     * Lookup571: pallet_preimage::pallet::Error<T>
     **/
    PalletPreimageError: {
        _enum: string[];
    };
    /**
     * Lookup573: pallet_conviction_voting::vote::Voting<Balance, account::AccountId20, BlockNumber, PollIndex, MaxVotes>
     **/
    PalletConvictionVotingVoteVoting: {
        _enum: {
            Casting: string;
            Delegating: string;
        };
    };
    /**
     * Lookup574: pallet_conviction_voting::vote::Casting<Balance, BlockNumber, PollIndex, MaxVotes>
     **/
    PalletConvictionVotingVoteCasting: {
        votes: string;
        delegations: string;
        prior: string;
    };
    /**
     * Lookup578: pallet_conviction_voting::types::Delegations<Balance>
     **/
    PalletConvictionVotingDelegations: {
        votes: string;
        capital: string;
    };
    /**
     * Lookup579: pallet_conviction_voting::vote::PriorLock<BlockNumber, Balance>
     **/
    PalletConvictionVotingVotePriorLock: string;
    /**
     * Lookup580: pallet_conviction_voting::vote::Delegating<Balance, account::AccountId20, BlockNumber>
     **/
    PalletConvictionVotingVoteDelegating: {
        balance: string;
        target: string;
        conviction: string;
        delegations: string;
        prior: string;
    };
    /**
     * Lookup584: pallet_conviction_voting::pallet::Error<T, I>
     **/
    PalletConvictionVotingError: {
        _enum: string[];
    };
    /**
     * Lookup585: pallet_referenda::types::ReferendumInfo<TrackId, moonriver_runtime::OriginCaller, Moment, frame_support::traits::preimages::Bounded<moonriver_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance, pallet_conviction_voting::types::Tally<Votes, Total>, account::AccountId20, ScheduleAddress>
     **/
    PalletReferendaReferendumInfo: {
        _enum: {
            Ongoing: string;
            Approved: string;
            Rejected: string;
            Cancelled: string;
            TimedOut: string;
            Killed: string;
        };
    };
    /**
     * Lookup586: pallet_referenda::types::ReferendumStatus<TrackId, moonriver_runtime::OriginCaller, Moment, frame_support::traits::preimages::Bounded<moonriver_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance, pallet_conviction_voting::types::Tally<Votes, Total>, account::AccountId20, ScheduleAddress>
     **/
    PalletReferendaReferendumStatus: {
        track: string;
        origin: string;
        proposal: string;
        enactment: string;
        submitted: string;
        submissionDeposit: string;
        decisionDeposit: string;
        deciding: string;
        tally: string;
        inQueue: string;
        alarm: string;
    };
    /**
     * Lookup587: pallet_referenda::types::Deposit<account::AccountId20, Balance>
     **/
    PalletReferendaDeposit: {
        who: string;
        amount: string;
    };
    /**
     * Lookup590: pallet_referenda::types::DecidingStatus<BlockNumber>
     **/
    PalletReferendaDecidingStatus: {
        since: string;
        confirming: string;
    };
    /**
     * Lookup598: pallet_referenda::types::TrackInfo<Balance, Moment>
     **/
    PalletReferendaTrackInfo: {
        name: string;
        maxDeciding: string;
        decisionDeposit: string;
        preparePeriod: string;
        decisionPeriod: string;
        confirmPeriod: string;
        minEnactmentPeriod: string;
        minApproval: string;
        minSupport: string;
    };
    /**
     * Lookup599: pallet_referenda::types::Curve
     **/
    PalletReferendaCurve: {
        _enum: {
            LinearDecreasing: {
                length: string;
                floor: string;
                ceil: string;
            };
            SteppedDecreasing: {
                begin: string;
                end: string;
                step: string;
                period: string;
            };
            Reciprocal: {
                factor: string;
                xOffset: string;
                yOffset: string;
            };
        };
    };
    /**
     * Lookup602: pallet_referenda::pallet::Error<T, I>
     **/
    PalletReferendaError: {
        _enum: string[];
    };
    /**
     * Lookup603: pallet_whitelist::pallet::Error<T>
     **/
    PalletWhitelistError: {
        _enum: string[];
    };
    /**
     * Lookup605: pallet_collective::Votes<account::AccountId20, BlockNumber>
     **/
    PalletCollectiveVotes: {
        index: string;
        threshold: string;
        ayes: string;
        nays: string;
        end: string;
    };
    /**
     * Lookup606: pallet_collective::pallet::Error<T, I>
     **/
    PalletCollectiveError: {
        _enum: string[];
    };
    /**
     * Lookup609: pallet_treasury::Proposal<account::AccountId20, Balance>
     **/
    PalletTreasuryProposal: {
        proposer: string;
        value: string;
        beneficiary: string;
        bond: string;
    };
    /**
     * Lookup612: pallet_treasury::SpendStatus<AssetKind, AssetBalance, account::AccountId20, BlockNumber, PaymentId>
     **/
    PalletTreasurySpendStatus: {
        assetKind: string;
        amount: string;
        beneficiary: string;
        validFrom: string;
        expireAt: string;
        status: string;
    };
    /**
     * Lookup613: pallet_treasury::PaymentState<Id>
     **/
    PalletTreasuryPaymentState: {
        _enum: {
            Pending: string;
            Attempted: {
                id: string;
            };
            Failed: string;
        };
    };
    /**
     * Lookup615: frame_support::PalletId
     **/
    FrameSupportPalletId: string;
    /**
     * Lookup616: pallet_treasury::pallet::Error<T, I>
     **/
    PalletTreasuryError: {
        _enum: string[];
    };
    /**
     * Lookup617: pallet_crowdloan_rewards::pallet::RewardInfo<T>
     **/
    PalletCrowdloanRewardsRewardInfo: {
        totalReward: string;
        claimedReward: string;
        contributedRelayAddresses: string;
    };
    /**
     * Lookup619: pallet_crowdloan_rewards::pallet::Error<T>
     **/
    PalletCrowdloanRewardsError: {
        _enum: string[];
    };
    /**
     * Lookup624: cumulus_pallet_xcmp_queue::OutboundChannelDetails
     **/
    CumulusPalletXcmpQueueOutboundChannelDetails: {
        recipient: string;
        state: string;
        signalsExist: string;
        firstIndex: string;
        lastIndex: string;
    };
    /**
     * Lookup625: cumulus_pallet_xcmp_queue::OutboundState
     **/
    CumulusPalletXcmpQueueOutboundState: {
        _enum: string[];
    };
    /**
     * Lookup629: cumulus_pallet_xcmp_queue::QueueConfigData
     **/
    CumulusPalletXcmpQueueQueueConfigData: {
        suspendThreshold: string;
        dropThreshold: string;
        resumeThreshold: string;
    };
    /**
     * Lookup630: cumulus_pallet_xcmp_queue::pallet::Error<T>
     **/
    CumulusPalletXcmpQueueError: {
        _enum: string[];
    };
    /**
     * Lookup631: pallet_xcm::pallet::QueryStatus<BlockNumber>
     **/
    PalletXcmQueryStatus: {
        _enum: {
            Pending: {
                responder: string;
                maybeMatchQuerier: string;
                maybeNotify: string;
                timeout: string;
            };
            VersionNotifier: {
                origin: string;
                isActive: string;
            };
            Ready: {
                response: string;
                at: string;
            };
        };
    };
    /**
     * Lookup635: xcm::VersionedResponse
     **/
    XcmVersionedResponse: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            V2: string;
            V3: string;
            V4: string;
        };
    };
    /**
     * Lookup641: pallet_xcm::pallet::VersionMigrationStage
     **/
    PalletXcmVersionMigrationStage: {
        _enum: {
            MigrateSupportedVersion: string;
            MigrateVersionNotifiers: string;
            NotifyCurrentTargets: string;
            MigrateAndNotifyOldTargets: string;
        };
    };
    /**
     * Lookup644: pallet_xcm::pallet::RemoteLockedFungibleRecord<ConsumerIdentifier, MaxConsumers>
     **/
    PalletXcmRemoteLockedFungibleRecord: {
        amount: string;
        owner: string;
        locker: string;
        consumers: string;
    };
    /**
     * Lookup651: pallet_xcm::pallet::Error<T>
     **/
    PalletXcmError: {
        _enum: string[];
    };
    /**
     * Lookup652: pallet_assets::types::AssetDetails<Balance, account::AccountId20, DepositBalance>
     **/
    PalletAssetsAssetDetails: {
        owner: string;
        issuer: string;
        admin: string;
        freezer: string;
        supply: string;
        deposit: string;
        minBalance: string;
        isSufficient: string;
        accounts: string;
        sufficients: string;
        approvals: string;
        status: string;
    };
    /**
     * Lookup653: pallet_assets::types::AssetStatus
     **/
    PalletAssetsAssetStatus: {
        _enum: string[];
    };
    /**
     * Lookup655: pallet_assets::types::AssetAccount<Balance, DepositBalance, Extra, account::AccountId20>
     **/
    PalletAssetsAssetAccount: {
        balance: string;
        status: string;
        reason: string;
        extra: string;
    };
    /**
     * Lookup656: pallet_assets::types::AccountStatus
     **/
    PalletAssetsAccountStatus: {
        _enum: string[];
    };
    /**
     * Lookup657: pallet_assets::types::ExistenceReason<Balance, account::AccountId20>
     **/
    PalletAssetsExistenceReason: {
        _enum: {
            Consumer: string;
            Sufficient: string;
            DepositHeld: string;
            DepositRefunded: string;
            DepositFrom: string;
        };
    };
    /**
     * Lookup659: pallet_assets::types::Approval<Balance, DepositBalance>
     **/
    PalletAssetsApproval: {
        amount: string;
        deposit: string;
    };
    /**
     * Lookup660: pallet_assets::types::AssetMetadata<DepositBalance, bounded_collections::bounded_vec::BoundedVec<T, S>>
     **/
    PalletAssetsAssetMetadata: {
        deposit: string;
        name: string;
        symbol: string;
        decimals: string;
        isFrozen: string;
    };
    /**
     * Lookup662: pallet_assets::pallet::Error<T, I>
     **/
    PalletAssetsError: {
        _enum: string[];
    };
    /**
     * Lookup663: pallet_asset_manager::pallet::Error<T>
     **/
    PalletAssetManagerError: {
        _enum: string[];
    };
    /**
     * Lookup664: pallet_xcm_transactor::relay_indices::RelayChainIndices
     **/
    PalletXcmTransactorRelayIndicesRelayChainIndices: {
        staking: string;
        utility: string;
        hrmp: string;
        bond: string;
        bondExtra: string;
        unbond: string;
        withdrawUnbonded: string;
        validate: string;
        nominate: string;
        chill: string;
        setPayee: string;
        setController: string;
        rebond: string;
        asDerivative: string;
        initOpenChannel: string;
        acceptOpenChannel: string;
        closeChannel: string;
        cancelOpenRequest: string;
    };
    /**
     * Lookup665: pallet_xcm_transactor::pallet::Error<T>
     **/
    PalletXcmTransactorError: {
        _enum: string[];
    };
    /**
     * Lookup666: pallet_ethereum_xcm::pallet::Error<T>
     **/
    PalletEthereumXcmError: {
        _enum: string[];
    };
    /**
     * Lookup667: pallet_message_queue::BookState<cumulus_primitives_core::AggregateMessageOrigin>
     **/
    PalletMessageQueueBookState: {
        _alias: {
            size_: string;
        };
        begin: string;
        end: string;
        count: string;
        readyNeighbours: string;
        messageCount: string;
        size_: string;
    };
    /**
     * Lookup669: pallet_message_queue::Neighbours<cumulus_primitives_core::AggregateMessageOrigin>
     **/
    PalletMessageQueueNeighbours: {
        prev: string;
        next: string;
    };
    /**
     * Lookup671: pallet_message_queue::Page<Size, HeapSize>
     **/
    PalletMessageQueuePage: {
        remaining: string;
        remainingSize: string;
        firstIndex: string;
        first: string;
        last: string;
        heap: string;
    };
    /**
     * Lookup673: pallet_message_queue::pallet::Error<T>
     **/
    PalletMessageQueueError: {
        _enum: string[];
    };
    /**
     * Lookup675: pallet_moonbeam_foreign_assets::AssetStatus
     **/
    PalletMoonbeamForeignAssetsAssetStatus: {
        _enum: string[];
    };
    /**
     * Lookup676: pallet_moonbeam_foreign_assets::pallet::Error<T>
     **/
    PalletMoonbeamForeignAssetsError: {
        _enum: string[];
    };
    /**
     * Lookup678: pallet_xcm_weight_trader::pallet::Error<T>
     **/
    PalletXcmWeightTraderError: {
        _enum: string[];
    };
    /**
     * Lookup679: pallet_emergency_para_xcm::XcmMode
     **/
    PalletEmergencyParaXcmXcmMode: {
        _enum: string[];
    };
    /**
     * Lookup680: pallet_emergency_para_xcm::pallet::Error<T>
     **/
    PalletEmergencyParaXcmError: {
        _enum: string[];
    };
    /**
     * Lookup682: pallet_precompile_benchmarks::pallet::Error<T>
     **/
    PalletPrecompileBenchmarksError: {
        _enum: string[];
    };
    /**
     * Lookup683: pallet_randomness::types::RequestState<T>
     **/
    PalletRandomnessRequestState: {
        request: string;
        deposit: string;
    };
    /**
     * Lookup684: pallet_randomness::types::Request<Balance, pallet_randomness::types::RequestInfo<T>>
     **/
    PalletRandomnessRequest: {
        refundAddress: string;
        contractAddress: string;
        fee: string;
        gasLimit: string;
        numWords: string;
        salt: string;
        info: string;
    };
    /**
     * Lookup685: pallet_randomness::types::RequestInfo<T>
     **/
    PalletRandomnessRequestInfo: {
        _enum: {
            BabeEpoch: string;
            Local: string;
        };
    };
    /**
     * Lookup686: pallet_randomness::types::RequestType<T>
     **/
    PalletRandomnessRequestType: {
        _enum: {
            BabeEpoch: string;
            Local: string;
        };
    };
    /**
     * Lookup687: pallet_randomness::types::RandomnessResult<primitive_types::H256>
     **/
    PalletRandomnessRandomnessResult: {
        randomness: string;
        requestCount: string;
    };
    /**
     * Lookup688: pallet_randomness::pallet::Error<T>
     **/
    PalletRandomnessError: {
        _enum: string[];
    };
    /**
     * Lookup691: frame_system::extensions::check_non_zero_sender::CheckNonZeroSender<T>
     **/
    FrameSystemExtensionsCheckNonZeroSender: string;
    /**
     * Lookup692: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
     **/
    FrameSystemExtensionsCheckSpecVersion: string;
    /**
     * Lookup693: frame_system::extensions::check_tx_version::CheckTxVersion<T>
     **/
    FrameSystemExtensionsCheckTxVersion: string;
    /**
     * Lookup694: frame_system::extensions::check_genesis::CheckGenesis<T>
     **/
    FrameSystemExtensionsCheckGenesis: string;
    /**
     * Lookup697: frame_system::extensions::check_nonce::CheckNonce<T>
     **/
    FrameSystemExtensionsCheckNonce: string;
    /**
     * Lookup698: frame_system::extensions::check_weight::CheckWeight<T>
     **/
    FrameSystemExtensionsCheckWeight: string;
    /**
     * Lookup699: pallet_transaction_payment::ChargeTransactionPayment<T>
     **/
    PalletTransactionPaymentChargeTransactionPayment: string;
    /**
     * Lookup700: frame_metadata_hash_extension::CheckMetadataHash<T>
     **/
    FrameMetadataHashExtensionCheckMetadataHash: {
        mode: string;
    };
    /**
     * Lookup701: frame_metadata_hash_extension::Mode
     **/
    FrameMetadataHashExtensionMode: {
        _enum: string[];
    };
    /**
     * Lookup702: cumulus_primitives_storage_weight_reclaim::StorageWeightReclaim<T>
     **/
    CumulusPrimitivesStorageWeightReclaimStorageWeightReclaim: string;
    /**
     * Lookup704: moonriver_runtime::Runtime
     **/
    MoonriverRuntimeRuntime: string;
};
export default _default;
//# sourceMappingURL=lookup.d.ts.map