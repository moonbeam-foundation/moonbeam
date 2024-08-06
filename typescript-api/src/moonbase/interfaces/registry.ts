// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/types/types/registry";

import type {
  AccountEthereumSignature,
  CumulusPalletDmpQueueCall,
  CumulusPalletDmpQueueEvent,
  CumulusPalletDmpQueueMigrationState,
  CumulusPalletParachainSystemCall,
  CumulusPalletParachainSystemError,
  CumulusPalletParachainSystemEvent,
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot,
  CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity,
  CumulusPalletParachainSystemUnincludedSegmentAncestor,
  CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate,
  CumulusPalletParachainSystemUnincludedSegmentSegmentTracker,
  CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth,
  CumulusPalletXcmEvent,
  CumulusPalletXcmOrigin,
  CumulusPalletXcmpQueueCall,
  CumulusPalletXcmpQueueError,
  CumulusPalletXcmpQueueEvent,
  CumulusPalletXcmpQueueOutboundChannelDetails,
  CumulusPalletXcmpQueueOutboundState,
  CumulusPalletXcmpQueueQueueConfigData,
  CumulusPrimitivesCoreAggregateMessageOrigin,
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
  FrameSupportMessagesProcessMessageError,
  FrameSupportPalletId,
  FrameSupportPreimagesBounded,
  FrameSupportScheduleDispatchTime,
  FrameSupportTokensMiscBalanceStatus,
  FrameSystemAccountInfo,
  FrameSystemCall,
  FrameSystemCodeUpgradeAuthorization,
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
  MoonbaseRuntimeAssetConfigAssetRegistrarMetadata,
  MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin,
  MoonbaseRuntimeOriginCaller,
  MoonbaseRuntimeProxyType,
  MoonbaseRuntimeRuntime,
  MoonbaseRuntimeRuntimeHoldReason,
  MoonbaseRuntimeXcmConfigAssetType,
  MoonbaseRuntimeXcmConfigCurrencyId,
  MoonbaseRuntimeXcmConfigTransactors,
  NimbusPrimitivesNimbusCryptoPublic,
  OrmlXtokensModuleCall,
  OrmlXtokensModuleError,
  OrmlXtokensModuleEvent,
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
  PalletBalancesAdjustmentDirection,
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
  PalletEmergencyParaXcmCall,
  PalletEmergencyParaXcmError,
  PalletEmergencyParaXcmEvent,
  PalletEmergencyParaXcmXcmMode,
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
  PalletIdentityAuthorityProperties,
  PalletIdentityCall,
  PalletIdentityError,
  PalletIdentityEvent,
  PalletIdentityJudgement,
  PalletIdentityLegacyIdentityInfo,
  PalletIdentityRegistrarInfo,
  PalletIdentityRegistration,
  PalletMaintenanceModeCall,
  PalletMaintenanceModeError,
  PalletMaintenanceModeEvent,
  PalletMessageQueueBookState,
  PalletMessageQueueCall,
  PalletMessageQueueError,
  PalletMessageQueueEvent,
  PalletMessageQueueNeighbours,
  PalletMessageQueuePage,
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
  PalletPrecompileBenchmarksError,
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
  PalletRootTestingEvent,
  PalletSchedulerCall,
  PalletSchedulerError,
  PalletSchedulerEvent,
  PalletSchedulerRetryConfig,
  PalletSchedulerScheduled,
  PalletSudoCall,
  PalletSudoError,
  PalletSudoEvent,
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
  PolkadotPrimitivesV7AbridgedHostConfiguration,
  PolkadotPrimitivesV7AbridgedHrmpChannel,
  PolkadotPrimitivesV7AsyncBackingAsyncBackingParams,
  PolkadotPrimitivesV7PersistedValidationData,
  PolkadotPrimitivesV7UpgradeGoAhead,
  PolkadotPrimitivesV7UpgradeRestriction,
  SessionKeysPrimitivesVrfVrfCryptoPublic,
  SpArithmeticArithmeticError,
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
  StagingXcmExecutorAssetTransferTransferType,
  StagingXcmV3MultiLocation,
  StagingXcmV4Asset,
  StagingXcmV4AssetAssetFilter,
  StagingXcmV4AssetAssetId,
  StagingXcmV4AssetAssetInstance,
  StagingXcmV4AssetAssets,
  StagingXcmV4AssetFungibility,
  StagingXcmV4AssetWildAsset,
  StagingXcmV4AssetWildFungibility,
  StagingXcmV4Instruction,
  StagingXcmV4Junction,
  StagingXcmV4JunctionNetworkId,
  StagingXcmV4Junctions,
  StagingXcmV4Location,
  StagingXcmV4PalletInfo,
  StagingXcmV4QueryResponseInfo,
  StagingXcmV4Response,
  StagingXcmV4TraitsOutcome,
  StagingXcmV4Xcm,
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
  XcmV3WeightLimit,
  XcmV3Xcm,
  XcmVersionedAsset,
  XcmVersionedAssetId,
  XcmVersionedAssets,
  XcmVersionedLocation,
  XcmVersionedResponse,
  XcmVersionedXcm,
} from "@polkadot/types/lookup";

