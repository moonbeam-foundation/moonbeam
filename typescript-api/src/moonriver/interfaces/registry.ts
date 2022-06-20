// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type {
  AccountEthereumSignature,
  CumulusPalletDmpQueueCall,
  CumulusPalletDmpQueueConfigData,
  CumulusPalletDmpQueueError,
  CumulusPalletDmpQueueEvent,
  CumulusPalletDmpQueuePageIndexData,
  CumulusPalletParachainSystemCall,
  CumulusPalletParachainSystemError,
  CumulusPalletParachainSystemEvent,
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot,
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
  FrameSupportDispatchRawOrigin,
  FrameSupportPalletId,
  FrameSupportScheduleLookupError,
  FrameSupportScheduleMaybeHashed,
  FrameSupportTokensMiscBalanceStatus,
  FrameSupportWeightsDispatchClass,
  FrameSupportWeightsDispatchInfo,
  FrameSupportWeightsPays,
  FrameSupportWeightsPerDispatchClassU32,
  FrameSupportWeightsPerDispatchClassU64,
  FrameSupportWeightsPerDispatchClassWeightsPerClass,
  FrameSupportWeightsRuntimeDbWeight,
  FrameSupportWeightsWeightToFeeCoefficient,
  FrameSystemAccountInfo,
  FrameSystemCall,
  FrameSystemError,
  FrameSystemEvent,
  FrameSystemEventRecord,
  FrameSystemExtensionsCheckGenesis,
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
  MoonriverRuntimeOriginCaller,
  MoonriverRuntimeProxyType,
  MoonriverRuntimeRuntime,
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
  PalletAssetsApproval,
  PalletAssetsAssetAccount,
  PalletAssetsAssetDetails,
  PalletAssetsAssetMetadata,
  PalletAssetsCall,
  PalletAssetsDestroyWitness,
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
  PalletBalancesReasons,
  PalletBalancesReleases,
  PalletBalancesReserveData,
  PalletBaseFeeCall,
  PalletBaseFeeEvent,
  PalletCollectiveCall,
  PalletCollectiveError,
  PalletCollectiveEvent,
  PalletCollectiveRawOrigin,
  PalletCollectiveVotes,
  PalletCrowdloanRewardsCall,
  PalletCrowdloanRewardsError,
  PalletCrowdloanRewardsEvent,
  PalletCrowdloanRewardsRewardInfo,
  PalletDemocracyCall,
  PalletDemocracyConviction,
  PalletDemocracyDelegations,
  PalletDemocracyError,
  PalletDemocracyEvent,
  PalletDemocracyPreimageStatus,
  PalletDemocracyReferendumInfo,
  PalletDemocracyReferendumStatus,
  PalletDemocracyReleases,
  PalletDemocracyTally,
  PalletDemocracyVoteAccountVote,
  PalletDemocracyVotePriorLock,
  PalletDemocracyVoteThreshold,
  PalletDemocracyVoteVoting,
  PalletEthereumCall,
  PalletEthereumError,
  PalletEthereumEvent,
  PalletEthereumRawOrigin,
  PalletEvmCall,
  PalletEvmError,
  PalletEvmEvent,
  PalletIdentityBitFlags,
  PalletIdentityCall,
  PalletIdentityError,
  PalletIdentityEvent,
  PalletIdentityIdentityField,
  PalletIdentityIdentityInfo,
  PalletIdentityJudgement,
  PalletIdentityRegistrarInfo,
  PalletIdentityRegistration,
  PalletMaintenanceModeCall,
  PalletMaintenanceModeError,
  PalletMaintenanceModeEvent,
  PalletMigrationsEvent,
  PalletMoonbeamOrbitersCall,
  PalletMoonbeamOrbitersCollatorPoolInfo,
  PalletMoonbeamOrbitersCurrentOrbiter,
  PalletMoonbeamOrbitersError,
  PalletMoonbeamOrbitersEvent,
  PalletProxyAnnouncement,
  PalletProxyCall,
  PalletProxyError,
  PalletProxyEvent,
  PalletProxyProxyDefinition,
  PalletSchedulerCall,
  PalletSchedulerError,
  PalletSchedulerEvent,
  PalletSchedulerScheduledV3,
  PalletTimestampCall,
  PalletTransactionPaymentChargeTransactionPayment,
  PalletTransactionPaymentReleases,
  PalletTreasuryCall,
  PalletTreasuryError,
  PalletTreasuryEvent,
  PalletTreasuryProposal,
  PalletUtilityCall,
  PalletUtilityError,
  PalletUtilityEvent,
  PalletXcmCall,
  PalletXcmError,
  PalletXcmEvent,
  PalletXcmOrigin,
  PalletXcmQueryStatus,
  PalletXcmVersionMigrationStage,
  ParachainStakingBond,
  ParachainStakingCall,
  ParachainStakingCandidateBondLessRequest,
  ParachainStakingCandidateMetadata,
  ParachainStakingCapacityStatus,
  ParachainStakingCollator2,
  ParachainStakingCollatorCandidate,
  ParachainStakingCollatorSnapshot,
  ParachainStakingCollatorStatus,
  ParachainStakingDelayedPayout,
  ParachainStakingDelegationRequestsCancelledScheduledRequest,
  ParachainStakingDelegationRequestsDelegationAction,
  ParachainStakingDelegationRequestsScheduledRequest,
  ParachainStakingDelegations,
  ParachainStakingDelegator,
  ParachainStakingDelegatorAdded,
  ParachainStakingDelegatorStatus,
  ParachainStakingError,
  ParachainStakingEvent,
  ParachainStakingInflationInflationInfo,
  ParachainStakingNominator2,
  ParachainStakingParachainBondConfig,
  ParachainStakingRoundInfo,
  ParachainStakingSetOrderedSetAccountId20,
  ParachainStakingSetOrderedSetBond,
  PolkadotCorePrimitivesInboundDownwardMessage,
  PolkadotCorePrimitivesInboundHrmpMessage,
  PolkadotCorePrimitivesOutboundHrmpMessage,
  PolkadotParachainPrimitivesXcmpMessageFormat,
  PolkadotPrimitivesV1AbridgedHostConfiguration,
  PolkadotPrimitivesV1AbridgedHrmpChannel,
  PolkadotPrimitivesV1PersistedValidationData,
  PolkadotPrimitivesV1UpgradeRestriction,
  SpCoreEcdsaSignature,
  SpCoreEd25519Signature,
  SpCoreSr25519Public,
  SpCoreSr25519Signature,
  SpCoreVoid,
  SpRuntimeArithmeticError,
  SpRuntimeDigest,
  SpRuntimeDigestDigestItem,
  SpRuntimeDispatchError,
  SpRuntimeModuleError,
  SpRuntimeMultiSignature,
  SpRuntimeTokenError,
  SpTrieStorageProof,
  SpVersionRuntimeVersion,
  XcmDoubleEncoded,
  XcmTransactorCall,
  XcmTransactorError,
  XcmTransactorEvent,
  XcmTransactorRemoteTransactInfoWithMaxWeight,
  XcmV0Junction,
  XcmV0JunctionBodyId,
  XcmV0JunctionBodyPart,
  XcmV0JunctionNetworkId,
  XcmV0MultiAsset,
  XcmV0MultiLocation,
  XcmV0Order,
  XcmV0OriginKind,
  XcmV0Response,
  XcmV0Xcm,
  XcmV1Junction,
  XcmV1MultiAsset,
  XcmV1MultiLocation,
  XcmV1MultiassetAssetId,
  XcmV1MultiassetAssetInstance,
  XcmV1MultiassetFungibility,
  XcmV1MultiassetMultiAssetFilter,
  XcmV1MultiassetMultiAssets,
  XcmV1MultiassetWildFungibility,
  XcmV1MultiassetWildMultiAsset,
  XcmV1MultilocationJunctions,
  XcmV1Order,
  XcmV1Response,
  XcmV1Xcm,
  XcmV2Instruction,
  XcmV2Response,
  XcmV2TraitsError,
  XcmV2TraitsOutcome,
  XcmV2WeightLimit,
  XcmV2Xcm,
  XcmVersionedMultiAsset,
  XcmVersionedMultiAssets,
  XcmVersionedMultiLocation,
  XcmVersionedResponse,
  XcmVersionedXcm,
} from "@polkadot/types/lookup";

