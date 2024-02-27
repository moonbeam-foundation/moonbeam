// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/types/types/registry";

import type {
  AccountEthereumSignature,
  CumulusPalletDmpQueueCall,
  CumulusPalletDmpQueueConfigData,
  CumulusPalletDmpQueueError,
  CumulusPalletDmpQueueEvent,
  CumulusPalletDmpQueuePageIndexData,
  CumulusPalletParachainSystemCall,
  CumulusPalletParachainSystemCodeUpgradeAuthorization,
  CumulusPalletParachainSystemError,
  CumulusPalletParachainSystemEvent,
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot,
  CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity,
  CumulusPalletParachainSystemUnincludedSegmentAncestor,
  CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate,
  CumulusPalletParachainSystemUnincludedSegmentSegmentTracker,
  CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth,
  CumulusPalletXcmError,
  CumulusPalletXcmEvent,
  CumulusPalletXcmOrigin,
  CumulusPalletXcmpQueueError,
  CumulusPalletXcmpQueueEvent,
  CumulusPalletXcmpQueueInboundChannelDetails,
  CumulusPalletXcmpQueueInboundState,
  CumulusPalletXcmpQueueOutboundChannelDetails,
  CumulusPalletXcmpQueueOutboundState,
  CumulusPalletXcmpQueueQueueConfigData,
  CumulusPrimitivesParachainInherentParachainInherentData,
  EthbloomBloom,
  EthereumBlock,
  EthereumHeader,
  EthereumLog,
  EthereumReceiptEip658ReceiptData,
  EthereumReceiptReceiptV3,
  EthereumTransactionAccessListItem,
  EthereumTransactionEip1559Transaction,
  EthereumTransactionEip2930Transaction,
  EthereumTransactionLegacyTransaction,
  EthereumTransactionTransactionAction,
  EthereumTransactionTransactionSignature,
  EthereumTransactionTransactionV2,
  EthereumTypesHashH64,
  EvmCoreErrorExitError,
  EvmCoreErrorExitFatal,
  EvmCoreErrorExitReason,
  EvmCoreErrorExitRevert,
  EvmCoreErrorExitSucceed,
  FpRpcTransactionStatus,
  FrameSupportDispatchDispatchClass,
  FrameSupportDispatchDispatchInfo,
  FrameSupportDispatchPays,
  FrameSupportDispatchPerDispatchClassU32,
  FrameSupportDispatchPerDispatchClassWeight,
  FrameSupportDispatchPerDispatchClassWeightsPerClass,
  FrameSupportDispatchPostDispatchInfo,
  FrameSupportDispatchRawOrigin,
  FrameSupportPalletId,
  FrameSupportPreimagesBounded,
  FrameSupportScheduleDispatchTime,
  FrameSupportTokensMiscBalanceStatus,
  FrameSystemAccountInfo,
  FrameSystemCall,
  FrameSystemError,
  FrameSystemEvent,
  FrameSystemEventRecord,
  FrameSystemExtensionsCheckGenesis,
  FrameSystemExtensionsCheckNonZeroSender,
  FrameSystemExtensionsCheckNonce,
  FrameSystemExtensionsCheckSpecVersion,
  FrameSystemExtensionsCheckTxVersion,
  FrameSystemExtensionsCheckWeight,
  FrameSystemLastRuntimeUpgradeInfo,
  FrameSystemLimitsBlockLength,
  FrameSystemLimitsBlockWeights,
  FrameSystemLimitsWeightsPerClass,
  FrameSystemPhase,
  MoonriverRuntimeAssetConfigAssetRegistrarMetadata,
  MoonriverRuntimeGovernanceOriginsCustomOriginsOrigin,
  MoonriverRuntimeOriginCaller,
  MoonriverRuntimeProxyType,
  MoonriverRuntimeRuntime,
  MoonriverRuntimeRuntimeHoldReason,
  MoonriverRuntimeXcmConfigAssetType,
  MoonriverRuntimeXcmConfigCurrencyId,
  MoonriverRuntimeXcmConfigTransactors,
  NimbusPrimitivesNimbusCryptoPublic,
  OrmlXtokensModuleCall,
  OrmlXtokensModuleError,
  OrmlXtokensModuleEvent,
  PalletAssetManagerAssetInfo,
  PalletAssetManagerCall,
  PalletAssetManagerError,
  PalletAssetManagerEvent,
  PalletAssetsAccountStatus,
  PalletAssetsApproval,
  PalletAssetsAssetAccount,
  PalletAssetsAssetDetails,
  PalletAssetsAssetMetadata,
  PalletAssetsAssetStatus,
  PalletAssetsCall,
  PalletAssetsError,
  PalletAssetsEvent,
  PalletAssetsExistenceReason,
  PalletAuthorInherentCall,
  PalletAuthorInherentError,
  PalletAuthorMappingCall,
  PalletAuthorMappingError,
  PalletAuthorMappingEvent,
  PalletAuthorMappingRegistrationInfo,
  PalletAuthorSlotFilterCall,
  PalletAuthorSlotFilterEvent,
  PalletBalancesAccountData,
  PalletBalancesBalanceLock,
  PalletBalancesCall,
  PalletBalancesError,
  PalletBalancesEvent,
  PalletBalancesIdAmount,
  PalletBalancesReasons,
  PalletBalancesReserveData,
  PalletCollectiveCall,
  PalletCollectiveError,
  PalletCollectiveEvent,
  PalletCollectiveRawOrigin,
  PalletCollectiveVotes,
  PalletConvictionVotingCall,
  PalletConvictionVotingConviction,
  PalletConvictionVotingDelegations,
  PalletConvictionVotingError,
  PalletConvictionVotingEvent,
  PalletConvictionVotingTally,
  PalletConvictionVotingVoteAccountVote,
  PalletConvictionVotingVoteCasting,
  PalletConvictionVotingVoteDelegating,
  PalletConvictionVotingVotePriorLock,
  PalletConvictionVotingVoteVoting,
  PalletCrowdloanRewardsCall,
  PalletCrowdloanRewardsError,
  PalletCrowdloanRewardsEvent,
  PalletCrowdloanRewardsRewardInfo,
  PalletDemocracyCall,
  PalletDemocracyConviction,
  PalletDemocracyDelegations,
  PalletDemocracyError,
  PalletDemocracyEvent,
  PalletDemocracyMetadataOwner,
  PalletDemocracyReferendumInfo,
  PalletDemocracyReferendumStatus,
  PalletDemocracyTally,
  PalletDemocracyVoteAccountVote,
  PalletDemocracyVotePriorLock,
  PalletDemocracyVoteThreshold,
  PalletDemocracyVoteVoting,
  PalletEthereumCall,
  PalletEthereumError,
  PalletEthereumEvent,
  PalletEthereumRawOrigin,
  PalletEthereumXcmCall,
  PalletEthereumXcmError,
  PalletEthereumXcmRawOrigin,
  PalletEvmCall,
  PalletEvmCodeMetadata,
  PalletEvmError,
  PalletEvmEvent,
  PalletIdentityBitFlags,
  PalletIdentityCall,
  PalletIdentityError,
  PalletIdentityEvent,
  PalletIdentityJudgement,
  PalletIdentityRegistrarInfo,
  PalletIdentityRegistration,
  PalletIdentitySimpleIdentityField,
  PalletIdentitySimpleIdentityInfo,
  PalletMaintenanceModeCall,
  PalletMaintenanceModeError,
  PalletMaintenanceModeEvent,
  PalletMigrationsError,
  PalletMigrationsEvent,
  PalletMoonbeamLazyMigrationsCall,
  PalletMoonbeamLazyMigrationsError,
  PalletMoonbeamOrbitersCall,
  PalletMoonbeamOrbitersCollatorPoolInfo,
  PalletMoonbeamOrbitersCurrentOrbiter,
  PalletMoonbeamOrbitersError,
  PalletMoonbeamOrbitersEvent,
  PalletMultisigCall,
  PalletMultisigError,
  PalletMultisigEvent,
  PalletMultisigMultisig,
  PalletMultisigTimepoint,
  PalletParachainStakingAutoCompoundAutoCompoundConfig,
  PalletParachainStakingBond,
  PalletParachainStakingBondWithAutoCompound,
  PalletParachainStakingCall,
  PalletParachainStakingCandidateBondLessRequest,
  PalletParachainStakingCandidateMetadata,
  PalletParachainStakingCapacityStatus,
  PalletParachainStakingCollatorSnapshot,
  PalletParachainStakingCollatorStatus,
  PalletParachainStakingDelayedPayout,
  PalletParachainStakingDelegationRequestsCancelledScheduledRequest,
  PalletParachainStakingDelegationRequestsDelegationAction,
  PalletParachainStakingDelegationRequestsScheduledRequest,
  PalletParachainStakingDelegations,
  PalletParachainStakingDelegator,
  PalletParachainStakingDelegatorAdded,
  PalletParachainStakingDelegatorStatus,
  PalletParachainStakingError,
  PalletParachainStakingEvent,
  PalletParachainStakingInflationInflationInfo,
  PalletParachainStakingParachainBondConfig,
  PalletParachainStakingRoundInfo,
  PalletParachainStakingSetBoundedOrderedSet,
  PalletParachainStakingSetOrderedSet,
  PalletPreimageCall,
  PalletPreimageError,
  PalletPreimageEvent,
  PalletPreimageHoldReason,
  PalletPreimageOldRequestStatus,
  PalletPreimageRequestStatus,
  PalletProxyAnnouncement,
  PalletProxyCall,
  PalletProxyError,
  PalletProxyEvent,
  PalletProxyProxyDefinition,
  PalletRandomnessCall,
  PalletRandomnessError,
  PalletRandomnessEvent,
  PalletRandomnessRandomnessResult,
  PalletRandomnessRequest,
  PalletRandomnessRequestInfo,
  PalletRandomnessRequestState,
  PalletRandomnessRequestType,
  PalletReferendaCall,
  PalletReferendaCurve,
  PalletReferendaDecidingStatus,
  PalletReferendaDeposit,
  PalletReferendaError,
  PalletReferendaEvent,
  PalletReferendaReferendumInfo,
  PalletReferendaReferendumStatus,
  PalletReferendaTrackInfo,
  PalletRootTestingCall,
  PalletSchedulerCall,
  PalletSchedulerError,
  PalletSchedulerEvent,
  PalletSchedulerScheduled,
  PalletTimestampCall,
  PalletTransactionPaymentChargeTransactionPayment,
  PalletTransactionPaymentEvent,
  PalletTransactionPaymentReleases,
  PalletTreasuryCall,
  PalletTreasuryError,
  PalletTreasuryEvent,
  PalletTreasuryPaymentState,
  PalletTreasuryProposal,
  PalletTreasurySpendStatus,
  PalletUtilityCall,
  PalletUtilityError,
  PalletUtilityEvent,
  PalletWhitelistCall,
  PalletWhitelistError,
  PalletWhitelistEvent,
  PalletXcmCall,
  PalletXcmError,
  PalletXcmEvent,
  PalletXcmOrigin,
  PalletXcmQueryStatus,
  PalletXcmRemoteLockedFungibleRecord,
  PalletXcmTransactorCall,
  PalletXcmTransactorCurrency,
  PalletXcmTransactorCurrencyPayment,
  PalletXcmTransactorError,
  PalletXcmTransactorEvent,
  PalletXcmTransactorHrmpInitParams,
  PalletXcmTransactorHrmpOperation,
  PalletXcmTransactorRelayIndicesRelayChainIndices,
  PalletXcmTransactorRemoteTransactInfoWithMaxWeight,
  PalletXcmTransactorTransactWeights,
  PalletXcmVersionMigrationStage,
  PolkadotCorePrimitivesInboundDownwardMessage,
  PolkadotCorePrimitivesInboundHrmpMessage,
  PolkadotCorePrimitivesOutboundHrmpMessage,
  PolkadotParachainPrimitivesPrimitivesHrmpChannelId,
  PolkadotParachainPrimitivesPrimitivesXcmpMessageFormat,
  PolkadotPrimitivesV6AbridgedHostConfiguration,
  PolkadotPrimitivesV6AbridgedHrmpChannel,
  PolkadotPrimitivesV6AsyncBackingAsyncBackingParams,
  PolkadotPrimitivesV6PersistedValidationData,
  PolkadotPrimitivesV6UpgradeGoAhead,
  PolkadotPrimitivesV6UpgradeRestriction,
  SessionKeysPrimitivesVrfVrfCryptoPublic,
  SpArithmeticArithmeticError,
  SpCoreEcdsaSignature,
  SpCoreEd25519Signature,
  SpCoreSr25519Public,
  SpCoreSr25519Signature,
  SpCoreVoid,
  SpRuntimeBlakeTwo256,
  SpRuntimeDigest,
  SpRuntimeDigestDigestItem,
  SpRuntimeDispatchError,
  SpRuntimeDispatchErrorWithPostInfo,
  SpRuntimeModuleError,
  SpRuntimeMultiSignature,
  SpRuntimeTokenError,
  SpRuntimeTransactionalError,
  SpTrieStorageProof,
  SpVersionRuntimeVersion,
  SpWeightsRuntimeDbWeight,
  SpWeightsWeightV2Weight,
  StagingXcmV3MultiLocation,
  XcmDoubleEncoded,
  XcmPrimitivesEthereumXcmEthereumXcmFee,
  XcmPrimitivesEthereumXcmEthereumXcmTransaction,
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV1,
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV2,
  XcmPrimitivesEthereumXcmManualEthereumXcmFee,
  XcmV2BodyId,
  XcmV2BodyPart,
  XcmV2Instruction,
  XcmV2Junction,
  XcmV2MultiAsset,
  XcmV2MultiLocation,
  XcmV2MultiassetAssetId,
  XcmV2MultiassetAssetInstance,
  XcmV2MultiassetFungibility,
  XcmV2MultiassetMultiAssetFilter,
  XcmV2MultiassetMultiAssets,
  XcmV2MultiassetWildFungibility,
  XcmV2MultiassetWildMultiAsset,
  XcmV2MultilocationJunctions,
  XcmV2NetworkId,
  XcmV2OriginKind,
  XcmV2Response,
  XcmV2TraitsError,
  XcmV2WeightLimit,
  XcmV2Xcm,
  XcmV3Instruction,
  XcmV3Junction,
  XcmV3JunctionBodyId,
  XcmV3JunctionBodyPart,
  XcmV3JunctionNetworkId,
  XcmV3Junctions,
  XcmV3MaybeErrorCode,
  XcmV3MultiAsset,
  XcmV3MultiassetAssetId,
  XcmV3MultiassetAssetInstance,
  XcmV3MultiassetFungibility,
  XcmV3MultiassetMultiAssetFilter,
  XcmV3MultiassetMultiAssets,
  XcmV3MultiassetWildFungibility,
  XcmV3MultiassetWildMultiAsset,
  XcmV3PalletInfo,
  XcmV3QueryResponseInfo,
  XcmV3Response,
  XcmV3TraitsError,
  XcmV3TraitsOutcome,
  XcmV3WeightLimit,
  XcmV3Xcm,
  XcmVersionedAssetId,
  XcmVersionedMultiAsset,
  XcmVersionedMultiAssets,
  XcmVersionedMultiLocation,
  XcmVersionedResponse,
  XcmVersionedXcm,
} from "@polkadot/types/lookup";

