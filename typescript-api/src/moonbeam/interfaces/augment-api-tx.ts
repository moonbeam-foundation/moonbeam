// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/submittable";

import type {
  ApiTypes,
  AugmentedSubmittable,
  SubmittableExtrinsic,
  SubmittableExtrinsicFunction,
} from "@polkadot/api-base/types";
import type { Data } from "@polkadot/types";
import type {
  Bytes,
  Compact,
  Null,
  Option,
  Struct,
  U256,
  U8aFixed,
  Vec,
  bool,
  u128,
  u16,
  u32,
  u64,
  u8,
} from "@polkadot/types-codec";
import type { AnyNumber, IMethod, ITuple } from "@polkadot/types-codec/types";
import type {
  AccountId20,
  Call,
  H160,
  H256,
  Perbill,
  Percent,
} from "@polkadot/types/interfaces/runtime";
import type {
  CumulusPrimitivesParachainInherentParachainInherentData,
  EthereumTransactionTransactionV2,
  FrameSupportPreimagesBounded,
  FrameSupportScheduleDispatchTime,
  MoonbeamRuntimeAssetConfigAssetRegistrarMetadata,
  MoonbeamRuntimeOriginCaller,
  MoonbeamRuntimeProxyType,
  MoonbeamRuntimeXcmConfigAssetType,
  MoonbeamRuntimeXcmConfigCurrencyId,
  MoonbeamRuntimeXcmConfigTransactors,
  NimbusPrimitivesNimbusCryptoPublic,
  PalletConvictionVotingConviction,
  PalletConvictionVotingVoteAccountVote,
  PalletDemocracyConviction,
  PalletDemocracyMetadataOwner,
  PalletDemocracyVoteAccountVote,
  PalletIdentityBitFlags,
  PalletIdentityJudgement,
  PalletIdentitySimpleIdentityInfo,
  PalletMultisigTimepoint,
  PalletXcmTransactorCurrencyPayment,
  PalletXcmTransactorHrmpOperation,
  PalletXcmTransactorTransactWeights,
  SpRuntimeMultiSignature,
  SpWeightsWeightV2Weight,
  StagingXcmV3MultiLocation,
  XcmPrimitivesEthereumXcmEthereumXcmTransaction,
  XcmV2OriginKind,
  XcmV3WeightLimit,
  XcmVersionedMultiAsset,
  XcmVersionedMultiAssets,
  XcmVersionedMultiLocation,
  XcmVersionedXcm,
} from "@polkadot/types/lookup";

export type __AugmentedSubmittable = AugmentedSubmittable<() => unknown>;
export type __SubmittableExtrinsic<ApiType extends ApiTypes> = SubmittableExtrinsic<ApiType>;
export type __SubmittableExtrinsicFunction<ApiType extends ApiTypes> =
  SubmittableExtrinsicFunction<ApiType>;

