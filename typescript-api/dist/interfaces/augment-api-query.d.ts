import "@polkadot/api-base/types/storage";
import type { ApiTypes, AugmentedQuery, QueryableStorageEntry } from "@polkadot/api-base/types";
import type { Data } from "@polkadot/types";
import type { BTreeMap, BTreeSet, Bytes, Null, Option, Struct, U256, U8aFixed, Vec, bool, u128, u16, u32, u64 } from "@polkadot/types-codec";
import type { AnyNumber, ITuple } from "@polkadot/types-codec/types";
import type { AccountId20, Call, H160, H256, Perbill, Percent } from "@polkadot/types/interfaces/runtime";
import type { CumulusPalletDmpQueueMigrationState, CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot, CumulusPalletParachainSystemUnincludedSegmentAncestor, CumulusPalletParachainSystemUnincludedSegmentSegmentTracker, CumulusPalletXcmpQueueOutboundChannelDetails, CumulusPalletXcmpQueueQueueConfigData, CumulusPrimitivesCoreAggregateMessageOrigin, EthereumBlock, EthereumReceiptReceiptV3, EthereumTransactionTransactionV2, FpRpcTransactionStatus, FrameSupportDispatchPerDispatchClassWeight, FrameSystemAccountInfo, FrameSystemCodeUpgradeAuthorization, FrameSystemEventRecord, FrameSystemLastRuntimeUpgradeInfo, FrameSystemPhase, MoonbeamRuntimeRuntimeHoldReason, MoonbeamRuntimeXcmConfigAssetType, NimbusPrimitivesNimbusCryptoPublic, PalletAssetsApproval, PalletAssetsAssetAccount, PalletAssetsAssetDetails, PalletAssetsAssetMetadata, PalletAuthorMappingRegistrationInfo, PalletBalancesAccountData, PalletBalancesBalanceLock, PalletBalancesIdAmount, PalletBalancesReserveData, PalletCollectiveVotes, PalletConvictionVotingVoteVoting, PalletCrowdloanRewardsRewardInfo, PalletEvmCodeMetadata, PalletIdentityAuthorityProperties, PalletIdentityRegistrarInfo, PalletIdentityRegistration, PalletMessageQueueBookState, PalletMessageQueuePage, PalletMoonbeamOrbitersCollatorPoolInfo, PalletMultisigMultisig, PalletParachainStakingAutoCompoundAutoCompoundConfig, PalletParachainStakingBond, PalletParachainStakingCandidateMetadata, PalletParachainStakingCollatorSnapshot, PalletParachainStakingDelayedPayout, PalletParachainStakingDelegationRequestsScheduledRequest, PalletParachainStakingDelegations, PalletParachainStakingDelegator, PalletParachainStakingInflationInflationInfo, PalletParachainStakingParachainBondConfig, PalletParachainStakingRoundInfo, PalletPreimageOldRequestStatus, PalletPreimageRequestStatus, PalletProxyAnnouncement, PalletProxyProxyDefinition, PalletRandomnessRandomnessResult, PalletRandomnessRequestState, PalletRandomnessRequestType, PalletReferendaReferendumInfo, PalletSchedulerRetryConfig, PalletSchedulerScheduled, PalletTransactionPaymentReleases, PalletTreasuryProposal, PalletTreasurySpendStatus, PalletXcmQueryStatus, PalletXcmRemoteLockedFungibleRecord, PalletXcmTransactorRelayIndicesRelayChainIndices, PalletXcmTransactorRemoteTransactInfoWithMaxWeight, PalletXcmVersionMigrationStage, PolkadotCorePrimitivesOutboundHrmpMessage, PolkadotPrimitivesV7AbridgedHostConfiguration, PolkadotPrimitivesV7PersistedValidationData, PolkadotPrimitivesV7UpgradeGoAhead, PolkadotPrimitivesV7UpgradeRestriction, SpRuntimeDigest, SpTrieStorageProof, SpWeightsWeightV2Weight, StagingXcmV4Location, XcmVersionedAssetId, XcmVersionedLocation } from "@polkadot/types/lookup";
import type { Observable } from "@polkadot/types/types";
export type __AugmentedQuery<ApiType extends ApiTypes> = AugmentedQuery<ApiType, () => unknown>;
export type __QueryableStorageEntry<ApiType extends ApiTypes> = QueryableStorageEntry<ApiType>;
declare module "@polkadot/api-base/types/storage" {
    interface AugmentedQueries<ApiType extends ApiTypes> {
        assetManager: {
            /**
             * Mapping from an asset id to asset type. This is mostly used when receiving transaction
             * specifying an asset directly, like transferring an asset from this chain to another.
             */
            assetIdType: AugmentedQuery<ApiType, (arg: u128 | AnyNumber | Uint8Array) => Observable<Option<MoonbeamRuntimeXcmConfigAssetType>>, [
                u128
            ]> & QueryableStorageEntry<ApiType, [u128]>;
            /**
             * Reverse mapping of AssetIdType. Mapping from an asset type to an asset id. This is mostly
             * used when receiving a multilocation XCM message to retrieve the corresponding asset in
             * which tokens should me minted.
             */
            assetTypeId: AugmentedQuery<ApiType, (arg: MoonbeamRuntimeXcmConfigAssetType | {
                Xcm: any;
            } | string | Uint8Array) => Observable<Option<u128>>, [
                MoonbeamRuntimeXcmConfigAssetType
            ]> & QueryableStorageEntry<ApiType, [MoonbeamRuntimeXcmConfigAssetType]>;
            /**
             * Stores the units per second for local execution for a AssetType. This is used to know how
             * to charge for XCM execution in a particular asset Not all assets might contain units per
             * second, hence the different storage
             */
            assetTypeUnitsPerSecond: AugmentedQuery<ApiType, (arg: MoonbeamRuntimeXcmConfigAssetType | {
                Xcm: any;
            } | string | Uint8Array) => Observable<Option<u128>>, [
                MoonbeamRuntimeXcmConfigAssetType
            ]> & QueryableStorageEntry<ApiType, [MoonbeamRuntimeXcmConfigAssetType]>;
            supportedFeePaymentAssets: AugmentedQuery<ApiType, () => Observable<Vec<MoonbeamRuntimeXcmConfigAssetType>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        assets: {
            /** The holdings of a specific account for a specific asset. */
            account: AugmentedQuery<ApiType, (arg1: u128 | AnyNumber | Uint8Array, arg2: AccountId20 | string | Uint8Array) => Observable<Option<PalletAssetsAssetAccount>>, [
                u128,
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [u128, AccountId20]>;
            /**
             * Approved balance transfers. First balance is the amount approved for transfer. Second is
             * the amount of `T::Currency` reserved for storing this. First key is the asset ID, second
             * key is the owner and third key is the delegate.
             */
            approvals: AugmentedQuery<ApiType, (arg1: u128 | AnyNumber | Uint8Array, arg2: AccountId20 | string | Uint8Array, arg3: AccountId20 | string | Uint8Array) => Observable<Option<PalletAssetsApproval>>, [
                u128,
                AccountId20,
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [u128, AccountId20, AccountId20]>;
            /** Details of an asset. */
            asset: AugmentedQuery<ApiType, (arg: u128 | AnyNumber | Uint8Array) => Observable<Option<PalletAssetsAssetDetails>>, [
                u128
            ]> & QueryableStorageEntry<ApiType, [u128]>;
            /** Metadata of an asset. */
            metadata: AugmentedQuery<ApiType, (arg: u128 | AnyNumber | Uint8Array) => Observable<PalletAssetsAssetMetadata>, [
                u128
            ]> & QueryableStorageEntry<ApiType, [u128]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        asyncBacking: {
            /**
             * First tuple element is the highest slot that has been seen in the history of this chain.
             * Second tuple element is the number of authored blocks so far. This is a strictly-increasing
             * value if T::AllowMultipleBlocksPerSlot = false.
             */
            slotInfo: AugmentedQuery<ApiType, () => Observable<Option<ITuple<[u64, u32]>>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        authorFilter: {
            /** The number of active authors that will be eligible at each height. */
            eligibleCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            eligibleRatio: AugmentedQuery<ApiType, () => Observable<Percent>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        authorInherent: {
            /** Author of current block. */
            author: AugmentedQuery<ApiType, () => Observable<Option<AccountId20>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Check if the inherent was included */
            inherentIncluded: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        authorMapping: {
            /** We maintain a mapping from the NimbusIds used in the consensus layer to the AccountIds runtime. */
            mappingWithDeposit: AugmentedQuery<ApiType, (arg: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array) => Observable<Option<PalletAuthorMappingRegistrationInfo>>, [
                NimbusPrimitivesNimbusCryptoPublic
            ]> & QueryableStorageEntry<ApiType, [NimbusPrimitivesNimbusCryptoPublic]>;
            /** We maintain a reverse mapping from AccountIds to NimbusIDS */
            nimbusLookup: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<NimbusPrimitivesNimbusCryptoPublic>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        balances: {
            /**
             * The Balances pallet example of storing the balance of an account.
             *
             * # Example
             *
             * ```nocompile
             * impl pallet_balances::Config for Runtime {
             * type AccountStore = StorageMapShim<Self::Account<Runtime>, frame_system::Provider<Runtime>, AccountId, Self::AccountData<Balance>>
             * }
             * ```
             *
             * You can also store the balance of an account in the `System` pallet.
             *
             * # Example
             *
             * ```nocompile
             * impl pallet_balances::Config for Runtime {
             * type AccountStore = System
             * }
             * ```
             *
             * But this comes with tradeoffs, storing account balances in the system pallet stores
             * `frame_system` data alongside the account data contrary to storing account balances in the
             * `Balances` pallet, which uses a `StorageMap` to store balances data only. NOTE: This is
             * only used in the case that this pallet is used to store balances.
             */
            account: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<PalletBalancesAccountData>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Freeze locks on account balances. */
            freezes: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Vec<PalletBalancesIdAmount>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Holds on account balances. */
            holds: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Vec<{
                readonly id: MoonbeamRuntimeRuntimeHoldReason;
                readonly amount: u128;
            } & Struct>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** The total units of outstanding deactivated balance in the system. */
            inactiveIssuance: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Any liquidity locks on some account balances. NOTE: Should only be accessed when setting,
             * changing and freeing a lock.
             *
             * Use of locks is deprecated in favour of freezes. See
             * `https://github.com/paritytech/substrate/pull/12951/`
             */
            locks: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Vec<PalletBalancesBalanceLock>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /**
             * Named reserves on some account balances.
             *
             * Use of reserves is deprecated in favour of holds. See
             * `https://github.com/paritytech/substrate/pull/12951/`
             */
            reserves: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Vec<PalletBalancesReserveData>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** The total units issued in the system. */
            totalIssuance: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        convictionVoting: {
            /**
             * The voting classes which have a non-zero lock requirement and the lock amounts which they
             * require. The actual amount locked on behalf of this pallet should always be the maximum of
             * this list.
             */
            classLocksFor: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Vec<ITuple<[u16, u128]>>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /**
             * All voting for a particular voter in a particular voting class. We store the balance for
             * the number of votes that we have recorded.
             */
            votingFor: AugmentedQuery<ApiType, (arg1: AccountId20 | string | Uint8Array, arg2: u16 | AnyNumber | Uint8Array) => Observable<PalletConvictionVotingVoteVoting>, [
                AccountId20,
                u16
            ]> & QueryableStorageEntry<ApiType, [AccountId20, u16]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        crowdloanRewards: {
            accountsPayable: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<PalletCrowdloanRewardsRewardInfo>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            claimedRelayChainIds: AugmentedQuery<ApiType, (arg: U8aFixed | string | Uint8Array) => Observable<Option<Null>>, [
                U8aFixed
            ]> & QueryableStorageEntry<ApiType, [U8aFixed]>;
            /** Vesting block height at the initialization of the pallet */
            endRelayBlock: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            initialized: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Total initialized amount so far. We store this to make pallet funds == contributors reward
             * check easier and more efficient
             */
            initializedRewardAmount: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
            /** Vesting block height at the initialization of the pallet */
            initRelayBlock: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Total number of contributors to aid hinting benchmarking */
            totalContributors: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            unassociatedContributions: AugmentedQuery<ApiType, (arg: U8aFixed | string | Uint8Array) => Observable<Option<PalletCrowdloanRewardsRewardInfo>>, [
                U8aFixed
            ]> & QueryableStorageEntry<ApiType, [U8aFixed]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        dmpQueue: {
            /** The migration state of this pallet. */
            migrationStatus: AugmentedQuery<ApiType, () => Observable<CumulusPalletDmpQueueMigrationState>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        ethereum: {
            blockHash: AugmentedQuery<ApiType, (arg: U256 | AnyNumber | Uint8Array) => Observable<H256>, [
                U256
            ]> & QueryableStorageEntry<ApiType, [U256]>;
            /** The current Ethereum block. */
            currentBlock: AugmentedQuery<ApiType, () => Observable<Option<EthereumBlock>>, []> & QueryableStorageEntry<ApiType, []>;
            /** The current Ethereum receipts. */
            currentReceipts: AugmentedQuery<ApiType, () => Observable<Option<Vec<EthereumReceiptReceiptV3>>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** The current transaction statuses. */
            currentTransactionStatuses: AugmentedQuery<ApiType, () => Observable<Option<Vec<FpRpcTransactionStatus>>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Current building block's transactions and receipts. */
            pending: AugmentedQuery<ApiType, () => Observable<Vec<ITuple<[
                EthereumTransactionTransactionV2,
                FpRpcTransactionStatus,
                EthereumReceiptReceiptV3
            ]>>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        ethereumChainId: {
            /** The EVM chain ID. */
            chainId: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        ethereumXcm: {
            /** Whether or not Ethereum-XCM is suspended from executing */
            ethereumXcmSuspended: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Global nonce used for building Ethereum transaction payload. */
            nonce: AugmentedQuery<ApiType, () => Observable<U256>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        evm: {
            accountCodes: AugmentedQuery<ApiType, (arg: H160 | string | Uint8Array) => Observable<Bytes>, [
                H160
            ]> & QueryableStorageEntry<ApiType, [H160]>;
            accountCodesMetadata: AugmentedQuery<ApiType, (arg: H160 | string | Uint8Array) => Observable<Option<PalletEvmCodeMetadata>>, [
                H160
            ]> & QueryableStorageEntry<ApiType, [H160]>;
            accountStorages: AugmentedQuery<ApiType, (arg1: H160 | string | Uint8Array, arg2: H256 | string | Uint8Array) => Observable<H256>, [
                H160,
                H256
            ]> & QueryableStorageEntry<ApiType, [H160, H256]>;
            suicided: AugmentedQuery<ApiType, (arg: H160 | string | Uint8Array) => Observable<Option<Null>>, [
                H160
            ]> & QueryableStorageEntry<ApiType, [H160]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        identity: {
            /**
             * Reverse lookup from `username` to the `AccountId` that has registered it. The value should
             * be a key in the `IdentityOf` map, but it may not if the user has cleared their identity.
             *
             * Multiple usernames may map to the same `AccountId`, but `IdentityOf` will only map to one
             * primary username.
             */
            accountOfUsername: AugmentedQuery<ApiType, (arg: Bytes | string | Uint8Array) => Observable<Option<AccountId20>>, [
                Bytes
            ]> & QueryableStorageEntry<ApiType, [Bytes]>;
            /**
             * Information that is pertinent to identify the entity behind an account. First item is the
             * registration, second is the account's primary username.
             *
             * TWOX-NOTE: OK ― `AccountId` is a secure hash.
             */
            identityOf: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<ITuple<[PalletIdentityRegistration, Option<Bytes>]>>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /**
             * Usernames that an authority has granted, but that the account controller has not confirmed
             * that they want it. Used primarily in cases where the `AccountId` cannot provide a signature
             * because they are a pure proxy, multisig, etc. In order to confirm it, they should call
             * [`Call::accept_username`].
             *
             * First tuple item is the account and second is the acceptance deadline.
             */
            pendingUsernames: AugmentedQuery<ApiType, (arg: Bytes | string | Uint8Array) => Observable<Option<ITuple<[AccountId20, u32]>>>, [
                Bytes
            ]> & QueryableStorageEntry<ApiType, [Bytes]>;
            /**
             * The set of registrars. Not expected to get very big as can only be added through a special
             * origin (likely a council motion).
             *
             * The index into this can be cast to `RegistrarIndex` to get a valid value.
             */
            registrars: AugmentedQuery<ApiType, () => Observable<Vec<Option<PalletIdentityRegistrarInfo>>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * Alternative "sub" identities of this account.
             *
             * The first item is the deposit, the second is a vector of the accounts.
             *
             * TWOX-NOTE: OK ― `AccountId` is a secure hash.
             */
            subsOf: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<ITuple<[u128, Vec<AccountId20>]>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /**
             * The super-identity of an alternative "sub" identity together with its name, within that
             * context. If the account is not some other account's sub-identity, then just `None`.
             */
            superOf: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<ITuple<[AccountId20, Data]>>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** A map of the accounts who are authorized to grant usernames. */
            usernameAuthorities: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<PalletIdentityAuthorityProperties>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        maintenanceMode: {
            /** Whether the site is in maintenance mode */
            maintenanceMode: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        messageQueue: {
            /** The index of the first and last (non-empty) pages. */
            bookStateFor: AugmentedQuery<ApiType, (arg: CumulusPrimitivesCoreAggregateMessageOrigin | {
                Here: any;
            } | {
                Parent: any;
            } | {
                Sibling: any;
            } | string | Uint8Array) => Observable<PalletMessageQueueBookState>, [
                CumulusPrimitivesCoreAggregateMessageOrigin
            ]> & QueryableStorageEntry<ApiType, [CumulusPrimitivesCoreAggregateMessageOrigin]>;
            /** The map of page indices to pages. */
            pages: AugmentedQuery<ApiType, (arg1: CumulusPrimitivesCoreAggregateMessageOrigin | {
                Here: any;
            } | {
                Parent: any;
            } | {
                Sibling: any;
            } | string | Uint8Array, arg2: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletMessageQueuePage>>, [
                CumulusPrimitivesCoreAggregateMessageOrigin,
                u32
            ]> & QueryableStorageEntry<ApiType, [CumulusPrimitivesCoreAggregateMessageOrigin, u32]>;
            /** The origin at which we should begin servicing. */
            serviceHead: AugmentedQuery<ApiType, () => Observable<Option<CumulusPrimitivesCoreAggregateMessageOrigin>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        migrations: {
            /** True if all required migrations have completed */
            fullyUpgraded: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * MigrationState tracks the progress of a migration. Maps name (Vec<u8>) -> whether or not
             * migration has been completed (bool)
             */
            migrationState: AugmentedQuery<ApiType, (arg: Bytes | string | Uint8Array) => Observable<bool>, [
                Bytes
            ]> & QueryableStorageEntry<ApiType, [Bytes]>;
            /**
             * Temporary value that is set to true at the beginning of the block during which the
             * execution of xcm messages must be paused.
             */
            shouldPauseXcm: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        moonbeamLazyMigrations: {
            /** The total number of suicided contracts that were removed */
            suicidedContractsRemoved: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        moonbeamOrbiters: {
            /** Account lookup override */
            accountLookupOverride: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<Option<AccountId20>>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Current orbiters, with their "parent" collator */
            collatorsPool: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<PalletMoonbeamOrbitersCollatorPoolInfo>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Counter for the related counted storage map */
            counterForCollatorsPool: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Current round index */
            currentRound: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * If true, it forces the rotation at the next round. A use case: when changing RotatePeriod,
             * you need a migration code that sets this value to true to avoid holes in OrbiterPerRound.
             */
            forceRotation: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Minimum deposit required to be registered as an orbiter */
            minOrbiterDeposit: AugmentedQuery<ApiType, () => Observable<Option<u128>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Store active orbiter per round and per parent collator */
            orbiterPerRound: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId20 | string | Uint8Array) => Observable<Option<AccountId20>>, [
                u32,
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [u32, AccountId20]>;
            /** Check if account is an orbiter */
            registeredOrbiter: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<bool>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        multisig: {
            /** The set of open multisig operations. */
            multisigs: AugmentedQuery<ApiType, (arg1: AccountId20 | string | Uint8Array, arg2: U8aFixed | string | Uint8Array) => Observable<Option<PalletMultisigMultisig>>, [
                AccountId20,
                U8aFixed
            ]> & QueryableStorageEntry<ApiType, [AccountId20, U8aFixed]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        openTechCommitteeCollective: {
            /** The current members of the collective. This is stored sorted (just by value). */
            members: AugmentedQuery<ApiType, () => Observable<Vec<AccountId20>>, []> & QueryableStorageEntry<ApiType, []>;
            /** The prime member that helps determine the default vote behavior in case of abstentions. */
            prime: AugmentedQuery<ApiType, () => Observable<Option<AccountId20>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Proposals so far. */
            proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Actual proposal for a given hash, if it's current. */
            proposalOf: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<Call>>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** The hashes of the active proposals. */
            proposals: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Votes on a given proposal, if it is ongoing. */
            voting: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletCollectiveVotes>>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        parachainInfo: {
            parachainId: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        parachainStaking: {
            /** Snapshot of collator delegation stake at the start of the round */
            atStake: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId20 | string | Uint8Array) => Observable<Option<PalletParachainStakingCollatorSnapshot>>, [
                u32,
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [u32, AccountId20]>;
            /** Stores auto-compounding configuration per collator. */
            autoCompoundingDelegations: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Vec<PalletParachainStakingAutoCompoundAutoCompoundConfig>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Points for each collator per round */
            awardedPts: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId20 | string | Uint8Array) => Observable<u32>, [
                u32,
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [u32, AccountId20]>;
            /** Bottom delegations for collator candidate */
            bottomDelegations: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<PalletParachainStakingDelegations>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Get collator candidate info associated with an account if account is candidate else None */
            candidateInfo: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<PalletParachainStakingCandidateMetadata>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** The pool of collator candidates, each with their total backing stake */
            candidatePool: AugmentedQuery<ApiType, () => Observable<Vec<PalletParachainStakingBond>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Commission percent taken off of rewards for all collators */
            collatorCommission: AugmentedQuery<ApiType, () => Observable<Perbill>, []> & QueryableStorageEntry<ApiType, []>;
            /** Delayed payouts */
            delayedPayouts: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletParachainStakingDelayedPayout>>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /** Stores outstanding delegation requests per collator. */
            delegationScheduledRequests: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Vec<PalletParachainStakingDelegationRequestsScheduledRequest>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Get delegator state associated with an account if account is delegating else None */
            delegatorState: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<PalletParachainStakingDelegator>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Killswitch to enable/disable marking offline feature. */
            enableMarkingOffline: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Inflation configuration */
            inflationConfig: AugmentedQuery<ApiType, () => Observable<PalletParachainStakingInflationInflationInfo>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Parachain bond config info { account, percent_of_inflation } */
            parachainBondInfo: AugmentedQuery<ApiType, () => Observable<PalletParachainStakingParachainBondConfig>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Total points awarded to collators for block production in the round */
            points: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<u32>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /** Current round index and next round scheduled transition */
            round: AugmentedQuery<ApiType, () => Observable<PalletParachainStakingRoundInfo>, []> & QueryableStorageEntry<ApiType, []>;
            /** The collator candidates selected for the current round */
            selectedCandidates: AugmentedQuery<ApiType, () => Observable<Vec<AccountId20>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Top delegations for collator candidate */
            topDelegations: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<PalletParachainStakingDelegations>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Total capital locked by this staking pallet */
            total: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
            /** The total candidates selected every round */
            totalSelected: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        parachainSystem: {
            /**
             * Storage field that keeps track of bandwidth used by the unincluded segment along with the
             * latest HRMP watermark. Used for limiting the acceptance of new blocks with respect to relay
             * chain constraints.
             */
            aggregatedUnincludedSegment: AugmentedQuery<ApiType, () => Observable<Option<CumulusPalletParachainSystemUnincludedSegmentSegmentTracker>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * The number of HRMP messages we observed in `on_initialize` and thus used that number for
             * announcing the weight of `on_initialize` and `on_finalize`.
             */
            announcedHrmpMessagesPerCandidate: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * A custom head data that should be returned as result of `validate_block`.
             *
             * See `Pallet::set_custom_validation_head_data` for more information.
             */
            customValidationHeadData: AugmentedQuery<ApiType, () => Observable<Option<Bytes>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Were the validation data set to notify the relay chain? */
            didSetValidationCode: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * The parachain host configuration that was obtained from the relay parent.
             *
             * This field is meant to be updated each block with the validation data inherent. Therefore,
             * before processing of the inherent, e.g. in `on_initialize` this data may be stale.
             *
             * This data is also absent from the genesis.
             */
            hostConfiguration: AugmentedQuery<ApiType, () => Observable<Option<PolkadotPrimitivesV7AbridgedHostConfiguration>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * HRMP messages that were sent in a block.
             *
             * This will be cleared in `on_initialize` of each new block.
             */
            hrmpOutboundMessages: AugmentedQuery<ApiType, () => Observable<Vec<PolkadotCorePrimitivesOutboundHrmpMessage>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * HRMP watermark that was set in a block.
             *
             * This will be cleared in `on_initialize` of each new block.
             */
            hrmpWatermark: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * The last downward message queue chain head we have observed.
             *
             * This value is loaded before and saved after processing inbound downward messages carried by
             * the system inherent.
             */
            lastDmqMqcHead: AugmentedQuery<ApiType, () => Observable<H256>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * The message queue chain heads we have observed per each channel incoming channel.
             *
             * This value is loaded before and saved after processing inbound downward messages carried by
             * the system inherent.
             */
            lastHrmpMqcHeads: AugmentedQuery<ApiType, () => Observable<BTreeMap<u32, H256>>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * The relay chain block number associated with the last parachain block.
             *
             * This is updated in `on_finalize`.
             */
            lastRelayChainBlockNumber: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Validation code that is set by the parachain and is to be communicated to collator and
             * consequently the relay-chain.
             *
             * This will be cleared in `on_initialize` of each new block if no other pallet already set the value.
             */
            newValidationCode: AugmentedQuery<ApiType, () => Observable<Option<Bytes>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Upward messages that are still pending and not yet send to the relay chain. */
            pendingUpwardMessages: AugmentedQuery<ApiType, () => Observable<Vec<Bytes>>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * In case of a scheduled upgrade, this storage field contains the validation code to be applied.
             *
             * As soon as the relay chain gives us the go-ahead signal, we will overwrite the
             * [`:code`][sp_core::storage::well_known_keys::CODE] which will result the next block process
             * with the new validation code. This concludes the upgrade process.
             */
            pendingValidationCode: AugmentedQuery<ApiType, () => Observable<Bytes>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Number of downward messages processed in a block.
             *
             * This will be cleared in `on_initialize` of each new block.
             */
            processedDownwardMessages: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * The state proof for the last relay parent block.
             *
             * This field is meant to be updated each block with the validation data inherent. Therefore,
             * before processing of the inherent, e.g. in `on_initialize` this data may be stale.
             *
             * This data is also absent from the genesis.
             */
            relayStateProof: AugmentedQuery<ApiType, () => Observable<Option<SpTrieStorageProof>>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * The snapshot of some state related to messaging relevant to the current parachain as per
             * the relay parent.
             *
             * This field is meant to be updated each block with the validation data inherent. Therefore,
             * before processing of the inherent, e.g. in `on_initialize` this data may be stale.
             *
             * This data is also absent from the genesis.
             */
            relevantMessagingState: AugmentedQuery<ApiType, () => Observable<Option<CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * The weight we reserve at the beginning of the block for processing DMP messages. This
             * overrides the amount set in the Config trait.
             */
            reservedDmpWeightOverride: AugmentedQuery<ApiType, () => Observable<Option<SpWeightsWeightV2Weight>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * The weight we reserve at the beginning of the block for processing XCMP messages. This
             * overrides the amount set in the Config trait.
             */
            reservedXcmpWeightOverride: AugmentedQuery<ApiType, () => Observable<Option<SpWeightsWeightV2Weight>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * Latest included block descendants the runtime accepted. In other words, these are ancestors
             * of the currently executing block which have not been included in the observed relay-chain state.
             *
             * The segment length is limited by the capacity returned from the [`ConsensusHook`]
             * configured in the pallet.
             */
            unincludedSegment: AugmentedQuery<ApiType, () => Observable<Vec<CumulusPalletParachainSystemUnincludedSegmentAncestor>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * Optional upgrade go-ahead signal from the relay-chain.
             *
             * This storage item is a mirror of the corresponding value for the current parachain from the
             * relay-chain. This value is ephemeral which means it doesn't hit the storage. This value is
             * set after the inherent.
             */
            upgradeGoAhead: AugmentedQuery<ApiType, () => Observable<Option<PolkadotPrimitivesV7UpgradeGoAhead>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * An option which indicates if the relay-chain restricts signalling a validation code
             * upgrade. In other words, if this is `Some` and [`NewValidationCode`] is `Some` then the
             * produced candidate will be invalid.
             *
             * This storage item is a mirror of the corresponding value for the current parachain from the
             * relay-chain. This value is ephemeral which means it doesn't hit the storage. This value is
             * set after the inherent.
             */
            upgradeRestrictionSignal: AugmentedQuery<ApiType, () => Observable<Option<PolkadotPrimitivesV7UpgradeRestriction>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** The factor to multiply the base delivery fee by for UMP. */
            upwardDeliveryFeeFactor: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Upward messages that were sent in a block.
             *
             * This will be cleared in `on_initialize` of each new block.
             */
            upwardMessages: AugmentedQuery<ApiType, () => Observable<Vec<Bytes>>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * The [`PersistedValidationData`] set for this block. This value is expected to be set only
             * once per block and it's never stored in the trie.
             */
            validationData: AugmentedQuery<ApiType, () => Observable<Option<PolkadotPrimitivesV7PersistedValidationData>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        polkadotXcm: {
            /**
             * The existing asset traps.
             *
             * Key is the blake2 256 hash of (origin, versioned `Assets`) pair. Value is the number of
             * times this pair has been trapped (usually just 1 if it exists at all).
             */
            assetTraps: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<u32>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** The current migration's stage, if any. */
            currentMigration: AugmentedQuery<ApiType, () => Observable<Option<PalletXcmVersionMigrationStage>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Fungible assets which we know are locked on this chain. */
            lockedFungibles: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<Option<Vec<ITuple<[u128, XcmVersionedLocation]>>>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** The ongoing queries. */
            queries: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<PalletXcmQueryStatus>>, [
                u64
            ]> & QueryableStorageEntry<ApiType, [u64]>;
            /** The latest available query index. */
            queryCounter: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
            /** Fungible assets which we know are locked on a remote chain. */
            remoteLockedFungibles: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId20 | string | Uint8Array, arg3: XcmVersionedAssetId | {
                V3: any;
            } | {
                V4: any;
            } | string | Uint8Array) => Observable<Option<PalletXcmRemoteLockedFungibleRecord>>, [
                u32,
                AccountId20,
                XcmVersionedAssetId
            ]> & QueryableStorageEntry<ApiType, [u32, AccountId20, XcmVersionedAssetId]>;
            /**
             * Default version to encode XCM when latest version of destination is unknown. If `None`,
             * then the destinations whose XCM version is unknown are considered unreachable.
             */
            safeXcmVersion: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
            /** The Latest versions that we know various locations support. */
            supportedVersion: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: XcmVersionedLocation | {
                V2: any;
            } | {
                V3: any;
            } | {
                V4: any;
            } | string | Uint8Array) => Observable<Option<u32>>, [
                u32,
                XcmVersionedLocation
            ]> & QueryableStorageEntry<ApiType, [u32, XcmVersionedLocation]>;
            /**
             * Destinations whose latest XCM version we would like to know. Duplicates not allowed, and
             * the `u32` counter is the number of times that a send to the destination has been attempted,
             * which is used as a prioritization.
             */
            versionDiscoveryQueue: AugmentedQuery<ApiType, () => Observable<Vec<ITuple<[XcmVersionedLocation, u32]>>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** All locations that we have requested version notifications from. */
            versionNotifiers: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: XcmVersionedLocation | {
                V2: any;
            } | {
                V3: any;
            } | {
                V4: any;
            } | string | Uint8Array) => Observable<Option<u64>>, [
                u32,
                XcmVersionedLocation
            ]> & QueryableStorageEntry<ApiType, [u32, XcmVersionedLocation]>;
            /**
             * The target locations that are subscribed to our version changes, as well as the most recent
             * of our versions we informed them of.
             */
            versionNotifyTargets: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: XcmVersionedLocation | {
                V2: any;
            } | {
                V3: any;
            } | {
                V4: any;
            } | string | Uint8Array) => Observable<Option<ITuple<[u64, SpWeightsWeightV2Weight, u32]>>>, [
                u32,
                XcmVersionedLocation
            ]> & QueryableStorageEntry<ApiType, [u32, XcmVersionedLocation]>;
            /** Global suspension state of the XCM executor. */
            xcmExecutionSuspended: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        preimage: {
            preimageFor: AugmentedQuery<ApiType, (arg: ITuple<[H256, u32]> | [H256 | string | Uint8Array, u32 | AnyNumber | Uint8Array]) => Observable<Option<Bytes>>, [
                ITuple<[H256, u32]>
            ]> & QueryableStorageEntry<ApiType, [ITuple<[H256, u32]>]>;
            /** The request status of a given hash. */
            requestStatusFor: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletPreimageRequestStatus>>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** The request status of a given hash. */
            statusFor: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletPreimageOldRequestStatus>>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        proxy: {
            /** The announcements made by the proxy (key). */
            announcements: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<ITuple<[Vec<PalletProxyAnnouncement>, u128]>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /**
             * The set of account proxies. Maps the account which has delegated to the accounts which are
             * being delegated to, together with the amount held on deposit.
             */
            proxies: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<ITuple<[Vec<PalletProxyProxyDefinition>, u128]>>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        randomness: {
            /** Ensures the mandatory inherent was included in the block */
            inherentIncluded: AugmentedQuery<ApiType, () => Observable<Option<Null>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Current local per-block VRF randomness Set in `on_initialize` */
            localVrfOutput: AugmentedQuery<ApiType, () => Observable<Option<H256>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Records whether this is the first block (genesis or runtime upgrade) */
            notFirstBlock: AugmentedQuery<ApiType, () => Observable<Option<Null>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Previous local per-block VRF randomness Set in `on_finalize` of last block */
            previousLocalVrfOutput: AugmentedQuery<ApiType, () => Observable<H256>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Snapshot of randomness to fulfill all requests that are for the same raw randomness Removed
             * once $value.request_count == 0
             */
            randomnessResults: AugmentedQuery<ApiType, (arg: PalletRandomnessRequestType | {
                BabeEpoch: any;
            } | {
                Local: any;
            } | string | Uint8Array) => Observable<Option<PalletRandomnessRandomnessResult>>, [
                PalletRandomnessRequestType
            ]> & QueryableStorageEntry<ApiType, [PalletRandomnessRequestType]>;
            /** Relay epoch */
            relayEpoch: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
            /** Number of randomness requests made so far, used to generate the next request's uid */
            requestCount: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
            /** Randomness requests not yet fulfilled or purged */
            requests: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<PalletRandomnessRequestState>>, [
                u64
            ]> & QueryableStorageEntry<ApiType, [u64]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        referenda: {
            /** The number of referenda being decided currently. */
            decidingCount: AugmentedQuery<ApiType, (arg: u16 | AnyNumber | Uint8Array) => Observable<u32>, [
                u16
            ]> & QueryableStorageEntry<ApiType, [u16]>;
            /**
             * The metadata is a general information concerning the referendum. The `Hash` refers to the
             * preimage of the `Preimages` provider which can be a JSON dump or IPFS hash of a JSON file.
             *
             * Consider a garbage collection for a metadata of finished referendums to `unrequest`
             * (remove) large preimages.
             */
            metadataOf: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<H256>>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /** The next free referendum index, aka the number of referenda started so far. */
            referendumCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Information concerning any given referendum. */
            referendumInfoFor: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletReferendaReferendumInfo>>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /**
             * The sorted list of referenda ready to be decided but not yet being decided, ordered by
             * conviction-weighted approvals.
             *
             * This should be empty if `DecidingCount` is less than `TrackInfo::max_deciding`.
             */
            trackQueue: AugmentedQuery<ApiType, (arg: u16 | AnyNumber | Uint8Array) => Observable<Vec<ITuple<[u32, u128]>>>, [
                u16
            ]> & QueryableStorageEntry<ApiType, [u16]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        relayStorageRoots: {
            /** Map of relay block number to relay storage root */
            relayStorageRoot: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<H256>>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /**
             * List of all the keys in `RelayStorageRoot`. Used to remove the oldest key without having to
             * iterate over all of them.
             */
            relayStorageRootKeys: AugmentedQuery<ApiType, () => Observable<Vec<u32>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        rootTesting: {
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        scheduler: {
            /** Items to be executed, indexed by the block number that they should be executed on. */
            agenda: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Vec<Option<PalletSchedulerScheduled>>>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            incompleteSince: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Lookup from a name to the block number and index of the task.
             *
             * For v3 -> v4 the previously unbounded identities are Blake2-256 hashed to form the v4 identities.
             */
            lookup: AugmentedQuery<ApiType, (arg: U8aFixed | string | Uint8Array) => Observable<Option<ITuple<[u32, u32]>>>, [
                U8aFixed
            ]> & QueryableStorageEntry<ApiType, [U8aFixed]>;
            /** Retry configurations for items to be executed, indexed by task address. */
            retries: AugmentedQuery<ApiType, (arg: ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array]) => Observable<Option<PalletSchedulerRetryConfig>>, [
                ITuple<[u32, u32]>
            ]> & QueryableStorageEntry<ApiType, [ITuple<[u32, u32]>]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        system: {
            /** The full account information for a particular account ID. */
            account: AugmentedQuery<ApiType, (arg: AccountId20 | string | Uint8Array) => Observable<FrameSystemAccountInfo>, [
                AccountId20
            ]> & QueryableStorageEntry<ApiType, [AccountId20]>;
            /** Total length (in bytes) for all extrinsics put together, for the current block. */
            allExtrinsicsLen: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
            /** `Some` if a code upgrade has been authorized. */
            authorizedUpgrade: AugmentedQuery<ApiType, () => Observable<Option<FrameSystemCodeUpgradeAuthorization>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Map of block numbers to block hashes. */
            blockHash: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<H256>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /** The current weight for the block. */
            blockWeight: AugmentedQuery<ApiType, () => Observable<FrameSupportDispatchPerDispatchClassWeight>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Digest of the current block, also part of the block header. */
            digest: AugmentedQuery<ApiType, () => Observable<SpRuntimeDigest>, []> & QueryableStorageEntry<ApiType, []>;
            /** The number of events in the `Events<T>` list. */
            eventCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Events deposited for the current block.
             *
             * NOTE: The item is unbound and should therefore never be read on chain. It could otherwise
             * inflate the PoV size of a block.
             *
             * Events have a large in-memory size. Box the events to not go out-of-memory just in case
             * someone still reads them from within the runtime.
             */
            events: AugmentedQuery<ApiType, () => Observable<Vec<FrameSystemEventRecord>>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * Mapping between a topic (represented by T::Hash) and a vector of indexes of events in the
             * `<Events<T>>` list.
             *
             * All topic vectors have deterministic storage locations depending on the topic. This allows
             * light-clients to leverage the changes trie storage tracking mechanism and in case of
             * changes fetch the list of events of interest.
             *
             * The value has the type `(BlockNumberFor<T>, EventIndex)` because if we used only just the
             * `EventIndex` then in case if the topic has the same contents on the next block no
             * notification will be triggered thus the event might be lost.
             */
            eventTopics: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Vec<ITuple<[u32, u32]>>>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** The execution phase of the block. */
            executionPhase: AugmentedQuery<ApiType, () => Observable<Option<FrameSystemPhase>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Total extrinsics count for the current block. */
            extrinsicCount: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Extrinsics data for the current block (maps an extrinsic's index to its data). */
            extrinsicData: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Bytes>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /** Whether all inherents have been applied. */
            inherentsApplied: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Stores the `spec_version` and `spec_name` of when the last runtime upgrade happened. */
            lastRuntimeUpgrade: AugmentedQuery<ApiType, () => Observable<Option<FrameSystemLastRuntimeUpgradeInfo>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** The current block number being processed. Set by `execute_block`. */
            number: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Hash of the previous block. */
            parentHash: AugmentedQuery<ApiType, () => Observable<H256>, []> & QueryableStorageEntry<ApiType, []>;
            /**
             * True if we have upgraded so that AccountInfo contains three types of `RefCount`. False
             * (default) if not.
             */
            upgradedToTripleRefCount: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** True if we have upgraded so that `type RefCount` is `u32`. False (default) if not. */
            upgradedToU32RefCount: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        timestamp: {
            /**
             * Whether the timestamp has been updated in this block.
             *
             * This value is updated to `true` upon successful submission of a timestamp by a node. It is
             * then checked at the end of each block execution in the `on_finalize` hook.
             */
            didUpdate: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** The current time for the current block. */
            now: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        transactionPayment: {
            nextFeeMultiplier: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
            storageVersion: AugmentedQuery<ApiType, () => Observable<PalletTransactionPaymentReleases>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        treasury: {
            /** Proposal indices that have been approved but not yet awarded. */
            approvals: AugmentedQuery<ApiType, () => Observable<Vec<u32>>, []> & QueryableStorageEntry<ApiType, []>;
            /** The amount which has been reported as inactive to Currency. */
            deactivated: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
            /** Number of proposals that have been made. */
            proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Proposals that have been made. */
            proposals: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletTreasuryProposal>>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /** The count of spends that have been made. */
            spendCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Spends that have been approved and being processed. */
            spends: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletTreasurySpendStatus>>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        treasuryCouncilCollective: {
            /** The current members of the collective. This is stored sorted (just by value). */
            members: AugmentedQuery<ApiType, () => Observable<Vec<AccountId20>>, []> & QueryableStorageEntry<ApiType, []>;
            /** The prime member that helps determine the default vote behavior in case of abstentions. */
            prime: AugmentedQuery<ApiType, () => Observable<Option<AccountId20>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Proposals so far. */
            proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
            /** Actual proposal for a given hash, if it's current. */
            proposalOf: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<Call>>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** The hashes of the active proposals. */
            proposals: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> & QueryableStorageEntry<ApiType, []>;
            /** Votes on a given proposal, if it is ongoing. */
            voting: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletCollectiveVotes>>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        whitelist: {
            whitelistedCall: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<Null>>, [
                H256
            ]> & QueryableStorageEntry<ApiType, [H256]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        xcmpQueue: {
            /** The factor to multiply the base delivery fee by. */
            deliveryFeeFactor: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<u128>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /**
             * The suspended inbound XCMP channels. All others are not suspended.
             *
             * This is a `StorageValue` instead of a `StorageMap` since we expect multiple reads per block
             * to different keys with a one byte payload. The access to `BoundedBTreeSet` will be cached
             * within the block and therefore only included once in the proof size.
             *
             * NOTE: The PoV benchmarking cannot know this and will over-estimate, but the actual proof
             * will be smaller.
             */
            inboundXcmpSuspended: AugmentedQuery<ApiType, () => Observable<BTreeSet<u32>>, []> & QueryableStorageEntry<ApiType, []>;
            /** The messages outbound in a given XCMP channel. */
            outboundXcmpMessages: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: u16 | AnyNumber | Uint8Array) => Observable<Bytes>, [
                u32,
                u16
            ]> & QueryableStorageEntry<ApiType, [u32, u16]>;
            /**
             * The non-empty XCMP channels in order of becoming non-empty, and the index of the first and
             * last outbound message. If the two indices are equal, then it indicates an empty queue and
             * there must be a non-`Ok` `OutboundStatus`. We assume queues grow no greater than 65535
             * items. Queue indices for normal messages begin at one; zero is reserved in case of the need
             * to send a high-priority signal message this block. The bool is true if there is a signal
             * message waiting to be sent.
             */
            outboundXcmpStatus: AugmentedQuery<ApiType, () => Observable<Vec<CumulusPalletXcmpQueueOutboundChannelDetails>>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** The configuration which controls the dynamics of the outbound queue. */
            queueConfig: AugmentedQuery<ApiType, () => Observable<CumulusPalletXcmpQueueQueueConfigData>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /** Whether or not the XCMP queue is suspended from executing incoming XCMs or not. */
            queueSuspended: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
            /** Any signal messages waiting to be sent. */
            signalMessages: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Bytes>, [
                u32
            ]> & QueryableStorageEntry<ApiType, [u32]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        xcmTransactor: {
            /**
             * Stores the fee per second for an asset in its reserve chain. This allows us to convert from
             * weight to fee
             */
            destinationAssetFeePerSecond: AugmentedQuery<ApiType, (arg: StagingXcmV4Location | {
                parents?: any;
                interior?: any;
            } | string | Uint8Array) => Observable<Option<u128>>, [
                StagingXcmV4Location
            ]> & QueryableStorageEntry<ApiType, [StagingXcmV4Location]>;
            /**
             * Since we are using pallet-utility for account derivation (through AsDerivative), we need to
             * provide an index for the account derivation. This storage item stores the index assigned
             * for a given local account. These indices are usable as derivative in the relay chain
             */
            indexToAccount: AugmentedQuery<ApiType, (arg: u16 | AnyNumber | Uint8Array) => Observable<Option<AccountId20>>, [
                u16
            ]> & QueryableStorageEntry<ApiType, [u16]>;
            /** Stores the indices of relay chain pallets */
            relayIndices: AugmentedQuery<ApiType, () => Observable<PalletXcmTransactorRelayIndicesRelayChainIndices>, [
            ]> & QueryableStorageEntry<ApiType, []>;
            /**
             * Stores the transact info of a Location. This defines how much extra weight we need to add
             * when we want to transact in the destination chain and maximum amount of weight allowed by
             * the destination chain
             */
            transactInfoWithWeightLimit: AugmentedQuery<ApiType, (arg: StagingXcmV4Location | {
                parents?: any;
                interior?: any;
            } | string | Uint8Array) => Observable<Option<PalletXcmTransactorRemoteTransactInfoWithMaxWeight>>, [
                StagingXcmV4Location
            ]> & QueryableStorageEntry<ApiType, [StagingXcmV4Location]>;
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
        xTokens: {
            /** Generic query */
            [key: string]: QueryableStorageEntry<ApiType>;
        };
    }
}
