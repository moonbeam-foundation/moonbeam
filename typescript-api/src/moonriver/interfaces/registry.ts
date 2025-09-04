// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/types/types/registry";

import type {
  AccountEthereumSignature,
  BpHeaderChainAuthoritySet,
  BpHeaderChainHeaderChainError,
  BpHeaderChainHeaderFinalityInfo,
  BpHeaderChainInitializationData,
  BpHeaderChainJustificationGrandpaJustification,
  BpHeaderChainStoredHeaderData,
  BpMessagesDeliveredMessages,
  BpMessagesInboundLaneData,
  BpMessagesLaneLaneState,
  BpMessagesMessageKey,
  BpMessagesMessagesOperatingMode,
  BpMessagesOutboundLaneData,
  BpMessagesReceivedMessages,
  BpMessagesReceptionResult,
  BpMessagesSourceChainFromBridgedChainMessagesDeliveryProof,
  BpMessagesTargetChainFromBridgedChainMessagesProof,
  BpMessagesUnrewardedRelayer,
  BpMessagesUnrewardedRelayersState,
  BpMessagesVerificationError,
  BpParachainsBestParaHeadHash,
  BpParachainsParaInfo,
  BpPolkadotCoreParachainsParaHeadsProof,
  BpRuntimeBasicOperatingMode,
  BpRuntimeHeaderId,
  BpRuntimeMessagesMessageDispatchResult,
  BpRuntimeOwnedBridgeModuleError,
  BpRuntimeStorageProofStorageProofError,
  BpXcmBridgeHubBridge,
  BpXcmBridgeHubBridgeLocationsError,
  BpXcmBridgeHubBridgeState,
  CumulusPalletParachainSystemCall,
  CumulusPalletParachainSystemError,
  CumulusPalletParachainSystemEvent,
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot,
  CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity,
  CumulusPalletParachainSystemUnincludedSegmentAncestor,
  CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate,
  CumulusPalletParachainSystemUnincludedSegmentSegmentTracker,
  CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth,
  CumulusPalletWeightReclaimStorageWeightReclaim,
  CumulusPalletXcmEvent,
  CumulusPalletXcmOrigin,
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
  EthereumReceiptReceiptV4,
  EthereumTransactionEip1559Eip1559Transaction,
  EthereumTransactionEip2930AccessListItem,
  EthereumTransactionEip2930Eip2930Transaction,
  EthereumTransactionEip2930MalleableTransactionSignature,
  EthereumTransactionEip2930TransactionSignature,
  EthereumTransactionEip7702AuthorizationListItem,
  EthereumTransactionEip7702Eip7702Transaction,
  EthereumTransactionLegacyLegacyTransaction,
  EthereumTransactionLegacyTransactionAction,
  EthereumTransactionLegacyTransactionSignature,
  EthereumTransactionTransactionV3,
  EthereumTypesHashH64,
  EvmCoreErrorExitError,
  EvmCoreErrorExitFatal,
  EvmCoreErrorExitReason,
  EvmCoreErrorExitRevert,
  EvmCoreErrorExitSucceed,
  FinalityGrandpaCommit,
  FinalityGrandpaPrecommit,
  FinalityGrandpaSignedPrecommit,
  FpRpcTransactionStatus,
  FrameMetadataHashExtensionCheckMetadataHash,
  FrameMetadataHashExtensionMode,
  FrameSupportDispatchDispatchClass,
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
  FrameSupportStorageDisabled,
  FrameSupportTokensFungibleUnionOfNativeOrWithId,
  FrameSupportTokensMiscBalanceStatus,
  FrameSupportTokensMiscIdAmount,
  FrameSystemAccountInfo,
  FrameSystemCall,
  FrameSystemCodeUpgradeAuthorization,
  FrameSystemDispatchEventInfo,
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
  MoonriverRuntimeBridgeRejectObsoleteHeadersAndMessages,
  MoonriverRuntimeGovernanceOriginsCustomOriginsOrigin,
  MoonriverRuntimeOriginCaller,
  MoonriverRuntimeProxyType,
  MoonriverRuntimeRuntime,
  MoonriverRuntimeRuntimeHoldReason,
  MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessDeposit,
  MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParameters,
  MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersKey,
  MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersValue,
  MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigFeesTreasuryProportion,
  MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParameters,
  MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersKey,
  MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersValue,
  MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigForeignAssetCreationDeposit,
  MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParameters,
  MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParametersKey,
  MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParametersValue,
  MoonriverRuntimeRuntimeParamsRuntimeParameters,
  MoonriverRuntimeRuntimeParamsRuntimeParametersKey,
  MoonriverRuntimeRuntimeParamsRuntimeParametersValue,
  MoonriverRuntimeXcmConfigCurrencyId,
  MoonriverRuntimeXcmConfigTransactors,
  NimbusPrimitivesNimbusCryptoPublic,
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
  PalletBalancesReasons,
  PalletBalancesReserveData,
  PalletBridgeGrandpaCall,
  PalletBridgeGrandpaError,
  PalletBridgeGrandpaEvent,
  PalletBridgeGrandpaStorageTypesStoredAuthoritySet,
  PalletBridgeMessagesCall,
  PalletBridgeMessagesError,
  PalletBridgeMessagesEvent,
  PalletBridgeMessagesLanesManagerLanesManagerError,
  PalletBridgeMessagesOutboundLaneReceptionConfirmationError,
  PalletBridgeParachainsCall,
  PalletBridgeParachainsError,
  PalletBridgeParachainsEvent,
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
  PalletEthereumXcmEvent,
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
  PalletIdentityProvider,
  PalletIdentityRegistrarInfo,
  PalletIdentityRegistration,
  PalletIdentityUsernameInformation,
  PalletMaintenanceModeCall,
  PalletMaintenanceModeError,
  PalletMaintenanceModeEvent,
  PalletMessageQueueBookState,
  PalletMessageQueueCall,
  PalletMessageQueueError,
  PalletMessageQueueEvent,
  PalletMessageQueueNeighbours,
  PalletMessageQueuePage,
  PalletMigrationsActiveCursor,
  PalletMigrationsCall,
  PalletMigrationsError,
  PalletMigrationsEvent,
  PalletMigrationsHistoricCleanupSelector,
  PalletMigrationsMigrationCursor,
  PalletMoonbeamForeignAssetsAssetDepositDetails,
  PalletMoonbeamForeignAssetsAssetStatus,
  PalletMoonbeamForeignAssetsCall,
  PalletMoonbeamForeignAssetsError,
  PalletMoonbeamForeignAssetsEvent,
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
  PalletParachainStakingInflationDistributionAccount,
  PalletParachainStakingInflationDistributionConfig,
  PalletParachainStakingInflationInflationInfo,
  PalletParachainStakingRoundInfo,
  PalletParachainStakingSetBoundedOrderedSet,
  PalletParachainStakingSetOrderedSet,
  PalletParametersCall,
  PalletParametersEvent,
  PalletPreimageCall,
  PalletPreimageError,
  PalletPreimageEvent,
  PalletPreimageHoldReason,
  PalletPreimageOldRequestStatus,
  PalletPreimageRequestStatus,
  PalletProxyAnnouncement,
  PalletProxyCall,
  PalletProxyDepositKind,
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
  PalletReferendaTrackDetails,
  PalletRootTestingCall,
  PalletRootTestingEvent,
  PalletSchedulerCall,
  PalletSchedulerError,
  PalletSchedulerEvent,
  PalletSchedulerRetryConfig,
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
  PalletXcmAuthorizedAliasesEntry,
  PalletXcmBridgeHubCall,
  PalletXcmBridgeHubDispatcherXcmBlobMessageDispatchResult,
  PalletXcmBridgeHubError,
  PalletXcmBridgeHubEvent,
  PalletXcmBridgeHubHoldReason,
  PalletXcmCall,
  PalletXcmError,
  PalletXcmEvent,
  PalletXcmMaxAuthorizedAliases,
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
  PalletXcmWeightTraderCall,
  PalletXcmWeightTraderError,
  PalletXcmWeightTraderEvent,
  PolkadotCorePrimitivesInboundDownwardMessage,
  PolkadotCorePrimitivesInboundHrmpMessage,
  PolkadotCorePrimitivesOutboundHrmpMessage,
  PolkadotParachainPrimitivesPrimitivesHrmpChannelId,
  PolkadotPrimitivesV8AbridgedHostConfiguration,
  PolkadotPrimitivesV8AbridgedHrmpChannel,
  PolkadotPrimitivesV8AsyncBackingAsyncBackingParams,
  PolkadotPrimitivesV8PersistedValidationData,
  PolkadotPrimitivesV8UpgradeGoAhead,
  PolkadotPrimitivesV8UpgradeRestriction,
  SessionKeysPrimitivesVrfVrfCryptoPublic,
  SpArithmeticArithmeticError,
  SpConsensusGrandpaAppPublic,
  SpConsensusGrandpaAppSignature,
  SpRuntimeBlakeTwo256,
  SpRuntimeDigest,
  SpRuntimeDigestDigestItem,
  SpRuntimeDispatchError,
  SpRuntimeDispatchErrorWithPostInfo,
  SpRuntimeHeader,
  SpRuntimeModuleError,
  SpRuntimeMultiSignature,
  SpRuntimeProvingTrieTrieError,
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
  StagingXcmV4Xcm,
  StagingXcmV5Asset,
  StagingXcmV5AssetAssetFilter,
  StagingXcmV5AssetAssetId,
  StagingXcmV5AssetAssetInstance,
  StagingXcmV5AssetAssetTransferFilter,
  StagingXcmV5AssetAssets,
  StagingXcmV5AssetFungibility,
  StagingXcmV5AssetWildAsset,
  StagingXcmV5AssetWildFungibility,
  StagingXcmV5Hint,
  StagingXcmV5Instruction,
  StagingXcmV5Junction,
  StagingXcmV5JunctionNetworkId,
  StagingXcmV5Junctions,
  StagingXcmV5Location,
  StagingXcmV5PalletInfo,
  StagingXcmV5QueryResponseInfo,
  StagingXcmV5Response,
  StagingXcmV5TraitsOutcome,
  StagingXcmV5Xcm,
  XcmDoubleEncoded,
  XcmPrimitivesEthereumXcmEthereumXcmFee,
  XcmPrimitivesEthereumXcmEthereumXcmTransaction,
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV1,
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV2,
  XcmPrimitivesEthereumXcmEthereumXcmTransactionV3,
  XcmPrimitivesEthereumXcmManualEthereumXcmFee,
  XcmRuntimeApisAuthorizedAliasesOriginAliaser,
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
  XcmV3OriginKind,
  XcmV3PalletInfo,
  XcmV3QueryResponseInfo,
  XcmV3Response,
  XcmV3TraitsError,
  XcmV3TraitsSendError,
  XcmV3WeightLimit,
  XcmV3Xcm,
  XcmV5TraitsError,
  XcmVersionedAssetId,
  XcmVersionedAssets,
  XcmVersionedInteriorLocation,
  XcmVersionedLocation,
  XcmVersionedResponse,
  XcmVersionedXcm
} from "@polkadot/types/lookup";