declare module "@polkadot/types/types/registry" {
  interface InterfaceTypes {
    AccountEthereumSignature: AccountEthereumSignature;
    CumulusPalletDmpQueueCall: CumulusPalletDmpQueueCall;
    CumulusPalletDmpQueueConfigData: CumulusPalletDmpQueueConfigData;
    CumulusPalletDmpQueueError: CumulusPalletDmpQueueError;
    CumulusPalletDmpQueueEvent: CumulusPalletDmpQueueEvent;
    CumulusPalletDmpQueuePageIndexData: CumulusPalletDmpQueuePageIndexData;
    CumulusPalletParachainSystemCall: CumulusPalletParachainSystemCall;
    CumulusPalletParachainSystemCodeUpgradeAuthorization: CumulusPalletParachainSystemCodeUpgradeAuthorization;
    CumulusPalletParachainSystemError: CumulusPalletParachainSystemError;
    CumulusPalletParachainSystemEvent: CumulusPalletParachainSystemEvent;
    CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot;
    CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity: CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity;
    CumulusPalletParachainSystemUnincludedSegmentAncestor: CumulusPalletParachainSystemUnincludedSegmentAncestor;
    CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate: CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate;
    CumulusPalletParachainSystemUnincludedSegmentSegmentTracker: CumulusPalletParachainSystemUnincludedSegmentSegmentTracker;
    CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth: CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth;
    CumulusPalletXcmError: CumulusPalletXcmError;
    CumulusPalletXcmEvent: CumulusPalletXcmEvent;
    CumulusPalletXcmOrigin: CumulusPalletXcmOrigin;
    CumulusPalletXcmpQueueError: CumulusPalletXcmpQueueError;
    CumulusPalletXcmpQueueEvent: CumulusPalletXcmpQueueEvent;
    CumulusPalletXcmpQueueInboundChannelDetails: CumulusPalletXcmpQueueInboundChannelDetails;
    CumulusPalletXcmpQueueInboundState: CumulusPalletXcmpQueueInboundState;
    CumulusPalletXcmpQueueOutboundChannelDetails: CumulusPalletXcmpQueueOutboundChannelDetails;
    CumulusPalletXcmpQueueOutboundState: CumulusPalletXcmpQueueOutboundState;
    CumulusPalletXcmpQueueQueueConfigData: CumulusPalletXcmpQueueQueueConfigData;
    CumulusPrimitivesParachainInherentParachainInherentData: CumulusPrimitivesParachainInherentParachainInherentData;
    EthbloomBloom: EthbloomBloom;
    EthereumBlock: EthereumBlock;
    EthereumHeader: EthereumHeader;
    EthereumLog: EthereumLog;
    EthereumReceiptEip658ReceiptData: EthereumReceiptEip658ReceiptData;
    EthereumReceiptReceiptV3: EthereumReceiptReceiptV3;
    EthereumTransactionAccessListItem: EthereumTransactionAccessListItem;
    EthereumTransactionEip1559Transaction: EthereumTransactionEip1559Transaction;
    EthereumTransactionEip2930Transaction: EthereumTransactionEip2930Transaction;
    EthereumTransactionLegacyTransaction: EthereumTransactionLegacyTransaction;
    EthereumTransactionTransactionAction: EthereumTransactionTransactionAction;
    EthereumTransactionTransactionSignature: EthereumTransactionTransactionSignature;
    EthereumTransactionTransactionV2: EthereumTransactionTransactionV2;
    EthereumTypesHashH64: EthereumTypesHashH64;
    EvmCoreErrorExitError: EvmCoreErrorExitError;
    EvmCoreErrorExitFatal: EvmCoreErrorExitFatal;
    EvmCoreErrorExitReason: EvmCoreErrorExitReason;
    EvmCoreErrorExitRevert: EvmCoreErrorExitRevert;
    EvmCoreErrorExitSucceed: EvmCoreErrorExitSucceed;
    FpRpcTransactionStatus: FpRpcTransactionStatus;
    FrameSupportDispatchDispatchClass: FrameSupportDispatchDispatchClass;
    FrameSupportDispatchDispatchInfo: FrameSupportDispatchDispatchInfo;
    FrameSupportDispatchPays: FrameSupportDispatchPays;
    FrameSupportDispatchPerDispatchClassU32: FrameSupportDispatchPerDispatchClassU32;
    FrameSupportDispatchPerDispatchClassWeight: FrameSupportDispatchPerDispatchClassWeight;
    FrameSupportDispatchPerDispatchClassWeightsPerClass: FrameSupportDispatchPerDispatchClassWeightsPerClass;
    FrameSupportDispatchPostDispatchInfo: FrameSupportDispatchPostDispatchInfo;
    FrameSupportDispatchRawOrigin: FrameSupportDispatchRawOrigin;
    FrameSupportPalletId: FrameSupportPalletId;
    FrameSupportPreimagesBounded: FrameSupportPreimagesBounded;
    FrameSupportScheduleDispatchTime: FrameSupportScheduleDispatchTime;
    FrameSupportTokensMiscBalanceStatus: FrameSupportTokensMiscBalanceStatus;
    FrameSystemAccountInfo: FrameSystemAccountInfo;
    FrameSystemCall: FrameSystemCall;
    FrameSystemError: FrameSystemError;
    FrameSystemEvent: FrameSystemEvent;
    FrameSystemEventRecord: FrameSystemEventRecord;
    FrameSystemExtensionsCheckGenesis: FrameSystemExtensionsCheckGenesis;
    FrameSystemExtensionsCheckNonZeroSender: FrameSystemExtensionsCheckNonZeroSender;
    FrameSystemExtensionsCheckNonce: FrameSystemExtensionsCheckNonce;
    FrameSystemExtensionsCheckSpecVersion: FrameSystemExtensionsCheckSpecVersion;
    FrameSystemExtensionsCheckTxVersion: FrameSystemExtensionsCheckTxVersion;
    FrameSystemExtensionsCheckWeight: FrameSystemExtensionsCheckWeight;
    FrameSystemLastRuntimeUpgradeInfo: FrameSystemLastRuntimeUpgradeInfo;
    FrameSystemLimitsBlockLength: FrameSystemLimitsBlockLength;
    FrameSystemLimitsBlockWeights: FrameSystemLimitsBlockWeights;
    FrameSystemLimitsWeightsPerClass: FrameSystemLimitsWeightsPerClass;
    FrameSystemPhase: FrameSystemPhase;
    MoonriverRuntimeAssetConfigAssetRegistrarMetadata: MoonriverRuntimeAssetConfigAssetRegistrarMetadata;
    MoonriverRuntimeGovernanceOriginsCustomOriginsOrigin: MoonriverRuntimeGovernanceOriginsCustomOriginsOrigin;
    MoonriverRuntimeOriginCaller: MoonriverRuntimeOriginCaller;
    MoonriverRuntimeProxyType: MoonriverRuntimeProxyType;
    MoonriverRuntimeRuntime: MoonriverRuntimeRuntime;
    MoonriverRuntimeRuntimeHoldReason: MoonriverRuntimeRuntimeHoldReason;
    MoonriverRuntimeXcmConfigAssetType: MoonriverRuntimeXcmConfigAssetType;
    MoonriverRuntimeXcmConfigCurrencyId: MoonriverRuntimeXcmConfigCurrencyId;
    MoonriverRuntimeXcmConfigTransactors: MoonriverRuntimeXcmConfigTransactors;
    NimbusPrimitivesNimbusCryptoPublic: NimbusPrimitivesNimbusCryptoPublic;
    OrmlXtokensModuleCall: OrmlXtokensModuleCall;
    OrmlXtokensModuleError: OrmlXtokensModuleError;
    OrmlXtokensModuleEvent: OrmlXtokensModuleEvent;
    PalletAssetManagerAssetInfo: PalletAssetManagerAssetInfo;
    PalletAssetManagerCall: PalletAssetManagerCall;
    PalletAssetManagerError: PalletAssetManagerError;
    PalletAssetManagerEvent: PalletAssetManagerEvent;
    PalletAssetsAccountStatus: PalletAssetsAccountStatus;
    PalletAssetsApproval: PalletAssetsApproval;
    PalletAssetsAssetAccount: PalletAssetsAssetAccount;
    PalletAssetsAssetDetails: PalletAssetsAssetDetails;
    PalletAssetsAssetMetadata: PalletAssetsAssetMetadata;
    PalletAssetsAssetStatus: PalletAssetsAssetStatus;
    PalletAssetsCall: PalletAssetsCall;
    PalletAssetsError: PalletAssetsError;
    PalletAssetsEvent: PalletAssetsEvent;
    PalletAssetsExistenceReason: PalletAssetsExistenceReason;
    PalletAuthorInherentCall: PalletAuthorInherentCall;
    PalletAuthorInherentError: PalletAuthorInherentError;
    PalletAuthorMappingCall: PalletAuthorMappingCall;
    PalletAuthorMappingError: PalletAuthorMappingError;
    PalletAuthorMappingEvent: PalletAuthorMappingEvent;
    PalletAuthorMappingRegistrationInfo: PalletAuthorMappingRegistrationInfo;
    PalletAuthorSlotFilterCall: PalletAuthorSlotFilterCall;
    PalletAuthorSlotFilterEvent: PalletAuthorSlotFilterEvent;
    PalletBalancesAccountData: PalletBalancesAccountData;
    PalletBalancesBalanceLock: PalletBalancesBalanceLock;
    PalletBalancesCall: PalletBalancesCall;
    PalletBalancesError: PalletBalancesError;
    PalletBalancesEvent: PalletBalancesEvent;
    PalletBalancesIdAmount: PalletBalancesIdAmount;
    PalletBalancesReasons: PalletBalancesReasons;
    PalletBalancesReserveData: PalletBalancesReserveData;
    PalletCollectiveCall: PalletCollectiveCall;
    PalletCollectiveError: PalletCollectiveError;
    PalletCollectiveEvent: PalletCollectiveEvent;
    PalletCollectiveRawOrigin: PalletCollectiveRawOrigin;
    PalletCollectiveVotes: PalletCollectiveVotes;
    PalletConvictionVotingCall: PalletConvictionVotingCall;
    PalletConvictionVotingConviction: PalletConvictionVotingConviction;
    PalletConvictionVotingDelegations: PalletConvictionVotingDelegations;
    PalletConvictionVotingError: PalletConvictionVotingError;
    PalletConvictionVotingEvent: PalletConvictionVotingEvent;
    PalletConvictionVotingTally: PalletConvictionVotingTally;
    PalletConvictionVotingVoteAccountVote: PalletConvictionVotingVoteAccountVote;
    PalletConvictionVotingVoteCasting: PalletConvictionVotingVoteCasting;
    PalletConvictionVotingVoteDelegating: PalletConvictionVotingVoteDelegating;
    PalletConvictionVotingVotePriorLock: PalletConvictionVotingVotePriorLock;
    PalletConvictionVotingVoteVoting: PalletConvictionVotingVoteVoting;
    PalletCrowdloanRewardsCall: PalletCrowdloanRewardsCall;
    PalletCrowdloanRewardsError: PalletCrowdloanRewardsError;
    PalletCrowdloanRewardsEvent: PalletCrowdloanRewardsEvent;
    PalletCrowdloanRewardsRewardInfo: PalletCrowdloanRewardsRewardInfo;
    PalletDemocracyCall: PalletDemocracyCall;
    PalletDemocracyConviction: PalletDemocracyConviction;
    PalletDemocracyDelegations: PalletDemocracyDelegations;
    PalletDemocracyError: PalletDemocracyError;
    PalletDemocracyEvent: PalletDemocracyEvent;
    PalletDemocracyMetadataOwner: PalletDemocracyMetadataOwner;
    PalletDemocracyReferendumInfo: PalletDemocracyReferendumInfo;
    PalletDemocracyReferendumStatus: PalletDemocracyReferendumStatus;
    PalletDemocracyTally: PalletDemocracyTally;
    PalletDemocracyVoteAccountVote: PalletDemocracyVoteAccountVote;
    PalletDemocracyVotePriorLock: PalletDemocracyVotePriorLock;
    PalletDemocracyVoteThreshold: PalletDemocracyVoteThreshold;
    PalletDemocracyVoteVoting: PalletDemocracyVoteVoting;
    PalletEthereumCall: PalletEthereumCall;
    PalletEthereumError: PalletEthereumError;
    PalletEthereumEvent: PalletEthereumEvent;
    PalletEthereumRawOrigin: PalletEthereumRawOrigin;
    PalletEthereumXcmCall: PalletEthereumXcmCall;
    PalletEthereumXcmError: PalletEthereumXcmError;
    PalletEthereumXcmRawOrigin: PalletEthereumXcmRawOrigin;
    PalletEvmCall: PalletEvmCall;
    PalletEvmCodeMetadata: PalletEvmCodeMetadata;
    PalletEvmError: PalletEvmError;
    PalletEvmEvent: PalletEvmEvent;
    PalletIdentityBitFlags: PalletIdentityBitFlags;
    PalletIdentityCall: PalletIdentityCall;
    PalletIdentityError: PalletIdentityError;
    PalletIdentityEvent: PalletIdentityEvent;
    PalletIdentityJudgement: PalletIdentityJudgement;
    PalletIdentityRegistrarInfo: PalletIdentityRegistrarInfo;
    PalletIdentityRegistration: PalletIdentityRegistration;
    PalletIdentitySimpleIdentityField: PalletIdentitySimpleIdentityField;
    PalletIdentitySimpleIdentityInfo: PalletIdentitySimpleIdentityInfo;
    PalletMaintenanceModeCall: PalletMaintenanceModeCall;
    PalletMaintenanceModeError: PalletMaintenanceModeError;
    PalletMaintenanceModeEvent: PalletMaintenanceModeEvent;
    PalletMigrationsError: PalletMigrationsError;
    PalletMigrationsEvent: PalletMigrationsEvent;
    PalletMoonbeamLazyMigrationsCall: PalletMoonbeamLazyMigrationsCall;
    PalletMoonbeamLazyMigrationsError: PalletMoonbeamLazyMigrationsError;
    PalletMoonbeamOrbitersCall: PalletMoonbeamOrbitersCall;
    PalletMoonbeamOrbitersCollatorPoolInfo: PalletMoonbeamOrbitersCollatorPoolInfo;
    PalletMoonbeamOrbitersCurrentOrbiter: PalletMoonbeamOrbitersCurrentOrbiter;
    PalletMoonbeamOrbitersError: PalletMoonbeamOrbitersError;
    PalletMoonbeamOrbitersEvent: PalletMoonbeamOrbitersEvent;
    PalletMultisigCall: PalletMultisigCall;
    PalletMultisigError: PalletMultisigError;
    PalletMultisigEvent: PalletMultisigEvent;
    PalletMultisigMultisig: PalletMultisigMultisig;
    PalletMultisigTimepoint: PalletMultisigTimepoint;
    PalletParachainStakingAutoCompoundAutoCompoundConfig: PalletParachainStakingAutoCompoundAutoCompoundConfig;
    PalletParachainStakingBond: PalletParachainStakingBond;
    PalletParachainStakingBondWithAutoCompound: PalletParachainStakingBondWithAutoCompound;
    PalletParachainStakingCall: PalletParachainStakingCall;
    PalletParachainStakingCandidateBondLessRequest: PalletParachainStakingCandidateBondLessRequest;
    PalletParachainStakingCandidateMetadata: PalletParachainStakingCandidateMetadata;
    PalletParachainStakingCapacityStatus: PalletParachainStakingCapacityStatus;
    PalletParachainStakingCollatorSnapshot: PalletParachainStakingCollatorSnapshot;
    PalletParachainStakingCollatorStatus: PalletParachainStakingCollatorStatus;
    PalletParachainStakingDelayedPayout: PalletParachainStakingDelayedPayout;
    PalletParachainStakingDelegationRequestsCancelledScheduledRequest: PalletParachainStakingDelegationRequestsCancelledScheduledRequest;
    PalletParachainStakingDelegationRequestsDelegationAction: PalletParachainStakingDelegationRequestsDelegationAction;
    PalletParachainStakingDelegationRequestsScheduledRequest: PalletParachainStakingDelegationRequestsScheduledRequest;
    PalletParachainStakingDelegations: PalletParachainStakingDelegations;
    PalletParachainStakingDelegator: PalletParachainStakingDelegator;
    PalletParachainStakingDelegatorAdded: PalletParachainStakingDelegatorAdded;
    PalletParachainStakingDelegatorStatus: PalletParachainStakingDelegatorStatus;
    PalletParachainStakingError: PalletParachainStakingError;
    PalletParachainStakingEvent: PalletParachainStakingEvent;
    PalletParachainStakingInflationInflationInfo: PalletParachainStakingInflationInflationInfo;
    PalletParachainStakingParachainBondConfig: PalletParachainStakingParachainBondConfig;
    PalletParachainStakingRoundInfo: PalletParachainStakingRoundInfo;
    PalletParachainStakingSetBoundedOrderedSet: PalletParachainStakingSetBoundedOrderedSet;
    PalletParachainStakingSetOrderedSet: PalletParachainStakingSetOrderedSet;
    PalletPreimageCall: PalletPreimageCall;
    PalletPreimageError: PalletPreimageError;
    PalletPreimageEvent: PalletPreimageEvent;
    PalletPreimageHoldReason: PalletPreimageHoldReason;
    PalletPreimageOldRequestStatus: PalletPreimageOldRequestStatus;
    PalletPreimageRequestStatus: PalletPreimageRequestStatus;
    PalletProxyAnnouncement: PalletProxyAnnouncement;
    PalletProxyCall: PalletProxyCall;
    PalletProxyError: PalletProxyError;
    PalletProxyEvent: PalletProxyEvent;
    PalletProxyProxyDefinition: PalletProxyProxyDefinition;
    PalletRandomnessCall: PalletRandomnessCall;
    PalletRandomnessError: PalletRandomnessError;
    PalletRandomnessEvent: PalletRandomnessEvent;
    PalletRandomnessRandomnessResult: PalletRandomnessRandomnessResult;
    PalletRandomnessRequest: PalletRandomnessRequest;
    PalletRandomnessRequestInfo: PalletRandomnessRequestInfo;
    PalletRandomnessRequestState: PalletRandomnessRequestState;
    PalletRandomnessRequestType: PalletRandomnessRequestType;
    PalletReferendaCall: PalletReferendaCall;
    PalletReferendaCurve: PalletReferendaCurve;
    PalletReferendaDecidingStatus: PalletReferendaDecidingStatus;
    PalletReferendaDeposit: PalletReferendaDeposit;
    PalletReferendaError: PalletReferendaError;
    PalletReferendaEvent: PalletReferendaEvent;
    PalletReferendaReferendumInfo: PalletReferendaReferendumInfo;
    PalletReferendaReferendumStatus: PalletReferendaReferendumStatus;
    PalletReferendaTrackInfo: PalletReferendaTrackInfo;
    PalletRootTestingCall: PalletRootTestingCall;
    PalletSchedulerCall: PalletSchedulerCall;
    PalletSchedulerError: PalletSchedulerError;
    PalletSchedulerEvent: PalletSchedulerEvent;
    PalletSchedulerScheduled: PalletSchedulerScheduled;
    PalletTimestampCall: PalletTimestampCall;
    PalletTransactionPaymentChargeTransactionPayment: PalletTransactionPaymentChargeTransactionPayment;
    PalletTransactionPaymentEvent: PalletTransactionPaymentEvent;
    PalletTransactionPaymentReleases: PalletTransactionPaymentReleases;
    PalletTreasuryCall: PalletTreasuryCall;
    PalletTreasuryError: PalletTreasuryError;
    PalletTreasuryEvent: PalletTreasuryEvent;
    PalletTreasuryPaymentState: PalletTreasuryPaymentState;
    PalletTreasuryProposal: PalletTreasuryProposal;
    PalletTreasurySpendStatus: PalletTreasurySpendStatus;
    PalletUtilityCall: PalletUtilityCall;
    PalletUtilityError: PalletUtilityError;
    PalletUtilityEvent: PalletUtilityEvent;
    PalletWhitelistCall: PalletWhitelistCall;
    PalletWhitelistError: PalletWhitelistError;
    PalletWhitelistEvent: PalletWhitelistEvent;
    PalletXcmCall: PalletXcmCall;
    PalletXcmError: PalletXcmError;
    PalletXcmEvent: PalletXcmEvent;
    PalletXcmOrigin: PalletXcmOrigin;
    PalletXcmQueryStatus: PalletXcmQueryStatus;
    PalletXcmRemoteLockedFungibleRecord: PalletXcmRemoteLockedFungibleRecord;
    PalletXcmTransactorCall: PalletXcmTransactorCall;
    PalletXcmTransactorCurrency: PalletXcmTransactorCurrency;
    PalletXcmTransactorCurrencyPayment: PalletXcmTransactorCurrencyPayment;
    PalletXcmTransactorError: PalletXcmTransactorError;
    PalletXcmTransactorEvent: PalletXcmTransactorEvent;
    PalletXcmTransactorHrmpInitParams: PalletXcmTransactorHrmpInitParams;
    PalletXcmTransactorHrmpOperation: PalletXcmTransactorHrmpOperation;
    PalletXcmTransactorRelayIndicesRelayChainIndices: PalletXcmTransactorRelayIndicesRelayChainIndices;
    PalletXcmTransactorRemoteTransactInfoWithMaxWeight: PalletXcmTransactorRemoteTransactInfoWithMaxWeight;
    PalletXcmTransactorTransactWeights: PalletXcmTransactorTransactWeights;
    PalletXcmVersionMigrationStage: PalletXcmVersionMigrationStage;
    PolkadotCorePrimitivesInboundDownwardMessage: PolkadotCorePrimitivesInboundDownwardMessage;
    PolkadotCorePrimitivesInboundHrmpMessage: PolkadotCorePrimitivesInboundHrmpMessage;
    PolkadotCorePrimitivesOutboundHrmpMessage: PolkadotCorePrimitivesOutboundHrmpMessage;
    PolkadotParachainPrimitivesPrimitivesHrmpChannelId: PolkadotParachainPrimitivesPrimitivesHrmpChannelId;
    PolkadotParachainPrimitivesPrimitivesXcmpMessageFormat: PolkadotParachainPrimitivesPrimitivesXcmpMessageFormat;
    PolkadotPrimitivesV6AbridgedHostConfiguration: PolkadotPrimitivesV6AbridgedHostConfiguration;
    PolkadotPrimitivesV6AbridgedHrmpChannel: PolkadotPrimitivesV6AbridgedHrmpChannel;
    PolkadotPrimitivesV6AsyncBackingAsyncBackingParams: PolkadotPrimitivesV6AsyncBackingAsyncBackingParams;
    PolkadotPrimitivesV6PersistedValidationData: PolkadotPrimitivesV6PersistedValidationData;
    PolkadotPrimitivesV6UpgradeGoAhead: PolkadotPrimitivesV6UpgradeGoAhead;
    PolkadotPrimitivesV6UpgradeRestriction: PolkadotPrimitivesV6UpgradeRestriction;
    SessionKeysPrimitivesVrfVrfCryptoPublic: SessionKeysPrimitivesVrfVrfCryptoPublic;
    SpArithmeticArithmeticError: SpArithmeticArithmeticError;
    SpCoreEcdsaSignature: SpCoreEcdsaSignature;
    SpCoreEd25519Signature: SpCoreEd25519Signature;
    SpCoreSr25519Public: SpCoreSr25519Public;
    SpCoreSr25519Signature: SpCoreSr25519Signature;
    SpCoreVoid: SpCoreVoid;
    SpRuntimeBlakeTwo256: SpRuntimeBlakeTwo256;
    SpRuntimeDigest: SpRuntimeDigest;
    SpRuntimeDigestDigestItem: SpRuntimeDigestDigestItem;
    SpRuntimeDispatchError: SpRuntimeDispatchError;
    SpRuntimeDispatchErrorWithPostInfo: SpRuntimeDispatchErrorWithPostInfo;
    SpRuntimeModuleError: SpRuntimeModuleError;
    SpRuntimeMultiSignature: SpRuntimeMultiSignature;
    SpRuntimeTokenError: SpRuntimeTokenError;
    SpRuntimeTransactionalError: SpRuntimeTransactionalError;
    SpTrieStorageProof: SpTrieStorageProof;
    SpVersionRuntimeVersion: SpVersionRuntimeVersion;
    SpWeightsRuntimeDbWeight: SpWeightsRuntimeDbWeight;
    SpWeightsWeightV2Weight: SpWeightsWeightV2Weight;
    StagingXcmV3MultiLocation: StagingXcmV3MultiLocation;
    XcmDoubleEncoded: XcmDoubleEncoded;
    XcmPrimitivesEthereumXcmEthereumXcmFee: XcmPrimitivesEthereumXcmEthereumXcmFee;
    XcmPrimitivesEthereumXcmEthereumXcmTransaction: XcmPrimitivesEthereumXcmEthereumXcmTransaction;
    XcmPrimitivesEthereumXcmEthereumXcmTransactionV1: XcmPrimitivesEthereumXcmEthereumXcmTransactionV1;
    XcmPrimitivesEthereumXcmEthereumXcmTransactionV2: XcmPrimitivesEthereumXcmEthereumXcmTransactionV2;
    XcmPrimitivesEthereumXcmManualEthereumXcmFee: XcmPrimitivesEthereumXcmManualEthereumXcmFee;
    XcmV2BodyId: XcmV2BodyId;
    XcmV2BodyPart: XcmV2BodyPart;
    XcmV2Instruction: XcmV2Instruction;
    XcmV2Junction: XcmV2Junction;
    XcmV2MultiAsset: XcmV2MultiAsset;
    XcmV2MultiLocation: XcmV2MultiLocation;
    XcmV2MultiassetAssetId: XcmV2MultiassetAssetId;
    XcmV2MultiassetAssetInstance: XcmV2MultiassetAssetInstance;
    XcmV2MultiassetFungibility: XcmV2MultiassetFungibility;
    XcmV2MultiassetMultiAssetFilter: XcmV2MultiassetMultiAssetFilter;
    XcmV2MultiassetMultiAssets: XcmV2MultiassetMultiAssets;
    XcmV2MultiassetWildFungibility: XcmV2MultiassetWildFungibility;
    XcmV2MultiassetWildMultiAsset: XcmV2MultiassetWildMultiAsset;
    XcmV2MultilocationJunctions: XcmV2MultilocationJunctions;
    XcmV2NetworkId: XcmV2NetworkId;
    XcmV2OriginKind: XcmV2OriginKind;
    XcmV2Response: XcmV2Response;
    XcmV2TraitsError: XcmV2TraitsError;
    XcmV2WeightLimit: XcmV2WeightLimit;
    XcmV2Xcm: XcmV2Xcm;
    XcmV3Instruction: XcmV3Instruction;
    XcmV3Junction: XcmV3Junction;
    XcmV3JunctionBodyId: XcmV3JunctionBodyId;
    XcmV3JunctionBodyPart: XcmV3JunctionBodyPart;
    XcmV3JunctionNetworkId: XcmV3JunctionNetworkId;
    XcmV3Junctions: XcmV3Junctions;
    XcmV3MaybeErrorCode: XcmV3MaybeErrorCode;
    XcmV3MultiAsset: XcmV3MultiAsset;
    XcmV3MultiassetAssetId: XcmV3MultiassetAssetId;
    XcmV3MultiassetAssetInstance: XcmV3MultiassetAssetInstance;
    XcmV3MultiassetFungibility: XcmV3MultiassetFungibility;
    XcmV3MultiassetMultiAssetFilter: XcmV3MultiassetMultiAssetFilter;
    XcmV3MultiassetMultiAssets: XcmV3MultiassetMultiAssets;
    XcmV3MultiassetWildFungibility: XcmV3MultiassetWildFungibility;
    XcmV3MultiassetWildMultiAsset: XcmV3MultiassetWildMultiAsset;
    XcmV3PalletInfo: XcmV3PalletInfo;
    XcmV3QueryResponseInfo: XcmV3QueryResponseInfo;
    XcmV3Response: XcmV3Response;
    XcmV3TraitsError: XcmV3TraitsError;
    XcmV3TraitsOutcome: XcmV3TraitsOutcome;
    XcmV3WeightLimit: XcmV3WeightLimit;
    XcmV3Xcm: XcmV3Xcm;
    XcmVersionedAssetId: XcmVersionedAssetId;
    XcmVersionedMultiAsset: XcmVersionedMultiAsset;
    XcmVersionedMultiAssets: XcmVersionedMultiAssets;
    XcmVersionedMultiLocation: XcmVersionedMultiLocation;
    XcmVersionedResponse: XcmVersionedResponse;
    XcmVersionedXcm: XcmVersionedXcm;
  } // InterfaceTypes
} // declare module