declare module "@polkadot/types/types/registry" {
  export interface InterfaceTypes {
    AccountEthereumSignature: AccountEthereumSignature;
    CumulusPalletDmpQueueCall: CumulusPalletDmpQueueCall;
    CumulusPalletDmpQueueConfigData: CumulusPalletDmpQueueConfigData;
    CumulusPalletDmpQueueError: CumulusPalletDmpQueueError;
    CumulusPalletDmpQueueEvent: CumulusPalletDmpQueueEvent;
    CumulusPalletDmpQueuePageIndexData: CumulusPalletDmpQueuePageIndexData;
    CumulusPalletParachainSystemCall: CumulusPalletParachainSystemCall;
    CumulusPalletParachainSystemError: CumulusPalletParachainSystemError;
    CumulusPalletParachainSystemEvent: CumulusPalletParachainSystemEvent;
    CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot;
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
    FrameSupportDispatchRawOrigin: FrameSupportDispatchRawOrigin;
    FrameSupportPalletId: FrameSupportPalletId;
    FrameSupportScheduleLookupError: FrameSupportScheduleLookupError;
    FrameSupportScheduleMaybeHashed: FrameSupportScheduleMaybeHashed;
    FrameSupportTokensMiscBalanceStatus: FrameSupportTokensMiscBalanceStatus;
    FrameSupportWeightsDispatchClass: FrameSupportWeightsDispatchClass;
    FrameSupportWeightsDispatchInfo: FrameSupportWeightsDispatchInfo;
    FrameSupportWeightsPays: FrameSupportWeightsPays;
    FrameSupportWeightsPerDispatchClassU32: FrameSupportWeightsPerDispatchClassU32;
    FrameSupportWeightsPerDispatchClassU64: FrameSupportWeightsPerDispatchClassU64;
    FrameSupportWeightsPerDispatchClassWeightsPerClass: FrameSupportWeightsPerDispatchClassWeightsPerClass;
    FrameSupportWeightsRuntimeDbWeight: FrameSupportWeightsRuntimeDbWeight;
    FrameSupportWeightsWeightToFeeCoefficient: FrameSupportWeightsWeightToFeeCoefficient;
    FrameSystemAccountInfo: FrameSystemAccountInfo;
    FrameSystemCall: FrameSystemCall;
    FrameSystemError: FrameSystemError;
    FrameSystemEvent: FrameSystemEvent;
    FrameSystemEventRecord: FrameSystemEventRecord;
    FrameSystemExtensionsCheckGenesis: FrameSystemExtensionsCheckGenesis;
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
    MoonriverRuntimeOriginCaller: MoonriverRuntimeOriginCaller;
    MoonriverRuntimeProxyType: MoonriverRuntimeProxyType;
    MoonriverRuntimeRuntime: MoonriverRuntimeRuntime;
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
    PalletAssetsApproval: PalletAssetsApproval;
    PalletAssetsAssetAccount: PalletAssetsAssetAccount;
    PalletAssetsAssetDetails: PalletAssetsAssetDetails;
    PalletAssetsAssetMetadata: PalletAssetsAssetMetadata;
    PalletAssetsCall: PalletAssetsCall;
    PalletAssetsDestroyWitness: PalletAssetsDestroyWitness;
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
    PalletBalancesReasons: PalletBalancesReasons;
    PalletBalancesReleases: PalletBalancesReleases;
    PalletBalancesReserveData: PalletBalancesReserveData;
    PalletBaseFeeCall: PalletBaseFeeCall;
    PalletBaseFeeEvent: PalletBaseFeeEvent;
    PalletCollectiveCall: PalletCollectiveCall;
    PalletCollectiveError: PalletCollectiveError;
    PalletCollectiveEvent: PalletCollectiveEvent;
    PalletCollectiveRawOrigin: PalletCollectiveRawOrigin;
    PalletCollectiveVotes: PalletCollectiveVotes;
    PalletCrowdloanRewardsCall: PalletCrowdloanRewardsCall;
    PalletCrowdloanRewardsError: PalletCrowdloanRewardsError;
    PalletCrowdloanRewardsEvent: PalletCrowdloanRewardsEvent;
    PalletCrowdloanRewardsRewardInfo: PalletCrowdloanRewardsRewardInfo;
    PalletDemocracyCall: PalletDemocracyCall;
    PalletDemocracyConviction: PalletDemocracyConviction;
    PalletDemocracyDelegations: PalletDemocracyDelegations;
    PalletDemocracyError: PalletDemocracyError;
    PalletDemocracyEvent: PalletDemocracyEvent;
    PalletDemocracyPreimageStatus: PalletDemocracyPreimageStatus;
    PalletDemocracyReferendumInfo: PalletDemocracyReferendumInfo;
    PalletDemocracyReferendumStatus: PalletDemocracyReferendumStatus;
    PalletDemocracyReleases: PalletDemocracyReleases;
    PalletDemocracyTally: PalletDemocracyTally;
    PalletDemocracyVoteAccountVote: PalletDemocracyVoteAccountVote;
    PalletDemocracyVotePriorLock: PalletDemocracyVotePriorLock;
    PalletDemocracyVoteThreshold: PalletDemocracyVoteThreshold;
    PalletDemocracyVoteVoting: PalletDemocracyVoteVoting;
    PalletEthereumCall: PalletEthereumCall;
    PalletEthereumError: PalletEthereumError;
    PalletEthereumEvent: PalletEthereumEvent;
    PalletEthereumRawOrigin: PalletEthereumRawOrigin;
    PalletEvmCall: PalletEvmCall;
    PalletEvmError: PalletEvmError;
    PalletEvmEvent: PalletEvmEvent;
    PalletIdentityBitFlags: PalletIdentityBitFlags;
    PalletIdentityCall: PalletIdentityCall;
    PalletIdentityError: PalletIdentityError;
    PalletIdentityEvent: PalletIdentityEvent;
    PalletIdentityIdentityField: PalletIdentityIdentityField;
    PalletIdentityIdentityInfo: PalletIdentityIdentityInfo;
    PalletIdentityJudgement: PalletIdentityJudgement;
    PalletIdentityRegistrarInfo: PalletIdentityRegistrarInfo;
    PalletIdentityRegistration: PalletIdentityRegistration;
    PalletMaintenanceModeCall: PalletMaintenanceModeCall;
    PalletMaintenanceModeError: PalletMaintenanceModeError;
    PalletMaintenanceModeEvent: PalletMaintenanceModeEvent;
    PalletMigrationsEvent: PalletMigrationsEvent;
    PalletMoonbeamOrbitersCall: PalletMoonbeamOrbitersCall;
    PalletMoonbeamOrbitersCollatorPoolInfo: PalletMoonbeamOrbitersCollatorPoolInfo;
    PalletMoonbeamOrbitersCurrentOrbiter: PalletMoonbeamOrbitersCurrentOrbiter;
    PalletMoonbeamOrbitersError: PalletMoonbeamOrbitersError;
    PalletMoonbeamOrbitersEvent: PalletMoonbeamOrbitersEvent;
    PalletProxyAnnouncement: PalletProxyAnnouncement;
    PalletProxyCall: PalletProxyCall;
    PalletProxyError: PalletProxyError;
    PalletProxyEvent: PalletProxyEvent;
    PalletProxyProxyDefinition: PalletProxyProxyDefinition;
    PalletSchedulerCall: PalletSchedulerCall;
    PalletSchedulerError: PalletSchedulerError;
    PalletSchedulerEvent: PalletSchedulerEvent;
    PalletSchedulerScheduledV3: PalletSchedulerScheduledV3;
    PalletTimestampCall: PalletTimestampCall;
    PalletTransactionPaymentChargeTransactionPayment: PalletTransactionPaymentChargeTransactionPayment;
    PalletTransactionPaymentReleases: PalletTransactionPaymentReleases;
    PalletTreasuryCall: PalletTreasuryCall;
    PalletTreasuryError: PalletTreasuryError;
    PalletTreasuryEvent: PalletTreasuryEvent;
    PalletTreasuryProposal: PalletTreasuryProposal;
    PalletUtilityCall: PalletUtilityCall;
    PalletUtilityError: PalletUtilityError;
    PalletUtilityEvent: PalletUtilityEvent;
    PalletXcmCall: PalletXcmCall;
    PalletXcmError: PalletXcmError;
    PalletXcmEvent: PalletXcmEvent;
    PalletXcmOrigin: PalletXcmOrigin;
    PalletXcmQueryStatus: PalletXcmQueryStatus;
    PalletXcmVersionMigrationStage: PalletXcmVersionMigrationStage;
    ParachainStakingBond: ParachainStakingBond;
    ParachainStakingCall: ParachainStakingCall;
    ParachainStakingCandidateBondLessRequest: ParachainStakingCandidateBondLessRequest;
    ParachainStakingCandidateMetadata: ParachainStakingCandidateMetadata;
    ParachainStakingCapacityStatus: ParachainStakingCapacityStatus;
    ParachainStakingCollator2: ParachainStakingCollator2;
    ParachainStakingCollatorCandidate: ParachainStakingCollatorCandidate;
    ParachainStakingCollatorSnapshot: ParachainStakingCollatorSnapshot;
    ParachainStakingCollatorStatus: ParachainStakingCollatorStatus;
    ParachainStakingDelayedPayout: ParachainStakingDelayedPayout;
    ParachainStakingDelegationRequestsCancelledScheduledRequest: ParachainStakingDelegationRequestsCancelledScheduledRequest;
    ParachainStakingDelegationRequestsDelegationAction: ParachainStakingDelegationRequestsDelegationAction;
    ParachainStakingDelegationRequestsScheduledRequest: ParachainStakingDelegationRequestsScheduledRequest;
    ParachainStakingDelegations: ParachainStakingDelegations;
    ParachainStakingDelegator: ParachainStakingDelegator;
    ParachainStakingDelegatorAdded: ParachainStakingDelegatorAdded;
    ParachainStakingDelegatorStatus: ParachainStakingDelegatorStatus;
    ParachainStakingError: ParachainStakingError;
    ParachainStakingEvent: ParachainStakingEvent;
    ParachainStakingInflationInflationInfo: ParachainStakingInflationInflationInfo;
    ParachainStakingNominator2: ParachainStakingNominator2;
    ParachainStakingParachainBondConfig: ParachainStakingParachainBondConfig;
    ParachainStakingRoundInfo: ParachainStakingRoundInfo;
    ParachainStakingSetOrderedSetAccountId20: ParachainStakingSetOrderedSetAccountId20;
    ParachainStakingSetOrderedSetBond: ParachainStakingSetOrderedSetBond;
    PolkadotCorePrimitivesInboundDownwardMessage: PolkadotCorePrimitivesInboundDownwardMessage;
    PolkadotCorePrimitivesInboundHrmpMessage: PolkadotCorePrimitivesInboundHrmpMessage;
    PolkadotCorePrimitivesOutboundHrmpMessage: PolkadotCorePrimitivesOutboundHrmpMessage;
    PolkadotParachainPrimitivesXcmpMessageFormat: PolkadotParachainPrimitivesXcmpMessageFormat;
    PolkadotPrimitivesV1AbridgedHostConfiguration: PolkadotPrimitivesV1AbridgedHostConfiguration;
    PolkadotPrimitivesV1AbridgedHrmpChannel: PolkadotPrimitivesV1AbridgedHrmpChannel;
    PolkadotPrimitivesV1PersistedValidationData: PolkadotPrimitivesV1PersistedValidationData;
    PolkadotPrimitivesV1UpgradeRestriction: PolkadotPrimitivesV1UpgradeRestriction;
    SpCoreEcdsaSignature: SpCoreEcdsaSignature;
    SpCoreEd25519Signature: SpCoreEd25519Signature;
    SpCoreSr25519Public: SpCoreSr25519Public;
    SpCoreSr25519Signature: SpCoreSr25519Signature;
    SpCoreVoid: SpCoreVoid;
    SpRuntimeArithmeticError: SpRuntimeArithmeticError;
    SpRuntimeDigest: SpRuntimeDigest;
    SpRuntimeDigestDigestItem: SpRuntimeDigestDigestItem;
    SpRuntimeDispatchError: SpRuntimeDispatchError;
    SpRuntimeModuleError: SpRuntimeModuleError;
    SpRuntimeMultiSignature: SpRuntimeMultiSignature;
    SpRuntimeTokenError: SpRuntimeTokenError;
    SpTrieStorageProof: SpTrieStorageProof;
    SpVersionRuntimeVersion: SpVersionRuntimeVersion;
    XcmDoubleEncoded: XcmDoubleEncoded;
    XcmTransactorCall: XcmTransactorCall;
    XcmTransactorError: XcmTransactorError;
    XcmTransactorEvent: XcmTransactorEvent;
    XcmTransactorRemoteTransactInfoWithMaxWeight: XcmTransactorRemoteTransactInfoWithMaxWeight;
    XcmV0Junction: XcmV0Junction;
    XcmV0JunctionBodyId: XcmV0JunctionBodyId;
    XcmV0JunctionBodyPart: XcmV0JunctionBodyPart;
    XcmV0JunctionNetworkId: XcmV0JunctionNetworkId;
    XcmV0MultiAsset: XcmV0MultiAsset;
    XcmV0MultiLocation: XcmV0MultiLocation;
    XcmV0Order: XcmV0Order;
    XcmV0OriginKind: XcmV0OriginKind;
    XcmV0Response: XcmV0Response;
    XcmV0Xcm: XcmV0Xcm;
    XcmV1Junction: XcmV1Junction;
    XcmV1MultiAsset: XcmV1MultiAsset;
    XcmV1MultiLocation: XcmV1MultiLocation;
    XcmV1MultiassetAssetId: XcmV1MultiassetAssetId;
    XcmV1MultiassetAssetInstance: XcmV1MultiassetAssetInstance;
    XcmV1MultiassetFungibility: XcmV1MultiassetFungibility;
    XcmV1MultiassetMultiAssetFilter: XcmV1MultiassetMultiAssetFilter;
    XcmV1MultiassetMultiAssets: XcmV1MultiassetMultiAssets;
    XcmV1MultiassetWildFungibility: XcmV1MultiassetWildFungibility;
    XcmV1MultiassetWildMultiAsset: XcmV1MultiassetWildMultiAsset;
    XcmV1MultilocationJunctions: XcmV1MultilocationJunctions;
    XcmV1Order: XcmV1Order;
    XcmV1Response: XcmV1Response;
    XcmV1Xcm: XcmV1Xcm;
    XcmV2Instruction: XcmV2Instruction;
    XcmV2Response: XcmV2Response;
    XcmV2TraitsError: XcmV2TraitsError;
    XcmV2TraitsOutcome: XcmV2TraitsOutcome;
    XcmV2WeightLimit: XcmV2WeightLimit;
    XcmV2Xcm: XcmV2Xcm;
    XcmVersionedMultiAsset: XcmVersionedMultiAsset;
    XcmVersionedMultiAssets: XcmVersionedMultiAssets;
    XcmVersionedMultiLocation: XcmVersionedMultiLocation;
    XcmVersionedResponse: XcmVersionedResponse;
    XcmVersionedXcm: XcmVersionedXcm;
  } // InterfaceTypes
} // declare module