declare module "@polkadot/types/types/registry" {
  interface InterfaceTypes {
    AccountEthereumSignature: AccountEthereumSignature;
    BpHeaderChainAuthoritySet: BpHeaderChainAuthoritySet;
    BpHeaderChainHeaderChainError: BpHeaderChainHeaderChainError;
    BpHeaderChainHeaderFinalityInfo: BpHeaderChainHeaderFinalityInfo;
    BpHeaderChainInitializationData: BpHeaderChainInitializationData;
    BpHeaderChainJustificationGrandpaJustification: BpHeaderChainJustificationGrandpaJustification;
    BpHeaderChainStoredHeaderData: BpHeaderChainStoredHeaderData;
    BpMessagesDeliveredMessages: BpMessagesDeliveredMessages;
    BpMessagesInboundLaneData: BpMessagesInboundLaneData;
    BpMessagesLaneLaneState: BpMessagesLaneLaneState;
    BpMessagesMessageKey: BpMessagesMessageKey;
    BpMessagesMessagesOperatingMode: BpMessagesMessagesOperatingMode;
    BpMessagesOutboundLaneData: BpMessagesOutboundLaneData;
    BpMessagesReceivedMessages: BpMessagesReceivedMessages;
    BpMessagesReceptionResult: BpMessagesReceptionResult;
    BpMessagesSourceChainFromBridgedChainMessagesDeliveryProof: BpMessagesSourceChainFromBridgedChainMessagesDeliveryProof;
    BpMessagesTargetChainFromBridgedChainMessagesProof: BpMessagesTargetChainFromBridgedChainMessagesProof;
    BpMessagesUnrewardedRelayer: BpMessagesUnrewardedRelayer;
    BpMessagesUnrewardedRelayersState: BpMessagesUnrewardedRelayersState;
    BpMessagesVerificationError: BpMessagesVerificationError;
    BpParachainsBestParaHeadHash: BpParachainsBestParaHeadHash;
    BpParachainsParaInfo: BpParachainsParaInfo;
    BpPolkadotCoreParachainsParaHeadsProof: BpPolkadotCoreParachainsParaHeadsProof;
    BpRuntimeBasicOperatingMode: BpRuntimeBasicOperatingMode;
    BpRuntimeHeaderId: BpRuntimeHeaderId;
    BpRuntimeMessagesMessageDispatchResult: BpRuntimeMessagesMessageDispatchResult;
    BpRuntimeOwnedBridgeModuleError: BpRuntimeOwnedBridgeModuleError;
    BpRuntimeStorageProofStorageProofError: BpRuntimeStorageProofStorageProofError;
    BpXcmBridgeHubBridge: BpXcmBridgeHubBridge;
    BpXcmBridgeHubBridgeLocationsError: BpXcmBridgeHubBridgeLocationsError;
    BpXcmBridgeHubBridgeState: BpXcmBridgeHubBridgeState;
    CumulusPalletParachainSystemCall: CumulusPalletParachainSystemCall;
    CumulusPalletParachainSystemError: CumulusPalletParachainSystemError;
    CumulusPalletParachainSystemEvent: CumulusPalletParachainSystemEvent;
    CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot;
    CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity: CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity;
    CumulusPalletParachainSystemUnincludedSegmentAncestor: CumulusPalletParachainSystemUnincludedSegmentAncestor;
    CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate: CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate;
    CumulusPalletParachainSystemUnincludedSegmentSegmentTracker: CumulusPalletParachainSystemUnincludedSegmentSegmentTracker;
    CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth: CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth;
    CumulusPalletWeightReclaimStorageWeightReclaim: CumulusPalletWeightReclaimStorageWeightReclaim;
    CumulusPalletXcmEvent: CumulusPalletXcmEvent;
    CumulusPalletXcmOrigin: CumulusPalletXcmOrigin;
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
    EthereumReceiptReceiptV4: EthereumReceiptReceiptV4;
    EthereumTransactionEip1559Eip1559Transaction: EthereumTransactionEip1559Eip1559Transaction;
    EthereumTransactionEip2930AccessListItem: EthereumTransactionEip2930AccessListItem;
    EthereumTransactionEip2930Eip2930Transaction: EthereumTransactionEip2930Eip2930Transaction;
    EthereumTransactionEip2930MalleableTransactionSignature: EthereumTransactionEip2930MalleableTransactionSignature;
    EthereumTransactionEip2930TransactionSignature: EthereumTransactionEip2930TransactionSignature;
    EthereumTransactionEip7702AuthorizationListItem: EthereumTransactionEip7702AuthorizationListItem;
    EthereumTransactionEip7702Eip7702Transaction: EthereumTransactionEip7702Eip7702Transaction;
    EthereumTransactionLegacyLegacyTransaction: EthereumTransactionLegacyLegacyTransaction;
    EthereumTransactionLegacyTransactionAction: EthereumTransactionLegacyTransactionAction;
    EthereumTransactionLegacyTransactionSignature: EthereumTransactionLegacyTransactionSignature;
    EthereumTransactionTransactionV3: EthereumTransactionTransactionV3;
    EthereumTypesHashH64: EthereumTypesHashH64;
    EvmCoreErrorExitError: EvmCoreErrorExitError;
    EvmCoreErrorExitFatal: EvmCoreErrorExitFatal;
    EvmCoreErrorExitReason: EvmCoreErrorExitReason;
    EvmCoreErrorExitRevert: EvmCoreErrorExitRevert;
    EvmCoreErrorExitSucceed: EvmCoreErrorExitSucceed;
    FinalityGrandpaCommit: FinalityGrandpaCommit;
    FinalityGrandpaPrecommit: FinalityGrandpaPrecommit;
    FinalityGrandpaSignedPrecommit: FinalityGrandpaSignedPrecommit;
    FpRpcTransactionStatus: FpRpcTransactionStatus;
    FrameMetadataHashExtensionCheckMetadataHash: FrameMetadataHashExtensionCheckMetadataHash;
    FrameMetadataHashExtensionMode: FrameMetadataHashExtensionMode;
    FrameSupportDispatchDispatchClass: FrameSupportDispatchDispatchClass;
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
    FrameSupportStorageDisabled: FrameSupportStorageDisabled;
    FrameSupportTokensFungibleUnionOfNativeOrWithId: FrameSupportTokensFungibleUnionOfNativeOrWithId;
    FrameSupportTokensMiscBalanceStatus: FrameSupportTokensMiscBalanceStatus;
    FrameSupportTokensMiscIdAmount: FrameSupportTokensMiscIdAmount;
    FrameSystemAccountInfo: FrameSystemAccountInfo;
    FrameSystemCall: FrameSystemCall;
    FrameSystemCodeUpgradeAuthorization: FrameSystemCodeUpgradeAuthorization;
    FrameSystemDispatchEventInfo: FrameSystemDispatchEventInfo;
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
    MoonriverRuntimeBridgeRejectObsoleteHeadersAndMessages: MoonriverRuntimeBridgeRejectObsoleteHeadersAndMessages;
    MoonriverRuntimeGovernanceOriginsCustomOriginsOrigin: MoonriverRuntimeGovernanceOriginsCustomOriginsOrigin;
    MoonriverRuntimeOriginCaller: MoonriverRuntimeOriginCaller;
    MoonriverRuntimeProxyType: MoonriverRuntimeProxyType;
    MoonriverRuntimeRuntime: MoonriverRuntimeRuntime;
    MoonriverRuntimeRuntimeHoldReason: MoonriverRuntimeRuntimeHoldReason;
    MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessDeposit: MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessDeposit;
    MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParameters: MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParameters;
    MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersKey: MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersKey;
    MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersValue: MoonriverRuntimeRuntimeParamsDynamicParamsPalletRandomnessParametersValue;
    MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigFeesTreasuryProportion: MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigFeesTreasuryProportion;
    MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParameters: MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParameters;
    MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersKey: MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersKey;
    MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersValue: MoonriverRuntimeRuntimeParamsDynamicParamsRuntimeConfigParametersValue;
    MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigForeignAssetCreationDeposit: MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigForeignAssetCreationDeposit;
    MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParameters: MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParameters;
    MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParametersKey: MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParametersKey;
    MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParametersValue: MoonriverRuntimeRuntimeParamsDynamicParamsXcmConfigParametersValue;
    MoonriverRuntimeRuntimeParamsRuntimeParameters: MoonriverRuntimeRuntimeParamsRuntimeParameters;
    MoonriverRuntimeRuntimeParamsRuntimeParametersKey: MoonriverRuntimeRuntimeParamsRuntimeParametersKey;
    MoonriverRuntimeRuntimeParamsRuntimeParametersValue: MoonriverRuntimeRuntimeParamsRuntimeParametersValue;
    MoonriverRuntimeXcmConfigCurrencyId: MoonriverRuntimeXcmConfigCurrencyId;
    MoonriverRuntimeXcmConfigTransactors: MoonriverRuntimeXcmConfigTransactors;
    NimbusPrimitivesNimbusCryptoPublic: NimbusPrimitivesNimbusCryptoPublic;
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
    PalletBalancesReasons: PalletBalancesReasons;
    PalletBalancesReserveData: PalletBalancesReserveData;
    PalletBridgeGrandpaCall: PalletBridgeGrandpaCall;
    PalletBridgeGrandpaError: PalletBridgeGrandpaError;
    PalletBridgeGrandpaEvent: PalletBridgeGrandpaEvent;
    PalletBridgeGrandpaStorageTypesStoredAuthoritySet: PalletBridgeGrandpaStorageTypesStoredAuthoritySet;
    PalletBridgeMessagesCall: PalletBridgeMessagesCall;
    PalletBridgeMessagesError: PalletBridgeMessagesError;
    PalletBridgeMessagesEvent: PalletBridgeMessagesEvent;
    PalletBridgeMessagesLanesManagerLanesManagerError: PalletBridgeMessagesLanesManagerLanesManagerError;
    PalletBridgeMessagesOutboundLaneReceptionConfirmationError: PalletBridgeMessagesOutboundLaneReceptionConfirmationError;
    PalletBridgeParachainsCall: PalletBridgeParachainsCall;
    PalletBridgeParachainsError: PalletBridgeParachainsError;
    PalletBridgeParachainsEvent: PalletBridgeParachainsEvent;
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
    PalletEthereumXcmEvent: PalletEthereumXcmEvent;
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
    PalletIdentityProvider: PalletIdentityProvider;
    PalletIdentityRegistrarInfo: PalletIdentityRegistrarInfo;
    PalletIdentityRegistration: PalletIdentityRegistration;
    PalletIdentityUsernameInformation: PalletIdentityUsernameInformation;
    PalletMaintenanceModeCall: PalletMaintenanceModeCall;
    PalletMaintenanceModeError: PalletMaintenanceModeError;
    PalletMaintenanceModeEvent: PalletMaintenanceModeEvent;
    PalletMessageQueueBookState: PalletMessageQueueBookState;
    PalletMessageQueueCall: PalletMessageQueueCall;
    PalletMessageQueueError: PalletMessageQueueError;
    PalletMessageQueueEvent: PalletMessageQueueEvent;
    PalletMessageQueueNeighbours: PalletMessageQueueNeighbours;
    PalletMessageQueuePage: PalletMessageQueuePage;
    PalletMigrationsActiveCursor: PalletMigrationsActiveCursor;
    PalletMigrationsCall: PalletMigrationsCall;
    PalletMigrationsError: PalletMigrationsError;
    PalletMigrationsEvent: PalletMigrationsEvent;
    PalletMigrationsHistoricCleanupSelector: PalletMigrationsHistoricCleanupSelector;
    PalletMigrationsMigrationCursor: PalletMigrationsMigrationCursor;
    PalletMoonbeamForeignAssetsAssetDepositDetails: PalletMoonbeamForeignAssetsAssetDepositDetails;
    PalletMoonbeamForeignAssetsAssetStatus: PalletMoonbeamForeignAssetsAssetStatus;
    PalletMoonbeamForeignAssetsCall: PalletMoonbeamForeignAssetsCall;
    PalletMoonbeamForeignAssetsError: PalletMoonbeamForeignAssetsError;
    PalletMoonbeamForeignAssetsEvent: PalletMoonbeamForeignAssetsEvent;
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
    PalletParachainStakingInflationDistributionAccount: PalletParachainStakingInflationDistributionAccount;
    PalletParachainStakingInflationDistributionConfig: PalletParachainStakingInflationDistributionConfig;
    PalletParachainStakingInflationInflationInfo: PalletParachainStakingInflationInflationInfo;
    PalletParachainStakingRoundInfo: PalletParachainStakingRoundInfo;
    PalletParachainStakingSetBoundedOrderedSet: PalletParachainStakingSetBoundedOrderedSet;
    PalletParachainStakingSetOrderedSet: PalletParachainStakingSetOrderedSet;
    PalletParametersCall: PalletParametersCall;
    PalletParametersEvent: PalletParametersEvent;
    PalletPreimageCall: PalletPreimageCall;
    PalletPreimageError: PalletPreimageError;
    PalletPreimageEvent: PalletPreimageEvent;
    PalletPreimageHoldReason: PalletPreimageHoldReason;
    PalletPreimageOldRequestStatus: PalletPreimageOldRequestStatus;
    PalletPreimageRequestStatus: PalletPreimageRequestStatus;
    PalletProxyAnnouncement: PalletProxyAnnouncement;
    PalletProxyCall: PalletProxyCall;
    PalletProxyDepositKind: PalletProxyDepositKind;
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
    PalletReferendaTrackDetails: PalletReferendaTrackDetails;
    PalletRootTestingCall: PalletRootTestingCall;
    PalletRootTestingEvent: PalletRootTestingEvent;
    PalletSchedulerCall: PalletSchedulerCall;
    PalletSchedulerError: PalletSchedulerError;
    PalletSchedulerEvent: PalletSchedulerEvent;
    PalletSchedulerRetryConfig: PalletSchedulerRetryConfig;
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
    PalletXcmAuthorizedAliasesEntry: PalletXcmAuthorizedAliasesEntry;
    PalletXcmBridgeHubCall: PalletXcmBridgeHubCall;
    PalletXcmBridgeHubDispatcherXcmBlobMessageDispatchResult: PalletXcmBridgeHubDispatcherXcmBlobMessageDispatchResult;
    PalletXcmBridgeHubError: PalletXcmBridgeHubError;
    PalletXcmBridgeHubEvent: PalletXcmBridgeHubEvent;
    PalletXcmBridgeHubHoldReason: PalletXcmBridgeHubHoldReason;
    PalletXcmCall: PalletXcmCall;
    PalletXcmError: PalletXcmError;
    PalletXcmEvent: PalletXcmEvent;
    PalletXcmMaxAuthorizedAliases: PalletXcmMaxAuthorizedAliases;
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
    PalletXcmWeightTraderCall: PalletXcmWeightTraderCall;
    PalletXcmWeightTraderError: PalletXcmWeightTraderError;
    PalletXcmWeightTraderEvent: PalletXcmWeightTraderEvent;
    PolkadotCorePrimitivesInboundDownwardMessage: PolkadotCorePrimitivesInboundDownwardMessage;
    PolkadotCorePrimitivesInboundHrmpMessage: PolkadotCorePrimitivesInboundHrmpMessage;
    PolkadotCorePrimitivesOutboundHrmpMessage: PolkadotCorePrimitivesOutboundHrmpMessage;
    PolkadotParachainPrimitivesPrimitivesHrmpChannelId: PolkadotParachainPrimitivesPrimitivesHrmpChannelId;
    PolkadotPrimitivesV8AbridgedHostConfiguration: PolkadotPrimitivesV8AbridgedHostConfiguration;
    PolkadotPrimitivesV8AbridgedHrmpChannel: PolkadotPrimitivesV8AbridgedHrmpChannel;
    PolkadotPrimitivesV8AsyncBackingAsyncBackingParams: PolkadotPrimitivesV8AsyncBackingAsyncBackingParams;
    PolkadotPrimitivesV8PersistedValidationData: PolkadotPrimitivesV8PersistedValidationData;
    PolkadotPrimitivesV8UpgradeGoAhead: PolkadotPrimitivesV8UpgradeGoAhead;
    PolkadotPrimitivesV8UpgradeRestriction: PolkadotPrimitivesV8UpgradeRestriction;
    SessionKeysPrimitivesVrfVrfCryptoPublic: SessionKeysPrimitivesVrfVrfCryptoPublic;
    SpArithmeticArithmeticError: SpArithmeticArithmeticError;
    SpConsensusGrandpaAppPublic: SpConsensusGrandpaAppPublic;
    SpConsensusGrandpaAppSignature: SpConsensusGrandpaAppSignature;
    SpRuntimeBlakeTwo256: SpRuntimeBlakeTwo256;
    SpRuntimeDigest: SpRuntimeDigest;
    SpRuntimeDigestDigestItem: SpRuntimeDigestDigestItem;
    SpRuntimeDispatchError: SpRuntimeDispatchError;
    SpRuntimeDispatchErrorWithPostInfo: SpRuntimeDispatchErrorWithPostInfo;
    SpRuntimeHeader: SpRuntimeHeader;
    SpRuntimeModuleError: SpRuntimeModuleError;
    SpRuntimeMultiSignature: SpRuntimeMultiSignature;
    SpRuntimeProvingTrieTrieError: SpRuntimeProvingTrieTrieError;
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
    StagingXcmV4Xcm: StagingXcmV4Xcm;
    StagingXcmV5Asset: StagingXcmV5Asset;
    StagingXcmV5AssetAssetFilter: StagingXcmV5AssetAssetFilter;
    StagingXcmV5AssetAssetId: StagingXcmV5AssetAssetId;
    StagingXcmV5AssetAssetInstance: StagingXcmV5AssetAssetInstance;
    StagingXcmV5AssetAssetTransferFilter: StagingXcmV5AssetAssetTransferFilter;
    StagingXcmV5AssetAssets: StagingXcmV5AssetAssets;
    StagingXcmV5AssetFungibility: StagingXcmV5AssetFungibility;
    StagingXcmV5AssetWildAsset: StagingXcmV5AssetWildAsset;
    StagingXcmV5AssetWildFungibility: StagingXcmV5AssetWildFungibility;
    StagingXcmV5Hint: StagingXcmV5Hint;
    StagingXcmV5Instruction: StagingXcmV5Instruction;
    StagingXcmV5Junction: StagingXcmV5Junction;
    StagingXcmV5JunctionNetworkId: StagingXcmV5JunctionNetworkId;
    StagingXcmV5Junctions: StagingXcmV5Junctions;
    StagingXcmV5Location: StagingXcmV5Location;
    StagingXcmV5PalletInfo: StagingXcmV5PalletInfo;
    StagingXcmV5QueryResponseInfo: StagingXcmV5QueryResponseInfo;
    StagingXcmV5Response: StagingXcmV5Response;
    StagingXcmV5TraitsOutcome: StagingXcmV5TraitsOutcome;
    StagingXcmV5Xcm: StagingXcmV5Xcm;
    XcmDoubleEncoded: XcmDoubleEncoded;
    XcmPrimitivesEthereumXcmEthereumXcmFee: XcmPrimitivesEthereumXcmEthereumXcmFee;
    XcmPrimitivesEthereumXcmEthereumXcmTransaction: XcmPrimitivesEthereumXcmEthereumXcmTransaction;
    XcmPrimitivesEthereumXcmEthereumXcmTransactionV1: XcmPrimitivesEthereumXcmEthereumXcmTransactionV1;
    XcmPrimitivesEthereumXcmEthereumXcmTransactionV2: XcmPrimitivesEthereumXcmEthereumXcmTransactionV2;
    XcmPrimitivesEthereumXcmEthereumXcmTransactionV3: XcmPrimitivesEthereumXcmEthereumXcmTransactionV3;
    XcmPrimitivesEthereumXcmManualEthereumXcmFee: XcmPrimitivesEthereumXcmManualEthereumXcmFee;
    XcmRuntimeApisAuthorizedAliasesOriginAliaser: XcmRuntimeApisAuthorizedAliasesOriginAliaser;
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
    XcmV3OriginKind: XcmV3OriginKind;
    XcmV3PalletInfo: XcmV3PalletInfo;
    XcmV3QueryResponseInfo: XcmV3QueryResponseInfo;
    XcmV3Response: XcmV3Response;
    XcmV3TraitsError: XcmV3TraitsError;
    XcmV3TraitsSendError: XcmV3TraitsSendError;
    XcmV3WeightLimit: XcmV3WeightLimit;
    XcmV3Xcm: XcmV3Xcm;
    XcmV5TraitsError: XcmV5TraitsError;
    XcmVersionedAssetId: XcmVersionedAssetId;
    XcmVersionedAssets: XcmVersionedAssets;
    XcmVersionedInteriorLocation: XcmVersionedInteriorLocation;
    XcmVersionedLocation: XcmVersionedLocation;
    XcmVersionedResponse: XcmVersionedResponse;
    XcmVersionedXcm: XcmVersionedXcm;
  } // InterfaceTypes
} // declare module