declare module "@polkadot/types/types/registry" {
  interface InterfaceTypes {
    AccountEthereumSignature: AccountEthereumSignature;
    CumulusPalletDmpQueueCall: CumulusPalletDmpQueueCall;
    CumulusPalletDmpQueueEvent: CumulusPalletDmpQueueEvent;
    CumulusPalletDmpQueueMigrationState: CumulusPalletDmpQueueMigrationState;
    CumulusPalletParachainSystemCall: CumulusPalletParachainSystemCall;
    CumulusPalletParachainSystemError: CumulusPalletParachainSystemError;
    CumulusPalletParachainSystemEvent: CumulusPalletParachainSystemEvent;
    CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot;
    CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity: CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity;
    CumulusPalletParachainSystemUnincludedSegmentAncestor: CumulusPalletParachainSystemUnincludedSegmentAncestor;
    CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate: CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate;
    CumulusPalletParachainSystemUnincludedSegmentSegmentTracker: CumulusPalletParachainSystemUnincludedSegmentSegmentTracker;
    CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth: CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth;
    CumulusPalletXcmEvent: CumulusPalletXcmEvent;
    CumulusPalletXcmOrigin: CumulusPalletXcmOrigin;
    CumulusPalletXcmpQueueCall: CumulusPalletXcmpQueueCall;
    CumulusPalletXcmpQueueError: CumulusPalletXcmpQueueError;
    CumulusPalletXcmpQueueEvent: CumulusPalletXcmpQueueEvent;
    CumulusPalletXcmpQueueOutboundChannelDetails: CumulusPalletXcmpQueueOutboundChannelDetails;
    CumulusPalletXcmpQueueOutboundState: CumulusPalletXcmpQueueOutboundState;
    CumulusPalletXcmpQueueQueueConfigData: CumulusPalletXcmpQueueQueueConfigData;
    CumulusPrimitivesCoreAggregateMessageOrigin: CumulusPrimitivesCoreAggregateMessageOrigin;
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
    FrameSupportMessagesProcessMessageError: FrameSupportMessagesProcessMessageError;
    FrameSupportPalletId: FrameSupportPalletId;
    FrameSupportPreimagesBounded: FrameSupportPreimagesBounded;
    FrameSupportScheduleDispatchTime: FrameSupportScheduleDispatchTime;
    FrameSupportTokensMiscBalanceStatus: FrameSupportTokensMiscBalanceStatus;
    FrameSystemAccountInfo: FrameSystemAccountInfo;
    FrameSystemCall: FrameSystemCall;
    FrameSystemCodeUpgradeAuthorization: FrameSystemCodeUpgradeAuthorization;
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
    MoonbaseRuntimeAssetConfigAssetRegistrarMetadata: MoonbaseRuntimeAssetConfigAssetRegistrarMetadata;
    MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin: MoonbaseRuntimeGovernanceOriginsCustomOriginsOrigin;
    MoonbaseRuntimeOriginCaller: MoonbaseRuntimeOriginCaller;
    MoonbaseRuntimeProxyType: MoonbaseRuntimeProxyType;
    MoonbaseRuntimeRuntime: MoonbaseRuntimeRuntime;
    MoonbaseRuntimeRuntimeHoldReason: MoonbaseRuntimeRuntimeHoldReason;
    MoonbaseRuntimeXcmConfigAssetType: MoonbaseRuntimeXcmConfigAssetType;
    MoonbaseRuntimeXcmConfigCurrencyId: MoonbaseRuntimeXcmConfigCurrencyId;
    MoonbaseRuntimeXcmConfigTransactors: MoonbaseRuntimeXcmConfigTransactors;
    NimbusPrimitivesNimbusCryptoPublic: NimbusPrimitivesNimbusCryptoPublic;
    OrmlXtokensModuleCall: OrmlXtokensModuleCall;
    OrmlXtokensModuleError: OrmlXtokensModuleError;
    OrmlXtokensModuleEvent: OrmlXtokensModuleEvent;
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
    PalletBalancesAdjustmentDirection: PalletBalancesAdjustmentDirection;
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
    PalletEmergencyParaXcmCall: PalletEmergencyParaXcmCall;
    PalletEmergencyParaXcmError: PalletEmergencyParaXcmError;
    PalletEmergencyParaXcmEvent: PalletEmergencyParaXcmEvent;
    PalletEmergencyParaXcmXcmMode: PalletEmergencyParaXcmXcmMode;
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
    PalletIdentityAuthorityProperties: PalletIdentityAuthorityProperties;
    PalletIdentityCall: PalletIdentityCall;
    PalletIdentityError: PalletIdentityError;
    PalletIdentityEvent: PalletIdentityEvent;
    PalletIdentityJudgement: PalletIdentityJudgement;
    PalletIdentityLegacyIdentityInfo: PalletIdentityLegacyIdentityInfo;
    PalletIdentityRegistrarInfo: PalletIdentityRegistrarInfo;
    PalletIdentityRegistration: PalletIdentityRegistration;
    PalletMaintenanceModeCall: PalletMaintenanceModeCall;
    PalletMaintenanceModeError: PalletMaintenanceModeError;
    PalletMaintenanceModeEvent: PalletMaintenanceModeEvent;
    PalletMessageQueueBookState: PalletMessageQueueBookState;
    PalletMessageQueueCall: PalletMessageQueueCall;
    PalletMessageQueueError: PalletMessageQueueError;
    PalletMessageQueueEvent: PalletMessageQueueEvent;
    PalletMessageQueueNeighbours: PalletMessageQueueNeighbours;
    PalletMessageQueuePage: PalletMessageQueuePage;
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
    PalletPrecompileBenchmarksError: PalletPrecompileBenchmarksError;
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
    PalletRootTestingEvent: PalletRootTestingEvent;
    PalletSchedulerCall: PalletSchedulerCall;
    PalletSchedulerError: PalletSchedulerError;
    PalletSchedulerEvent: PalletSchedulerEvent;
    PalletSchedulerRetryConfig: PalletSchedulerRetryConfig;
    PalletSchedulerScheduled: PalletSchedulerScheduled;
    PalletSudoCall: PalletSudoCall;
    PalletSudoError: PalletSudoError;
    PalletSudoEvent: PalletSudoEvent;
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
    PolkadotPrimitivesV7AbridgedHostConfiguration: PolkadotPrimitivesV7AbridgedHostConfiguration;
    PolkadotPrimitivesV7AbridgedHrmpChannel: PolkadotPrimitivesV7AbridgedHrmpChannel;
    PolkadotPrimitivesV7AsyncBackingAsyncBackingParams: PolkadotPrimitivesV7AsyncBackingAsyncBackingParams;
    PolkadotPrimitivesV7PersistedValidationData: PolkadotPrimitivesV7PersistedValidationData;
    PolkadotPrimitivesV7UpgradeGoAhead: PolkadotPrimitivesV7UpgradeGoAhead;
    PolkadotPrimitivesV7UpgradeRestriction: PolkadotPrimitivesV7UpgradeRestriction;
    SessionKeysPrimitivesVrfVrfCryptoPublic: SessionKeysPrimitivesVrfVrfCryptoPublic;
    SpArithmeticArithmeticError: SpArithmeticArithmeticError;
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
    StagingXcmExecutorAssetTransferTransferType: StagingXcmExecutorAssetTransferTransferType;
    StagingXcmV3MultiLocation: StagingXcmV3MultiLocation;
    StagingXcmV4Asset: StagingXcmV4Asset;
    StagingXcmV4AssetAssetFilter: StagingXcmV4AssetAssetFilter;
    StagingXcmV4AssetAssetId: StagingXcmV4AssetAssetId;
    StagingXcmV4AssetAssetInstance: StagingXcmV4AssetAssetInstance;
    StagingXcmV4AssetAssets: StagingXcmV4AssetAssets;
    StagingXcmV4AssetFungibility: StagingXcmV4AssetFungibility;
    StagingXcmV4AssetWildAsset: StagingXcmV4AssetWildAsset;
    StagingXcmV4AssetWildFungibility: StagingXcmV4AssetWildFungibility;
    StagingXcmV4Instruction: StagingXcmV4Instruction;
    StagingXcmV4Junction: StagingXcmV4Junction;
    StagingXcmV4JunctionNetworkId: StagingXcmV4JunctionNetworkId;
    StagingXcmV4Junctions: StagingXcmV4Junctions;
    StagingXcmV4Location: StagingXcmV4Location;
    StagingXcmV4PalletInfo: StagingXcmV4PalletInfo;
    StagingXcmV4QueryResponseInfo: StagingXcmV4QueryResponseInfo;
    StagingXcmV4Response: StagingXcmV4Response;
    StagingXcmV4TraitsOutcome: StagingXcmV4TraitsOutcome;
    StagingXcmV4Xcm: StagingXcmV4Xcm;
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
    XcmV3WeightLimit: XcmV3WeightLimit;
    XcmV3Xcm: XcmV3Xcm;
    XcmVersionedAsset: XcmVersionedAsset;
    XcmVersionedAssetId: XcmVersionedAssetId;
    XcmVersionedAssets: XcmVersionedAssets;
    XcmVersionedLocation: XcmVersionedLocation;
    XcmVersionedResponse: XcmVersionedResponse;
    XcmVersionedXcm: XcmVersionedXcm;
  } // InterfaceTypes
} // declare module
