import '@polkadot/api-base/types/consts';
import type { ApiTypes, AugmentedConst } from '@polkadot/api-base/types';
import type { Bytes, Option, Vec, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { Codec, ITuple } from '@polkadot/types-codec/types';
import type { Perbill, Permill } from '@polkadot/types/interfaces/runtime';
import type { FrameSupportPalletId, FrameSystemLimitsBlockLength, FrameSystemLimitsBlockWeights, PalletReferendaTrackInfo, SpVersionRuntimeVersion, SpWeightsRuntimeDbWeight, SpWeightsWeightV2Weight, StagingXcmV4Location } from '@polkadot/types/lookup';
export type __AugmentedConst<ApiType extends ApiTypes> = AugmentedConst<ApiType>;
declare module '@polkadot/api-base/types/consts' {
    interface AugmentedConsts<ApiType extends ApiTypes> {
        assets: {
            /**
             * The amount of funds that must be reserved when creating a new approval.
             **/
            approvalDeposit: u128 & AugmentedConst<ApiType>;
            /**
             * The amount of funds that must be reserved for a non-provider asset account to be
             * maintained.
             **/
            assetAccountDeposit: u128 & AugmentedConst<ApiType>;
            /**
             * The basic amount of funds that must be reserved for an asset.
             **/
            assetDeposit: u128 & AugmentedConst<ApiType>;
            /**
             * The basic amount of funds that must be reserved when adding metadata to your asset.
             **/
            metadataDepositBase: u128 & AugmentedConst<ApiType>;
            /**
             * The additional funds that must be reserved for the number of bytes you store in your
             * metadata.
             **/
            metadataDepositPerByte: u128 & AugmentedConst<ApiType>;
            /**
             * Max number of items to destroy per `destroy_accounts` and `destroy_approvals` call.
             *
             * Must be configured to result in a weight that makes each call fit in a block.
             **/
            removeItemsLimit: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum length of a name or symbol stored on-chain.
             **/
            stringLimit: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        asyncBacking: {
            /**
             * Purely informative, but used by mocking tools like chospticks to allow knowing how to mock
             * blocks
             **/
            expectedBlockTime: u64 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        balances: {
            /**
             * The minimum amount required to keep an account open. MUST BE GREATER THAN ZERO!
             *
             * If you *really* need it to be zero, you can enable the feature `insecure_zero_ed` for
             * this pallet. However, you do so at your own risk: this will open up a major DoS vector.
             * In case you have multiple sources of provider references, you may also get unexpected
             * behaviour if you set this to zero.
             *
             * Bottom line: Do yourself a favour and make it at least one!
             **/
            existentialDeposit: u128 & AugmentedConst<ApiType>;
            /**
             * The maximum number of individual freeze locks that can exist on an account at any time.
             **/
            maxFreezes: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum number of locks that should exist on an account.
             * Not strictly enforced, but used for weight estimation.
             *
             * Use of locks is deprecated in favour of freezes. See `https://github.com/paritytech/substrate/pull/12951/`
             **/
            maxLocks: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum number of named reserves that can exist on an account.
             *
             * Use of reserves is deprecated in favour of holds. See `https://github.com/paritytech/substrate/pull/12951/`
             **/
            maxReserves: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        convictionVoting: {
            /**
             * The maximum number of concurrent votes an account may have.
             *
             * Also used to compute weight, an overly large value can lead to extrinsics with large
             * weight estimation: see `delegate` for instance.
             **/
            maxVotes: u32 & AugmentedConst<ApiType>;
            /**
             * The minimum period of vote locking.
             *
             * It should be no shorter than enactment period to ensure that in the case of an approval,
             * those successful voters are locked into the consequences that their votes entail.
             **/
            voteLockingPeriod: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        crowdloanRewards: {
            /**
             * Percentage to be payed at initialization
             **/
            initializationPayment: Perbill & AugmentedConst<ApiType>;
            maxInitContributors: u32 & AugmentedConst<ApiType>;
            /**
             * A fraction representing the percentage of proofs
             * that need to be presented to change a reward address through the relay keys
             **/
            rewardAddressRelayVoteThreshold: Perbill & AugmentedConst<ApiType>;
            /**
             * Network Identifier to be appended into the signatures for reward address change/association
             * Prevents replay attacks from one network to the other
             **/
            signatureNetworkIdentifier: Bytes & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        identity: {
            /**
             * The amount held on deposit for a registered identity.
             **/
            basicDeposit: u128 & AugmentedConst<ApiType>;
            /**
             * The amount held on deposit per encoded byte for a registered identity.
             **/
            byteDeposit: u128 & AugmentedConst<ApiType>;
            /**
             * Maximum number of registrars allowed in the system. Needed to bound the complexity
             * of, e.g., updating judgements.
             **/
            maxRegistrars: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum number of sub-accounts allowed per identified account.
             **/
            maxSubAccounts: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum length of a suffix.
             **/
            maxSuffixLength: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum length of a username, including its suffix and any system-added delimiters.
             **/
            maxUsernameLength: u32 & AugmentedConst<ApiType>;
            /**
             * The number of blocks within which a username grant must be accepted.
             **/
            pendingUsernameExpiration: u32 & AugmentedConst<ApiType>;
            /**
             * The amount held on deposit for a registered subaccount. This should account for the fact
             * that one storage item's value will increase by the size of an account ID, and there will
             * be another trie item whose value is the size of an account ID plus 32 bytes.
             **/
            subAccountDeposit: u128 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        messageQueue: {
            /**
             * The size of the page; this implies the maximum message size which can be sent.
             *
             * A good value depends on the expected message sizes, their weights, the weight that is
             * available for processing them and the maximal needed message size. The maximal message
             * size is slightly lower than this as defined by [`MaxMessageLenOf`].
             **/
            heapSize: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum amount of weight (if any) to be used from remaining weight `on_idle` which
             * should be provided to the message queue for servicing enqueued items `on_idle`.
             * Useful for parachains to process messages at the same block they are received.
             *
             * If `None`, it will not call `ServiceQueues::service_queues` in `on_idle`.
             **/
            idleMaxServiceWeight: Option<SpWeightsWeightV2Weight> & AugmentedConst<ApiType>;
            /**
             * The maximum number of stale pages (i.e. of overweight messages) allowed before culling
             * can happen. Once there are more stale pages than this, then historical pages may be
             * dropped, even if they contain unprocessed overweight messages.
             **/
            maxStale: u32 & AugmentedConst<ApiType>;
            /**
             * The amount of weight (if any) which should be provided to the message queue for
             * servicing enqueued items `on_initialize`.
             *
             * This may be legitimately `None` in the case that you will call
             * `ServiceQueues::service_queues` manually or set [`Self::IdleMaxServiceWeight`] to have
             * it run in `on_idle`.
             **/
            serviceWeight: Option<SpWeightsWeightV2Weight> & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        moonbeamOrbiters: {
            /**
             * Maximum number of orbiters per collator.
             **/
            maxPoolSize: u32 & AugmentedConst<ApiType>;
            /**
             * Maximum number of round to keep on storage.
             **/
            maxRoundArchive: u32 & AugmentedConst<ApiType>;
            /**
             * Number of rounds before changing the selected orbiter.
             * WARNING: when changing `RotatePeriod`, you need a migration code that sets
             * `ForceRotation` to true to avoid holes in `OrbiterPerRound`.
             **/
            rotatePeriod: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        multisig: {
            /**
             * The base amount of currency needed to reserve for creating a multisig execution or to
             * store a dispatch call for later.
             *
             * This is held for an additional storage item whose value size is
             * `4 + sizeof((BlockNumber, Balance, AccountId))` bytes and whose key size is
             * `32 + sizeof(AccountId)` bytes.
             **/
            depositBase: u128 & AugmentedConst<ApiType>;
            /**
             * The amount of currency needed per unit threshold when creating a multisig execution.
             *
             * This is held for adding 32 bytes more into a pre-existing storage value.
             **/
            depositFactor: u128 & AugmentedConst<ApiType>;
            /**
             * The maximum amount of signatories allowed in the multisig.
             **/
            maxSignatories: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        openTechCommitteeCollective: {
            /**
             * The maximum weight of a dispatch call that can be proposed and executed.
             **/
            maxProposalWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        parachainStaking: {
            /**
             * Get the average time beetween 2 blocks in milliseconds
             **/
            blockTime: u64 & AugmentedConst<ApiType>;
            /**
             * Number of rounds candidate requests to decrease self-bond must wait to be executable
             **/
            candidateBondLessDelay: u32 & AugmentedConst<ApiType>;
            /**
             * Number of rounds that delegation less requests must wait before executable
             **/
            delegationBondLessDelay: u32 & AugmentedConst<ApiType>;
            /**
             * Number of rounds that candidates remain bonded before exit request is executable
             **/
            leaveCandidatesDelay: u32 & AugmentedConst<ApiType>;
            /**
             * Number of rounds that delegators remain bonded before exit request is executable
             **/
            leaveDelegatorsDelay: u32 & AugmentedConst<ApiType>;
            /**
             * Maximum bottom delegations (not counted) per candidate
             **/
            maxBottomDelegationsPerCandidate: u32 & AugmentedConst<ApiType>;
            /**
             * Maximum candidates
             **/
            maxCandidates: u32 & AugmentedConst<ApiType>;
            /**
             * Maximum delegations per delegator
             **/
            maxDelegationsPerDelegator: u32 & AugmentedConst<ApiType>;
            /**
             * If a collator doesn't produce any block on this number of rounds, it is notified as inactive.
             * This value must be less than or equal to RewardPaymentDelay.
             **/
            maxOfflineRounds: u32 & AugmentedConst<ApiType>;
            /**
             * Maximum top delegations counted per candidate
             **/
            maxTopDelegationsPerCandidate: u32 & AugmentedConst<ApiType>;
            /**
             * Minimum number of blocks per round
             **/
            minBlocksPerRound: u32 & AugmentedConst<ApiType>;
            /**
             * Minimum stake required for any account to be a collator candidate
             **/
            minCandidateStk: u128 & AugmentedConst<ApiType>;
            /**
             * Minimum stake for any registered on-chain account to delegate
             **/
            minDelegation: u128 & AugmentedConst<ApiType>;
            /**
             * Minimum number of selected candidates every round
             **/
            minSelectedCandidates: u32 & AugmentedConst<ApiType>;
            /**
             * Number of rounds that delegations remain bonded before revocation request is executable
             **/
            revokeDelegationDelay: u32 & AugmentedConst<ApiType>;
            /**
             * Number of rounds after which block authors are rewarded
             **/
            rewardPaymentDelay: u32 & AugmentedConst<ApiType>;
            /**
             * Get the slot duration in milliseconds
             **/
            slotDuration: u64 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        parachainSystem: {
            /**
             * Returns the parachain ID we are running with.
             **/
            selfParaId: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        proxy: {
            /**
             * The base amount of currency needed to reserve for creating an announcement.
             *
             * This is held when a new storage item holding a `Balance` is created (typically 16
             * bytes).
             **/
            announcementDepositBase: u128 & AugmentedConst<ApiType>;
            /**
             * The amount of currency needed per announcement made.
             *
             * This is held for adding an `AccountId`, `Hash` and `BlockNumber` (typically 68 bytes)
             * into a pre-existing storage value.
             **/
            announcementDepositFactor: u128 & AugmentedConst<ApiType>;
            /**
             * The maximum amount of time-delayed announcements that are allowed to be pending.
             **/
            maxPending: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum amount of proxies allowed for a single account.
             **/
            maxProxies: u32 & AugmentedConst<ApiType>;
            /**
             * The base amount of currency needed to reserve for creating a proxy.
             *
             * This is held for an additional storage item whose value size is
             * `sizeof(Balance)` bytes and whose key size is `sizeof(AccountId)` bytes.
             **/
            proxyDepositBase: u128 & AugmentedConst<ApiType>;
            /**
             * The amount of currency needed per proxy added.
             *
             * This is held for adding 32 bytes plus an instance of `ProxyType` more into a
             * pre-existing storage value. Thus, when configuring `ProxyDepositFactor` one should take
             * into account `32 + proxy_type.encode().len()` bytes of data.
             **/
            proxyDepositFactor: u128 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        randomness: {
            /**
             * Local requests expire and can be purged from storage after this many blocks/epochs
             **/
            blockExpirationDelay: u32 & AugmentedConst<ApiType>;
            /**
             * The amount that should be taken as a security deposit when requesting randomness.
             **/
            deposit: u128 & AugmentedConst<ApiType>;
            /**
             * Babe requests expire and can be purged from storage after this many blocks/epochs
             **/
            epochExpirationDelay: u64 & AugmentedConst<ApiType>;
            /**
             * Local per-block VRF requests must be at most this many blocks after the block in which
             * they were requested
             **/
            maxBlockDelay: u32 & AugmentedConst<ApiType>;
            /**
             * Maximum number of random words that can be requested per request
             **/
            maxRandomWords: u8 & AugmentedConst<ApiType>;
            /**
             * Local per-block VRF requests must be at least this many blocks after the block in which
             * they were requested
             **/
            minBlockDelay: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        referenda: {
            /**
             * Quantization level for the referendum wakeup scheduler. A higher number will result in
             * fewer storage reads/writes needed for smaller voters, but also result in delays to the
             * automatic referendum status changes. Explicit servicing instructions are unaffected.
             **/
            alarmInterval: u32 & AugmentedConst<ApiType>;
            /**
             * Maximum size of the referendum queue for a single track.
             **/
            maxQueued: u32 & AugmentedConst<ApiType>;
            /**
             * The minimum amount to be used as a deposit for a public referendum proposal.
             **/
            submissionDeposit: u128 & AugmentedConst<ApiType>;
            /**
             * Information concerning the different referendum tracks.
             **/
            tracks: Vec<ITuple<[u16, PalletReferendaTrackInfo]>> & AugmentedConst<ApiType>;
            /**
             * The number of blocks after submission that a referendum must begin being decided by.
             * Once this passes, then anyone may cancel the referendum.
             **/
            undecidingTimeout: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        relayStorageRoots: {
            /**
             * Limit the number of relay storage roots that will be stored.
             * This limit applies to the number of items, not to their age. Decreasing the value of
             * `MaxStorageRoots` is a breaking change and needs a migration to clean the
             * `RelayStorageRoots` mapping.
             **/
            maxStorageRoots: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        scheduler: {
            /**
             * The maximum weight that may be scheduled per block for any dispatchables.
             **/
            maximumWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
            /**
             * The maximum number of scheduled calls in the queue for a single block.
             *
             * NOTE:
             * + Dependent pallets' benchmarks might require a higher limit for the setting. Set a
             * higher limit under `runtime-benchmarks` feature.
             **/
            maxScheduledPerBlock: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        system: {
            /**
             * Maximum number of block number to block hash mappings to keep (oldest pruned first).
             **/
            blockHashCount: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum length of a block (in bytes).
             **/
            blockLength: FrameSystemLimitsBlockLength & AugmentedConst<ApiType>;
            /**
             * Block & extrinsics weights: base values and limits.
             **/
            blockWeights: FrameSystemLimitsBlockWeights & AugmentedConst<ApiType>;
            /**
             * The weight of runtime database operations the runtime can invoke.
             **/
            dbWeight: SpWeightsRuntimeDbWeight & AugmentedConst<ApiType>;
            /**
             * The designated SS58 prefix of this chain.
             *
             * This replaces the "ss58Format" property declared in the chain spec. Reason is
             * that the runtime should know about the prefix in order to make use of it as
             * an identifier of the chain.
             **/
            ss58Prefix: u16 & AugmentedConst<ApiType>;
            /**
             * Get the chain's in-code version.
             **/
            version: SpVersionRuntimeVersion & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        timestamp: {
            /**
             * The minimum period between blocks.
             *
             * Be aware that this is different to the *expected* period that the block production
             * apparatus provides. Your chosen consensus system will generally work with this to
             * determine a sensible block time. For example, in the Aura pallet it will be double this
             * period on default settings.
             **/
            minimumPeriod: u64 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        transactionPayment: {
            /**
             * A fee multiplier for `Operational` extrinsics to compute "virtual tip" to boost their
             * `priority`
             *
             * This value is multiplied by the `final_fee` to obtain a "virtual tip" that is later
             * added to a tip component in regular `priority` calculations.
             * It means that a `Normal` transaction can front-run a similarly-sized `Operational`
             * extrinsic (with no tip), by including a tip value greater than the virtual tip.
             *
             * ```rust,ignore
             * // For `Normal`
             * let priority = priority_calc(tip);
             *
             * // For `Operational`
             * let virtual_tip = (inclusion_fee + tip) * OperationalFeeMultiplier;
             * let priority = priority_calc(tip + virtual_tip);
             * ```
             *
             * Note that since we use `final_fee` the multiplier applies also to the regular `tip`
             * sent with the transaction. So, not only does the transaction get a priority bump based
             * on the `inclusion_fee`, but we also amplify the impact of tips applied to `Operational`
             * transactions.
             **/
            operationalFeeMultiplier: u8 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        treasury: {
            /**
             * Percentage of spare funds (if any) that are burnt per spend period.
             **/
            burn: Permill & AugmentedConst<ApiType>;
            /**
             * The maximum number of approvals that can wait in the spending queue.
             *
             * NOTE: This parameter is also used within the Bounties Pallet extension if enabled.
             **/
            maxApprovals: u32 & AugmentedConst<ApiType>;
            /**
             * The treasury's pallet id, used for deriving its sovereign account ID.
             **/
            palletId: FrameSupportPalletId & AugmentedConst<ApiType>;
            /**
             * The period during which an approved treasury spend has to be claimed.
             **/
            payoutPeriod: u32 & AugmentedConst<ApiType>;
            /**
             * Period between successive spends.
             **/
            spendPeriod: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        treasuryCouncilCollective: {
            /**
             * The maximum weight of a dispatch call that can be proposed and executed.
             **/
            maxProposalWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        utility: {
            /**
             * The limit on the number of batched calls.
             **/
            batchedCallsLimit: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        xcmpQueue: {
            /**
             * Maximal number of outbound XCMP channels that can have messages queued at the same time.
             *
             * If this is reached, then no further messages can be sent to channels that do not yet
             * have a message queued. This should be set to the expected maximum of outbound channels
             * which is determined by [`Self::ChannelInfo`]. It is important to set this large enough,
             * since otherwise the congestion control protocol will not work as intended and messages
             * may be dropped. This value increases the PoV and should therefore not be picked too
             * high. Governance needs to pay attention to not open more channels than this value.
             **/
            maxActiveOutboundChannels: u32 & AugmentedConst<ApiType>;
            /**
             * The maximum number of inbound XCMP channels that can be suspended simultaneously.
             *
             * Any further channel suspensions will fail and messages may get dropped without further
             * notice. Choosing a high value (1000) is okay; the trade-off that is described in
             * [`InboundXcmpSuspended`] still applies at that scale.
             **/
            maxInboundSuspended: u32 & AugmentedConst<ApiType>;
            /**
             * The maximal page size for HRMP message pages.
             *
             * A lower limit can be set dynamically, but this is the hard-limit for the PoV worst case
             * benchmarking. The limit for the size of a message is slightly below this, since some
             * overhead is incurred for encoding the format.
             **/
            maxPageSize: u32 & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
        xcmTransactor: {
            /**
             *
             * The actual weight for an XCM message is `T::BaseXcmWeight +
             * T::Weigher::weight(&msg)`.
             **/
            baseXcmWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
            /**
             * Self chain location.
             **/
            selfLocation: StagingXcmV4Location & AugmentedConst<ApiType>;
            /**
             * Generic const
             **/
            [key: string]: Codec;
        };
    }
}
//# sourceMappingURL=augment-api-consts.d.ts.map