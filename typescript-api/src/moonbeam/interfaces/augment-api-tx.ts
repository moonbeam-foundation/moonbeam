// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from "@polkadot/api-base/types";
import type { Data } from "@polkadot/types";
import type {
  Bytes,
  Compact,
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
  Permill,
} from "@polkadot/types/interfaces/runtime";
import type {
  CumulusPrimitivesParachainInherentParachainInherentData,
  EthereumTransactionTransactionV2,
  FrameSupportScheduleMaybeHashed,
  MoonbeamRuntimeAssetConfigAssetRegistrarMetadata,
  MoonbeamRuntimeOriginCaller,
  MoonbeamRuntimeProxyType,
  MoonbeamRuntimeXcmConfigAssetType,
  MoonbeamRuntimeXcmConfigCurrencyId,
  MoonbeamRuntimeXcmConfigTransactors,
  NimbusPrimitivesNimbusCryptoPublic,
  PalletAssetsDestroyWitness,
  PalletDemocracyConviction,
  PalletDemocracyVoteAccountVote,
  PalletIdentityBitFlags,
  PalletIdentityIdentityInfo,
  PalletIdentityJudgement,
  PalletXcmTransactorCurrencyPayment,
  PalletXcmTransactorTransactWeights,
  SpRuntimeMultiSignature,
  XcmV0OriginKind,
  XcmV1MultiLocation,
  XcmV2WeightLimit,
  XcmVersionedMultiAsset,
  XcmVersionedMultiAssets,
  XcmVersionedMultiLocation,
  XcmVersionedXcm,
} from "@polkadot/types/lookup";