declare module "@polkadot/api-base/types/submittable" {
  interface AugmentedSubmittables<ApiType extends ApiTypes> {
    assetManager: {
      /** See [`Pallet::change_existing_asset_type`]. */
      changeExistingAssetType: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          newAssetType: MoonbeamRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, MoonbeamRuntimeXcmConfigAssetType, u32]
      >;
      /** See [`Pallet::destroy_foreign_asset`]. */
      destroyForeignAsset: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, u32]
      >;
      /** See [`Pallet::destroy_local_asset`]. */
      destroyLocalAsset: AugmentedSubmittable<
        (assetId: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u128]
      >;
      /** See [`Pallet::register_foreign_asset`]. */
      registerForeignAsset: AugmentedSubmittable<
        (
          asset: MoonbeamRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          metadata:
            | MoonbeamRuntimeAssetConfigAssetRegistrarMetadata
            | { name?: any; symbol?: any; decimals?: any; isFrozen?: any }
            | string
            | Uint8Array,
          minAmount: u128 | AnyNumber | Uint8Array,
          isSufficient: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          MoonbeamRuntimeXcmConfigAssetType,
          MoonbeamRuntimeAssetConfigAssetRegistrarMetadata,
          u128,
          bool
        ]
      >;
      /** See [`Pallet::register_local_asset`]. */
      registerLocalAsset: AugmentedSubmittable<
        (
          creator: AccountId20 | string | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          isSufficient: bool | boolean | Uint8Array,
          minBalance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20, bool, u128]
      >;
      /** See [`Pallet::remove_existing_asset_type`]. */
      removeExistingAssetType: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, u32]
      >;
      /** See [`Pallet::remove_supported_asset`]. */
      removeSupportedAsset: AugmentedSubmittable<
        (
          assetType: MoonbeamRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeXcmConfigAssetType, u32]
      >;
      /** See [`Pallet::set_asset_units_per_second`]. */
      setAssetUnitsPerSecond: AugmentedSubmittable<
        (
          assetType: MoonbeamRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          unitsPerSecond: u128 | AnyNumber | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeXcmConfigAssetType, u128, u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    assets: {
      /** See [`Pallet::approve_transfer`]. */
      approveTransfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          delegate: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::block`]. */
      block: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** See [`Pallet::burn`]. */
      burn: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::cancel_approval`]. */
      cancelApproval: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          delegate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** See [`Pallet::clear_metadata`]. */
      clearMetadata: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::create`]. */
      create: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          admin: AccountId20 | string | Uint8Array,
          minBalance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, u128]
      >;
      /** See [`Pallet::destroy_accounts`]. */
      destroyAccounts: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::destroy_approvals`]. */
      destroyApprovals: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::finish_destroy`]. */
      finishDestroy: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::force_asset_status`]. */
      forceAssetStatus: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          issuer: AccountId20 | string | Uint8Array,
          admin: AccountId20 | string | Uint8Array,
          freezer: AccountId20 | string | Uint8Array,
          minBalance: Compact<u128> | AnyNumber | Uint8Array,
          isSufficient: bool | boolean | Uint8Array,
          isFrozen: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          Compact<u128>,
          AccountId20,
          AccountId20,
          AccountId20,
          AccountId20,
          Compact<u128>,
          bool,
          bool
        ]
      >;
      /** See [`Pallet::force_cancel_approval`]. */
      forceCancelApproval: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          delegate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20]
      >;
      /** See [`Pallet::force_clear_metadata`]. */
      forceClearMetadata: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::force_create`]. */
      forceCreate: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          isSufficient: bool | boolean | Uint8Array,
          minBalance: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, bool, Compact<u128>]
      >;
      /** See [`Pallet::force_set_metadata`]. */
      forceSetMetadata: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          name: Bytes | string | Uint8Array,
          symbol: Bytes | string | Uint8Array,
          decimals: u8 | AnyNumber | Uint8Array,
          isFrozen: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, Bytes, Bytes, u8, bool]
      >;
      /** See [`Pallet::force_transfer`]. */
      forceTransfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          source: AccountId20 | string | Uint8Array,
          dest: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::freeze`]. */
      freeze: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** See [`Pallet::freeze_asset`]. */
      freezeAsset: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::mint`]. */
      mint: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::refund`]. */
      refund: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          allowBurn: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, bool]
      >;
      /** See [`Pallet::refund_other`]. */
      refundOther: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** See [`Pallet::set_metadata`]. */
      setMetadata: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          name: Bytes | string | Uint8Array,
          symbol: Bytes | string | Uint8Array,
          decimals: u8 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, Bytes, Bytes, u8]
      >;
      /** See [`Pallet::set_min_balance`]. */
      setMinBalance: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          minBalance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, u128]
      >;
      /** See [`Pallet::set_team`]. */
      setTeam: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          issuer: AccountId20 | string | Uint8Array,
          admin: AccountId20 | string | Uint8Array,
          freezer: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, AccountId20]
      >;
      /** See [`Pallet::start_destroy`]. */
      startDestroy: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::thaw`]. */
      thaw: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** See [`Pallet::thaw_asset`]. */
      thawAsset: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::touch`]. */
      touch: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /** See [`Pallet::touch_other`]. */
      touchOther: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** See [`Pallet::transfer`]. */
      transfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::transfer_approved`]. */
      transferApproved: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          destination: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::transfer_keep_alive`]. */
      transferKeepAlive: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::transfer_ownership`]. */
      transferOwnership: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorFilter: {
      /** See [`Pallet::set_eligible`]. */
      setEligible: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorInherent: {
      /** See [`Pallet::kick_off_authorship_validation`]. */
      kickOffAuthorshipValidation: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorMapping: {
      /** See [`Pallet::add_association`]. */
      addAssociation: AugmentedSubmittable<
        (
          nimbusId: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [NimbusPrimitivesNimbusCryptoPublic]
      >;
      /** See [`Pallet::clear_association`]. */
      clearAssociation: AugmentedSubmittable<
        (
          nimbusId: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [NimbusPrimitivesNimbusCryptoPublic]
      >;
      /** See [`Pallet::remove_keys`]. */
      removeKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::set_keys`]. */
      setKeys: AugmentedSubmittable<
        (keys: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** See [`Pallet::update_association`]. */
      updateAssociation: AugmentedSubmittable<
        (
          oldNimbusId: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array,
          newNimbusId: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [NimbusPrimitivesNimbusCryptoPublic, NimbusPrimitivesNimbusCryptoPublic]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    balances: {
      /** See [`Pallet::force_set_balance`]. */
      forceSetBalance: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          newFree: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::force_transfer`]. */
      forceTransfer: AugmentedSubmittable<
        (
          source: AccountId20 | string | Uint8Array,
          dest: AccountId20 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::force_unreserve`]. */
      forceUnreserve: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128]
      >;
      /** See [`Pallet::transfer_all`]. */
      transferAll: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          keepAlive: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, bool]
      >;
      /** See [`Pallet::transfer_allow_death`]. */
      transferAllowDeath: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::transfer_keep_alive`]. */
      transferKeepAlive: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>]
      >;
      /** See [`Pallet::upgrade_accounts`]. */
      upgradeAccounts: AugmentedSubmittable<
        (
          who: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    convictionVoting: {
      /** See [`Pallet::delegate`]. */
      delegate: AugmentedSubmittable<
        (
          clazz: u16 | AnyNumber | Uint8Array,
          to: AccountId20 | string | Uint8Array,
          conviction:
            | PalletConvictionVotingConviction
            | "None"
            | "Locked1x"
            | "Locked2x"
            | "Locked3x"
            | "Locked4x"
            | "Locked5x"
            | "Locked6x"
            | number
            | Uint8Array,
          balance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, AccountId20, PalletConvictionVotingConviction, u128]
      >;
      /** See [`Pallet::remove_other_vote`]. */
      removeOtherVote: AugmentedSubmittable<
        (
          target: AccountId20 | string | Uint8Array,
          clazz: u16 | AnyNumber | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u16, u32]
      >;
      /** See [`Pallet::remove_vote`]. */
      removeVote: AugmentedSubmittable<
        (
          clazz: Option<u16> | null | Uint8Array | u16 | AnyNumber,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Option<u16>, u32]
      >;
      /** See [`Pallet::undelegate`]. */
      undelegate: AugmentedSubmittable<
        (clazz: u16 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u16]
      >;
      /** See [`Pallet::unlock`]. */
      unlock: AugmentedSubmittable<
        (
          clazz: u16 | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, AccountId20]
      >;
      /** See [`Pallet::vote`]. */
      vote: AugmentedSubmittable<
        (
          pollIndex: Compact<u32> | AnyNumber | Uint8Array,
          vote:
            | PalletConvictionVotingVoteAccountVote
            | { Standard: any }
            | { Split: any }
            | { SplitAbstain: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, PalletConvictionVotingVoteAccountVote]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    crowdloanRewards: {
      /** See [`Pallet::associate_native_identity`]. */
      associateNativeIdentity: AugmentedSubmittable<
        (
          rewardAccount: AccountId20 | string | Uint8Array,
          relayAccount: U8aFixed | string | Uint8Array,
          proof:
            | SpRuntimeMultiSignature
            | { Ed25519: any }
            | { Sr25519: any }
            | { Ecdsa: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, U8aFixed, SpRuntimeMultiSignature]
      >;
      /** See [`Pallet::change_association_with_relay_keys`]. */
      changeAssociationWithRelayKeys: AugmentedSubmittable<
        (
          rewardAccount: AccountId20 | string | Uint8Array,
          previousAccount: AccountId20 | string | Uint8Array,
          proofs:
            | Vec<ITuple<[U8aFixed, SpRuntimeMultiSignature]>>
            | [
                U8aFixed | string | Uint8Array,
                (
                  | SpRuntimeMultiSignature
                  | { Ed25519: any }
                  | { Sr25519: any }
                  | { Ecdsa: any }
                  | string
                  | Uint8Array
                )
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20, Vec<ITuple<[U8aFixed, SpRuntimeMultiSignature]>>]
      >;
      /** See [`Pallet::claim`]. */
      claim: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::complete_initialization`]. */
      completeInitialization: AugmentedSubmittable<
        (leaseEndingBlock: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::initialize_reward_vec`]. */
      initializeRewardVec: AugmentedSubmittable<
        (
          rewards:
            | Vec<ITuple<[U8aFixed, Option<AccountId20>, u128]>>
            | [
                U8aFixed | string | Uint8Array,
                Option<AccountId20> | null | Uint8Array | AccountId20 | string,
                u128 | AnyNumber | Uint8Array
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[U8aFixed, Option<AccountId20>, u128]>>]
      >;
      /** See [`Pallet::update_reward_address`]. */
      updateRewardAddress: AugmentedSubmittable<
        (newRewardAccount: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    democracy: {
      /** See [`Pallet::blacklist`]. */
      blacklist: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          maybeRefIndex: Option<u32> | null | Uint8Array | u32 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Option<u32>]
      >;
      /** See [`Pallet::cancel_proposal`]. */
      cancelProposal: AugmentedSubmittable<
        (propIndex: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /** See [`Pallet::cancel_referendum`]. */
      cancelReferendum: AugmentedSubmittable<
        (refIndex: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /** See [`Pallet::clear_public_proposals`]. */
      clearPublicProposals: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::delegate`]. */
      delegate: AugmentedSubmittable<
        (
          to: AccountId20 | string | Uint8Array,
          conviction:
            | PalletDemocracyConviction
            | "None"
            | "Locked1x"
            | "Locked2x"
            | "Locked3x"
            | "Locked4x"
            | "Locked5x"
            | "Locked6x"
            | number
            | Uint8Array,
          balance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, PalletDemocracyConviction, u128]
      >;
      /** See [`Pallet::emergency_cancel`]. */
      emergencyCancel: AugmentedSubmittable<
        (refIndex: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::external_propose`]. */
      externalPropose: AugmentedSubmittable<
        (
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [FrameSupportPreimagesBounded]
      >;
      /** See [`Pallet::external_propose_default`]. */
      externalProposeDefault: AugmentedSubmittable<
        (
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [FrameSupportPreimagesBounded]
      >;
      /** See [`Pallet::external_propose_majority`]. */
      externalProposeMajority: AugmentedSubmittable<
        (
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [FrameSupportPreimagesBounded]
      >;
      /** See [`Pallet::fast_track`]. */
      fastTrack: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          votingPeriod: u32 | AnyNumber | Uint8Array,
          delay: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, u32, u32]
      >;
      /** See [`Pallet::propose`]. */
      propose: AugmentedSubmittable<
        (
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [FrameSupportPreimagesBounded, Compact<u128>]
      >;
      /** See [`Pallet::remove_other_vote`]. */
      removeOtherVote: AugmentedSubmittable<
        (
          target: AccountId20 | string | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u32]
      >;
      /** See [`Pallet::remove_vote`]. */
      removeVote: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::second`]. */
      second: AugmentedSubmittable<
        (proposal: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /** See [`Pallet::set_metadata`]. */
      setMetadata: AugmentedSubmittable<
        (
          owner:
            | PalletDemocracyMetadataOwner
            | { External: any }
            | { Proposal: any }
            | { Referendum: any }
            | string
            | Uint8Array,
          maybeHash: Option<H256> | null | Uint8Array | H256 | string
        ) => SubmittableExtrinsic<ApiType>,
        [PalletDemocracyMetadataOwner, Option<H256>]
      >;
      /** See [`Pallet::undelegate`]. */
      undelegate: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::unlock`]. */
      unlock: AugmentedSubmittable<
        (target: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::veto_external`]. */
      vetoExternal: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** See [`Pallet::vote`]. */
      vote: AugmentedSubmittable<
        (
          refIndex: Compact<u32> | AnyNumber | Uint8Array,
          vote:
            | PalletDemocracyVoteAccountVote
            | { Standard: any }
            | { Split: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, PalletDemocracyVoteAccountVote]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    dmpQueue: {
      /** See [`Pallet::service_overweight`]. */
      serviceOverweight: AugmentedSubmittable<
        (
          index: u64 | AnyNumber | Uint8Array,
          weightLimit:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u64, SpWeightsWeightV2Weight]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    ethereum: {
      /** See [`Pallet::transact`]. */
      transact: AugmentedSubmittable<
        (
          transaction:
            | EthereumTransactionTransactionV2
            | { Legacy: any }
            | { EIP2930: any }
            | { EIP1559: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [EthereumTransactionTransactionV2]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    ethereumXcm: {
      /** See `Pallet::resume_ethereum_xcm_execution`. */
      resumeEthereumXcmExecution: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See `Pallet::suspend_ethereum_xcm_execution`. */
      suspendEthereumXcmExecution: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See `Pallet::transact`. */
      transact: AugmentedSubmittable<
        (
          xcmTransaction:
            | XcmPrimitivesEthereumXcmEthereumXcmTransaction
            | { V1: any }
            | { V2: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmPrimitivesEthereumXcmEthereumXcmTransaction]
      >;
      /** See `Pallet::transact_through_proxy`. */
      transactThroughProxy: AugmentedSubmittable<
        (
          transactAs: H160 | string | Uint8Array,
          xcmTransaction:
            | XcmPrimitivesEthereumXcmEthereumXcmTransaction
            | { V1: any }
            | { V2: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H160, XcmPrimitivesEthereumXcmEthereumXcmTransaction]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    evm: {
      /** See [`Pallet::call`]. */
      call: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          target: H160 | string | Uint8Array,
          input: Bytes | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          nonce: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          accessList:
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [H160 | string | Uint8Array, Vec<H256> | (H256 | string | Uint8Array)[]][]
        ) => SubmittableExtrinsic<ApiType>,
        [
          H160,
          H160,
          Bytes,
          U256,
          u64,
          U256,
          Option<U256>,
          Option<U256>,
          Vec<ITuple<[H160, Vec<H256>]>>
        ]
      >;
      /** See [`Pallet::create`]. */
      create: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          init: Bytes | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          nonce: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          accessList:
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [H160 | string | Uint8Array, Vec<H256> | (H256 | string | Uint8Array)[]][]
        ) => SubmittableExtrinsic<ApiType>,
        [H160, Bytes, U256, u64, U256, Option<U256>, Option<U256>, Vec<ITuple<[H160, Vec<H256>]>>]
      >;
      /** See [`Pallet::create2`]. */
      create2: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          init: Bytes | string | Uint8Array,
          salt: H256 | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          nonce: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          accessList:
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [H160 | string | Uint8Array, Vec<H256> | (H256 | string | Uint8Array)[]][]
        ) => SubmittableExtrinsic<ApiType>,
        [
          H160,
          Bytes,
          H256,
          U256,
          u64,
          U256,
          Option<U256>,
          Option<U256>,
          Vec<ITuple<[H160, Vec<H256>]>>
        ]
      >;
      /** See [`Pallet::withdraw`]. */
      withdraw: AugmentedSubmittable<
        (
          address: H160 | string | Uint8Array,
          value: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H160, u128]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    identity: {
      /** See [`Pallet::add_registrar`]. */
      addRegistrar: AugmentedSubmittable<
        (account: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::add_sub`]. */
      addSub: AugmentedSubmittable<
        (
          sub: AccountId20 | string | Uint8Array,
          data:
            | Data
            | { None: any }
            | { Raw: any }
            | { BlakeTwo256: any }
            | { Sha256: any }
            | { Keccak256: any }
            | { ShaThree256: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Data]
      >;
      /** See [`Pallet::cancel_request`]. */
      cancelRequest: AugmentedSubmittable<
        (regIndex: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::clear_identity`]. */
      clearIdentity: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::kill_identity`]. */
      killIdentity: AugmentedSubmittable<
        (target: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::provide_judgement`]. */
      provideJudgement: AugmentedSubmittable<
        (
          regIndex: Compact<u32> | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array,
          judgement:
            | PalletIdentityJudgement
            | { Unknown: any }
            | { FeePaid: any }
            | { Reasonable: any }
            | { KnownGood: any }
            | { OutOfDate: any }
            | { LowQuality: any }
            | { Erroneous: any }
            | string
            | Uint8Array,
          identity: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, AccountId20, PalletIdentityJudgement, H256]
      >;
      /** See [`Pallet::quit_sub`]. */
      quitSub: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::remove_sub`]. */
      removeSub: AugmentedSubmittable<
        (sub: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::rename_sub`]. */
      renameSub: AugmentedSubmittable<
        (
          sub: AccountId20 | string | Uint8Array,
          data:
            | Data
            | { None: any }
            | { Raw: any }
            | { BlakeTwo256: any }
            | { Sha256: any }
            | { Keccak256: any }
            | { ShaThree256: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Data]
      >;
      /** See [`Pallet::request_judgement`]. */
      requestJudgement: AugmentedSubmittable<
        (
          regIndex: Compact<u32> | AnyNumber | Uint8Array,
          maxFee: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u128>]
      >;
      /** See [`Pallet::set_account_id`]. */
      setAccountId: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          updated: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, AccountId20]
      >;
      /** See [`Pallet::set_fee`]. */
      setFee: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          fee: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u128>]
      >;
      /** See [`Pallet::set_fields`]. */
      setFields: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          fields: PalletIdentityBitFlags
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, PalletIdentityBitFlags]
      >;
      /** See [`Pallet::set_identity`]. */
      setIdentity: AugmentedSubmittable<
        (
          info:
            | PalletIdentitySimpleIdentityInfo
            | {
                additional?: any;
                display?: any;
                legal?: any;
                web?: any;
                riot?: any;
                email?: any;
                pgpFingerprint?: any;
                image?: any;
                twitter?: any;
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletIdentitySimpleIdentityInfo]
      >;
      /** See [`Pallet::set_subs`]. */
      setSubs: AugmentedSubmittable<
        (
          subs:
            | Vec<ITuple<[AccountId20, Data]>>
            | [
                AccountId20 | string | Uint8Array,
                (
                  | Data
                  | { None: any }
                  | { Raw: any }
                  | { BlakeTwo256: any }
                  | { Sha256: any }
                  | { Keccak256: any }
                  | { ShaThree256: any }
                  | string
                  | Uint8Array
                )
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[AccountId20, Data]>>]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    maintenanceMode: {
      /** See [`Pallet::enter_maintenance_mode`]. */
      enterMaintenanceMode: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::resume_normal_operation`]. */
      resumeNormalOperation: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    moonbeamLazyMigrations: {
      /** See [`Pallet::clear_local_assets_storage`]. */
      clearLocalAssetsStorage: AugmentedSubmittable<
        (limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::clear_suicided_storage`]. */
      clearSuicidedStorage: AugmentedSubmittable<
        (
          addresses: Vec<H160> | (H160 | string | Uint8Array)[],
          limit: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<H160>, u32]
      >;
      /** See [`Pallet::unlock_democracy_funds`]. */
      unlockDemocracyFunds: AugmentedSubmittable<
        (limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    moonbeamOrbiters: {
      /** See [`Pallet::add_collator`]. */
      addCollator: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::collator_add_orbiter`]. */
      collatorAddOrbiter: AugmentedSubmittable<
        (orbiter: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::collator_remove_orbiter`]. */
      collatorRemoveOrbiter: AugmentedSubmittable<
        (orbiter: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::orbiter_leave_collator_pool`]. */
      orbiterLeaveCollatorPool: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::orbiter_register`]. */
      orbiterRegister: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::orbiter_unregister`]. */
      orbiterUnregister: AugmentedSubmittable<
        (collatorsPoolCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::remove_collator`]. */
      removeCollator: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    multisig: {
      /** See [`Pallet::approve_as_multi`]. */
      approveAsMulti: AugmentedSubmittable<
        (
          threshold: u16 | AnyNumber | Uint8Array,
          otherSignatories: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          maybeTimepoint:
            | Option<PalletMultisigTimepoint>
            | null
            | Uint8Array
            | PalletMultisigTimepoint
            | { height?: any; index?: any }
            | string,
          callHash: U8aFixed | string | Uint8Array,
          maxWeight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, Vec<AccountId20>, Option<PalletMultisigTimepoint>, U8aFixed, SpWeightsWeightV2Weight]
      >;
      /** See [`Pallet::as_multi`]. */
      asMulti: AugmentedSubmittable<
        (
          threshold: u16 | AnyNumber | Uint8Array,
          otherSignatories: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          maybeTimepoint:
            | Option<PalletMultisigTimepoint>
            | null
            | Uint8Array
            | PalletMultisigTimepoint
            | { height?: any; index?: any }
            | string,
          call: Call | IMethod | string | Uint8Array,
          maxWeight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, Vec<AccountId20>, Option<PalletMultisigTimepoint>, Call, SpWeightsWeightV2Weight]
      >;
      /** See [`Pallet::as_multi_threshold_1`]. */
      asMultiThreshold1: AugmentedSubmittable<
        (
          otherSignatories: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Call]
      >;
      /** See [`Pallet::cancel_as_multi`]. */
      cancelAsMulti: AugmentedSubmittable<
        (
          threshold: u16 | AnyNumber | Uint8Array,
          otherSignatories: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          timepoint: PalletMultisigTimepoint | { height?: any; index?: any } | string | Uint8Array,
          callHash: U8aFixed | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, Vec<AccountId20>, PalletMultisigTimepoint, U8aFixed]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    openTechCommitteeCollective: {
      /** See [`Pallet::close`]. */
      close: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          proposalWeightBound:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, SpWeightsWeightV2Weight, Compact<u32>]
      >;
      /** See [`Pallet::disapprove_proposal`]. */
      disapproveProposal: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** See [`Pallet::execute`]. */
      execute: AugmentedSubmittable<
        (
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, Compact<u32>]
      >;
      /** See [`Pallet::propose`]. */
      propose: AugmentedSubmittable<
        (
          threshold: Compact<u32> | AnyNumber | Uint8Array,
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Call, Compact<u32>]
      >;
      /** See [`Pallet::set_members`]. */
      setMembers: AugmentedSubmittable<
        (
          newMembers: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          prime: Option<AccountId20> | null | Uint8Array | AccountId20 | string,
          oldCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Option<AccountId20>, u32]
      >;
      /** See [`Pallet::vote`]. */
      vote: AugmentedSubmittable<
        (
          proposal: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          approve: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, bool]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    parachainStaking: {
      /** See [`Pallet::cancel_candidate_bond_less`]. */
      cancelCandidateBondLess: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::cancel_delegation_request`]. */
      cancelDelegationRequest: AugmentedSubmittable<
        (candidate: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::cancel_leave_candidates`]. */
      cancelLeaveCandidates: AugmentedSubmittable<
        (candidateCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::candidate_bond_more`]. */
      candidateBondMore: AugmentedSubmittable<
        (more: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u128]
      >;
      /** See [`Pallet::delegate`]. */
      delegate: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          candidateDelegationCount: u32 | AnyNumber | Uint8Array,
          delegationCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128, u32, u32]
      >;
      /** See [`Pallet::delegate_with_auto_compound`]. */
      delegateWithAutoCompound: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          autoCompound: Percent | AnyNumber | Uint8Array,
          candidateDelegationCount: u32 | AnyNumber | Uint8Array,
          candidateAutoCompoundingDelegationCount: u32 | AnyNumber | Uint8Array,
          delegationCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128, Percent, u32, u32, u32]
      >;
      /** See [`Pallet::delegator_bond_more`]. */
      delegatorBondMore: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          more: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128]
      >;
      /** See [`Pallet::enable_marking_offline`]. */
      enableMarkingOffline: AugmentedSubmittable<
        (value: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [bool]
      >;
      /** See [`Pallet::execute_candidate_bond_less`]. */
      executeCandidateBondLess: AugmentedSubmittable<
        (candidate: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::execute_delegation_request`]. */
      executeDelegationRequest: AugmentedSubmittable<
        (
          delegator: AccountId20 | string | Uint8Array,
          candidate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20]
      >;
      /** See [`Pallet::execute_leave_candidates`]. */
      executeLeaveCandidates: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          candidateDelegationCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u32]
      >;
      /** See [`Pallet::force_join_candidates`]. */
      forceJoinCandidates: AugmentedSubmittable<
        (
          account: AccountId20 | string | Uint8Array,
          bond: u128 | AnyNumber | Uint8Array,
          candidateCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128, u32]
      >;
      /** See [`Pallet::go_offline`]. */
      goOffline: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::go_online`]. */
      goOnline: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::hotfix_remove_delegation_requests_exited_candidates`]. */
      hotfixRemoveDelegationRequestsExitedCandidates: AugmentedSubmittable<
        (
          candidates: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>]
      >;
      /** See [`Pallet::join_candidates`]. */
      joinCandidates: AugmentedSubmittable<
        (
          bond: u128 | AnyNumber | Uint8Array,
          candidateCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, u32]
      >;
      /** See [`Pallet::notify_inactive_collator`]. */
      notifyInactiveCollator: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::removed_call_19`]. */
      removedCall19: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::removed_call_20`]. */
      removedCall20: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::removed_call_21`]. */
      removedCall21: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::schedule_candidate_bond_less`]. */
      scheduleCandidateBondLess: AugmentedSubmittable<
        (less: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u128]
      >;
      /** See [`Pallet::schedule_delegator_bond_less`]. */
      scheduleDelegatorBondLess: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          less: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128]
      >;
      /** See [`Pallet::schedule_leave_candidates`]. */
      scheduleLeaveCandidates: AugmentedSubmittable<
        (candidateCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::schedule_revoke_delegation`]. */
      scheduleRevokeDelegation: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::set_auto_compound`]. */
      setAutoCompound: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          value: Percent | AnyNumber | Uint8Array,
          candidateAutoCompoundingDelegationCountHint: u32 | AnyNumber | Uint8Array,
          delegationCountHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Percent, u32, u32]
      >;
      /** See [`Pallet::set_blocks_per_round`]. */
      setBlocksPerRound: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::set_collator_commission`]. */
      setCollatorCommission: AugmentedSubmittable<
        (updated: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Perbill]
      >;
      /** See [`Pallet::set_inflation`]. */
      setInflation: AugmentedSubmittable<
        (
          schedule:
            | ({
                readonly min: Perbill;
                readonly ideal: Perbill;
                readonly max: Perbill;
              } & Struct)
            | { min?: any; ideal?: any; max?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          {
            readonly min: Perbill;
            readonly ideal: Perbill;
            readonly max: Perbill;
          } & Struct
        ]
      >;
      /** See [`Pallet::set_parachain_bond_account`]. */
      setParachainBondAccount: AugmentedSubmittable<
        (updated: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** See [`Pallet::set_parachain_bond_reserve_percent`]. */
      setParachainBondReservePercent: AugmentedSubmittable<
        (updated: Percent | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Percent]
      >;
      /** See [`Pallet::set_staking_expectations`]. */
      setStakingExpectations: AugmentedSubmittable<
        (
          expectations:
            | ({
                readonly min: u128;
                readonly ideal: u128;
                readonly max: u128;
              } & Struct)
            | { min?: any; ideal?: any; max?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          {
            readonly min: u128;
            readonly ideal: u128;
            readonly max: u128;
          } & Struct
        ]
      >;
      /** See [`Pallet::set_total_selected`]. */
      setTotalSelected: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    parachainSystem: {
      /** See [`Pallet::authorize_upgrade`]. */
      authorizeUpgrade: AugmentedSubmittable<
        (
          codeHash: H256 | string | Uint8Array,
          checkVersion: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, bool]
      >;
      /** See [`Pallet::enact_authorized_upgrade`]. */
      enactAuthorizedUpgrade: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** See [`Pallet::set_validation_data`]. */
      setValidationData: AugmentedSubmittable<
        (
          data:
            | CumulusPrimitivesParachainInherentParachainInherentData
            | {
                validationData?: any;
                relayChainState?: any;
                downwardMessages?: any;
                horizontalMessages?: any;
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [CumulusPrimitivesParachainInherentParachainInherentData]
      >;
      /** See [`Pallet::sudo_send_upward_message`]. */
      sudoSendUpwardMessage: AugmentedSubmittable<
        (message: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polkadotXcm: {
      /** See [`Pallet::execute`]. */
      execute: AugmentedSubmittable<
        (
          message: XcmVersionedXcm | { V2: any } | { V3: any } | string | Uint8Array,
          maxWeight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedXcm, SpWeightsWeightV2Weight]
      >;
      /** See [`Pallet::force_default_xcm_version`]. */
      forceDefaultXcmVersion: AugmentedSubmittable<
        (
          maybeXcmVersion: Option<u32> | null | Uint8Array | u32 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [Option<u32>]
      >;
      /** See [`Pallet::force_subscribe_version_notify`]. */
      forceSubscribeVersionNotify: AugmentedSubmittable<
        (
          location: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation]
      >;
      /** See [`Pallet::force_suspension`]. */
      forceSuspension: AugmentedSubmittable<
        (suspended: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [bool]
      >;
      /** See [`Pallet::force_unsubscribe_version_notify`]. */
      forceUnsubscribeVersionNotify: AugmentedSubmittable<
        (
          location: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation]
      >;
      /** See [`Pallet::force_xcm_version`]. */
      forceXcmVersion: AugmentedSubmittable<
        (
          location:
            | StagingXcmV3MultiLocation
            | { parents?: any; interior?: any }
            | string
            | Uint8Array,
          version: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [StagingXcmV3MultiLocation, u32]
      >;
      /** See [`Pallet::limited_reserve_transfer_assets`]. */
      limitedReserveTransferAssets: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          beneficiary: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          assets: XcmVersionedMultiAssets | { V2: any } | { V3: any } | string | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array,
          weightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedMultiLocation,
          XcmVersionedMultiLocation,
          XcmVersionedMultiAssets,
          u32,
          XcmV3WeightLimit
        ]
      >;
      /** See [`Pallet::limited_teleport_assets`]. */
      limitedTeleportAssets: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          beneficiary: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          assets: XcmVersionedMultiAssets | { V2: any } | { V3: any } | string | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array,
          weightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedMultiLocation,
          XcmVersionedMultiLocation,
          XcmVersionedMultiAssets,
          u32,
          XcmV3WeightLimit
        ]
      >;
      /** See [`Pallet::reserve_transfer_assets`]. */
      reserveTransferAssets: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          beneficiary: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          assets: XcmVersionedMultiAssets | { V2: any } | { V3: any } | string | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, XcmVersionedMultiLocation, XcmVersionedMultiAssets, u32]
      >;
      /** See [`Pallet::send`]. */
      send: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          message: XcmVersionedXcm | { V2: any } | { V3: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, XcmVersionedXcm]
      >;
      /** See [`Pallet::teleport_assets`]. */
      teleportAssets: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          beneficiary: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          assets: XcmVersionedMultiAssets | { V2: any } | { V3: any } | string | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, XcmVersionedMultiLocation, XcmVersionedMultiAssets, u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    preimage: {
      /** See [`Pallet::ensure_updated`]. */
      ensureUpdated: AugmentedSubmittable<
        (hashes: Vec<H256> | (H256 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>,
        [Vec<H256>]
      >;
      /** See [`Pallet::note_preimage`]. */
      notePreimage: AugmentedSubmittable<
        (bytes: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** See [`Pallet::request_preimage`]. */
      requestPreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** See [`Pallet::unnote_preimage`]. */
      unnotePreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** See [`Pallet::unrequest_preimage`]. */
      unrequestPreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    proxy: {
      /** See [`Pallet::add_proxy`]. */
      addProxy: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          proxyType:
            | MoonbeamRuntimeProxyType
            | "Any"
            | "NonTransfer"
            | "Governance"
            | "Staking"
            | "CancelProxy"
            | "Balances"
            | "AuthorMapping"
            | "IdentityJudgement"
            | number
            | Uint8Array,
          delay: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, MoonbeamRuntimeProxyType, u32]
      >;
      /** See [`Pallet::announce`]. */
      announce: AugmentedSubmittable<
        (
          real: AccountId20 | string | Uint8Array,
          callHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, H256]
      >;
      /** See [`Pallet::create_pure`]. */
      createPure: AugmentedSubmittable<
        (
          proxyType:
            | MoonbeamRuntimeProxyType
            | "Any"
            | "NonTransfer"
            | "Governance"
            | "Staking"
            | "CancelProxy"
            | "Balances"
            | "AuthorMapping"
            | "IdentityJudgement"
            | number
            | Uint8Array,
          delay: u32 | AnyNumber | Uint8Array,
          index: u16 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeProxyType, u32, u16]
      >;
      /** See [`Pallet::kill_pure`]. */
      killPure: AugmentedSubmittable<
        (
          spawner: AccountId20 | string | Uint8Array,
          proxyType:
            | MoonbeamRuntimeProxyType
            | "Any"
            | "NonTransfer"
            | "Governance"
            | "Staking"
            | "CancelProxy"
            | "Balances"
            | "AuthorMapping"
            | "IdentityJudgement"
            | number
            | Uint8Array,
          index: u16 | AnyNumber | Uint8Array,
          height: Compact<u32> | AnyNumber | Uint8Array,
          extIndex: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, MoonbeamRuntimeProxyType, u16, Compact<u32>, Compact<u32>]
      >;
      /** See [`Pallet::proxy`]. */
      proxy: AugmentedSubmittable<
        (
          real: AccountId20 | string | Uint8Array,
          forceProxyType:
            | Option<MoonbeamRuntimeProxyType>
            | null
            | Uint8Array
            | MoonbeamRuntimeProxyType
            | "Any"
            | "NonTransfer"
            | "Governance"
            | "Staking"
            | "CancelProxy"
            | "Balances"
            | "AuthorMapping"
            | "IdentityJudgement"
            | number,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Option<MoonbeamRuntimeProxyType>, Call]
      >;
      /** See [`Pallet::proxy_announced`]. */
      proxyAnnounced: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          real: AccountId20 | string | Uint8Array,
          forceProxyType:
            | Option<MoonbeamRuntimeProxyType>
            | null
            | Uint8Array
            | MoonbeamRuntimeProxyType
            | "Any"
            | "NonTransfer"
            | "Governance"
            | "Staking"
            | "CancelProxy"
            | "Balances"
            | "AuthorMapping"
            | "IdentityJudgement"
            | number,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20, Option<MoonbeamRuntimeProxyType>, Call]
      >;
      /** See [`Pallet::reject_announcement`]. */
      rejectAnnouncement: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          callHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, H256]
      >;
      /** See [`Pallet::remove_announcement`]. */
      removeAnnouncement: AugmentedSubmittable<
        (
          real: AccountId20 | string | Uint8Array,
          callHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, H256]
      >;
      /** See [`Pallet::remove_proxies`]. */
      removeProxies: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** See [`Pallet::remove_proxy`]. */
      removeProxy: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          proxyType:
            | MoonbeamRuntimeProxyType
            | "Any"
            | "NonTransfer"
            | "Governance"
            | "Staking"
            | "CancelProxy"
            | "Balances"
            | "AuthorMapping"
            | "IdentityJudgement"
            | number
            | Uint8Array,
          delay: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, MoonbeamRuntimeProxyType, u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    randomness: {
      /** See [`Pallet::set_babe_randomness_results`]. */
      setBabeRandomnessResults: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    referenda: {
      /** See [`Pallet::cancel`]. */
      cancel: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::kill`]. */
      kill: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::nudge_referendum`]. */
      nudgeReferendum: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::one_fewer_deciding`]. */
      oneFewerDeciding: AugmentedSubmittable<
        (track: u16 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u16]
      >;
      /** See [`Pallet::place_decision_deposit`]. */
      placeDecisionDeposit: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::refund_decision_deposit`]. */
      refundDecisionDeposit: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::refund_submission_deposit`]. */
      refundSubmissionDeposit: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::set_metadata`]. */
      setMetadata: AugmentedSubmittable<
        (
          index: u32 | AnyNumber | Uint8Array,
          maybeHash: Option<H256> | null | Uint8Array | H256 | string
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Option<H256>]
      >;
      /** See [`Pallet::submit`]. */
      submit: AugmentedSubmittable<
        (
          proposalOrigin:
            | MoonbeamRuntimeOriginCaller
            | { system: any }
            | { Void: any }
            | { Ethereum: any }
            | { Origins: any }
            | { TreasuryCouncilCollective: any }
            | { OpenTechCommitteeCollective: any }
            | { CumulusXcm: any }
            | { PolkadotXcm: any }
            | { EthereumXcm: any }
            | string
            | Uint8Array,
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array,
          enactmentMoment:
            | FrameSupportScheduleDispatchTime
            | { At: any }
            | { After: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          MoonbeamRuntimeOriginCaller,
          FrameSupportPreimagesBounded,
          FrameSupportScheduleDispatchTime
        ]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    rootTesting: {
      /** See `Pallet::fill_block`. */
      fillBlock: AugmentedSubmittable<
        (ratio: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Perbill]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    scheduler: {
      /** See [`Pallet::cancel`]. */
      cancel: AugmentedSubmittable<
        (
          when: u32 | AnyNumber | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, u32]
      >;
      /** See [`Pallet::cancel_named`]. */
      cancelNamed: AugmentedSubmittable<
        (id: U8aFixed | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [U8aFixed]
      >;
      /** See [`Pallet::schedule`]. */
      schedule: AugmentedSubmittable<
        (
          when: u32 | AnyNumber | Uint8Array,
          maybePeriodic:
            | Option<ITuple<[u32, u32]>>
            | null
            | Uint8Array
            | ITuple<[u32, u32]>
            | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          priority: u8 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Option<ITuple<[u32, u32]>>, u8, Call]
      >;
      /** See [`Pallet::schedule_after`]. */
      scheduleAfter: AugmentedSubmittable<
        (
          after: u32 | AnyNumber | Uint8Array,
          maybePeriodic:
            | Option<ITuple<[u32, u32]>>
            | null
            | Uint8Array
            | ITuple<[u32, u32]>
            | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          priority: u8 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Option<ITuple<[u32, u32]>>, u8, Call]
      >;
      /** See [`Pallet::schedule_named`]. */
      scheduleNamed: AugmentedSubmittable<
        (
          id: U8aFixed | string | Uint8Array,
          when: u32 | AnyNumber | Uint8Array,
          maybePeriodic:
            | Option<ITuple<[u32, u32]>>
            | null
            | Uint8Array
            | ITuple<[u32, u32]>
            | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          priority: u8 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [U8aFixed, u32, Option<ITuple<[u32, u32]>>, u8, Call]
      >;
      /** See [`Pallet::schedule_named_after`]. */
      scheduleNamedAfter: AugmentedSubmittable<
        (
          id: U8aFixed | string | Uint8Array,
          after: u32 | AnyNumber | Uint8Array,
          maybePeriodic:
            | Option<ITuple<[u32, u32]>>
            | null
            | Uint8Array
            | ITuple<[u32, u32]>
            | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          priority: u8 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [U8aFixed, u32, Option<ITuple<[u32, u32]>>, u8, Call]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    system: {
      /** See [`Pallet::kill_prefix`]. */
      killPrefix: AugmentedSubmittable<
        (
          prefix: Bytes | string | Uint8Array,
          subkeys: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, u32]
      >;
      /** See [`Pallet::kill_storage`]. */
      killStorage: AugmentedSubmittable<
        (keys: Vec<Bytes> | (Bytes | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>,
        [Vec<Bytes>]
      >;
      /** See [`Pallet::remark`]. */
      remark: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** See [`Pallet::remark_with_event`]. */
      remarkWithEvent: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** See [`Pallet::set_code`]. */
      setCode: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** See [`Pallet::set_code_without_checks`]. */
      setCodeWithoutChecks: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** See [`Pallet::set_heap_pages`]. */
      setHeapPages: AugmentedSubmittable<
        (pages: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u64]
      >;
      /** See [`Pallet::set_storage`]. */
      setStorage: AugmentedSubmittable<
        (
          items:
            | Vec<ITuple<[Bytes, Bytes]>>
            | [Bytes | string | Uint8Array, Bytes | string | Uint8Array][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[Bytes, Bytes]>>]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    timestamp: {
      /** See [`Pallet::set`]. */
      set: AugmentedSubmittable<
        (now: Compact<u64> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u64>]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    treasury: {
      /** See [`Pallet::approve_proposal`]. */
      approveProposal: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /** See [`Pallet::check_status`]. */
      checkStatus: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::payout`]. */
      payout: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** See [`Pallet::propose_spend`]. */
      proposeSpend: AugmentedSubmittable<
        (
          value: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** See [`Pallet::reject_proposal`]. */
      rejectProposal: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /** See [`Pallet::remove_approval`]. */
      removeApproval: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /** See [`Pallet::spend`]. */
      spend: AugmentedSubmittable<
        (
          assetKind: Null | null,
          amount: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array,
          validFrom: Option<u32> | null | Uint8Array | u32 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [Null, Compact<u128>, AccountId20, Option<u32>]
      >;
      /** See [`Pallet::spend_local`]. */
      spendLocal: AugmentedSubmittable<
        (
          amount: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /** See [`Pallet::void_spend`]. */
      voidSpend: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    treasuryCouncilCollective: {
      /** See [`Pallet::close`]. */
      close: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          proposalWeightBound:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, SpWeightsWeightV2Weight, Compact<u32>]
      >;
      /** See [`Pallet::disapprove_proposal`]. */
      disapproveProposal: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** See [`Pallet::execute`]. */
      execute: AugmentedSubmittable<
        (
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, Compact<u32>]
      >;
      /** See [`Pallet::propose`]. */
      propose: AugmentedSubmittable<
        (
          threshold: Compact<u32> | AnyNumber | Uint8Array,
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Call, Compact<u32>]
      >;
      /** See [`Pallet::set_members`]. */
      setMembers: AugmentedSubmittable<
        (
          newMembers: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          prime: Option<AccountId20> | null | Uint8Array | AccountId20 | string,
          oldCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Option<AccountId20>, u32]
      >;
      /** See [`Pallet::vote`]. */
      vote: AugmentedSubmittable<
        (
          proposal: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          approve: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, bool]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    utility: {
      /** See [`Pallet::as_derivative`]. */
      asDerivative: AugmentedSubmittable<
        (
          index: u16 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, Call]
      >;
      /** See [`Pallet::batch`]. */
      batch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /** See [`Pallet::batch_all`]. */
      batchAll: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /** See [`Pallet::dispatch_as`]. */
      dispatchAs: AugmentedSubmittable<
        (
          asOrigin:
            | MoonbeamRuntimeOriginCaller
            | { system: any }
            | { Void: any }
            | { Ethereum: any }
            | { Origins: any }
            | { TreasuryCouncilCollective: any }
            | { OpenTechCommitteeCollective: any }
            | { CumulusXcm: any }
            | { PolkadotXcm: any }
            | { EthereumXcm: any }
            | string
            | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeOriginCaller, Call]
      >;
      /** See [`Pallet::force_batch`]. */
      forceBatch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /** See [`Pallet::with_weight`]. */
      withWeight: AugmentedSubmittable<
        (
          call: Call | IMethod | string | Uint8Array,
          weight: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, SpWeightsWeightV2Weight]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    whitelist: {
      /** See [`Pallet::dispatch_whitelisted_call`]. */
      dispatchWhitelistedCall: AugmentedSubmittable<
        (
          callHash: H256 | string | Uint8Array,
          callEncodedLen: u32 | AnyNumber | Uint8Array,
          callWeightWitness:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, u32, SpWeightsWeightV2Weight]
      >;
      /** See [`Pallet::dispatch_whitelisted_call_with_preimage`]. */
      dispatchWhitelistedCallWithPreimage: AugmentedSubmittable<
        (call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Call]
      >;
      /** See [`Pallet::remove_whitelisted_call`]. */
      removeWhitelistedCall: AugmentedSubmittable<
        (callHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** See [`Pallet::whitelist_call`]. */
      whitelistCall: AugmentedSubmittable<
        (callHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    xcmTransactor: {
      /** See [`Pallet::deregister`]. */
      deregister: AugmentedSubmittable<
        (index: u16 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u16]
      >;
      /** See [`Pallet::hrmp_manage`]. */
      hrmpManage: AugmentedSubmittable<
        (
          action:
            | PalletXcmTransactorHrmpOperation
            | { InitOpen: any }
            | { Accept: any }
            | { Close: any }
            | { Cancel: any }
            | string
            | Uint8Array,
          fee:
            | PalletXcmTransactorCurrencyPayment
            | { currency?: any; feeAmount?: any }
            | string
            | Uint8Array,
          weightInfo:
            | PalletXcmTransactorTransactWeights
            | { transactRequiredWeightAtMost?: any; overallWeight?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          PalletXcmTransactorHrmpOperation,
          PalletXcmTransactorCurrencyPayment,
          PalletXcmTransactorTransactWeights
        ]
      >;
      /** See [`Pallet::register`]. */
      register: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          index: u16 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u16]
      >;
      /** See [`Pallet::remove_fee_per_second`]. */
      removeFeePerSecond: AugmentedSubmittable<
        (
          assetLocation: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation]
      >;
      /** See [`Pallet::remove_transact_info`]. */
      removeTransactInfo: AugmentedSubmittable<
        (
          location: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation]
      >;
      /** See [`Pallet::set_fee_per_second`]. */
      setFeePerSecond: AugmentedSubmittable<
        (
          assetLocation:
            | XcmVersionedMultiLocation
            | { V2: any }
            | { V3: any }
            | string
            | Uint8Array,
          feePerSecond: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, u128]
      >;
      /** See [`Pallet::set_transact_info`]. */
      setTransactInfo: AugmentedSubmittable<
        (
          location: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          transactExtraWeight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array,
          maxWeight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array,
          transactExtraWeightSigned:
            | Option<SpWeightsWeightV2Weight>
            | null
            | Uint8Array
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedMultiLocation,
          SpWeightsWeightV2Weight,
          SpWeightsWeightV2Weight,
          Option<SpWeightsWeightV2Weight>
        ]
      >;
      /** See [`Pallet::transact_through_derivative`]. */
      transactThroughDerivative: AugmentedSubmittable<
        (
          dest: MoonbeamRuntimeXcmConfigTransactors | "Relay" | number | Uint8Array,
          index: u16 | AnyNumber | Uint8Array,
          fee:
            | PalletXcmTransactorCurrencyPayment
            | { currency?: any; feeAmount?: any }
            | string
            | Uint8Array,
          innerCall: Bytes | string | Uint8Array,
          weightInfo:
            | PalletXcmTransactorTransactWeights
            | { transactRequiredWeightAtMost?: any; overallWeight?: any }
            | string
            | Uint8Array,
          refund: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          MoonbeamRuntimeXcmConfigTransactors,
          u16,
          PalletXcmTransactorCurrencyPayment,
          Bytes,
          PalletXcmTransactorTransactWeights,
          bool
        ]
      >;
      /** See [`Pallet::transact_through_signed`]. */
      transactThroughSigned: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          fee:
            | PalletXcmTransactorCurrencyPayment
            | { currency?: any; feeAmount?: any }
            | string
            | Uint8Array,
          call: Bytes | string | Uint8Array,
          weightInfo:
            | PalletXcmTransactorTransactWeights
            | { transactRequiredWeightAtMost?: any; overallWeight?: any }
            | string
            | Uint8Array,
          refund: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedMultiLocation,
          PalletXcmTransactorCurrencyPayment,
          Bytes,
          PalletXcmTransactorTransactWeights,
          bool
        ]
      >;
      /** See [`Pallet::transact_through_sovereign`]. */
      transactThroughSovereign: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          feePayer: AccountId20 | string | Uint8Array,
          fee:
            | PalletXcmTransactorCurrencyPayment
            | { currency?: any; feeAmount?: any }
            | string
            | Uint8Array,
          call: Bytes | string | Uint8Array,
          originKind:
            | XcmV2OriginKind
            | "Native"
            | "SovereignAccount"
            | "Superuser"
            | "Xcm"
            | number
            | Uint8Array,
          weightInfo:
            | PalletXcmTransactorTransactWeights
            | { transactRequiredWeightAtMost?: any; overallWeight?: any }
            | string
            | Uint8Array,
          refund: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedMultiLocation,
          AccountId20,
          PalletXcmTransactorCurrencyPayment,
          Bytes,
          XcmV2OriginKind,
          PalletXcmTransactorTransactWeights,
          bool
        ]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    xTokens: {
      /** See [`Pallet::transfer`]. */
      transfer: AugmentedSubmittable<
        (
          currencyId:
            | MoonbeamRuntimeXcmConfigCurrencyId
            | { SelfReserve: any }
            | { ForeignAsset: any }
            | { DeprecatedLocalAssetReserve: any }
            | { Erc20: any }
            | string
            | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeXcmConfigCurrencyId, u128, XcmVersionedMultiLocation, XcmV3WeightLimit]
      >;
      /** See [`Pallet::transfer_multiasset`]. */
      transferMultiasset: AugmentedSubmittable<
        (
          asset: XcmVersionedMultiAsset | { V2: any } | { V3: any } | string | Uint8Array,
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiAsset, XcmVersionedMultiLocation, XcmV3WeightLimit]
      >;
      /** See [`Pallet::transfer_multiassets`]. */
      transferMultiassets: AugmentedSubmittable<
        (
          assets: XcmVersionedMultiAssets | { V2: any } | { V3: any } | string | Uint8Array,
          feeItem: u32 | AnyNumber | Uint8Array,
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiAssets, u32, XcmVersionedMultiLocation, XcmV3WeightLimit]
      >;
      /** See [`Pallet::transfer_multiasset_with_fee`]. */
      transferMultiassetWithFee: AugmentedSubmittable<
        (
          asset: XcmVersionedMultiAsset | { V2: any } | { V3: any } | string | Uint8Array,
          fee: XcmVersionedMultiAsset | { V2: any } | { V3: any } | string | Uint8Array,
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedMultiAsset,
          XcmVersionedMultiAsset,
          XcmVersionedMultiLocation,
          XcmV3WeightLimit
        ]
      >;
      /** See [`Pallet::transfer_multicurrencies`]. */
      transferMulticurrencies: AugmentedSubmittable<
        (
          currencies:
            | Vec<ITuple<[MoonbeamRuntimeXcmConfigCurrencyId, u128]>>
            | [
                (
                  | MoonbeamRuntimeXcmConfigCurrencyId
                  | { SelfReserve: any }
                  | { ForeignAsset: any }
                  | { DeprecatedLocalAssetReserve: any }
                  | { Erc20: any }
                  | string
                  | Uint8Array
                ),
                u128 | AnyNumber | Uint8Array
              ][],
          feeItem: u32 | AnyNumber | Uint8Array,
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          Vec<ITuple<[MoonbeamRuntimeXcmConfigCurrencyId, u128]>>,
          u32,
          XcmVersionedMultiLocation,
          XcmV3WeightLimit
        ]
      >;
      /** See [`Pallet::transfer_with_fee`]. */
      transferWithFee: AugmentedSubmittable<
        (
          currencyId:
            | MoonbeamRuntimeXcmConfigCurrencyId
            | { SelfReserve: any }
            | { ForeignAsset: any }
            | { DeprecatedLocalAssetReserve: any }
            | { Erc20: any }
            | string
            | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          fee: u128 | AnyNumber | Uint8Array,
          dest: XcmVersionedMultiLocation | { V2: any } | { V3: any } | string | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          MoonbeamRuntimeXcmConfigCurrencyId,
          u128,
          u128,
          XcmVersionedMultiLocation,
          XcmV3WeightLimit
        ]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
  } // AugmentedSubmittables
} // declare module