declare module "@polkadot/api-base/types/submittable" {
  export interface AugmentedSubmittables<ApiType extends ApiTypes> {
    assetManager: {
      /**
       * Change the xcm type mapping for a given assetId We also change this if
       * the previous units per second where pointing at the old assetType
       */
      changeExistingAssetType: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          newAssetType: MoonbeamRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, MoonbeamRuntimeXcmConfigAssetType, u32]
      >;
      /**
       * Destroy a given foreign assetId The weight in this case is the one
       * returned by the trait plus the db writes and reads from removing all
       * the associated data
       */
      destroyForeignAsset: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          destroyAssetWitness:
            | PalletAssetsDestroyWitness
            | { accounts?: any; sufficients?: any; approvals?: any }
            | string
            | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, PalletAssetsDestroyWitness, u32]
      >;
      /**
       * Destroy a given local assetId We do not store anything related to local
       * assets in this pallet other than the counter and the counter is not
       * used for destroying the asset, so no additional db reads/writes to be
       * counter here
       */
      destroyLocalAsset: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          destroyAssetWitness:
            | PalletAssetsDestroyWitness
            | { accounts?: any; sufficients?: any; approvals?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, PalletAssetsDestroyWitness]
      >;
      /**
       * Register new asset with the asset manager
       */
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
      /**
       * Register a new local asset No information is stored in this pallet
       * about the local asset The reason is that we dont need to hold a mapping
       * between the multilocation and the local asset, as this conversion is
       * deterministic Further, we dont allow xcm fee payment in local assets
       */
      registerLocalAsset: AugmentedSubmittable<
        (
          creator: AccountId20 | string | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          isSufficient: bool | boolean | Uint8Array,
          minBalance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20, bool, u128]
      >;
      /**
       * Remove a given assetId -> assetType association
       */
      removeExistingAssetType: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, u32]
      >;
      removeSupportedAsset: AugmentedSubmittable<
        (
          assetType: MoonbeamRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeXcmConfigAssetType, u32]
      >;
      /**
       * Change the amount of units we are charging per execution second for a
       * given ForeignAssetType
       */
      setAssetUnitsPerSecond: AugmentedSubmittable<
        (
          assetType: MoonbeamRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          unitsPerSecond: u128 | AnyNumber | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeXcmConfigAssetType, u128, u32]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    assets: {
      /**
       * Approve an amount of asset for transfer by a delegated third-party account.
       *
       * Origin must be Signed.
       *
       * Ensures that `ApprovalDeposit` worth of `Currency` is reserved from
       * signing account for the purpose of holding the approval. If some
       * non-zero amount of assets is already approved from signing account to
       * `delegate`, then it is topped up or unreserved to meet the right value.
       *
       * NOTE: The signing account does not need to own `amount` of assets at
       * the point of making this call.
       *
       * - `id`: The identifier of the asset.
       * - `delegate`: The account to delegate permission to transfer asset.
       * - `amount`: The amount of asset that may be transferred by `delegate`. If
       *   there is already an approval in place, then this acts additively.
       *
       * Emits `ApprovedTransfer` on success.
       *
       * Weight: `O(1)`
       */
      approveTransfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          delegate: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Reduce the balance of `who` by as much as possible up to `amount` assets of `id`.
       *
       * Origin must be Signed and the sender should be the Manager of the asset `id`.
       *
       * Bails with `NoAccount` if the `who` is already dead.
       *
       * - `id`: The identifier of the asset to have some amount burned.
       * - `who`: The account to be debited from.
       * - `amount`: The maximum amount by which `who`'s balance should be reduced.
       *
       * Emits `Burned` with the actual amount burned. If this takes the balance
       * to below the minimum for the asset, then the amount burned is increased
       * to take it to zero.
       *
       * Weight: `O(1)` Modes: Post-existence of `who`; Pre & post Zombie-status of `who`.
       */
      burn: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Cancel all of some asset approved for delegated transfer by a
       * third-party account.
       *
       * Origin must be Signed and there must be an approval in place between
       * signer and `delegate`.
       *
       * Unreserves any deposit previously reserved by `approve_transfer` for
       * the approval.
       *
       * - `id`: The identifier of the asset.
       * - `delegate`: The account delegated permission to transfer asset.
       *
       * Emits `ApprovalCancelled` on success.
       *
       * Weight: `O(1)`
       */
      cancelApproval: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          delegate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Clear the metadata for an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * Any deposit is freed for the asset owner.
       *
       * - `id`: The identifier of the asset to clear.
       *
       * Emits `MetadataCleared`.
       *
       * Weight: `O(1)`
       */
      clearMetadata: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Issue a new class of fungible assets from a public origin.
       *
       * This new asset class has no assets initially and its owner is the origin.
       *
       * The origin must be Signed and the sender must have sufficient funds free.
       *
       * Funds of sender are reserved by `AssetDeposit`.
       *
       * Parameters:
       *
       * - `id`: The identifier of the new asset. This must not be currently in
       *   use to identify an existing asset.
       * - `admin`: The admin of this class of assets. The admin is the initial
       *   address of each member of the asset class's admin team.
       * - `min_balance`: The minimum balance of this new asset that any single
       *   account must have. If an account's balance is reduced below this,
       *   then it collapses to zero.
       *
       * Emits `Created` event when successful.
       *
       * Weight: `O(1)`
       */
      create: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          admin: AccountId20 | string | Uint8Array,
          minBalance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, u128]
      >;
      /**
       * Destroy a class of fungible assets.
       *
       * The origin must conform to `ForceOrigin` or must be Signed and the
       * sender must be the owner of the asset `id`.
       *
       * - `id`: The identifier of the asset to be destroyed. This must identify
       *   an existing asset.
       *
       * Emits `Destroyed` event when successful.
       *
       * NOTE: It can be helpful to first freeze an asset before destroying it
       * so that you can provide accurate witness information and prevent users
       * from manipulating state in a way that can make it harder to destroy.
       *
       * Weight: `O(c + p + a)` where:
       *
       * - `c = (witness.accounts - witness.sufficients)`
       * - `s = witness.sufficients`
       * - `a = witness.approvals`
       */
      destroy: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          witness:
            | PalletAssetsDestroyWitness
            | { accounts?: any; sufficients?: any; approvals?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, PalletAssetsDestroyWitness]
      >;
      /**
       * Alter the attributes of a given asset.
       *
       * Origin must be `ForceOrigin`.
       *
       * - `id`: The identifier of the asset.
       * - `owner`: The new Owner of this asset.
       * - `issuer`: The new Issuer of this asset.
       * - `admin`: The new Admin of this asset.
       * - `freezer`: The new Freezer of this asset.
       * - `min_balance`: The minimum balance of this new asset that any single
       *   account must have. If an account's balance is reduced below this,
       *   then it collapses to zero.
       * - `is_sufficient`: Whether a non-zero balance of this asset is deposit of
       *   sufficient value to account for the state bloat associated with its
       *   balance storage. If set to `true`, then non-zero balances may be
       *   stored without a `consumer` reference (and thus an ED in the Balances
       *   pallet or whatever else is used to control user-account state growth).
       * - `is_frozen`: Whether this asset class is frozen except for
       *   permissioned/admin instructions.
       *
       * Emits `AssetStatusChanged` with the identity of the asset.
       *
       * Weight: `O(1)`
       */
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
      /**
       * Cancel all of some asset approved for delegated transfer by a
       * third-party account.
       *
       * Origin must be either ForceOrigin or Signed origin with the signer
       * being the Admin account of the asset `id`.
       *
       * Unreserves any deposit previously reserved by `approve_transfer` for
       * the approval.
       *
       * - `id`: The identifier of the asset.
       * - `delegate`: The account delegated permission to transfer asset.
       *
       * Emits `ApprovalCancelled` on success.
       *
       * Weight: `O(1)`
       */
      forceCancelApproval: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          delegate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20]
      >;
      /**
       * Clear the metadata for an asset.
       *
       * Origin must be ForceOrigin.
       *
       * Any deposit is returned.
       *
       * - `id`: The identifier of the asset to clear.
       *
       * Emits `MetadataCleared`.
       *
       * Weight: `O(1)`
       */
      forceClearMetadata: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Issue a new class of fungible assets from a privileged origin.
       *
       * This new asset class has no assets initially.
       *
       * The origin must conform to `ForceOrigin`.
       *
       * Unlike `create`, no funds are reserved.
       *
       * - `id`: The identifier of the new asset. This must not be currently in
       *   use to identify an existing asset.
       * - `owner`: The owner of this class of assets. The owner has full
       *   superuser permissions over this asset, but may later change and
       *   configure the permissions using `transfer_ownership` and `set_team`.
       * - `min_balance`: The minimum balance of this new asset that any single
       *   account must have. If an account's balance is reduced below this,
       *   then it collapses to zero.
       *
       * Emits `ForceCreated` event when successful.
       *
       * Weight: `O(1)`
       */
      forceCreate: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          isSufficient: bool | boolean | Uint8Array,
          minBalance: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, bool, Compact<u128>]
      >;
      /**
       * Force the metadata for an asset to some value.
       *
       * Origin must be ForceOrigin.
       *
       * Any deposit is left alone.
       *
       * - `id`: The identifier of the asset to update.
       * - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
       * - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
       * - `decimals`: The number of decimals this asset uses to represent one unit.
       *
       * Emits `MetadataSet`.
       *
       * Weight: `O(N + S)` where N and S are the length of the name and symbol
       * respectively.
       */
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
      /**
       * Move some assets from one account to another.
       *
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       *
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `source`: The account to be debited.
       * - `dest`: The account to be credited.
       * - `amount`: The amount by which the `source`'s balance of assets should
       *   be reduced and `dest`'s balance increased. The amount actually
       *   transferred may be slightly greater in the case that the transfer
       *   would otherwise take the `source` balance above zero but below the
       *   minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes
       * the source balance to below the minimum for the asset, then the amount
       * transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `dest`; Post-existence of
       * `source`; Account pre-existence of `dest`.
       */
      forceTransfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          source: AccountId20 | string | Uint8Array,
          dest: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, Compact<u128>]
      >;
      /**
       * Disallow further unprivileged transfers from an account.
       *
       * Origin must be Signed and the sender should be the Freezer of the asset `id`.
       *
       * - `id`: The identifier of the asset to be frozen.
       * - `who`: The account to be frozen.
       *
       * Emits `Frozen`.
       *
       * Weight: `O(1)`
       */
      freeze: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Disallow further unprivileged transfers for the asset class.
       *
       * Origin must be Signed and the sender should be the Freezer of the asset `id`.
       *
       * - `id`: The identifier of the asset to be frozen.
       *
       * Emits `Frozen`.
       *
       * Weight: `O(1)`
       */
      freezeAsset: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Mint assets of a particular class.
       *
       * The origin must be Signed and the sender must be the Issuer of the asset `id`.
       *
       * - `id`: The identifier of the asset to have some amount minted.
       * - `beneficiary`: The account to be credited with the minted assets.
       * - `amount`: The amount of the asset to be minted.
       *
       * Emits `Issued` event when successful.
       *
       * Weight: `O(1)` Modes: Pre-existing balance of `beneficiary`; Account
       * pre-existence of `beneficiary`.
       */
      mint: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Return the deposit (if any) of an asset account.
       *
       * The origin must be Signed.
       *
       * - `id`: The identifier of the asset for the account to be created.
       * - `allow_burn`: If `true` then assets may be destroyed in order to
       *   complete the refund.
       *
       * Emits `Refunded` event when successful.
       */
      refund: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          allowBurn: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, bool]
      >;
      /**
       * Set the metadata for an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * Funds of sender are reserved according to the formula:
       * `MetadataDepositBase + MetadataDepositPerByte * (name.len +
       * symbol.len)` taking into account any already reserved funds.
       *
       * - `id`: The identifier of the asset to update.
       * - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
       * - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
       * - `decimals`: The number of decimals this asset uses to represent one unit.
       *
       * Emits `MetadataSet`.
       *
       * Weight: `O(1)`
       */
      setMetadata: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          name: Bytes | string | Uint8Array,
          symbol: Bytes | string | Uint8Array,
          decimals: u8 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, Bytes, Bytes, u8]
      >;
      /**
       * Change the Issuer, Admin and Freezer of an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * - `id`: The identifier of the asset to be frozen.
       * - `issuer`: The new Issuer of this asset.
       * - `admin`: The new Admin of this asset.
       * - `freezer`: The new Freezer of this asset.
       *
       * Emits `TeamChanged`.
       *
       * Weight: `O(1)`
       */
      setTeam: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          issuer: AccountId20 | string | Uint8Array,
          admin: AccountId20 | string | Uint8Array,
          freezer: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, AccountId20]
      >;
      /**
       * Allow unprivileged transfers from an account again.
       *
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       *
       * - `id`: The identifier of the asset to be frozen.
       * - `who`: The account to be unfrozen.
       *
       * Emits `Thawed`.
       *
       * Weight: `O(1)`
       */
      thaw: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Allow unprivileged transfers for the asset again.
       *
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       *
       * - `id`: The identifier of the asset to be thawed.
       *
       * Emits `Thawed`.
       *
       * Weight: `O(1)`
       */
      thawAsset: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Create an asset account for non-provider assets.
       *
       * A deposit will be taken from the signer account.
       *
       * - `origin`: Must be Signed; the signer account must have sufficient funds
       *   for a deposit to be taken.
       * - `id`: The identifier of the asset for the account to be created.
       *
       * Emits `Touched` event when successful.
       */
      touch: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Move some assets from the sender account to another.
       *
       * Origin must be Signed.
       *
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `target`: The account to be credited.
       * - `amount`: The amount by which the sender's balance of assets should be
       *   reduced and `target`'s balance increased. The amount actually
       *   transferred may be slightly greater in the case that the transfer
       *   would otherwise take the sender balance above zero but below the
       *   minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes
       * the source balance to below the minimum for the asset, then the amount
       * transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `target`; Post-existence of
       * sender; Account pre-existence of `target`.
       */
      transfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Transfer some asset balance from a previously delegated account to some
       * third-party account.
       *
       * Origin must be Signed and there must be an approval in place by the
       * `owner` to the signer.
       *
       * If the entire amount approved for transfer is transferred, then any
       * deposit previously reserved by `approve_transfer` is unreserved.
       *
       * - `id`: The identifier of the asset.
       * - `owner`: The account which previously approved for a transfer of at
       *   least `amount` and from which the asset balance will be withdrawn.
       * - `destination`: The account to which the asset balance of `amount` will
       *   be transferred.
       * - `amount`: The amount of assets to transfer.
       *
       * Emits `TransferredApproved` on success.
       *
       * Weight: `O(1)`
       */
      transferApproved: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          destination: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, Compact<u128>]
      >;
      /**
       * Move some assets from the sender account to another, keeping the sender
       * account alive.
       *
       * Origin must be Signed.
       *
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `target`: The account to be credited.
       * - `amount`: The amount by which the sender's balance of assets should be
       *   reduced and `target`'s balance increased. The amount actually
       *   transferred may be slightly greater in the case that the transfer
       *   would otherwise take the sender balance above zero but below the
       *   minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes
       * the source balance to below the minimum for the asset, then the amount
       * transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `target`; Post-existence of
       * sender; Account pre-existence of `target`.
       */
      transferKeepAlive: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Change the Owner of an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * - `id`: The identifier of the asset.
       * - `owner`: The new Owner of this asset.
       *
       * Emits `OwnerChanged`.
       *
       * Weight: `O(1)`
       */
      transferOwnership: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorFilter: {
      /**
       * Update the eligible count. Intended to be called by governance.
       */
      setEligible: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorInherent: {
      /**
       * This inherent is a workaround to run code after the "real" inherents
       * have executed, but before transactions are executed.
       */
      kickOffAuthorshipValidation: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorMapping: {
      /**
       * Register your NimbusId onchain so blocks you author are associated with
       * your account.
       *
       * Users who have been (or will soon be) elected active collators in
       * staking, should submit this extrinsic to have their blocks accepted and
       * earn rewards.
       */
      addAssociation: AugmentedSubmittable<
        (
          nimbusId: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [NimbusPrimitivesNimbusCryptoPublic]
      >;
      /**
       * Clear your Mapping.
       *
       * This is useful when you are no longer an author and would like to
       * re-claim your security deposit.
       */
      clearAssociation: AugmentedSubmittable<
        (
          nimbusId: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [NimbusPrimitivesNimbusCryptoPublic]
      >;
      /**
       * Remove your Mapping.
       *
       * This is useful when you are no longer an author and would like to
       * re-claim your security deposit.
       */
      removeKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Set association and session keys at once.
       *
       * This is useful for key rotation to update Nimbus and VRF keys in one
       * call. No new security deposit is required. Will replace
       * `update_association` which is kept now for backwards compatibility reasons.
       */
      setKeys: AugmentedSubmittable<
        (keys: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Change your Mapping.
       *
       * This is useful for normal key rotation or for when switching from one
       * physical collator machine to another. No new security deposit is
       * required. This sets keys to new_nimbus_id.into() by default.
       */
      updateAssociation: AugmentedSubmittable<
        (
          oldNimbusId: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array,
          newNimbusId: NimbusPrimitivesNimbusCryptoPublic | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [NimbusPrimitivesNimbusCryptoPublic, NimbusPrimitivesNimbusCryptoPublic]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    balances: {
      /**
       * Exactly as `transfer`, except the origin must be root and the source
       * account may be specified.
       *
       * # <weight>
       *
       * - Same as transfer, but additional read and write because the source
       *   account is not assumed to be in the overlay.
       *
       * # </weight>
       */
      forceTransfer: AugmentedSubmittable<
        (
          source: AccountId20 | string | Uint8Array,
          dest: AccountId20 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20, Compact<u128>]
      >;
      /**
       * Unreserve some balance from a user by force.
       *
       * Can only be called by ROOT.
       */
      forceUnreserve: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128]
      >;
      /**
       * Set the balances of a given account.
       *
       * This will alter `FreeBalance` and `ReservedBalance` in storage. it will
       * also alter the total issuance of the system (`TotalIssuance`)
       * appropriately. If the new free or reserved balance is below the
       * existential deposit, it will reset the account nonce
       * (`frame_system::AccountNonce`).
       *
       * The dispatch origin for this call is `root`.
       */
      setBalance: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          newFree: Compact<u128> | AnyNumber | Uint8Array,
          newReserved: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>, Compact<u128>]
      >;
      /**
       * Transfer some liquid free balance to another account.
       *
       * `transfer` will set the `FreeBalance` of the sender and receiver. If
       * the sender's account is below the existential deposit as a result of
       * the transfer, the account will be reaped.
       *
       * The dispatch origin for this call must be `Signed` by the transactor.
       *
       * # <weight>
       *
       * - Dependent on arguments but not critical, given proper implementations
       *   for input config types. See related functions below.
       * - It contains a limited number of reads and writes internally and no
       *   complex computation.
       *
       * Related functions:
       *
       * - `ensure_can_withdraw` is always called internally but has a bounded complexity.
       * - Transferring balances to accounts that did not exist before will cause
       *   `T::OnNewAccount::on_new_account` to be called.
       * - Removing enough funds from an account will trigger
       *   `T::DustRemoval::on_unbalanced`.
       * - `transfer_keep_alive` works the same way as `transfer`, but has an
       *   additional check that the transfer will not kill the origin account.
       *
       * - Origin account is already in memory, so no DB operations for them.
       *
       * # </weight>
       */
      transfer: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>]
      >;
      /**
       * Transfer the entire transferable balance from the caller account.
       *
       * NOTE: This function only attempts to transfer _transferable_ balances.
       * This means that any locked, reserved, or existential deposits (when
       * `keep_alive` is `true`), will not be transferred by this function. To
       * ensure that this function results in a killed account, you might need
       * to prepare the account by removing any reference counters, storage
       * deposits, etc...
       *
       * The dispatch origin of this call must be Signed.
       *
       * - `dest`: The recipient of the transfer.
       * - `keep_alive`: A boolean to determine if the `transfer_all` operation
       *   should send all of the funds the account has, causing the sender
       *   account to be killed (false), or transfer everything except at least
       *   the existential deposit, which will guarantee to keep the sender
       *   account alive (true). # <weight>
       * - O(1). Just like transfer, but reading the user's transferable balance
       *   first. #</weight>
       */
      transferAll: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          keepAlive: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, bool]
      >;
      /**
       * Same as the [`transfer`][`transfer`] call, but with a check that the
       * transfer will not kill the origin account.
       *
       * 99% of the time you want [`transfer`][`transfer`] instead.
       *
       * [`transfer`]: struct.Pallet.html#method.transfer
       */
      transferKeepAlive: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    baseFee: {
      setBaseFeePerGas: AugmentedSubmittable<
        (fee: U256 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [U256]
      >;
      setElasticity: AugmentedSubmittable<
        (elasticity: Permill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Permill]
      >;
      setIsActive: AugmentedSubmittable<
        (isActive: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [bool]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    councilCollective: {
      /**
       * Close a vote that is either approved, disapproved or whose voting
       * period has ended.
       *
       * May be called by any signed account in order to finish voting and close
       * the proposal.
       *
       * If called before the end of the voting period it will only close the
       * vote if it is has enough votes to be approved or disapproved.
       *
       * If called after the end of the voting period abstentions are counted as
       * rejections unless there is a prime member set and the prime member cast
       * an approval.
       *
       * If the close operation completes successfully with disapproval, the
       * transaction fee will be waived. Otherwise execution of the approved
       * operation will be charged to the caller.
       *
       * - `proposal_weight_bound`: The maximum amount of weight consumed by
       *   executing the closed proposal.
       * - `length_bound`: The upper bound for the length of the proposal in
       *   storage. Checked via `storage::read` so it is `size_of::<u32>() == 4`
       *   larger than the pure length.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(B + M + P1 + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - `P1` is the complexity of `proposal` preimage.
       * - `P2` is proposal-count (code-bounded)
       * - DB:
       * - 2 storage reads (`Members`: codec `O(M)`, `Prime`: codec `O(1)`)
       * - 3 mutations (`Voting`: codec `O(M)`, `ProposalOf`: codec `O(B)`,
       *   `Proposals`: codec `O(P2)`)
       * - Any mutations done while executing `proposal` (`P1`)
       * - Up to 3 events
       *
       * # </weight>
       */
      close: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          proposalWeightBound: Compact<u64> | AnyNumber | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, Compact<u64>, Compact<u32>]
      >;
      /**
       * Disapprove a proposal, close, and remove it from the system, regardless
       * of its current state.
       *
       * Must be called by the Root origin.
       *
       * Parameters:
       *
       * - `proposal_hash`: The hash of the proposal that should be disapproved.
       *
       * # <weight>
       *
       * Complexity: O(P) where P is the number of max proposals DB Weight:
       *
       * - Reads: Proposals
       * - Writes: Voting, Proposals, ProposalOf
       *
       * # </weight>
       */
      disapproveProposal: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Dispatch a proposal from a member using the `Member` origin.
       *
       * Origin must be a member of the collective.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(M + P)` where `M` members-count (code-bounded) and `P` complexity of
       *   dispatching `proposal`
       * - DB: 1 read (codec `O(M)`) + DB access of `proposal`
       * - 1 event
       *
       * # </weight>
       */
      execute: AugmentedSubmittable<
        (
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, Compact<u32>]
      >;
      /**
       * Add a new proposal to either be voted on or executed directly.
       *
       * Requires the sender to be member.
       *
       * `threshold` determines whether `proposal` is executed directly
       * (`threshold < 2`) or put up for voting.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(B + M + P1)` or `O(B + M + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - Branching is influenced by `threshold` where:
       * - `P1` is proposal execution complexity (`threshold < 2`)
       * - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
       * - DB:
       * - 1 storage read `is_member` (codec `O(M)`)
       * - 1 storage read `ProposalOf::contains_key` (codec `O(1)`)
       * - DB accesses influenced by `threshold`:
       * - EITHER storage accesses done by `proposal` (`threshold < 2`)
       * - OR proposal insertion (`threshold <= 2`)
       * - 1 storage mutation `Proposals` (codec `O(P2)`)
       * - 1 storage mutation `ProposalCount` (codec `O(1)`)
       * - 1 storage write `ProposalOf` (codec `O(B)`)
       * - 1 storage write `Voting` (codec `O(M)`)
       * - 1 event
       *
       * # </weight>
       */
      propose: AugmentedSubmittable<
        (
          threshold: Compact<u32> | AnyNumber | Uint8Array,
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Call, Compact<u32>]
      >;
      /**
       * Set the collective's membership.
       *
       * - `new_members`: The new member list. Be nice to the chain and provide it sorted.
       * - `prime`: The prime member whose vote sets the default.
       * - `old_count`: The upper bound for the previous number of members in
       *   storage. Used for weight estimation.
       *
       * Requires root origin.
       *
       * NOTE: Does not enforce the expected `MaxMembers` limit on the amount of
       * members, but the weight estimations rely on it to estimate dispatchable weight.
       *
       * # WARNING:
       *
       * The `pallet-collective` can also be managed by logic outside of the
       * pallet through the implementation of the trait [`ChangeMembers`]. Any
       * call to `set_members` must be careful that the member set doesn't get
       * out of sync with other logic managing the member set.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(MP + N)` where:
       * - `M` old-members-count (code- and governance-bounded)
       * - `N` new-members-count (code- and governance-bounded)
       * - `P` proposals-count (code-bounded)
       * - DB:
       * - 1 storage mutation (codec `O(M)` read, `O(N)` write) for reading and
       *   writing the members
       * - 1 storage read (codec `O(P)`) for reading the proposals
       * - `P` storage mutations (codec `O(M)`) for updating the votes for each proposal
       * - 1 storage write (codec `O(1)`) for deleting the old `prime` and setting
       *   the new one
       *
       * # </weight>
       */
      setMembers: AugmentedSubmittable<
        (
          newMembers: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          prime: Option<AccountId20> | null | object | string | Uint8Array,
          oldCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Option<AccountId20>, u32]
      >;
      /**
       * Add an aye or nay vote for the sender to the given proposal.
       *
       * Requires the sender to be a member.
       *
       * Transaction fees will be waived if the member is voting on any
       * particular proposal for the first time and the call is successful.
       * Subsequent vote changes will charge a fee.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(M)` where `M` is members-count (code- and governance-bounded)
       * - DB:
       * - 1 storage read `Members` (codec `O(M)`)
       * - 1 storage mutation `Voting` (codec `O(M)`)
       * - 1 event
       *
       * # </weight>
       */
      vote: AugmentedSubmittable<
        (
          proposal: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          approve: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, bool]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    crowdloanRewards: {
      /**
       * Associate a native rewards_destination identity with a crowdloan contribution.
       *
       * The caller needs to provide the unassociated relay account and a proof
       * to succeed with the association The proof is nothing but a signature
       * over the reward_address using the relay keys
       */
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
      /**
       * Change reward account by submitting proofs from relay accounts
       *
       * The number of valid proofs needs to be bigger than
       * 'RewardAddressRelayVoteThreshold' The account to be changed needs to be
       * submitted as 'previous_account' Origin must be RewardAddressChangeOrigin
       */
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
      /**
       * Collect whatever portion of your reward are currently vested.
       */
      claim: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * This extrinsic completes the initialization if some checks are
       * fullfiled. These checks are: -The reward contribution money matches the
       * crowdloan pot -The end vesting block is higher than the init vesting
       * block -The initialization has not complete yet
       */
      completeInitialization: AugmentedSubmittable<
        (leaseEndingBlock: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Initialize the reward distribution storage. It shortcuts whenever an
       * error is found This does not enforce any checks other than making sure
       * we dont go over funds complete_initialization should perform any additional
       */
      initializeRewardVec: AugmentedSubmittable<
        (
          rewards:
            | Vec<ITuple<[U8aFixed, Option<AccountId20>, u128]>>
            | [
                U8aFixed | string | Uint8Array,
                Option<AccountId20> | null | object | string | Uint8Array,
                u128 | AnyNumber | Uint8Array
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[U8aFixed, Option<AccountId20>, u128]>>]
      >;
      /**
       * Update reward address, proving that the caller owns the current native key
       */
      updateRewardAddress: AugmentedSubmittable<
        (newRewardAccount: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    democracy: {
      /**
       * Permanently place a proposal into the blacklist. This prevents it from
       * ever being proposed again.
       *
       * If called on a queued public or external proposal, then this will
       * result in it being removed. If the `ref_index` supplied is an active
       * referendum with the proposal hash, then it will be cancelled.
       *
       * The dispatch origin of this call must be `BlacklistOrigin`.
       *
       * - `proposal_hash`: The proposal hash to blacklist permanently.
       * - `ref_index`: An ongoing referendum whose hash is `proposal_hash`, which
       *   will be cancelled.
       *
       * Weight: `O(p)` (though as this is an high-privilege dispatch, we assume
       * it has a reasonable value).
       */
      blacklist: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          maybeRefIndex: Option<u32> | null | object | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Option<u32>]
      >;
      /**
       * Remove a proposal.
       *
       * The dispatch origin of this call must be `CancelProposalOrigin`.
       *
       * - `prop_index`: The index of the proposal to cancel.
       *
       * Weight: `O(p)` where `p = PublicProps::<T>::decode_len()`
       */
      cancelProposal: AugmentedSubmittable<
        (propIndex: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /**
       * Cancel a proposal queued for enactment.
       *
       * The dispatch origin of this call must be _Root_.
       *
       * - `which`: The index of the referendum to cancel.
       *
       * Weight: `O(D)` where `D` is the items in the dispatch queue. Weighted
       * as `D = 10`.
       */
      cancelQueued: AugmentedSubmittable<
        (which: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Remove a referendum.
       *
       * The dispatch origin of this call must be _Root_.
       *
       * - `ref_index`: The index of the referendum to cancel.
       *
       * # Weight: `O(1)`.
       */
      cancelReferendum: AugmentedSubmittable<
        (refIndex: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /**
       * Clears all public proposals.
       *
       * The dispatch origin of this call must be _Root_.
       *
       * Weight: `O(1)`.
       */
      clearPublicProposals: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Delegate the voting power (with some given conviction) of the sending account.
       *
       * The balance delegated is locked for as long as it's delegated, and
       * thereafter for the time appropriate for the conviction's lock period.
       *
       * The dispatch origin of this call must be _Signed_, and the signing
       * account must either:
       *
       * - Be delegating already; or
       * - Have no voting activity (if there is, then it will need to be
       *   removed/consolidated through `reap_vote` or `unvote`).
       * - `to`: The account whose voting the `target` account's voting power will follow.
       * - `conviction`: The conviction that will be attached to the delegated
       *   votes. When the account is undelegated, the funds will be locked for
       *   the corresponding period.
       * - `balance`: The amount of the account's balance to be used in
       *   delegating. This must not be more than the account's current balance.
       *
       * Emits `Delegated`.
       *
       * Weight: `O(R)` where R is the number of referendums the voter
       * delegating to has voted on. Weight is charged as if maximum votes.
       */
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
      /**
       * Schedule an emergency cancellation of a referendum. Cannot happen twice
       * to the same referendum.
       *
       * The dispatch origin of this call must be `CancellationOrigin`.
       *
       * -`ref_index`: The index of the referendum to cancel.
       *
       * Weight: `O(1)`.
       */
      emergencyCancel: AugmentedSubmittable<
        (refIndex: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Enact a proposal from a referendum. For now we just make the weight be
       * the maximum.
       */
      enactProposal: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, u32]
      >;
      /**
       * Schedule a referendum to be tabled once it is legal to schedule an
       * external referendum.
       *
       * The dispatch origin of this call must be `ExternalOrigin`.
       *
       * - `proposal_hash`: The preimage hash of the proposal.
       *
       * Weight: `O(V)` with V number of vetoers in the blacklist of proposal.
       * Decoding vec of length V. Charged as maximum
       */
      externalPropose: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Schedule a negative-turnout-bias referendum to be tabled next once it
       * is legal to schedule an external referendum.
       *
       * The dispatch of this call must be `ExternalDefaultOrigin`.
       *
       * - `proposal_hash`: The preimage hash of the proposal.
       *
       * Unlike `external_propose`, blacklisting has no effect on this and it
       * may replace a pre-scheduled `external_propose` call.
       *
       * Weight: `O(1)`
       */
      externalProposeDefault: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Schedule a majority-carries referendum to be tabled next once it is
       * legal to schedule an external referendum.
       *
       * The dispatch of this call must be `ExternalMajorityOrigin`.
       *
       * - `proposal_hash`: The preimage hash of the proposal.
       *
       * Unlike `external_propose`, blacklisting has no effect on this and it
       * may replace a pre-scheduled `external_propose` call.
       *
       * Weight: `O(1)`
       */
      externalProposeMajority: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Schedule the currently externally-proposed majority-carries referendum
       * to be tabled immediately. If there is no externally-proposed referendum
       * currently, or if there is one but it is not a majority-carries
       * referendum then it fails.
       *
       * The dispatch of this call must be `FastTrackOrigin`.
       *
       * - `proposal_hash`: The hash of the current external proposal.
       * - `voting_period`: The period that is allowed for voting on this
       *   proposal. Must be always greater than zero. For `FastTrackOrigin`
       *   must be equal or greater than `FastTrackVotingPeriod`.
       * - `delay`: The number of block after voting has ended in approval and
       *   this should be enacted. This doesn't have a minimum amount.
       *
       * Emits `Started`.
       *
       * Weight: `O(1)`
       */
      fastTrack: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          votingPeriod: u32 | AnyNumber | Uint8Array,
          delay: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, u32, u32]
      >;
      /**
       * Register the preimage for an upcoming proposal. This requires the
       * proposal to be in the dispatch queue. No deposit is needed. When this
       * call is successful, i.e. the preimage has not been uploaded before and
       * matches some imminent proposal, no fee is paid.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `encoded_proposal`: The preimage of a proposal.
       *
       * Emits `PreimageNoted`.
       *
       * Weight: `O(E)` with E size of `encoded_proposal` (protected by a
       * required deposit).
       */
      noteImminentPreimage: AugmentedSubmittable<
        (encodedProposal: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Same as `note_imminent_preimage` but origin is `OperationalPreimageOrigin`.
       */
      noteImminentPreimageOperational: AugmentedSubmittable<
        (encodedProposal: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Register the preimage for an upcoming proposal. This doesn't require
       * the proposal to be in the dispatch queue but does require a deposit,
       * returned once enacted.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `encoded_proposal`: The preimage of a proposal.
       *
       * Emits `PreimageNoted`.
       *
       * Weight: `O(E)` with E size of `encoded_proposal` (protected by a
       * required deposit).
       */
      notePreimage: AugmentedSubmittable<
        (encodedProposal: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Same as `note_preimage` but origin is `OperationalPreimageOrigin`.
       */
      notePreimageOperational: AugmentedSubmittable<
        (encodedProposal: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Propose a sensitive action to be taken.
       *
       * The dispatch origin of this call must be _Signed_ and the sender must
       * have funds to cover the deposit.
       *
       * - `proposal_hash`: The hash of the proposal preimage.
       * - `value`: The amount of deposit (must be at least `MinimumDeposit`).
       *
       * Emits `Proposed`.
       *
       * Weight: `O(p)`
       */
      propose: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u128>]
      >;
      /**
       * Remove an expired proposal preimage and collect the deposit.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `proposal_hash`: The preimage hash of a proposal.
       * - `proposal_length_upper_bound`: an upper bound on length of the
       *   proposal. Extrinsic is weighted according to this value with no refund.
       *
       * This will only work after `VotingPeriod` blocks from the time that the
       * preimage was noted, if it's the same account doing it. If it's a
       * different account, then it'll only work an additional `EnactmentPeriod` later.
       *
       * Emits `PreimageReaped`.
       *
       * Weight: `O(D)` where D is length of proposal.
       */
      reapPreimage: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          proposalLenUpperBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>]
      >;
      /**
       * Remove a vote for a referendum.
       *
       * If the `target` is equal to the signer, then this function is exactly
       * equivalent to `remove_vote`. If not equal to the signer, then the vote
       * must have expired, either because the referendum was cancelled, because
       * the voter lost the referendum or because the conviction period is over.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `target`: The account of the vote to be removed; this account must have
       *   voted for referendum `index`.
       * - `index`: The index of referendum of the vote to be removed.
       *
       * Weight: `O(R + log R)` where R is the number of referenda that `target`
       * has voted on. Weight is calculated for the maximum number of vote.
       */
      removeOtherVote: AugmentedSubmittable<
        (
          target: AccountId20 | string | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u32]
      >;
      /**
       * Remove a vote for a referendum.
       *
       * If:
       *
       * - The referendum was cancelled, or
       * - The referendum is ongoing, or
       * - The referendum has ended such that
       * - The vote of the account was in opposition to the result; or
       * - There was no conviction to the account's vote; or
       * - The account made a split vote ...then the vote is removed cleanly and a
       *   following call to `unlock` may result in more funds being available.
       *
       * If, however, the referendum has ended and:
       *
       * - It finished corresponding to the vote of the account, and
       * - The account made a standard vote with conviction, and
       * - The lock period of the conviction is not over ...then the lock will be
       *   aggregated into the overall account's lock, which may involve
       *   _overlocking_ (where the two locks are combined into a single lock
       *   that is the maximum of both the amount locked and the time is it locked for).
       *
       * The dispatch origin of this call must be _Signed_, and the signer must
       * have a vote registered for referendum `index`.
       *
       * - `index`: The index of referendum of the vote to be removed.
       *
       * Weight: `O(R + log R)` where R is the number of referenda that `target`
       * has voted on. Weight is calculated for the maximum number of vote.
       */
      removeVote: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Signals agreement with a particular proposal.
       *
       * The dispatch origin of this call must be _Signed_ and the sender must
       * have funds to cover the deposit, equal to the original deposit.
       *
       * - `proposal`: The index of the proposal to second.
       * - `seconds_upper_bound`: an upper bound on the current number of seconds
       *   on this proposal. Extrinsic is weighted according to this value with
       *   no refund.
       *
       * Weight: `O(S)` where S is the number of seconds a proposal already has.
       */
      second: AugmentedSubmittable<
        (
          proposal: Compact<u32> | AnyNumber | Uint8Array,
          secondsUpperBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u32>]
      >;
      /**
       * Undelegate the voting power of the sending account.
       *
       * Tokens may be unlocked following once an amount of time consistent with
       * the lock period of the conviction with which the delegation was issued.
       *
       * The dispatch origin of this call must be _Signed_ and the signing
       * account must be currently delegating.
       *
       * Emits `Undelegated`.
       *
       * Weight: `O(R)` where R is the number of referendums the voter
       * delegating to has voted on. Weight is charged as if maximum votes.
       */
      undelegate: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Unlock tokens that have an expired lock.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `target`: The account to remove the lock on.
       *
       * Weight: `O(R)` with R number of vote of target.
       */
      unlock: AugmentedSubmittable<
        (target: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Veto and blacklist the external proposal hash.
       *
       * The dispatch origin of this call must be `VetoOrigin`.
       *
       * - `proposal_hash`: The preimage hash of the proposal to veto and blacklist.
       *
       * Emits `Vetoed`.
       *
       * Weight: `O(V + log(V))` where V is number of `existing vetoers`
       */
      vetoExternal: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Vote in a referendum. If `vote.is_aye()`, the vote is to enact the
       * proposal; otherwise it is a vote to keep the status quo.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `ref_index`: The index of the referendum to vote for.
       * - `vote`: The vote configuration.
       *
       * Weight: `O(R)` where R is the number of referendums the voter has voted on.
       */
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
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    dmpQueue: {
      /**
       * Service a single overweight message.
       *
       * - `origin`: Must pass `ExecuteOverweightOrigin`.
       * - `index`: The index of the overweight message to service.
       * - `weight_limit`: The amount of weight that message execution may take.
       *
       * Errors:
       *
       * - `Unknown`: Message of `index` is unknown.
       * - `OverLimit`: Message execution may use greater than `weight_limit`.
       *
       * Events:
       *
       * - `OverweightServiced`: On success.
       */
      serviceOverweight: AugmentedSubmittable<
        (
          index: u64 | AnyNumber | Uint8Array,
          weightLimit: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u64, u64]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    ethereum: {
      /**
       * Transact an Ethereum transaction.
       */
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
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    evm: {
      /**
       * Issue an EVM call operation. This is similar to a message call
       * transaction in Ethereum.
       */
      call: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          target: H160 | string | Uint8Array,
          input: Bytes | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas: Option<U256> | null | object | string | Uint8Array,
          nonce: Option<U256> | null | object | string | Uint8Array,
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
      /**
       * Issue an EVM create operation. This is similar to a contract creation
       * transaction in Ethereum.
       */
      create: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          init: Bytes | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas: Option<U256> | null | object | string | Uint8Array,
          nonce: Option<U256> | null | object | string | Uint8Array,
          accessList:
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [H160 | string | Uint8Array, Vec<H256> | (H256 | string | Uint8Array)[]][]
        ) => SubmittableExtrinsic<ApiType>,
        [H160, Bytes, U256, u64, U256, Option<U256>, Option<U256>, Vec<ITuple<[H160, Vec<H256>]>>]
      >;
      /**
       * Issue an EVM create2 operation.
       */
      create2: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          init: Bytes | string | Uint8Array,
          salt: H256 | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas: Option<U256> | null | object | string | Uint8Array,
          nonce: Option<U256> | null | object | string | Uint8Array,
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
      /**
       * Withdraw balance from EVM into currency/balances pallet.
       */
      withdraw: AugmentedSubmittable<
        (
          address: H160 | string | Uint8Array,
          value: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H160, u128]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    identity: {
      /**
       * Add a registrar to the system.
       *
       * The dispatch origin for this call must be `T::RegistrarOrigin`.
       *
       * - `account`: the account of the registrar.
       *
       * Emits `RegistrarAdded` if successful.
       *
       * # <weight>
       *
       * - `O(R)` where `R` registrar-count (governance-bounded and code-bounded).
       * - One storage mutation (codec `O(R)`).
       * - One event.
       *
       * # </weight>
       */
      addRegistrar: AugmentedSubmittable<
        (account: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Add the given account to the sender's subs.
       *
       * Payment: Balance reserved by a previous `set_subs` call for one sub
       * will be repatriated to the sender.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * have a registered sub identity of `sub`.
       */
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
      /**
       * Cancel a previous request.
       *
       * Payment: A previously reserved deposit is returned on success.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * have a registered identity.
       *
       * - `reg_index`: The index of the registrar whose judgement is no longer requested.
       *
       * Emits `JudgementUnrequested` if successful.
       *
       * # <weight>
       *
       * - `O(R + X)`.
       * - One balance-reserve operation.
       * - One storage mutation `O(R + X)`.
       * - One event
       *
       * # </weight>
       */
      cancelRequest: AugmentedSubmittable<
        (regIndex: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Clear an account's identity info and all sub-accounts and return all deposits.
       *
       * Payment: All reserved balances on the account are returned.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * have a registered identity.
       *
       * Emits `IdentityCleared` if successful.
       *
       * # <weight>
       *
       * - `O(R + S + X)`
       * - Where `R` registrar-count (governance-bounded).
       * - Where `S` subs-count (hard- and deposit-bounded).
       * - Where `X` additional-field-count (deposit-bounded and code-bounded).
       * - One balance-unreserve operation.
       * - `2` storage reads and `S + 2` storage deletions.
       * - One event.
       *
       * # </weight>
       */
      clearIdentity: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Remove an account's identity and sub-account information and slash the deposits.
       *
       * Payment: Reserved balances from `set_subs` and `set_identity` are
       * slashed and handled by `Slash`. Verification request deposits are not
       * returned; they should be cancelled manually using `cancel_request`.
       *
       * The dispatch origin for this call must match `T::ForceOrigin`.
       *
       * - `target`: the account whose identity the judgement is upon. This must
       *   be an account with a registered identity.
       *
       * Emits `IdentityKilled` if successful.
       *
       * # <weight>
       *
       * - `O(R + S + X)`.
       * - One balance-reserve operation.
       * - `S + 2` storage mutations.
       * - One event.
       *
       * # </weight>
       */
      killIdentity: AugmentedSubmittable<
        (target: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Provide a judgement for an account's identity.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * be the account of the registrar whose index is `reg_index`.
       *
       * - `reg_index`: the index of the registrar whose judgement is being made.
       * - `target`: the account whose identity the judgement is upon. This must
       *   be an account with a registered identity.
       * - `judgement`: the judgement of the registrar of index `reg_index` about `target`.
       *
       * Emits `JudgementGiven` if successful.
       *
       * # <weight>
       *
       * - `O(R + X)`.
       * - One balance-transfer operation.
       * - Up to one account-lookup operation.
       * - Storage: 1 read `O(R)`, 1 mutate `O(R + X)`.
       * - One event.
       *
       * # </weight>
       */
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
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, AccountId20, PalletIdentityJudgement]
      >;
      /**
       * Remove the sender as a sub-account.
       *
       * Payment: Balance reserved by a previous `set_subs` call for one sub
       * will be repatriated to the sender (_not_ the original depositor).
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * have a registered super-identity.
       *
       * NOTE: This should not normally be used, but is provided in the case
       * that the non- controller of an account is maliciously registered as a
       * sub-account.
       */
      quitSub: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Remove the given account from the sender's subs.
       *
       * Payment: Balance reserved by a previous `set_subs` call for one sub
       * will be repatriated to the sender.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * have a registered sub identity of `sub`.
       */
      removeSub: AugmentedSubmittable<
        (sub: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Alter the associated name of the given sub-account.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * have a registered sub identity of `sub`.
       */
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
      /**
       * Request a judgement from a registrar.
       *
       * Payment: At most `max_fee` will be reserved for payment to the
       * registrar if judgement given.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * have a registered identity.
       *
       * - `reg_index`: The index of the registrar whose judgement is requested.
       * - `max_fee`: The maximum fee that may be paid. This should just be
       *   auto-populated as:
       *
       * ```nocompile
       * Self::registrars().get(reg_index).unwrap().fee;
       * ```
       *
       * Emits `JudgementRequested` if successful.
       *
       * # <weight>
       *
       * - `O(R + X)`.
       * - One balance-reserve operation.
       * - Storage: 1 read `O(R)`, 1 mutate `O(X + R)`.
       * - One event.
       *
       * # </weight>
       */
      requestJudgement: AugmentedSubmittable<
        (
          regIndex: Compact<u32> | AnyNumber | Uint8Array,
          maxFee: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u128>]
      >;
      /**
       * Change the account associated with a registrar.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * be the account of the registrar whose index is `index`.
       *
       * - `index`: the index of the registrar whose fee is to be set.
       * - `new`: the new account ID.
       *
       * # <weight>
       *
       * - `O(R)`.
       * - One storage mutation `O(R)`.
       * - Benchmark: 8.823 + R * 0.32 µs (min squares analysis)
       *
       * # </weight>
       */
      setAccountId: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          updated: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, AccountId20]
      >;
      /**
       * Set the fee required for a judgement to be requested from a registrar.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * be the account of the registrar whose index is `index`.
       *
       * - `index`: the index of the registrar whose fee is to be set.
       * - `fee`: the new fee.
       *
       * # <weight>
       *
       * - `O(R)`.
       * - One storage mutation `O(R)`.
       * - Benchmark: 7.315 + R * 0.329 µs (min squares analysis)
       *
       * # </weight>
       */
      setFee: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          fee: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u128>]
      >;
      /**
       * Set the field information for a registrar.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * be the account of the registrar whose index is `index`.
       *
       * - `index`: the index of the registrar whose fee is to be set.
       * - `fields`: the fields that the registrar concerns themselves with.
       *
       * # <weight>
       *
       * - `O(R)`.
       * - One storage mutation `O(R)`.
       * - Benchmark: 7.464 + R * 0.325 µs (min squares analysis)
       *
       * # </weight>
       */
      setFields: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          fields: PalletIdentityBitFlags
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, PalletIdentityBitFlags]
      >;
      /**
       * Set an account's identity information and reserve the appropriate deposit.
       *
       * If the account already has identity information, the deposit is taken
       * as part payment for the new deposit.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * - `info`: The identity information.
       *
       * Emits `IdentitySet` if successful.
       *
       * # <weight>
       *
       * - `O(X + X' + R)`
       * - Where `X` additional-field-count (deposit-bounded and code-bounded)
       * - Where `R` judgements-count (registrar-count-bounded)
       * - One balance reserve operation.
       * - One storage mutation (codec-read `O(X' + R)`, codec-write `O(X + R)`).
       * - One event.
       *
       * # </weight>
       */
      setIdentity: AugmentedSubmittable<
        (
          info:
            | PalletIdentityIdentityInfo
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
        [PalletIdentityIdentityInfo]
      >;
      /**
       * Set the sub-accounts of the sender.
       *
       * Payment: Any aggregate balance reserved by previous `set_subs` calls
       * will be returned and an amount `SubAccountDeposit` will be reserved for
       * each item in `subs`.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must
       * have a registered identity.
       *
       * - `subs`: The identity's (new) sub-accounts.
       *
       * # <weight>
       *
       * - `O(P + S)`
       * - Where `P` old-subs-count (hard- and deposit-bounded).
       * - Where `S` subs-count (hard- and deposit-bounded).
       * - At most one balance operations.
       * - DB:
       * - `P + S` storage mutations (codec complexity `O(1)`)
       * - One storage read (codec complexity `O(P)`).
       * - One storage write (codec complexity `O(S)`).
       * - One storage-exists (`IdentityOf::contains_key`).
       *
       * # </weight>
       */
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
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    localAssets: {
      /**
       * Approve an amount of asset for transfer by a delegated third-party account.
       *
       * Origin must be Signed.
       *
       * Ensures that `ApprovalDeposit` worth of `Currency` is reserved from
       * signing account for the purpose of holding the approval. If some
       * non-zero amount of assets is already approved from signing account to
       * `delegate`, then it is topped up or unreserved to meet the right value.
       *
       * NOTE: The signing account does not need to own `amount` of assets at
       * the point of making this call.
       *
       * - `id`: The identifier of the asset.
       * - `delegate`: The account to delegate permission to transfer asset.
       * - `amount`: The amount of asset that may be transferred by `delegate`. If
       *   there is already an approval in place, then this acts additively.
       *
       * Emits `ApprovedTransfer` on success.
       *
       * Weight: `O(1)`
       */
      approveTransfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          delegate: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Reduce the balance of `who` by as much as possible up to `amount` assets of `id`.
       *
       * Origin must be Signed and the sender should be the Manager of the asset `id`.
       *
       * Bails with `NoAccount` if the `who` is already dead.
       *
       * - `id`: The identifier of the asset to have some amount burned.
       * - `who`: The account to be debited from.
       * - `amount`: The maximum amount by which `who`'s balance should be reduced.
       *
       * Emits `Burned` with the actual amount burned. If this takes the balance
       * to below the minimum for the asset, then the amount burned is increased
       * to take it to zero.
       *
       * Weight: `O(1)` Modes: Post-existence of `who`; Pre & post Zombie-status of `who`.
       */
      burn: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Cancel all of some asset approved for delegated transfer by a
       * third-party account.
       *
       * Origin must be Signed and there must be an approval in place between
       * signer and `delegate`.
       *
       * Unreserves any deposit previously reserved by `approve_transfer` for
       * the approval.
       *
       * - `id`: The identifier of the asset.
       * - `delegate`: The account delegated permission to transfer asset.
       *
       * Emits `ApprovalCancelled` on success.
       *
       * Weight: `O(1)`
       */
      cancelApproval: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          delegate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Clear the metadata for an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * Any deposit is freed for the asset owner.
       *
       * - `id`: The identifier of the asset to clear.
       *
       * Emits `MetadataCleared`.
       *
       * Weight: `O(1)`
       */
      clearMetadata: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Issue a new class of fungible assets from a public origin.
       *
       * This new asset class has no assets initially and its owner is the origin.
       *
       * The origin must be Signed and the sender must have sufficient funds free.
       *
       * Funds of sender are reserved by `AssetDeposit`.
       *
       * Parameters:
       *
       * - `id`: The identifier of the new asset. This must not be currently in
       *   use to identify an existing asset.
       * - `admin`: The admin of this class of assets. The admin is the initial
       *   address of each member of the asset class's admin team.
       * - `min_balance`: The minimum balance of this new asset that any single
       *   account must have. If an account's balance is reduced below this,
       *   then it collapses to zero.
       *
       * Emits `Created` event when successful.
       *
       * Weight: `O(1)`
       */
      create: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          admin: AccountId20 | string | Uint8Array,
          minBalance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, u128]
      >;
      /**
       * Destroy a class of fungible assets.
       *
       * The origin must conform to `ForceOrigin` or must be Signed and the
       * sender must be the owner of the asset `id`.
       *
       * - `id`: The identifier of the asset to be destroyed. This must identify
       *   an existing asset.
       *
       * Emits `Destroyed` event when successful.
       *
       * NOTE: It can be helpful to first freeze an asset before destroying it
       * so that you can provide accurate witness information and prevent users
       * from manipulating state in a way that can make it harder to destroy.
       *
       * Weight: `O(c + p + a)` where:
       *
       * - `c = (witness.accounts - witness.sufficients)`
       * - `s = witness.sufficients`
       * - `a = witness.approvals`
       */
      destroy: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          witness:
            | PalletAssetsDestroyWitness
            | { accounts?: any; sufficients?: any; approvals?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, PalletAssetsDestroyWitness]
      >;
      /**
       * Alter the attributes of a given asset.
       *
       * Origin must be `ForceOrigin`.
       *
       * - `id`: The identifier of the asset.
       * - `owner`: The new Owner of this asset.
       * - `issuer`: The new Issuer of this asset.
       * - `admin`: The new Admin of this asset.
       * - `freezer`: The new Freezer of this asset.
       * - `min_balance`: The minimum balance of this new asset that any single
       *   account must have. If an account's balance is reduced below this,
       *   then it collapses to zero.
       * - `is_sufficient`: Whether a non-zero balance of this asset is deposit of
       *   sufficient value to account for the state bloat associated with its
       *   balance storage. If set to `true`, then non-zero balances may be
       *   stored without a `consumer` reference (and thus an ED in the Balances
       *   pallet or whatever else is used to control user-account state growth).
       * - `is_frozen`: Whether this asset class is frozen except for
       *   permissioned/admin instructions.
       *
       * Emits `AssetStatusChanged` with the identity of the asset.
       *
       * Weight: `O(1)`
       */
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
      /**
       * Cancel all of some asset approved for delegated transfer by a
       * third-party account.
       *
       * Origin must be either ForceOrigin or Signed origin with the signer
       * being the Admin account of the asset `id`.
       *
       * Unreserves any deposit previously reserved by `approve_transfer` for
       * the approval.
       *
       * - `id`: The identifier of the asset.
       * - `delegate`: The account delegated permission to transfer asset.
       *
       * Emits `ApprovalCancelled` on success.
       *
       * Weight: `O(1)`
       */
      forceCancelApproval: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          delegate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20]
      >;
      /**
       * Clear the metadata for an asset.
       *
       * Origin must be ForceOrigin.
       *
       * Any deposit is returned.
       *
       * - `id`: The identifier of the asset to clear.
       *
       * Emits `MetadataCleared`.
       *
       * Weight: `O(1)`
       */
      forceClearMetadata: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Issue a new class of fungible assets from a privileged origin.
       *
       * This new asset class has no assets initially.
       *
       * The origin must conform to `ForceOrigin`.
       *
       * Unlike `create`, no funds are reserved.
       *
       * - `id`: The identifier of the new asset. This must not be currently in
       *   use to identify an existing asset.
       * - `owner`: The owner of this class of assets. The owner has full
       *   superuser permissions over this asset, but may later change and
       *   configure the permissions using `transfer_ownership` and `set_team`.
       * - `min_balance`: The minimum balance of this new asset that any single
       *   account must have. If an account's balance is reduced below this,
       *   then it collapses to zero.
       *
       * Emits `ForceCreated` event when successful.
       *
       * Weight: `O(1)`
       */
      forceCreate: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          isSufficient: bool | boolean | Uint8Array,
          minBalance: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, bool, Compact<u128>]
      >;
      /**
       * Force the metadata for an asset to some value.
       *
       * Origin must be ForceOrigin.
       *
       * Any deposit is left alone.
       *
       * - `id`: The identifier of the asset to update.
       * - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
       * - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
       * - `decimals`: The number of decimals this asset uses to represent one unit.
       *
       * Emits `MetadataSet`.
       *
       * Weight: `O(N + S)` where N and S are the length of the name and symbol
       * respectively.
       */
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
      /**
       * Move some assets from one account to another.
       *
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       *
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `source`: The account to be debited.
       * - `dest`: The account to be credited.
       * - `amount`: The amount by which the `source`'s balance of assets should
       *   be reduced and `dest`'s balance increased. The amount actually
       *   transferred may be slightly greater in the case that the transfer
       *   would otherwise take the `source` balance above zero but below the
       *   minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes
       * the source balance to below the minimum for the asset, then the amount
       * transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `dest`; Post-existence of
       * `source`; Account pre-existence of `dest`.
       */
      forceTransfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          source: AccountId20 | string | Uint8Array,
          dest: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, Compact<u128>]
      >;
      /**
       * Disallow further unprivileged transfers from an account.
       *
       * Origin must be Signed and the sender should be the Freezer of the asset `id`.
       *
       * - `id`: The identifier of the asset to be frozen.
       * - `who`: The account to be frozen.
       *
       * Emits `Frozen`.
       *
       * Weight: `O(1)`
       */
      freeze: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Disallow further unprivileged transfers for the asset class.
       *
       * Origin must be Signed and the sender should be the Freezer of the asset `id`.
       *
       * - `id`: The identifier of the asset to be frozen.
       *
       * Emits `Frozen`.
       *
       * Weight: `O(1)`
       */
      freezeAsset: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Mint assets of a particular class.
       *
       * The origin must be Signed and the sender must be the Issuer of the asset `id`.
       *
       * - `id`: The identifier of the asset to have some amount minted.
       * - `beneficiary`: The account to be credited with the minted assets.
       * - `amount`: The amount of the asset to be minted.
       *
       * Emits `Issued` event when successful.
       *
       * Weight: `O(1)` Modes: Pre-existing balance of `beneficiary`; Account
       * pre-existence of `beneficiary`.
       */
      mint: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Return the deposit (if any) of an asset account.
       *
       * The origin must be Signed.
       *
       * - `id`: The identifier of the asset for the account to be created.
       * - `allow_burn`: If `true` then assets may be destroyed in order to
       *   complete the refund.
       *
       * Emits `Refunded` event when successful.
       */
      refund: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          allowBurn: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, bool]
      >;
      /**
       * Set the metadata for an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * Funds of sender are reserved according to the formula:
       * `MetadataDepositBase + MetadataDepositPerByte * (name.len +
       * symbol.len)` taking into account any already reserved funds.
       *
       * - `id`: The identifier of the asset to update.
       * - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
       * - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
       * - `decimals`: The number of decimals this asset uses to represent one unit.
       *
       * Emits `MetadataSet`.
       *
       * Weight: `O(1)`
       */
      setMetadata: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          name: Bytes | string | Uint8Array,
          symbol: Bytes | string | Uint8Array,
          decimals: u8 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, Bytes, Bytes, u8]
      >;
      /**
       * Change the Issuer, Admin and Freezer of an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * - `id`: The identifier of the asset to be frozen.
       * - `issuer`: The new Issuer of this asset.
       * - `admin`: The new Admin of this asset.
       * - `freezer`: The new Freezer of this asset.
       *
       * Emits `TeamChanged`.
       *
       * Weight: `O(1)`
       */
      setTeam: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          issuer: AccountId20 | string | Uint8Array,
          admin: AccountId20 | string | Uint8Array,
          freezer: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, AccountId20]
      >;
      /**
       * Allow unprivileged transfers from an account again.
       *
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       *
       * - `id`: The identifier of the asset to be frozen.
       * - `who`: The account to be unfrozen.
       *
       * Emits `Thawed`.
       *
       * Weight: `O(1)`
       */
      thaw: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Allow unprivileged transfers for the asset again.
       *
       * Origin must be Signed and the sender should be the Admin of the asset `id`.
       *
       * - `id`: The identifier of the asset to be thawed.
       *
       * Emits `Thawed`.
       *
       * Weight: `O(1)`
       */
      thawAsset: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Create an asset account for non-provider assets.
       *
       * A deposit will be taken from the signer account.
       *
       * - `origin`: Must be Signed; the signer account must have sufficient funds
       *   for a deposit to be taken.
       * - `id`: The identifier of the asset for the account to be created.
       *
       * Emits `Touched` event when successful.
       */
      touch: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Move some assets from the sender account to another.
       *
       * Origin must be Signed.
       *
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `target`: The account to be credited.
       * - `amount`: The amount by which the sender's balance of assets should be
       *   reduced and `target`'s balance increased. The amount actually
       *   transferred may be slightly greater in the case that the transfer
       *   would otherwise take the sender balance above zero but below the
       *   minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes
       * the source balance to below the minimum for the asset, then the amount
       * transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `target`; Post-existence of
       * sender; Account pre-existence of `target`.
       */
      transfer: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Transfer some asset balance from a previously delegated account to some
       * third-party account.
       *
       * Origin must be Signed and there must be an approval in place by the
       * `owner` to the signer.
       *
       * If the entire amount approved for transfer is transferred, then any
       * deposit previously reserved by `approve_transfer` is unreserved.
       *
       * - `id`: The identifier of the asset.
       * - `owner`: The account which previously approved for a transfer of at
       *   least `amount` and from which the asset balance will be withdrawn.
       * - `destination`: The account to which the asset balance of `amount` will
       *   be transferred.
       * - `amount`: The amount of assets to transfer.
       *
       * Emits `TransferredApproved` on success.
       *
       * Weight: `O(1)`
       */
      transferApproved: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array,
          destination: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, AccountId20, Compact<u128>]
      >;
      /**
       * Move some assets from the sender account to another, keeping the sender
       * account alive.
       *
       * Origin must be Signed.
       *
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `target`: The account to be credited.
       * - `amount`: The amount by which the sender's balance of assets should be
       *   reduced and `target`'s balance increased. The amount actually
       *   transferred may be slightly greater in the case that the transfer
       *   would otherwise take the sender balance above zero but below the
       *   minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes
       * the source balance to below the minimum for the asset, then the amount
       * transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `target`; Post-existence of
       * sender; Account pre-existence of `target`.
       */
      transferKeepAlive: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array,
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20, Compact<u128>]
      >;
      /**
       * Change the Owner of an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * - `id`: The identifier of the asset.
       * - `owner`: The new Owner of this asset.
       *
       * Emits `OwnerChanged`.
       *
       * Weight: `O(1)`
       */
      transferOwnership: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          owner: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    maintenanceMode: {
      /**
       * Place the chain in maintenance mode
       *
       * Weight cost is:
       *
       * - One DB read to ensure we're not already in maintenance mode
       * - Three DB writes - 1 for the mode, 1 for suspending xcm execution, 1 for the event
       */
      enterMaintenanceMode: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Return the chain to normal operating mode
       *
       * Weight cost is:
       *
       * - One DB read to ensure we're in maintenance mode
       * - Three DB writes - 1 for the mode, 1 for resuming xcm execution, 1 for the event
       */
      resumeNormalOperation: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    moonbeamOrbiters: {
      /**
       * Add a collator to orbiters program.
       */
      addCollator: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Add an orbiter in a collator pool
       */
      collatorAddOrbiter: AugmentedSubmittable<
        (orbiter: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Remove an orbiter from the caller collator pool
       */
      collatorRemoveOrbiter: AugmentedSubmittable<
        (orbiter: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Remove the caller from the specified collator pool
       */
      orbiterLeaveCollatorPool: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Registering as an orbiter
       */
      orbiterRegister: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Deregistering from orbiters
       */
      orbiterUnregister: AugmentedSubmittable<
        (collatorsPoolCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Remove a collator from orbiters program.
       */
      removeCollator: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    parachainStaking: {
      /**
       * Cancel pending request to adjust the collator candidate self bond
       */
      cancelCandidateBondLess: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Cancel request to change an existing delegation.
       */
      cancelDelegationRequest: AugmentedSubmittable<
        (candidate: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Cancel open request to leave candidates
       *
       * - Only callable by collator account
       * - Result upon successful call is the candidate is active in the candidate pool
       */
      cancelLeaveCandidates: AugmentedSubmittable<
        (candidateCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * DEPRECATED use batch util with cancel_delegation_request for all
       * delegations Cancel a pending request to exit the set of delegators.
       * Success clears the pending exit request (thereby resetting the delay
       * upon another `leave_delegators` call).
       */
      cancelLeaveDelegators: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Increase collator candidate self bond by `more`
       */
      candidateBondMore: AugmentedSubmittable<
        (more: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u128]
      >;
      /**
       * If caller is not a delegator and not a collator, then join the set of
       * delegators If caller is a delegator, then makes delegation to change
       * their delegation state
       */
      delegate: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          candidateDelegationCount: u32 | AnyNumber | Uint8Array,
          delegationCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128, u32, u32]
      >;
      /**
       * Bond more for delegators wrt a specific collator candidate.
       */
      delegatorBondMore: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          more: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128]
      >;
      /**
       * Execute pending request to adjust the collator candidate self bond
       */
      executeCandidateBondLess: AugmentedSubmittable<
        (candidate: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Execute pending request to change an existing delegation
       */
      executeDelegationRequest: AugmentedSubmittable<
        (
          delegator: AccountId20 | string | Uint8Array,
          candidate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20]
      >;
      /**
       * Execute leave candidates request
       */
      executeLeaveCandidates: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          candidateDelegationCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u32]
      >;
      /**
       * DEPRECATED use batch util with execute_delegation_request for all
       * delegations Execute the right to exit the set of delegators and revoke
       * all ongoing delegations.
       */
      executeLeaveDelegators: AugmentedSubmittable<
        (
          delegator: AccountId20 | string | Uint8Array,
          delegationCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u32]
      >;
      /**
       * Temporarily leave the set of collator candidates without unbonding
       */
      goOffline: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Rejoin the set of collator candidates if previously had called `go_offline`
       */
      goOnline: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Hotfix to remove existing empty entries for candidates that have left.
       */
      hotfixRemoveDelegationRequestsExitedCandidates: AugmentedSubmittable<
        (
          candidates: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>]
      >;
      /**
       * Join the set of collator candidates
       */
      joinCandidates: AugmentedSubmittable<
        (
          bond: u128 | AnyNumber | Uint8Array,
          candidateCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, u32]
      >;
      /**
       * Request by collator candidate to decrease self bond by `less`
       */
      scheduleCandidateBondLess: AugmentedSubmittable<
        (less: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u128]
      >;
      /**
       * Request bond less for delegators wrt a specific collator candidate.
       */
      scheduleDelegatorBondLess: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          less: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128]
      >;
      /**
       * Request to leave the set of candidates. If successful, the account is
       * immediately removed from the candidate pool to prevent selection as a collator.
       */
      scheduleLeaveCandidates: AugmentedSubmittable<
        (candidateCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * DEPRECATED use batch util with schedule_revoke_delegation for all
       * delegations Request to leave the set of delegators. If successful, the
       * caller is scheduled to be allowed to exit via a
       * [DelegationAction::Revoke] towards all existing delegations. Success
       * forbids future delegation requests until the request is invoked or cancelled.
       */
      scheduleLeaveDelegators: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Request to revoke an existing delegation. If successful, the delegation
       * is scheduled to be allowed to be revoked via the
       * `execute_delegation_request` extrinsic.
       */
      scheduleRevokeDelegation: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Set blocks per round
       *
       * - If called with `new` less than length of current round, will transition
       *   immediately in the next block
       * - Also updates per-round inflation config
       */
      setBlocksPerRound: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Set the commission for all collators
       */
      setCollatorCommission: AugmentedSubmittable<
        (updated: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Perbill]
      >;
      /**
       * Set the annual inflation rate to derive per-round inflation
       */
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
      /**
       * Set the account that will hold funds set aside for parachain bond
       */
      setParachainBondAccount: AugmentedSubmittable<
        (updated: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Set the percent of inflation set aside for parachain bond
       */
      setParachainBondReservePercent: AugmentedSubmittable<
        (updated: Percent | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Percent]
      >;
      /**
       * Set the expectations for total staked. These expectations determine the
       * issuance for the round according to logic in `fn compute_issuance`
       */
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
      /**
       * Set the total number of collator candidates selected per round
       *
       * - Changes are not applied until the start of the next round
       */
      setTotalSelected: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    parachainSystem: {
      authorizeUpgrade: AugmentedSubmittable<
        (codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      enactAuthorizedUpgrade: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the current validation data.
       *
       * This should be invoked exactly once per block. It will panic at the
       * finalization phase if the call was not invoked.
       *
       * The dispatch origin for this call must be `Inherent`
       *
       * As a side effect, this function upgrades the current validation
       * function if the appropriate time has come.
       */
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
      sudoSendUpwardMessage: AugmentedSubmittable<
        (message: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polkadotXcm: {
      /**
       * Execute an XCM message from a local, signed, origin.
       *
       * An event is deposited indicating whether `msg` could be executed
       * completely or only partially.
       *
       * No more than `max_weight` will be used in its attempted execution. If
       * this is less than the maximum amount of weight that the message could
       * take to be executed, then no execution attempt will be made.
       *
       * NOTE: A successful return to this does _not_ imply that the `msg` was
       * executed successfully to completion; only that _some_ of it was executed.
       */
      execute: AugmentedSubmittable<
        (
          message: XcmVersionedXcm | { V0: any } | { V1: any } | { V2: any } | string | Uint8Array,
          maxWeight: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedXcm, u64]
      >;
      /**
       * Set a safe XCM version (the version that XCM should be encoded with if
       * the most recent version a destination can accept is unknown).
       *
       * - `origin`: Must be Root.
       * - `maybe_xcm_version`: The default XCM encoding version, or `None` to disable.
       */
      forceDefaultXcmVersion: AugmentedSubmittable<
        (
          maybeXcmVersion: Option<u32> | null | object | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Option<u32>]
      >;
      /**
       * Ask a location to notify us regarding their XCM version and any changes to it.
       *
       * - `origin`: Must be Root.
       * - `location`: The location to which we should subscribe for XCM version
       *   notifications.
       */
      forceSubscribeVersionNotify: AugmentedSubmittable<
        (
          location: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation]
      >;
      /**
       * Require that a particular destination should no longer notify us
       * regarding any XCM version changes.
       *
       * - `origin`: Must be Root.
       * - `location`: The location to which we are currently subscribed for XCM
       *   version notifications which we no longer desire.
       */
      forceUnsubscribeVersionNotify: AugmentedSubmittable<
        (
          location: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation]
      >;
      /**
       * Extoll that a particular destination can be communicated with through a
       * particular version of XCM.
       *
       * - `origin`: Must be Root.
       * - `location`: The destination that is being described.
       * - `xcm_version`: The latest version of XCM that `location` supports.
       */
      forceXcmVersion: AugmentedSubmittable<
        (
          location: XcmV1MultiLocation | { parents?: any; interior?: any } | string | Uint8Array,
          xcmVersion: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmV1MultiLocation, u32]
      >;
      /**
       * Transfer some assets from the local chain to the sovereign account of a
       * destination chain and forward a notification XCM.
       *
       * Fee payment on the destination side is made from the asset in the
       * `assets` vector of index `fee_asset_item`, up to enough to pay for
       * `weight_limit` of weight. If more weight is needed than `weight_limit`,
       * then the operation will fail and the assets send may be at risk.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be
       *   `X2(Parent, Parachain(..))` to send from parachain to parachain, or
       *   `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of
       *   `dest`. Will generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets
       *   used to pay the fee on the `dest` side.
       * - `fee_asset_item`: The index into `assets` of the item which should be
       *   used to pay fees.
       * - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
       */
      limitedReserveTransferAssets: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          beneficiary: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array,
          weightLimit:
            | XcmV2WeightLimit
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
          XcmV2WeightLimit
        ]
      >;
      /**
       * Teleport some assets from the local chain to some destination chain.
       *
       * Fee payment on the destination side is made from the asset in the
       * `assets` vector of index `fee_asset_item`, up to enough to pay for
       * `weight_limit` of weight. If more weight is needed than `weight_limit`,
       * then the operation will fail and the assets send may be at risk.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be
       *   `X2(Parent, Parachain(..))` to send from parachain to parachain, or
       *   `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of
       *   `dest`. Will generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. The first item should be the
       *   currency used to to pay the fee on the `dest` side. May not be empty.
       * - `fee_asset_item`: The index into `assets` of the item which should be
       *   used to pay fees.
       * - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
       */
      limitedTeleportAssets: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          beneficiary: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array,
          weightLimit:
            | XcmV2WeightLimit
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
          XcmV2WeightLimit
        ]
      >;
      /**
       * Transfer some assets from the local chain to the sovereign account of a
       * destination chain and forward a notification XCM.
       *
       * Fee payment on the destination side is made from the asset in the
       * `assets` vector of index `fee_asset_item`. The weight limit for fees is
       * not provided and thus is unlimited, with all fees taken as needed from
       * the asset.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be
       *   `X2(Parent, Parachain(..))` to send from parachain to parachain, or
       *   `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of
       *   `dest`. Will generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets
       *   used to pay the fee on the `dest` side.
       * - `fee_asset_item`: The index into `assets` of the item which should be
       *   used to pay fees.
       */
      reserveTransferAssets: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          beneficiary: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, XcmVersionedMultiLocation, XcmVersionedMultiAssets, u32]
      >;
      send: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          message: XcmVersionedXcm | { V0: any } | { V1: any } | { V2: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, XcmVersionedXcm]
      >;
      /**
       * Teleport some assets from the local chain to some destination chain.
       *
       * Fee payment on the destination side is made from the asset in the
       * `assets` vector of index `fee_asset_item`. The weight limit for fees is
       * not provided and thus is unlimited, with all fees taken as needed from
       * the asset.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be
       *   `X2(Parent, Parachain(..))` to send from parachain to parachain, or
       *   `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of
       *   `dest`. Will generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. The first item should be the
       *   currency used to to pay the fee on the `dest` side. May not be empty.
       * - `fee_asset_item`: The index into `assets` of the item which should be
       *   used to pay fees.
       */
      teleportAssets: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          beneficiary: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, XcmVersionedMultiLocation, XcmVersionedMultiAssets, u32]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    proxy: {
      /**
       * Register a proxy account for the sender that is able to make calls on its behalf.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `proxy`: The account that the `caller` would like to make a proxy.
       * - `proxy_type`: The permissions allowed for this proxy account.
       * - `delay`: The announcement period required of the initial proxy. Will
       *   generally be zero.
       *
       * # <weight>
       *
       * Weight is a function of the number of proxies the user has (P).
       *
       * # </weight>
       */
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
      /**
       * Publish the hash of a proxy-call that will be made in the future.
       *
       * This must be called some number of blocks before the corresponding
       * `proxy` is attempted if the delay associated with the proxy
       * relationship is greater than zero.
       *
       * No more than `MaxPending` announcements may be made at any one time.
       *
       * This will take a deposit of `AnnouncementDepositFactor` as well as
       * `AnnouncementDepositBase` if there are no other pending announcements.
       *
       * The dispatch origin for this call must be _Signed_ and a proxy of `real`.
       *
       * Parameters:
       *
       * - `real`: The account that the proxy will make a call on behalf of.
       * - `call_hash`: The hash of the call to be made by the `real` account.
       *
       * # <weight>
       *
       * Weight is a function of:
       *
       * - A: the number of announcements made.
       * - P: the number of proxies the user has.
       *
       * # </weight>
       */
      announce: AugmentedSubmittable<
        (
          real: AccountId20 | string | Uint8Array,
          callHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, H256]
      >;
      /**
       * Spawn a fresh new account that is guaranteed to be otherwise
       * inaccessible, and initialize it with a proxy of `proxy_type` for
       * `origin` sender.
       *
       * Requires a `Signed` origin.
       *
       * - `proxy_type`: The type of the proxy that the sender will be registered
       *   as over the new account. This will almost always be the most
       *   permissive `ProxyType` possible to allow for maximum flexibility.
       * - `index`: A disambiguation index, in case this is called multiple times
       *   in the same transaction (e.g. with `utility::batch`). Unless you're
       *   using `batch` you probably just want to use `0`.
       * - `delay`: The announcement period required of the initial proxy. Will
       *   generally be zero.
       *
       * Fails with `Duplicate` if this has already been called in this
       * transaction, from the same sender, with the same parameters.
       *
       * Fails if there are insufficient funds to pay for deposit.
       *
       * # <weight>
       *
       * Weight is a function of the number of proxies the user has (P).
       *
       * # </weight>
       *
       * TODO: Might be over counting 1 read
       */
      anonymous: AugmentedSubmittable<
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
      /**
       * Removes a previously spawned anonymous proxy.
       *
       * WARNING: **All access to this account will be lost.** Any funds held in
       * it will be inaccessible.
       *
       * Requires a `Signed` origin, and the sender account must have been
       * created by a call to `anonymous` with corresponding parameters.
       *
       * - `spawner`: The account that originally called `anonymous` to create this account.
       * - `index`: The disambiguation index originally passed to `anonymous`. Probably `0`.
       * - `proxy_type`: The proxy type originally passed to `anonymous`.
       * - `height`: The height of the chain when the call to `anonymous` was processed.
       * - `ext_index`: The extrinsic index in which the call to `anonymous` was processed.
       *
       * Fails with `NoPermission` in case the caller is not a previously
       * created anonymous account whose `anonymous` call has corresponding parameters.
       *
       * # <weight>
       *
       * Weight is a function of the number of proxies the user has (P).
       *
       * # </weight>
       */
      killAnonymous: AugmentedSubmittable<
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
      /**
       * Dispatch the given `call` from an account that the sender is authorised
       * for through `add_proxy`.
       *
       * Removes any corresponding announcement(s).
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `real`: The account that the proxy will make a call on behalf of.
       * - `force_proxy_type`: Specify the exact proxy type to be used and checked
       *   for this call.
       * - `call`: The call to be made by the `real` account.
       *
       * # <weight>
       *
       * Weight is a function of the number of proxies the user has (P).
       *
       * # </weight>
       */
      proxy: AugmentedSubmittable<
        (
          real: AccountId20 | string | Uint8Array,
          forceProxyType: Option<MoonbeamRuntimeProxyType> | null | object | string | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Option<MoonbeamRuntimeProxyType>, Call]
      >;
      /**
       * Dispatch the given `call` from an account that the sender is authorized
       * for through `add_proxy`.
       *
       * Removes any corresponding announcement(s).
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `real`: The account that the proxy will make a call on behalf of.
       * - `force_proxy_type`: Specify the exact proxy type to be used and checked
       *   for this call.
       * - `call`: The call to be made by the `real` account.
       *
       * # <weight>
       *
       * Weight is a function of:
       *
       * - A: the number of announcements made.
       * - P: the number of proxies the user has.
       *
       * # </weight>
       */
      proxyAnnounced: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          real: AccountId20 | string | Uint8Array,
          forceProxyType: Option<MoonbeamRuntimeProxyType> | null | object | string | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20, Option<MoonbeamRuntimeProxyType>, Call]
      >;
      /**
       * Remove the given announcement of a delegate.
       *
       * May be called by a target (proxied) account to remove a call that one
       * of their delegates (`delegate`) has announced they want to execute. The
       * deposit is returned.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `delegate`: The account that previously announced the call.
       * - `call_hash`: The hash of the call to be made.
       *
       * # <weight>
       *
       * Weight is a function of:
       *
       * - A: the number of announcements made.
       * - P: the number of proxies the user has.
       *
       * # </weight>
       */
      rejectAnnouncement: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          callHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, H256]
      >;
      /**
       * Remove a given announcement.
       *
       * May be called by a proxy account to remove a call they previously
       * announced and return the deposit.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `real`: The account that the proxy will make a call on behalf of.
       * - `call_hash`: The hash of the call to be made by the `real` account.
       *
       * # <weight>
       *
       * Weight is a function of:
       *
       * - A: the number of announcements made.
       * - P: the number of proxies the user has.
       *
       * # </weight>
       */
      removeAnnouncement: AugmentedSubmittable<
        (
          real: AccountId20 | string | Uint8Array,
          callHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, H256]
      >;
      /**
       * Unregister all proxy accounts for the sender.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * WARNING: This may be called on accounts created by `anonymous`, however
       * if done, then the unreserved fees will be inaccessible. **All access to
       * this account will be lost.**
       *
       * # <weight>
       *
       * Weight is a function of the number of proxies the user has (P).
       *
       * # </weight>
       */
      removeProxies: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Unregister a proxy account for the sender.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `proxy`: The account that the `caller` would like to remove as a proxy.
       * - `proxy_type`: The permissions currently enabled for the removed proxy account.
       *
       * # <weight>
       *
       * Weight is a function of the number of proxies the user has (P).
       *
       * # </weight>
       */
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
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    randomness: {
      /**
       * Populates the `RandomnessResults` that are due this block with the raw values
       */
      setBabeRandomnessResults: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    scheduler: {
      /**
       * Cancel an anonymously scheduled task.
       */
      cancel: AugmentedSubmittable<
        (
          when: u32 | AnyNumber | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, u32]
      >;
      /**
       * Cancel a named scheduled task.
       */
      cancelNamed: AugmentedSubmittable<
        (id: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Anonymously schedule a task.
       */
      schedule: AugmentedSubmittable<
        (
          when: u32 | AnyNumber | Uint8Array,
          maybePeriodic: Option<ITuple<[u32, u32]>> | null | object | string | Uint8Array,
          priority: u8 | AnyNumber | Uint8Array,
          call:
            | FrameSupportScheduleMaybeHashed
            | { Value: any }
            | { Hash: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]
      >;
      /**
       * Anonymously schedule a task after a delay.
       *
       * # <weight>
       *
       * Same as [`schedule`].
       *
       * # </weight>
       */
      scheduleAfter: AugmentedSubmittable<
        (
          after: u32 | AnyNumber | Uint8Array,
          maybePeriodic: Option<ITuple<[u32, u32]>> | null | object | string | Uint8Array,
          priority: u8 | AnyNumber | Uint8Array,
          call:
            | FrameSupportScheduleMaybeHashed
            | { Value: any }
            | { Hash: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]
      >;
      /**
       * Schedule a named task.
       */
      scheduleNamed: AugmentedSubmittable<
        (
          id: Bytes | string | Uint8Array,
          when: u32 | AnyNumber | Uint8Array,
          maybePeriodic: Option<ITuple<[u32, u32]>> | null | object | string | Uint8Array,
          priority: u8 | AnyNumber | Uint8Array,
          call:
            | FrameSupportScheduleMaybeHashed
            | { Value: any }
            | { Hash: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]
      >;
      /**
       * Schedule a named task after a delay.
       *
       * # <weight>
       *
       * Same as [`schedule_named`](Self::schedule_named).
       *
       * # </weight>
       */
      scheduleNamedAfter: AugmentedSubmittable<
        (
          id: Bytes | string | Uint8Array,
          after: u32 | AnyNumber | Uint8Array,
          maybePeriodic: Option<ITuple<[u32, u32]>> | null | object | string | Uint8Array,
          priority: u8 | AnyNumber | Uint8Array,
          call:
            | FrameSupportScheduleMaybeHashed
            | { Value: any }
            | { Hash: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    system: {
      /**
       * A dispatch that will fill the block weight up to the given ratio.
       */
      fillBlock: AugmentedSubmittable<
        (ratio: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Perbill]
      >;
      /**
       * Kill all storage items with a key that starts with the given prefix.
       *
       * **NOTE:** We rely on the Root origin to provide us the number of
       * subkeys under the prefix we are removing to accurately calculate the
       * weight of this function.
       */
      killPrefix: AugmentedSubmittable<
        (
          prefix: Bytes | string | Uint8Array,
          subkeys: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, u32]
      >;
      /**
       * Kill some items from storage.
       */
      killStorage: AugmentedSubmittable<
        (keys: Vec<Bytes> | (Bytes | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>,
        [Vec<Bytes>]
      >;
      /**
       * Make some on-chain remark.
       *
       * # <weight>
       *
       * - `O(1)`
       *
       * # </weight>
       */
      remark: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Make some on-chain remark and emit event.
       */
      remarkWithEvent: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the new runtime code.
       *
       * # <weight>
       *
       * - `O(C + S)` where `C` length of `code` and `S` complexity of `can_set_code`
       * - 1 call to `can_set_code`: `O(S)` (calls `sp_io::misc::runtime_version`
       *   which is expensive).
       * - 1 storage write (codec `O(C)`).
       * - 1 digest item.
       * - 1 event. The weight of this function is dependent on the runtime, but
       *   generally this is very expensive. We will treat this as a full block.
       *
       * # </weight>
       */
      setCode: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the new runtime code without doing any checks of the given `code`.
       *
       * # <weight>
       *
       * - `O(C)` where `C` length of `code`
       * - 1 storage write (codec `O(C)`).
       * - 1 digest item.
       * - 1 event. The weight of this function is dependent on the runtime. We
       *   will treat this as a full block. # </weight>
       */
      setCodeWithoutChecks: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the number of pages in the WebAssembly environment's heap.
       */
      setHeapPages: AugmentedSubmittable<
        (pages: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u64]
      >;
      /**
       * Set some items of storage.
       */
      setStorage: AugmentedSubmittable<
        (
          items:
            | Vec<ITuple<[Bytes, Bytes]>>
            | [Bytes | string | Uint8Array, Bytes | string | Uint8Array][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[Bytes, Bytes]>>]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    techCommitteeCollective: {
      /**
       * Close a vote that is either approved, disapproved or whose voting
       * period has ended.
       *
       * May be called by any signed account in order to finish voting and close
       * the proposal.
       *
       * If called before the end of the voting period it will only close the
       * vote if it is has enough votes to be approved or disapproved.
       *
       * If called after the end of the voting period abstentions are counted as
       * rejections unless there is a prime member set and the prime member cast
       * an approval.
       *
       * If the close operation completes successfully with disapproval, the
       * transaction fee will be waived. Otherwise execution of the approved
       * operation will be charged to the caller.
       *
       * - `proposal_weight_bound`: The maximum amount of weight consumed by
       *   executing the closed proposal.
       * - `length_bound`: The upper bound for the length of the proposal in
       *   storage. Checked via `storage::read` so it is `size_of::<u32>() == 4`
       *   larger than the pure length.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(B + M + P1 + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - `P1` is the complexity of `proposal` preimage.
       * - `P2` is proposal-count (code-bounded)
       * - DB:
       * - 2 storage reads (`Members`: codec `O(M)`, `Prime`: codec `O(1)`)
       * - 3 mutations (`Voting`: codec `O(M)`, `ProposalOf`: codec `O(B)`,
       *   `Proposals`: codec `O(P2)`)
       * - Any mutations done while executing `proposal` (`P1`)
       * - Up to 3 events
       *
       * # </weight>
       */
      close: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          proposalWeightBound: Compact<u64> | AnyNumber | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, Compact<u64>, Compact<u32>]
      >;
      /**
       * Disapprove a proposal, close, and remove it from the system, regardless
       * of its current state.
       *
       * Must be called by the Root origin.
       *
       * Parameters:
       *
       * - `proposal_hash`: The hash of the proposal that should be disapproved.
       *
       * # <weight>
       *
       * Complexity: O(P) where P is the number of max proposals DB Weight:
       *
       * - Reads: Proposals
       * - Writes: Voting, Proposals, ProposalOf
       *
       * # </weight>
       */
      disapproveProposal: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Dispatch a proposal from a member using the `Member` origin.
       *
       * Origin must be a member of the collective.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(M + P)` where `M` members-count (code-bounded) and `P` complexity of
       *   dispatching `proposal`
       * - DB: 1 read (codec `O(M)`) + DB access of `proposal`
       * - 1 event
       *
       * # </weight>
       */
      execute: AugmentedSubmittable<
        (
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, Compact<u32>]
      >;
      /**
       * Add a new proposal to either be voted on or executed directly.
       *
       * Requires the sender to be member.
       *
       * `threshold` determines whether `proposal` is executed directly
       * (`threshold < 2`) or put up for voting.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(B + M + P1)` or `O(B + M + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - Branching is influenced by `threshold` where:
       * - `P1` is proposal execution complexity (`threshold < 2`)
       * - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
       * - DB:
       * - 1 storage read `is_member` (codec `O(M)`)
       * - 1 storage read `ProposalOf::contains_key` (codec `O(1)`)
       * - DB accesses influenced by `threshold`:
       * - EITHER storage accesses done by `proposal` (`threshold < 2`)
       * - OR proposal insertion (`threshold <= 2`)
       * - 1 storage mutation `Proposals` (codec `O(P2)`)
       * - 1 storage mutation `ProposalCount` (codec `O(1)`)
       * - 1 storage write `ProposalOf` (codec `O(B)`)
       * - 1 storage write `Voting` (codec `O(M)`)
       * - 1 event
       *
       * # </weight>
       */
      propose: AugmentedSubmittable<
        (
          threshold: Compact<u32> | AnyNumber | Uint8Array,
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Call, Compact<u32>]
      >;
      /**
       * Set the collective's membership.
       *
       * - `new_members`: The new member list. Be nice to the chain and provide it sorted.
       * - `prime`: The prime member whose vote sets the default.
       * - `old_count`: The upper bound for the previous number of members in
       *   storage. Used for weight estimation.
       *
       * Requires root origin.
       *
       * NOTE: Does not enforce the expected `MaxMembers` limit on the amount of
       * members, but the weight estimations rely on it to estimate dispatchable weight.
       *
       * # WARNING:
       *
       * The `pallet-collective` can also be managed by logic outside of the
       * pallet through the implementation of the trait [`ChangeMembers`]. Any
       * call to `set_members` must be careful that the member set doesn't get
       * out of sync with other logic managing the member set.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(MP + N)` where:
       * - `M` old-members-count (code- and governance-bounded)
       * - `N` new-members-count (code- and governance-bounded)
       * - `P` proposals-count (code-bounded)
       * - DB:
       * - 1 storage mutation (codec `O(M)` read, `O(N)` write) for reading and
       *   writing the members
       * - 1 storage read (codec `O(P)`) for reading the proposals
       * - `P` storage mutations (codec `O(M)`) for updating the votes for each proposal
       * - 1 storage write (codec `O(1)`) for deleting the old `prime` and setting
       *   the new one
       *
       * # </weight>
       */
      setMembers: AugmentedSubmittable<
        (
          newMembers: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          prime: Option<AccountId20> | null | object | string | Uint8Array,
          oldCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Option<AccountId20>, u32]
      >;
      /**
       * Add an aye or nay vote for the sender to the given proposal.
       *
       * Requires the sender to be a member.
       *
       * Transaction fees will be waived if the member is voting on any
       * particular proposal for the first time and the call is successful.
       * Subsequent vote changes will charge a fee.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(M)` where `M` is members-count (code- and governance-bounded)
       * - DB:
       * - 1 storage read `Members` (codec `O(M)`)
       * - 1 storage mutation `Voting` (codec `O(M)`)
       * - 1 event
       *
       * # </weight>
       */
      vote: AugmentedSubmittable<
        (
          proposal: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          approve: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, bool]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    timestamp: {
      /**
       * Set the current time.
       *
       * This call should be invoked exactly once per block. It will panic at
       * the finalization phase, if this call hasn't been invoked by that time.
       *
       * The timestamp should be greater than the previous one by the amount
       * specified by `MinimumPeriod`.
       *
       * The dispatch origin for this call must be `Inherent`.
       *
       * # <weight>
       *
       * - `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)
       * - 1 storage read and 1 storage mutation (codec `O(1)`). (because of
       *   `DidUpdate::take` in `on_finalize`)
       * - 1 event handler `on_timestamp_set`. Must be `O(1)`.
       *
       * # </weight>
       */
      set: AugmentedSubmittable<
        (now: Compact<u64> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u64>]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    treasury: {
      /**
       * Approve a proposal. At a later time, the proposal will be allocated to
       * the beneficiary and the original deposit will be returned.
       *
       * May only be called from `T::ApproveOrigin`.
       *
       * # <weight>
       *
       * - Complexity: O(1).
       * - DbReads: `Proposals`, `Approvals`
       * - DbWrite: `Approvals`
       *
       * # </weight>
       */
      approveProposal: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /**
       * Put forward a suggestion for spending. A deposit proportional to the
       * value is reserved and slashed if the proposal is rejected. It is
       * returned once the proposal is awarded.
       *
       * # <weight>
       *
       * - Complexity: O(1)
       * - DbReads: `ProposalCount`, `origin account`
       * - DbWrites: `ProposalCount`, `Proposals`, `origin account`
       *
       * # </weight>
       */
      proposeSpend: AugmentedSubmittable<
        (
          value: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Reject a proposed spend. The original deposit will be slashed.
       *
       * May only be called from `T::RejectOrigin`.
       *
       * # <weight>
       *
       * - Complexity: O(1)
       * - DbReads: `Proposals`, `rejected proposer account`
       * - DbWrites: `Proposals`, `rejected proposer account`
       *
       * # </weight>
       */
      rejectProposal: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /**
       * Force a previously approved proposal to be removed from the approval
       * queue. The original deposit will no longer be returned.
       *
       * May only be called from `T::RejectOrigin`.
       *
       * - `proposal_id`: The index of a proposal
       *
       * # <weight>
       *
       * - Complexity: O(A) where `A` is the number of approvals
       * - Db reads and writes: `Approvals`
       *
       * # </weight>
       *
       * Errors:
       *
       * - `ProposalNotApproved`: The `proposal_id` supplied was not found in the
       *   approval queue, i.e., the proposal has not been approved. This could
       *   also mean the proposal does not exist altogether, thus there is no
       *   way it would have been approved in the first place.
       */
      removeApproval: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /**
       * Propose and approve a spend of treasury funds.
       *
       * - `origin`: Must be `SpendOrigin` with the `Success` value being at least `amount`.
       * - `amount`: The amount to be transferred from the treasury to the `beneficiary`.
       * - `beneficiary`: The destination account for the transfer.
       *
       * NOTE: For record-keeping purposes, the proposer is deemed to be
       * equivalent to the beneficiary.
       */
      spend: AugmentedSubmittable<
        (
          amount: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    treasuryCouncilCollective: {
      /**
       * Close a vote that is either approved, disapproved or whose voting
       * period has ended.
       *
       * May be called by any signed account in order to finish voting and close
       * the proposal.
       *
       * If called before the end of the voting period it will only close the
       * vote if it is has enough votes to be approved or disapproved.
       *
       * If called after the end of the voting period abstentions are counted as
       * rejections unless there is a prime member set and the prime member cast
       * an approval.
       *
       * If the close operation completes successfully with disapproval, the
       * transaction fee will be waived. Otherwise execution of the approved
       * operation will be charged to the caller.
       *
       * - `proposal_weight_bound`: The maximum amount of weight consumed by
       *   executing the closed proposal.
       * - `length_bound`: The upper bound for the length of the proposal in
       *   storage. Checked via `storage::read` so it is `size_of::<u32>() == 4`
       *   larger than the pure length.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(B + M + P1 + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - `P1` is the complexity of `proposal` preimage.
       * - `P2` is proposal-count (code-bounded)
       * - DB:
       * - 2 storage reads (`Members`: codec `O(M)`, `Prime`: codec `O(1)`)
       * - 3 mutations (`Voting`: codec `O(M)`, `ProposalOf`: codec `O(B)`,
       *   `Proposals`: codec `O(P2)`)
       * - Any mutations done while executing `proposal` (`P1`)
       * - Up to 3 events
       *
       * # </weight>
       */
      close: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          proposalWeightBound: Compact<u64> | AnyNumber | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, Compact<u64>, Compact<u32>]
      >;
      /**
       * Disapprove a proposal, close, and remove it from the system, regardless
       * of its current state.
       *
       * Must be called by the Root origin.
       *
       * Parameters:
       *
       * - `proposal_hash`: The hash of the proposal that should be disapproved.
       *
       * # <weight>
       *
       * Complexity: O(P) where P is the number of max proposals DB Weight:
       *
       * - Reads: Proposals
       * - Writes: Voting, Proposals, ProposalOf
       *
       * # </weight>
       */
      disapproveProposal: AugmentedSubmittable<
        (proposalHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Dispatch a proposal from a member using the `Member` origin.
       *
       * Origin must be a member of the collective.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(M + P)` where `M` members-count (code-bounded) and `P` complexity of
       *   dispatching `proposal`
       * - DB: 1 read (codec `O(M)`) + DB access of `proposal`
       * - 1 event
       *
       * # </weight>
       */
      execute: AugmentedSubmittable<
        (
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, Compact<u32>]
      >;
      /**
       * Add a new proposal to either be voted on or executed directly.
       *
       * Requires the sender to be member.
       *
       * `threshold` determines whether `proposal` is executed directly
       * (`threshold < 2`) or put up for voting.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(B + M + P1)` or `O(B + M + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - Branching is influenced by `threshold` where:
       * - `P1` is proposal execution complexity (`threshold < 2`)
       * - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
       * - DB:
       * - 1 storage read `is_member` (codec `O(M)`)
       * - 1 storage read `ProposalOf::contains_key` (codec `O(1)`)
       * - DB accesses influenced by `threshold`:
       * - EITHER storage accesses done by `proposal` (`threshold < 2`)
       * - OR proposal insertion (`threshold <= 2`)
       * - 1 storage mutation `Proposals` (codec `O(P2)`)
       * - 1 storage mutation `ProposalCount` (codec `O(1)`)
       * - 1 storage write `ProposalOf` (codec `O(B)`)
       * - 1 storage write `Voting` (codec `O(M)`)
       * - 1 event
       *
       * # </weight>
       */
      propose: AugmentedSubmittable<
        (
          threshold: Compact<u32> | AnyNumber | Uint8Array,
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Call, Compact<u32>]
      >;
      /**
       * Set the collective's membership.
       *
       * - `new_members`: The new member list. Be nice to the chain and provide it sorted.
       * - `prime`: The prime member whose vote sets the default.
       * - `old_count`: The upper bound for the previous number of members in
       *   storage. Used for weight estimation.
       *
       * Requires root origin.
       *
       * NOTE: Does not enforce the expected `MaxMembers` limit on the amount of
       * members, but the weight estimations rely on it to estimate dispatchable weight.
       *
       * # WARNING:
       *
       * The `pallet-collective` can also be managed by logic outside of the
       * pallet through the implementation of the trait [`ChangeMembers`]. Any
       * call to `set_members` must be careful that the member set doesn't get
       * out of sync with other logic managing the member set.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(MP + N)` where:
       * - `M` old-members-count (code- and governance-bounded)
       * - `N` new-members-count (code- and governance-bounded)
       * - `P` proposals-count (code-bounded)
       * - DB:
       * - 1 storage mutation (codec `O(M)` read, `O(N)` write) for reading and
       *   writing the members
       * - 1 storage read (codec `O(P)`) for reading the proposals
       * - `P` storage mutations (codec `O(M)`) for updating the votes for each proposal
       * - 1 storage write (codec `O(1)`) for deleting the old `prime` and setting
       *   the new one
       *
       * # </weight>
       */
      setMembers: AugmentedSubmittable<
        (
          newMembers: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          prime: Option<AccountId20> | null | object | string | Uint8Array,
          oldCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Option<AccountId20>, u32]
      >;
      /**
       * Add an aye or nay vote for the sender to the given proposal.
       *
       * Requires the sender to be a member.
       *
       * Transaction fees will be waived if the member is voting on any
       * particular proposal for the first time and the call is successful.
       * Subsequent vote changes will charge a fee.
       *
       * # <weight>
       *
       * ## Weight
       *
       * - `O(M)` where `M` is members-count (code- and governance-bounded)
       * - DB:
       * - 1 storage read `Members` (codec `O(M)`)
       * - 1 storage mutation `Voting` (codec `O(M)`)
       * - 1 event
       *
       * # </weight>
       */
      vote: AugmentedSubmittable<
        (
          proposal: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          approve: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, bool]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    utility: {
      /**
       * Send a call through an indexed pseudonym of the sender.
       *
       * Filter from origin are passed along. The call will be dispatched with
       * an origin which use the same filter as the origin of this call.
       *
       * NOTE: If you need to ensure that any account-based filtering is not
       * honored (i.e. because you expect `proxy` to have been used prior in the
       * call stack and you do not want the call restrictions to apply to any
       * sub-accounts), then use `as_multi_threshold_1` in the Multisig pallet instead.
       *
       * NOTE: Prior to version *12, this was called `as_limited_sub`.
       *
       * The dispatch origin for this call must be _Signed_.
       */
      asDerivative: AugmentedSubmittable<
        (
          index: u16 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, Call]
      >;
      /**
       * Send a batch of dispatch calls.
       *
       * May be called from any origin.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of
       *   call must not exceed the constant: `batched_calls_limit` (available
       *   in constant metadata).
       *
       * If origin is root then call are dispatch without checking origin
       * filter. (This includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * # <weight>
       *
       * - Complexity: O(C) where C is the number of calls to be batched.
       *
       * # </weight>
       *
       * This will return `Ok` in all circumstances. To determine the success of
       * the batch, an event is deposited. If a call failed and the batch was
       * interrupted, then the `BatchInterrupted` event is deposited, along with
       * the number of successful calls made and the error of the failed call.
       * If all were successful, then the `BatchCompleted` event is deposited.
       */
      batch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /**
       * Send a batch of dispatch calls and atomically execute them. The whole
       * transaction will rollback and fail if any of the calls failed.
       *
       * May be called from any origin.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of
       *   call must not exceed the constant: `batched_calls_limit` (available
       *   in constant metadata).
       *
       * If origin is root then call are dispatch without checking origin
       * filter. (This includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * # <weight>
       *
       * - Complexity: O(C) where C is the number of calls to be batched.
       *
       * # </weight>
       */
      batchAll: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /**
       * Dispatches a function call with a provided origin.
       *
       * The dispatch origin for this call must be _Root_.
       *
       * # <weight>
       *
       * - O(1).
       * - Limited storage reads.
       * - One DB write (event).
       * - Weight of derivative `call` execution + T::WeightInfo::dispatch_as().
       *
       * # </weight>
       */
      dispatchAs: AugmentedSubmittable<
        (
          asOrigin:
            | MoonbeamRuntimeOriginCaller
            | { system: any }
            | { Void: any }
            | { Ethereum: any }
            | { CouncilCollective: any }
            | { TechCommitteeCollective: any }
            | { TreasuryCouncilCollective: any }
            | { CumulusXcm: any }
            | { PolkadotXcm: any }
            | string
            | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeOriginCaller, Call]
      >;
      /**
       * Send a batch of dispatch calls. Unlike `batch`, it allows errors and
       * won't interrupt.
       *
       * May be called from any origin.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of
       *   call must not exceed the constant: `batched_calls_limit` (available
       *   in constant metadata).
       *
       * If origin is root then call are dispatch without checking origin
       * filter. (This includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * # <weight>
       *
       * - Complexity: O(C) where C is the number of calls to be batched.
       *
       * # </weight>
       */
      forceBatch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    xcmTransactor: {
      /**
       * De-Register a derivative index. This prevents an account to use a
       * derivative address (represented by an index) from our of our sovereign
       * accounts anymore
       */
      deregister: AugmentedSubmittable<
        (index: u16 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u16]
      >;
      /**
       * Register a derivative index for an account id. Dispatchable by
       * DerivativeAddressRegistrationOrigin
       *
       * We do not store the derivative address, but only the index. We do not
       * need to store the derivative address to issue calls, only the index is enough
       *
       * For now an index is registered for all possible destinations and not
       * per-destination. We can change this in the future although it would
       * just make things more complicated
       */
      register: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          index: u16 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u16]
      >;
      /**
       * Remove the fee per second of an asset on its reserve chain
       */
      removeFeePerSecond: AugmentedSubmittable<
        (
          assetLocation: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation]
      >;
      /**
       * Remove the transact info of a location
       */
      removeTransactInfo: AugmentedSubmittable<
        (
          location: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation]
      >;
      /**
       * Set the fee per second of an asset on its reserve chain
       */
      setFeePerSecond: AugmentedSubmittable<
        (
          assetLocation:
            | XcmVersionedMultiLocation
            | { V0: any }
            | { V1: any }
            | string
            | Uint8Array,
          feePerSecond: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, u128]
      >;
      /**
       * Change the transact info of a location
       */
      setTransactInfo: AugmentedSubmittable<
        (
          location: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          transactExtraWeight: u64 | AnyNumber | Uint8Array,
          maxWeight: u64 | AnyNumber | Uint8Array,
          transactExtraWeightSigned: Option<u64> | null | object | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiLocation, u64, u64, Option<u64>]
      >;
      /**
       * Transact the inner call through a derivative account in a destination
       * chain, using 'fee_location' to pay for the fees. This fee_location is
       * given as a multilocation
       *
       * The caller needs to have the index registered in this pallet. The fee
       * multiasset needs to be a reserve asset for the destination
       * transactor::multilocation.
       */
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
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          MoonbeamRuntimeXcmConfigTransactors,
          u16,
          PalletXcmTransactorCurrencyPayment,
          Bytes,
          PalletXcmTransactorTransactWeights
        ]
      >;
      /**
       * Transact the call through the a signed origin in this chain that should
       * be converted to a transaction dispatch account in the destination chain
       * by any method implemented in the destination chains runtime
       *
       * This time we are giving the currency as a currencyId instead of multilocation
       */
      transactThroughSigned: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
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
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedMultiLocation,
          PalletXcmTransactorCurrencyPayment,
          Bytes,
          PalletXcmTransactorTransactWeights
        ]
      >;
      /**
       * Transact the call through the sovereign account in a destination chain,
       * 'fee_payer' pays for the fee
       *
       * SovereignAccountDispatcherOrigin callable only
       */
      transactThroughSovereign: AugmentedSubmittable<
        (
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          feePayer: AccountId20 | string | Uint8Array,
          fee:
            | PalletXcmTransactorCurrencyPayment
            | { currency?: any; feeAmount?: any }
            | string
            | Uint8Array,
          call: Bytes | string | Uint8Array,
          originKind:
            | XcmV0OriginKind
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
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedMultiLocation,
          AccountId20,
          PalletXcmTransactorCurrencyPayment,
          Bytes,
          XcmV0OriginKind,
          PalletXcmTransactorTransactWeights
        ]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    xTokens: {
      /**
       * Transfer native currencies.
       *
       * `dest_weight` is the weight for XCM execution on the dest chain, and it
       * would be charged from the transferred assets. If set below
       * requirements, the execution may fail and assets wouldn't be received.
       *
       * It's a no-op if any error on local XCM execution or message sending.
       * Note sending assets out per se doesn't guarantee they would be
       * received. Receiving depends on if the XCM message could be delivered by
       * the network, and if the receiving chain would handle messages correctly.
       */
      transfer: AugmentedSubmittable<
        (
          currencyId:
            | MoonbeamRuntimeXcmConfigCurrencyId
            | { SelfReserve: any }
            | { ForeignAsset: any }
            | { LocalAssetReserve: any }
            | string
            | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          destWeight: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeXcmConfigCurrencyId, u128, XcmVersionedMultiLocation, u64]
      >;
      /**
       * Transfer `MultiAsset`.
       *
       * `dest_weight` is the weight for XCM execution on the dest chain, and it
       * would be charged from the transferred assets. If set below
       * requirements, the execution may fail and assets wouldn't be received.
       *
       * It's a no-op if any error on local XCM execution or message sending.
       * Note sending assets out per se doesn't guarantee they would be
       * received. Receiving depends on if the XCM message could be delivered by
       * the network, and if the receiving chain would handle messages correctly.
       */
      transferMultiasset: AugmentedSubmittable<
        (
          asset: XcmVersionedMultiAsset | { V0: any } | { V1: any } | string | Uint8Array,
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          destWeight: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiAsset, XcmVersionedMultiLocation, u64]
      >;
      /**
       * Transfer several `MultiAsset` specifying the item to be used as fee
       *
       * `dest_weight` is the weight for XCM execution on the dest chain, and it
       * would be charged from the transferred assets. If set below
       * requirements, the execution may fail and assets wouldn't be received.
       *
       * `fee_item` is index of the MultiAssets that we want to use for payment
       *
       * It's a no-op if any error on local XCM execution or message sending.
       * Note sending assets out per se doesn't guarantee they would be
       * received. Receiving depends on if the XCM message could be delivered by
       * the network, and if the receiving chain would handle messages correctly.
       */
      transferMultiassets: AugmentedSubmittable<
        (
          assets: XcmVersionedMultiAssets | { V0: any } | { V1: any } | string | Uint8Array,
          feeItem: u32 | AnyNumber | Uint8Array,
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          destWeight: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiAssets, u32, XcmVersionedMultiLocation, u64]
      >;
      /**
       * Transfer `MultiAsset` specifying the fee and amount as separate.
       *
       * `dest_weight` is the weight for XCM execution on the dest chain, and it
       * would be charged from the transferred assets. If set below
       * requirements, the execution may fail and assets wouldn't be received.
       *
       * `fee` is the multiasset to be spent to pay for execution in destination
       * chain. Both fee and amount will be subtracted form the callers balance
       * For now we only accept fee and asset having the same `MultiLocation` id.
       *
       * If `fee` is not high enough to cover for the execution costs in the
       * destination chain, then the assets will be trapped in the destination chain
       *
       * It's a no-op if any error on local XCM execution or message sending.
       * Note sending assets out per se doesn't guarantee they would be
       * received. Receiving depends on if the XCM message could be delivered by
       * the network, and if the receiving chain would handle messages correctly.
       */
      transferMultiassetWithFee: AugmentedSubmittable<
        (
          asset: XcmVersionedMultiAsset | { V0: any } | { V1: any } | string | Uint8Array,
          fee: XcmVersionedMultiAsset | { V0: any } | { V1: any } | string | Uint8Array,
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          destWeight: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedMultiAsset, XcmVersionedMultiAsset, XcmVersionedMultiLocation, u64]
      >;
      /**
       * Transfer several currencies specifying the item to be used as fee
       *
       * `dest_weight` is the weight for XCM execution on the dest chain, and it
       * would be charged from the transferred assets. If set below
       * requirements, the execution may fail and assets wouldn't be received.
       *
       * `fee_item` is index of the currencies tuple that we want to use for payment
       *
       * It's a no-op if any error on local XCM execution or message sending.
       * Note sending assets out per se doesn't guarantee they would be
       * received. Receiving depends on if the XCM message could be delivered by
       * the network, and if the receiving chain would handle messages correctly.
       */
      transferMulticurrencies: AugmentedSubmittable<
        (
          currencies:
            | Vec<ITuple<[MoonbeamRuntimeXcmConfigCurrencyId, u128]>>
            | [
                (
                  | MoonbeamRuntimeXcmConfigCurrencyId
                  | { SelfReserve: any }
                  | { ForeignAsset: any }
                  | { LocalAssetReserve: any }
                  | string
                  | Uint8Array
                ),
                u128 | AnyNumber | Uint8Array
              ][],
          feeItem: u32 | AnyNumber | Uint8Array,
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          destWeight: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          Vec<ITuple<[MoonbeamRuntimeXcmConfigCurrencyId, u128]>>,
          u32,
          XcmVersionedMultiLocation,
          u64
        ]
      >;
      /**
       * Transfer native currencies specifying the fee and amount as separate.
       *
       * `dest_weight` is the weight for XCM execution on the dest chain, and it
       * would be charged from the transferred assets. If set below
       * requirements, the execution may fail and assets wouldn't be received.
       *
       * `fee` is the amount to be spent to pay for execution in destination
       * chain. Both fee and amount will be subtracted form the callers balance.
       *
       * If `fee` is not high enough to cover for the execution costs in the
       * destination chain, then the assets will be trapped in the destination chain
       *
       * It's a no-op if any error on local XCM execution or message sending.
       * Note sending assets out per se doesn't guarantee they would be
       * received. Receiving depends on if the XCM message could be delivered by
       * the network, and if the receiving chain would handle messages correctly.
       */
      transferWithFee: AugmentedSubmittable<
        (
          currencyId:
            | MoonbeamRuntimeXcmConfigCurrencyId
            | { SelfReserve: any }
            | { ForeignAsset: any }
            | { LocalAssetReserve: any }
            | string
            | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          fee: u128 | AnyNumber | Uint8Array,
          dest: XcmVersionedMultiLocation | { V0: any } | { V1: any } | string | Uint8Array,
          destWeight: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbeamRuntimeXcmConfigCurrencyId, u128, u128, XcmVersionedMultiLocation, u64]
      >;
      /**
       * Generic tx
       */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
  } // AugmentedSubmittables
} // declare module
