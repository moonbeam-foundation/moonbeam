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
  AccountEthereumSignature,
  CumulusPrimitivesCoreAggregateMessageOrigin,
  CumulusPrimitivesParachainInherentParachainInherentData,
  EthereumTransactionTransactionV2,
  FrameSupportPreimagesBounded,
  FrameSupportScheduleDispatchTime,
  MoonbaseRuntimeAssetConfigAssetRegistrarMetadata,
  MoonbaseRuntimeOriginCaller,
  MoonbaseRuntimeProxyType,
  MoonbaseRuntimeRuntimeParamsRuntimeParameters,
  MoonbaseRuntimeXcmConfigAssetType,
  MoonbaseRuntimeXcmConfigCurrencyId,
  MoonbaseRuntimeXcmConfigTransactors,
  NimbusPrimitivesNimbusCryptoPublic,
  PalletBalancesAdjustmentDirection,
  PalletConvictionVotingConviction,
  PalletConvictionVotingVoteAccountVote,
  PalletIdentityJudgement,
  PalletIdentityLegacyIdentityInfo,
  PalletMultisigTimepoint,
  PalletXcmTransactorCurrencyPayment,
  PalletXcmTransactorHrmpOperation,
  PalletXcmTransactorTransactWeights,
  SpRuntimeMultiSignature,
  SpWeightsWeightV2Weight,
  StagingXcmExecutorAssetTransferTransferType,
  StagingXcmV4Location,
  XcmPrimitivesEthereumXcmEthereumXcmTransaction,
  XcmV2OriginKind,
  XcmV3WeightLimit,
  XcmVersionedAsset,
  XcmVersionedAssetId,
  XcmVersionedAssets,
  XcmVersionedLocation,
  XcmVersionedXcm,
} from "@polkadot/types/lookup";

export type __AugmentedSubmittable = AugmentedSubmittable<() => unknown>;
export type __SubmittableExtrinsic<ApiType extends ApiTypes> = SubmittableExtrinsic<ApiType>;
export type __SubmittableExtrinsicFunction<ApiType extends ApiTypes> =
  SubmittableExtrinsicFunction<ApiType>;

declare module "@polkadot/api-base/types/submittable" {
  interface AugmentedSubmittables<ApiType extends ApiTypes> {
    assetManager: {
      /**
       * Change the xcm type mapping for a given assetId We also change this if the previous units
       * per second where pointing at the old assetType
       */
      changeExistingAssetType: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          newAssetType: MoonbaseRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, MoonbaseRuntimeXcmConfigAssetType, u32]
      >;
      /**
       * Destroy a given foreign assetId The weight in this case is the one returned by the trait
       * plus the db writes and reads from removing all the associated data
       */
      destroyForeignAsset: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, u32]
      >;
      /** Register new asset with the asset manager */
      registerForeignAsset: AugmentedSubmittable<
        (
          asset: MoonbaseRuntimeXcmConfigAssetType | { Xcm: any } | string | Uint8Array,
          metadata:
            | MoonbaseRuntimeAssetConfigAssetRegistrarMetadata
            | { name?: any; symbol?: any; decimals?: any; isFrozen?: any }
            | string
            | Uint8Array,
          minAmount: u128 | AnyNumber | Uint8Array,
          isSufficient: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          MoonbaseRuntimeXcmConfigAssetType,
          MoonbaseRuntimeAssetConfigAssetRegistrarMetadata,
          u128,
          bool
        ]
      >;
      /** Remove a given assetId -> assetType association */
      removeExistingAssetType: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          numAssetsWeightHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    assets: {
      /**
       * Approve an amount of asset for transfer by a delegated third-party account.
       *
       * Origin must be Signed.
       *
       * Ensures that `ApprovalDeposit` worth of `Currency` is reserved from signing account for the
       * purpose of holding the approval. If some non-zero amount of assets is already approved from
       * signing account to `delegate`, then it is topped up or unreserved to meet the right value.
       *
       * NOTE: The signing account does not need to own `amount` of assets at the point of making this call.
       *
       * - `id`: The identifier of the asset.
       * - `delegate`: The account to delegate permission to transfer asset.
       * - `amount`: The amount of asset that may be transferred by `delegate`. If there is already an
       *   approval in place, then this acts additively.
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
       * Disallow further unprivileged transfers of an asset `id` to and from an account `who`.
       *
       * Origin must be Signed and the sender should be the Freezer of the asset `id`.
       *
       * - `id`: The identifier of the account's asset.
       * - `who`: The account to be unblocked.
       *
       * Emits `Blocked`.
       *
       * Weight: `O(1)`
       */
      block: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
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
       * Emits `Burned` with the actual amount burned. If this takes the balance to below the
       * minimum for the asset, then the amount burned is increased to take it to zero.
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
       * Cancel all of some asset approved for delegated transfer by a third-party account.
       *
       * Origin must be Signed and there must be an approval in place between signer and `delegate`.
       *
       * Unreserves any deposit previously reserved by `approve_transfer` for the approval.
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
       * The origin must conform to the configured `CreateOrigin` and have sufficient funds free.
       *
       * Funds of sender are reserved by `AssetDeposit`.
       *
       * Parameters:
       *
       * - `id`: The identifier of the new asset. This must not be currently in use to identify an
       *   existing asset.
       * - `admin`: The admin of this class of assets. The admin is the initial address of each member
       *   of the asset class's admin team.
       * - `min_balance`: The minimum balance of this new asset that any single account must have. If
       *   an account's balance is reduced below this, then it collapses to zero.
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
       * Destroy all accounts associated with a given asset.
       *
       * `destroy_accounts` should only be called after `start_destroy` has been called, and the
       * asset is in a `Destroying` state.
       *
       * Due to weight restrictions, this function may need to be called multiple times to fully
       * destroy all accounts. It will destroy `RemoveItemsLimit` accounts at a time.
       *
       * - `id`: The identifier of the asset to be destroyed. This must identify an existing asset.
       *
       * Each call emits the `Event::DestroyedAccounts` event.
       */
      destroyAccounts: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Destroy all approvals associated with a given asset up to the max (T::RemoveItemsLimit).
       *
       * `destroy_approvals` should only be called after `start_destroy` has been called, and the
       * asset is in a `Destroying` state.
       *
       * Due to weight restrictions, this function may need to be called multiple times to fully
       * destroy all approvals. It will destroy `RemoveItemsLimit` approvals at a time.
       *
       * - `id`: The identifier of the asset to be destroyed. This must identify an existing asset.
       *
       * Each call emits the `Event::DestroyedApprovals` event.
       */
      destroyApprovals: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Complete destroying asset and unreserve currency.
       *
       * `finish_destroy` should only be called after `start_destroy` has been called, and the asset
       * is in a `Destroying` state. All accounts or approvals should be destroyed before hand.
       *
       * - `id`: The identifier of the asset to be destroyed. This must identify an existing asset.
       *
       * Each successful call emits the `Event::Destroyed` event.
       */
      finishDestroy: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
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
       * - `min_balance`: The minimum balance of this new asset that any single account must have. If
       *   an account's balance is reduced below this, then it collapses to zero.
       * - `is_sufficient`: Whether a non-zero balance of this asset is deposit of sufficient value to
       *   account for the state bloat associated with its balance storage. If set to `true`, then
       *   non-zero balances may be stored without a `consumer` reference (and thus an ED in the
       *   Balances pallet or whatever else is used to control user-account state growth).
       * - `is_frozen`: Whether this asset class is frozen except for permissioned/admin instructions.
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
       * Cancel all of some asset approved for delegated transfer by a third-party account.
       *
       * Origin must be either ForceOrigin or Signed origin with the signer being the Admin account
       * of the asset `id`.
       *
       * Unreserves any deposit previously reserved by `approve_transfer` for the approval.
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
       * - `id`: The identifier of the new asset. This must not be currently in use to identify an
       *   existing asset.
       * - `owner`: The owner of this class of assets. The owner has full superuser permissions over
       *   this asset, but may later change and configure the permissions using `transfer_ownership`
       *   and `set_team`.
       * - `min_balance`: The minimum balance of this new asset that any single account must have. If
       *   an account's balance is reduced below this, then it collapses to zero.
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
       * Weight: `O(N + S)` where N and S are the length of the name and symbol respectively.
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
       * - `amount`: The amount by which the `source`'s balance of assets should be reduced and
       *   `dest`'s balance increased. The amount actually transferred may be slightly greater in
       *   the case that the transfer would otherwise take the `source` balance above zero but below
       *   the minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes the source balance to
       * below the minimum for the asset, then the amount transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `dest`; Post-existence of `source`; Account
       * pre-existence of `dest`.
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
       * Disallow further unprivileged transfers of an asset `id` from an account `who`. `who` must
       * already exist as an entry in `Account`s of the asset. If you want to freeze an account that
       * does not have an entry, use `touch_other` first.
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
       * Weight: `O(1)` Modes: Pre-existing balance of `beneficiary`; Account pre-existence of `beneficiary`.
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
       * Return the deposit (if any) of an asset account or a consumer reference (if any) of an account.
       *
       * The origin must be Signed.
       *
       * - `id`: The identifier of the asset for which the caller would like the deposit refunded.
       * - `allow_burn`: If `true` then assets may be destroyed in order to complete the refund.
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
       * Return the deposit (if any) of a target asset account. Useful if you are the depositor.
       *
       * The origin must be Signed and either the account owner, depositor, or asset `Admin`. In
       * order to burn a non-zero balance of the asset, the caller must be the account and should
       * use `refund`.
       *
       * - `id`: The identifier of the asset for the account holding a deposit.
       * - `who`: The account to refund.
       *
       * Emits `Refunded` event when successful.
       */
      refundOther: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Set the metadata for an asset.
       *
       * Origin must be Signed and the sender should be the Owner of the asset `id`.
       *
       * Funds of sender are reserved according to the formula: `MetadataDepositBase +
       * MetadataDepositPerByte * (name.len + symbol.len)` taking into account any already reserved funds.
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
       * Sets the minimum balance of an asset.
       *
       * Only works if there aren't any accounts that are holding the asset or if the new value of
       * `min_balance` is less than the old one.
       *
       * Origin must be Signed and the sender has to be the Owner of the asset `id`.
       *
       * - `id`: The identifier of the asset.
       * - `min_balance`: The new value of `min_balance`.
       *
       * Emits `AssetMinBalanceChanged` event when successful.
       */
      setMinBalance: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          minBalance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, u128]
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
       * Start the process of destroying a fungible asset class.
       *
       * `start_destroy` is the first in a series of extrinsics that should be called, to allow
       * destruction of an asset class.
       *
       * The origin must conform to `ForceOrigin` or must be `Signed` by the asset's `owner`.
       *
       * - `id`: The identifier of the asset to be destroyed. This must identify an existing asset.
       *
       * The asset class must be frozen before calling `start_destroy`.
       */
      startDestroy: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Allow unprivileged transfers to and from an account again.
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
       * - `origin`: Must be Signed; the signer account must have sufficient funds for a deposit to be taken.
       * - `id`: The identifier of the asset for the account to be created.
       *
       * Emits `Touched` event when successful.
       */
      touch: AugmentedSubmittable<
        (id: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >;
      /**
       * Create an asset account for `who`.
       *
       * A deposit will be taken from the signer account.
       *
       * - `origin`: Must be Signed by `Freezer` or `Admin` of the asset `id`; the signer account must
       *   have sufficient funds for a deposit to be taken.
       * - `id`: The identifier of the asset for the account to be created.
       * - `who`: The account to be created.
       *
       * Emits `Touched` event when successful.
       */
      touchOther: AugmentedSubmittable<
        (
          id: Compact<u128> | AnyNumber | Uint8Array,
          who: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Move some assets from the sender account to another.
       *
       * Origin must be Signed.
       *
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `target`: The account to be credited.
       * - `amount`: The amount by which the sender's balance of assets should be reduced and
       *   `target`'s balance increased. The amount actually transferred may be slightly greater in
       *   the case that the transfer would otherwise take the sender balance above zero but below
       *   the minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes the source balance to
       * below the minimum for the asset, then the amount transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `target`; Post-existence of sender; Account
       * pre-existence of `target`.
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
       * Transfer some asset balance from a previously delegated account to some third-party account.
       *
       * Origin must be Signed and there must be an approval in place by the `owner` to the signer.
       *
       * If the entire amount approved for transfer is transferred, then any deposit previously
       * reserved by `approve_transfer` is unreserved.
       *
       * - `id`: The identifier of the asset.
       * - `owner`: The account which previously approved for a transfer of at least `amount` and from
       *   which the asset balance will be withdrawn.
       * - `destination`: The account to which the asset balance of `amount` will be transferred.
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
       * Move some assets from the sender account to another, keeping the sender account alive.
       *
       * Origin must be Signed.
       *
       * - `id`: The identifier of the asset to have some amount transferred.
       * - `target`: The account to be credited.
       * - `amount`: The amount by which the sender's balance of assets should be reduced and
       *   `target`'s balance increased. The amount actually transferred may be slightly greater in
       *   the case that the transfer would otherwise take the sender balance above zero but below
       *   the minimum balance. Must be greater than zero.
       *
       * Emits `Transferred` with the actual amount transferred. If this takes the source balance to
       * below the minimum for the asset, then the amount transferred is increased to take it to zero.
       *
       * Weight: `O(1)` Modes: Pre-existence of `target`; Post-existence of sender; Account
       * pre-existence of `target`.
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
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorFilter: {
      /** Update the eligible count. Intended to be called by governance. */
      setEligible: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorInherent: {
      /**
       * This inherent is a workaround to run code after the "real" inherents have executed, but
       * before transactions are executed.
       */
      kickOffAuthorshipValidation: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorMapping: {
      /**
       * Register your NimbusId onchain so blocks you author are associated with your account.
       *
       * Users who have been (or will soon be) elected active collators in staking, should submit
       * this extrinsic to have their blocks accepted and earn rewards.
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
       * This is useful when you are no longer an author and would like to re-claim your security deposit.
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
       * This is useful when you are no longer an author and would like to re-claim your security deposit.
       */
      removeKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Set association and session keys at once.
       *
       * This is useful for key rotation to update Nimbus and VRF keys in one call. No new security
       * deposit is required. Will replace `update_association` which is kept now for backwards
       * compatibility reasons.
       */
      setKeys: AugmentedSubmittable<
        (keys: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Change your Mapping.
       *
       * This is useful for normal key rotation or for when switching from one physical collator
       * machine to another. No new security deposit is required. This sets keys to
       * new_nimbus_id.into() by default.
       */
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
      /**
       * Adjust the total issuance in a saturating way.
       *
       * Can only be called by root and always needs a positive `delta`.
       *
       * # Example
       */
      forceAdjustTotalIssuance: AugmentedSubmittable<
        (
          direction:
            | PalletBalancesAdjustmentDirection
            | "Increase"
            | "Decrease"
            | number
            | Uint8Array,
          delta: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletBalancesAdjustmentDirection, Compact<u128>]
      >;
      /**
       * Set the regular balance of a given account.
       *
       * The dispatch origin for this call is `root`.
       */
      forceSetBalance: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          newFree: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>]
      >;
      /**
       * Exactly as `transfer_allow_death`, except the origin must be root and the source account
       * may be specified.
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
       * Transfer the entire transferable balance from the caller account.
       *
       * NOTE: This function only attempts to transfer _transferable_ balances. This means that any
       * locked, reserved, or existential deposits (when `keep_alive` is `true`), will not be
       * transferred by this function. To ensure that this function results in a killed account, you
       * might need to prepare the account by removing any reference counters, storage deposits, etc...
       *
       * The dispatch origin of this call must be Signed.
       *
       * - `dest`: The recipient of the transfer.
       * - `keep_alive`: A boolean to determine if the `transfer_all` operation should send all of the
       *   funds the account has, causing the sender account to be killed (false), or transfer
       *   everything except at least the existential deposit, which will guarantee to keep the
       *   sender account alive (true).
       */
      transferAll: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          keepAlive: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, bool]
      >;
      /**
       * Transfer some liquid free balance to another account.
       *
       * `transfer_allow_death` will set the `FreeBalance` of the sender and receiver. If the
       * sender's account is below the existential deposit as a result of the transfer, the account
       * will be reaped.
       *
       * The dispatch origin for this call must be `Signed` by the transactor.
       */
      transferAllowDeath: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>]
      >;
      /**
       * Same as the [`transfer_allow_death`][`transfer_allow_death`] call, but with a check that
       * the transfer will not kill the origin account.
       *
       * 99% of the time you want [`transfer_allow_death`][`transfer_allow_death`] instead.
       *
       * [`transfer_allow_death`]: struct.Pallet.html#method.transfer
       */
      transferKeepAlive: AugmentedSubmittable<
        (
          dest: AccountId20 | string | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Compact<u128>]
      >;
      /**
       * Upgrade a specified account.
       *
       * - `origin`: Must be `Signed`.
       * - `who`: The account to be upgraded.
       *
       * This will waive the transaction fee if at least all but 10% of the accounts needed to be
       * upgraded. (We let some not have to be upgraded just in order to allow for the possibility of churn).
       */
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
      /**
       * Delegate the voting power (with some given conviction) of the sending account for a
       * particular class of polls.
       *
       * The balance delegated is locked for as long as it's delegated, and thereafter for the time
       * appropriate for the conviction's lock period.
       *
       * The dispatch origin of this call must be _Signed_, and the signing account must either:
       *
       * - Be delegating already; or
       * - Have no voting activity (if there is, then it will need to be removed through `remove_vote`).
       * - `to`: The account whose voting the `target` account's voting power will follow.
       * - `class`: The class of polls to delegate. To delegate multiple classes, multiple calls to
       *   this function are required.
       * - `conviction`: The conviction that will be attached to the delegated votes. When the account
       *   is undelegated, the funds will be locked for the corresponding period.
       * - `balance`: The amount of the account's balance to be used in delegating. This must not be
       *   more than the account's current balance.
       *
       * Emits `Delegated`.
       *
       * Weight: `O(R)` where R is the number of polls the voter delegating to has voted on. Weight
       * is initially charged as if maximum votes, but is refunded later.
       */
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
      /**
       * Remove a vote for a poll.
       *
       * If the `target` is equal to the signer, then this function is exactly equivalent to
       * `remove_vote`. If not equal to the signer, then the vote must have expired, either because
       * the poll was cancelled, because the voter lost the poll or because the conviction period is over.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `target`: The account of the vote to be removed; this account must have voted for poll `index`.
       * - `index`: The index of poll of the vote to be removed.
       * - `class`: The class of the poll.
       *
       * Weight: `O(R + log R)` where R is the number of polls that `target` has voted on. Weight is
       * calculated for the maximum number of vote.
       */
      removeOtherVote: AugmentedSubmittable<
        (
          target: AccountId20 | string | Uint8Array,
          clazz: u16 | AnyNumber | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u16, u32]
      >;
      /**
       * Remove a vote for a poll.
       *
       * If:
       *
       * - The poll was cancelled, or
       * - The poll is ongoing, or
       * - The poll has ended such that
       * - The vote of the account was in opposition to the result; or
       * - There was no conviction to the account's vote; or
       * - The account made a split vote ...then the vote is removed cleanly and a following call to
       *   `unlock` may result in more funds being available.
       *
       * If, however, the poll has ended and:
       *
       * - It finished corresponding to the vote of the account, and
       * - The account made a standard vote with conviction, and
       * - The lock period of the conviction is not over ...then the lock will be aggregated into the
       *   overall account's lock, which may involve _overlocking_ (where the two locks are combined
       *   into a single lock that is the maximum of both the amount locked and the time is it locked for).
       *
       * The dispatch origin of this call must be _Signed_, and the signer must have a vote
       * registered for poll `index`.
       *
       * - `index`: The index of poll of the vote to be removed.
       * - `class`: Optional parameter, if given it indicates the class of the poll. For polls which
       *   have finished or are cancelled, this must be `Some`.
       *
       * Weight: `O(R + log R)` where R is the number of polls that `target` has voted on. Weight is
       * calculated for the maximum number of vote.
       */
      removeVote: AugmentedSubmittable<
        (
          clazz: Option<u16> | null | Uint8Array | u16 | AnyNumber,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Option<u16>, u32]
      >;
      /**
       * Undelegate the voting power of the sending account for a particular class of polls.
       *
       * Tokens may be unlocked following once an amount of time consistent with the lock period of
       * the conviction with which the delegation was issued has passed.
       *
       * The dispatch origin of this call must be _Signed_ and the signing account must be currently
       * delegating.
       *
       * - `class`: The class of polls to remove the delegation from.
       *
       * Emits `Undelegated`.
       *
       * Weight: `O(R)` where R is the number of polls the voter delegating to has voted on. Weight
       * is initially charged as if maximum votes, but is refunded later.
       */
      undelegate: AugmentedSubmittable<
        (clazz: u16 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u16]
      >;
      /**
       * Remove the lock caused by prior voting/delegating which has expired within a particular class.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `class`: The class of polls to unlock.
       * - `target`: The account to remove the lock on.
       *
       * Weight: `O(R)` with R number of vote of target.
       */
      unlock: AugmentedSubmittable<
        (
          clazz: u16 | AnyNumber | Uint8Array,
          target: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, AccountId20]
      >;
      /**
       * Vote in a poll. If `vote.is_aye()`, the vote is to enact the proposal; otherwise it is a
       * vote to keep the status quo.
       *
       * The dispatch origin of this call must be _Signed_.
       *
       * - `poll_index`: The index of the poll to vote for.
       * - `vote`: The vote configuration.
       *
       * Weight: `O(R)` where R is the number of polls the voter has voted on.
       */
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
      /**
       * Associate a native rewards_destination identity with a crowdloan contribution.
       *
       * The caller needs to provide the unassociated relay account and a proof to succeed with the
       * association The proof is nothing but a signature over the reward_address using the relay keys
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
       * The number of valid proofs needs to be bigger than 'RewardAddressRelayVoteThreshold' The
       * account to be changed needs to be submitted as 'previous_account' Origin must be
       * RewardAddressChangeOrigin
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
      /** Collect whatever portion of your reward are currently vested. */
      claim: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * This extrinsic completes the initialization if some checks are fullfiled. These checks are:
       * -The reward contribution money matches the crowdloan pot -The end vesting block is higher
       * than the init vesting block -The initialization has not complete yet
       */
      completeInitialization: AugmentedSubmittable<
        (leaseEndingBlock: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Initialize the reward distribution storage. It shortcuts whenever an error is found This
       * does not enforce any checks other than making sure we dont go over funds
       * complete_initialization should perform any additional
       */
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
      /** Update reward address, proving that the caller owns the current native key */
      updateRewardAddress: AugmentedSubmittable<
        (newRewardAccount: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    dmpQueue: {
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    emergencyParaXcm: {
      /** Authorize a runtime upgrade. Only callable in `Paused` mode */
      fastAuthorizeUpgrade: AugmentedSubmittable<
        (codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** Resume `Normal` mode */
      pausedToNormal: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    ethereum: {
      /** Transact an Ethereum transaction. */
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
      /**
       * Xcm Transact an Ethereum transaction, but allow to force the caller and create address.
       * This call should be restricted (callable only by the runtime or governance). Weight: Gas
       * limit plus the db reads involving the suspension and proxy checks
       */
      forceTransactAs: AugmentedSubmittable<
        (
          transactAs: H160 | string | Uint8Array,
          xcmTransaction:
            | XcmPrimitivesEthereumXcmEthereumXcmTransaction
            | { V1: any }
            | { V2: any }
            | string
            | Uint8Array,
          forceCreateAddress: Option<H160> | null | Uint8Array | H160 | string
        ) => SubmittableExtrinsic<ApiType>,
        [H160, XcmPrimitivesEthereumXcmEthereumXcmTransaction, Option<H160>]
      >;
      /**
       * Resumes all Ethereum executions from XCM.
       *
       * - `origin`: Must pass `ControllerOrigin`.
       */
      resumeEthereumXcmExecution: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Suspends all Ethereum executions from XCM.
       *
       * - `origin`: Must pass `ControllerOrigin`.
       */
      suspendEthereumXcmExecution: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Xcm Transact an Ethereum transaction. Weight: Gas limit plus the db read involving the
       * suspension check
       */
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
      /**
       * Xcm Transact an Ethereum transaction through proxy. Weight: Gas limit plus the db reads
       * involving the suspension and proxy checks
       */
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
      /** Issue an EVM call operation. This is similar to a message call transaction in Ethereum. */
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
      /** Issue an EVM create operation. This is similar to a contract creation transaction in Ethereum. */
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
      /** Issue an EVM create2 operation. */
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
      /** Withdraw balance from EVM into currency/balances pallet. */
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
    evmForeignAssets: {
      /**
       * Change the xcm type mapping for a given assetId We also change this if the previous units
       * per second where pointing at the old assetType
       */
      changeXcmLocation: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          newXcmLocation:
            | StagingXcmV4Location
            | { parents?: any; interior?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, StagingXcmV4Location]
      >;
      /** Create new asset with the ForeignAssetCreator */
      createForeignAsset: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          xcmLocation:
            | StagingXcmV4Location
            | { parents?: any; interior?: any }
            | string
            | Uint8Array,
          decimals: u8 | AnyNumber | Uint8Array,
          symbol: Bytes | string | Uint8Array,
          name: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, StagingXcmV4Location, u8, Bytes, Bytes]
      >;
      /** Freeze a given foreign assetId */
      freezeForeignAsset: AugmentedSubmittable<
        (
          assetId: u128 | AnyNumber | Uint8Array,
          allowXcmDeposit: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, bool]
      >;
      /** Unfreeze a given foreign assetId */
      unfreezeForeignAsset: AugmentedSubmittable<
        (assetId: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u128]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    identity: {
      /**
       * Accept a given username that an `authority` granted. The call must include the full
       * username, as in `username.suffix`.
       */
      acceptUsername: AugmentedSubmittable<
        (username: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Add a registrar to the system.
       *
       * The dispatch origin for this call must be `T::RegistrarOrigin`.
       *
       * - `account`: the account of the registrar.
       *
       * Emits `RegistrarAdded` if successful.
       */
      addRegistrar: AugmentedSubmittable<
        (account: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Add the given account to the sender's subs.
       *
       * Payment: Balance reserved by a previous `set_subs` call for one sub will be repatriated to
       * the sender.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must have a registered
       * sub identity of `sub`.
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
       * Add an `AccountId` with permission to grant usernames with a given `suffix` appended.
       *
       * The authority can grant up to `allocation` usernames. To top up their allocation, they
       * should just issue (or request via governance) a new `add_username_authority` call.
       */
      addUsernameAuthority: AugmentedSubmittable<
        (
          authority: AccountId20 | string | Uint8Array,
          suffix: Bytes | string | Uint8Array,
          allocation: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Bytes, u32]
      >;
      /**
       * Cancel a previous request.
       *
       * Payment: A previously reserved deposit is returned on success.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must have a registered identity.
       *
       * - `reg_index`: The index of the registrar whose judgement is no longer requested.
       *
       * Emits `JudgementUnrequested` if successful.
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
       * The dispatch origin for this call must be _Signed_ and the sender must have a registered identity.
       *
       * Emits `IdentityCleared` if successful.
       */
      clearIdentity: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Remove an account's identity and sub-account information and slash the deposits.
       *
       * Payment: Reserved balances from `set_subs` and `set_identity` are slashed and handled by
       * `Slash`. Verification request deposits are not returned; they should be cancelled manually
       * using `cancel_request`.
       *
       * The dispatch origin for this call must match `T::ForceOrigin`.
       *
       * - `target`: the account whose identity the judgement is upon. This must be an account with a
       *   registered identity.
       *
       * Emits `IdentityKilled` if successful.
       */
      killIdentity: AugmentedSubmittable<
        (target: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Provide a judgement for an account's identity.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must be the account of
       * the registrar whose index is `reg_index`.
       *
       * - `reg_index`: the index of the registrar whose judgement is being made.
       * - `target`: the account whose identity the judgement is upon. This must be an account with a
       *   registered identity.
       * - `judgement`: the judgement of the registrar of index `reg_index` about `target`.
       * - `identity`: The hash of the [`IdentityInformationProvider`] for that the judgement is provided.
       *
       * Note: Judgements do not apply to a username.
       *
       * Emits `JudgementGiven` if successful.
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
            | Uint8Array,
          identity: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, AccountId20, PalletIdentityJudgement, H256]
      >;
      /**
       * Remove the sender as a sub-account.
       *
       * Payment: Balance reserved by a previous `set_subs` call for one sub will be repatriated to
       * the sender (_not_ the original depositor).
       *
       * The dispatch origin for this call must be _Signed_ and the sender must have a registered
       * super-identity.
       *
       * NOTE: This should not normally be used, but is provided in the case that the non-
       * controller of an account is maliciously registered as a sub-account.
       */
      quitSub: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Remove a username that corresponds to an account with no identity. Exists when a user gets
       * a username but then calls `clear_identity`.
       */
      removeDanglingUsername: AugmentedSubmittable<
        (username: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Remove an expired username approval. The username was approved by an authority but never
       * accepted by the user and must now be beyond its expiration. The call must include the full
       * username, as in `username.suffix`.
       */
      removeExpiredApproval: AugmentedSubmittable<
        (username: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Remove the given account from the sender's subs.
       *
       * Payment: Balance reserved by a previous `set_subs` call for one sub will be repatriated to
       * the sender.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must have a registered
       * sub identity of `sub`.
       */
      removeSub: AugmentedSubmittable<
        (sub: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Remove `authority` from the username authorities. */
      removeUsernameAuthority: AugmentedSubmittable<
        (authority: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Alter the associated name of the given sub-account.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must have a registered
       * sub identity of `sub`.
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
       * Payment: At most `max_fee` will be reserved for payment to the registrar if judgement given.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must have a registered identity.
       *
       * - `reg_index`: The index of the registrar whose judgement is requested.
       * - `max_fee`: The maximum fee that may be paid. This should just be auto-populated as:
       *
       * ```nocompile
       * Self::registrars().get(reg_index).unwrap().fee;
       * ```
       *
       * Emits `JudgementRequested` if successful.
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
       * The dispatch origin for this call must be _Signed_ and the sender must be the account of
       * the registrar whose index is `index`.
       *
       * - `index`: the index of the registrar whose fee is to be set.
       * - `new`: the new account ID.
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
       * The dispatch origin for this call must be _Signed_ and the sender must be the account of
       * the registrar whose index is `index`.
       *
       * - `index`: the index of the registrar whose fee is to be set.
       * - `fee`: the new fee.
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
       * The dispatch origin for this call must be _Signed_ and the sender must be the account of
       * the registrar whose index is `index`.
       *
       * - `index`: the index of the registrar whose fee is to be set.
       * - `fields`: the fields that the registrar concerns themselves with.
       */
      setFields: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          fields: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, u64]
      >;
      /**
       * Set an account's identity information and reserve the appropriate deposit.
       *
       * If the account already has identity information, the deposit is taken as part payment for
       * the new deposit.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * - `info`: The identity information.
       *
       * Emits `IdentitySet` if successful.
       */
      setIdentity: AugmentedSubmittable<
        (
          info:
            | PalletIdentityLegacyIdentityInfo
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
        [PalletIdentityLegacyIdentityInfo]
      >;
      /** Set a given username as the primary. The username should include the suffix. */
      setPrimaryUsername: AugmentedSubmittable<
        (username: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the sub-accounts of the sender.
       *
       * Payment: Any aggregate balance reserved by previous `set_subs` calls will be returned and
       * an amount `SubAccountDeposit` will be reserved for each item in `subs`.
       *
       * The dispatch origin for this call must be _Signed_ and the sender must have a registered identity.
       *
       * - `subs`: The identity's (new) sub-accounts.
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
       * Set the username for `who`. Must be called by a username authority.
       *
       * The authority must have an `allocation`. Users can either pre-sign their usernames or
       * accept them later.
       *
       * Usernames must:
       *
       * - Only contain lowercase ASCII characters or digits.
       * - When combined with the suffix of the issuing authority be _less than_ the `MaxUsernameLength`.
       */
      setUsernameFor: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          username: Bytes | string | Uint8Array,
          signature:
            | Option<AccountEthereumSignature>
            | null
            | Uint8Array
            | AccountEthereumSignature
            | string
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Bytes, Option<AccountEthereumSignature>]
      >;
      /** Generic tx */
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
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    messageQueue: {
      /**
       * Execute an overweight message.
       *
       * Temporary processing errors will be propagated whereas permanent errors are treated as
       * success condition.
       *
       * - `origin`: Must be `Signed`.
       * - `message_origin`: The origin from which the message to be executed arrived.
       * - `page`: The page in the queue in which the message to be executed is sitting.
       * - `index`: The index into the queue of the message to be executed.
       * - `weight_limit`: The maximum amount of weight allowed to be consumed in the execution of the message.
       *
       * Benchmark complexity considerations: O(index + weight_limit).
       */
      executeOverweight: AugmentedSubmittable<
        (
          messageOrigin:
            | CumulusPrimitivesCoreAggregateMessageOrigin
            | { Here: any }
            | { Parent: any }
            | { Sibling: any }
            | string
            | Uint8Array,
          page: u32 | AnyNumber | Uint8Array,
          index: u32 | AnyNumber | Uint8Array,
          weightLimit:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [CumulusPrimitivesCoreAggregateMessageOrigin, u32, u32, SpWeightsWeightV2Weight]
      >;
      /** Remove a page which has no more messages remaining to be processed or is stale. */
      reapPage: AugmentedSubmittable<
        (
          messageOrigin:
            | CumulusPrimitivesCoreAggregateMessageOrigin
            | { Here: any }
            | { Parent: any }
            | { Sibling: any }
            | string
            | Uint8Array,
          pageIndex: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [CumulusPrimitivesCoreAggregateMessageOrigin, u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    moonbeamLazyMigrations: {
      clearSuicidedStorage: AugmentedSubmittable<
        (
          addresses: Vec<H160> | (H160 | string | Uint8Array)[],
          limit: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<H160>, u32]
      >;
      createContractMetadata: AugmentedSubmittable<
        (address: H160 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H160]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    moonbeamOrbiters: {
      /** Add a collator to orbiters program. */
      addCollator: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Add an orbiter in a collator pool */
      collatorAddOrbiter: AugmentedSubmittable<
        (orbiter: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Remove an orbiter from the caller collator pool */
      collatorRemoveOrbiter: AugmentedSubmittable<
        (orbiter: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Remove the caller from the specified collator pool */
      orbiterLeaveCollatorPool: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Registering as an orbiter */
      orbiterRegister: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Deregistering from orbiters */
      orbiterUnregister: AugmentedSubmittable<
        (collatorsPoolCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Remove a collator from orbiters program. */
      removeCollator: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    multisig: {
      /**
       * Register approval for a dispatch to be made from a deterministic composite account if
       * approved by a total of `threshold - 1` of `other_signatories`.
       *
       * Payment: `DepositBase` will be reserved if this is the first approval, plus `threshold`
       * times `DepositFactor`. It is returned once this dispatch happens or is cancelled.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * - `threshold`: The total number of approvals for this dispatch before it is executed.
       * - `other_signatories`: The accounts (other than the sender) who can approve this dispatch.
       *   May not be empty.
       * - `maybe_timepoint`: If this is the first approval, then this must be `None`. If it is not
       *   the first approval, then it must be `Some`, with the timepoint (block number and
       *   transaction index) of the first approval transaction.
       * - `call_hash`: The hash of the call to be executed.
       *
       * NOTE: If this is the final approval, you will want to use `as_multi` instead.
       *
       * ## Complexity
       *
       * - `O(S)`.
       * - Up to one balance-reserve or unreserve operation.
       * - One passthrough operation, one insert, both `O(S)` where `S` is the number of signatories.
       *   `S` is capped by `MaxSignatories`, with weight being proportional.
       * - One encode & hash, both of complexity `O(S)`.
       * - Up to one binary search and insert (`O(logS + S)`).
       * - I/O: 1 read `O(S)`, up to 1 mutate `O(S)`. Up to one remove.
       * - One event.
       * - Storage: inserts one item, value size bounded by `MaxSignatories`, with a deposit taken for
       *   its lifetime of `DepositBase + threshold * DepositFactor`.
       */
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
      /**
       * Register approval for a dispatch to be made from a deterministic composite account if
       * approved by a total of `threshold - 1` of `other_signatories`.
       *
       * If there are enough, then dispatch the call.
       *
       * Payment: `DepositBase` will be reserved if this is the first approval, plus `threshold`
       * times `DepositFactor`. It is returned once this dispatch happens or is cancelled.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * - `threshold`: The total number of approvals for this dispatch before it is executed.
       * - `other_signatories`: The accounts (other than the sender) who can approve this dispatch.
       *   May not be empty.
       * - `maybe_timepoint`: If this is the first approval, then this must be `None`. If it is not
       *   the first approval, then it must be `Some`, with the timepoint (block number and
       *   transaction index) of the first approval transaction.
       * - `call`: The call to be executed.
       *
       * NOTE: Unless this is the final approval, you will generally want to use `approve_as_multi`
       * instead, since it only requires a hash of the call.
       *
       * Result is equivalent to the dispatched result if `threshold` is exactly `1`. Otherwise on
       * success, result is `Ok` and the result from the interior call, if it was executed, may be
       * found in the deposited `MultisigExecuted` event.
       *
       * ## Complexity
       *
       * - `O(S + Z + Call)`.
       * - Up to one balance-reserve or unreserve operation.
       * - One passthrough operation, one insert, both `O(S)` where `S` is the number of signatories.
       *   `S` is capped by `MaxSignatories`, with weight being proportional.
       * - One call encode & hash, both of complexity `O(Z)` where `Z` is tx-len.
       * - One encode & hash, both of complexity `O(S)`.
       * - Up to one binary search and insert (`O(logS + S)`).
       * - I/O: 1 read `O(S)`, up to 1 mutate `O(S)`. Up to one remove.
       * - One event.
       * - The weight of the `call`.
       * - Storage: inserts one item, value size bounded by `MaxSignatories`, with a deposit taken for
       *   its lifetime of `DepositBase + threshold * DepositFactor`.
       */
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
      /**
       * Immediately dispatch a multi-signature call using a single approval from the caller.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * - `other_signatories`: The accounts (other than the sender) who are part of the
       *   multi-signature, but do not participate in the approval process.
       * - `call`: The call to be executed.
       *
       * Result is equivalent to the dispatched result.
       *
       * ## Complexity
       *
       * O(Z + C) where Z is the length of the call and C its execution weight.
       */
      asMultiThreshold1: AugmentedSubmittable<
        (
          otherSignatories: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Call]
      >;
      /**
       * Cancel a pre-existing, on-going multisig transaction. Any deposit reserved previously for
       * this operation will be unreserved on success.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * - `threshold`: The total number of approvals for this dispatch before it is executed.
       * - `other_signatories`: The accounts (other than the sender) who can approve this dispatch.
       *   May not be empty.
       * - `timepoint`: The timepoint (block number and transaction index) of the first approval
       *   transaction for this dispatch.
       * - `call_hash`: The hash of the call to be executed.
       *
       * ## Complexity
       *
       * - `O(S)`.
       * - Up to one balance-reserve or unreserve operation.
       * - One passthrough operation, one insert, both `O(S)` where `S` is the number of signatories.
       *   `S` is capped by `MaxSignatories`, with weight being proportional.
       * - One encode & hash, both of complexity `O(S)`.
       * - One event.
       * - I/O: 1 read `O(S)`, one remove.
       * - Storage: removes one item.
       */
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
      /**
       * Close a vote that is either approved, disapproved or whose voting period has ended.
       *
       * May be called by any signed account in order to finish voting and close the proposal.
       *
       * If called before the end of the voting period it will only close the vote if it is has
       * enough votes to be approved or disapproved.
       *
       * If called after the end of the voting period abstentions are counted as rejections unless
       * there is a prime member set and the prime member cast an approval.
       *
       * If the close operation completes successfully with disapproval, the transaction fee will be
       * waived. Otherwise execution of the approved operation will be charged to the caller.
       *
       * - `proposal_weight_bound`: The maximum amount of weight consumed by executing the closed proposal.
       * - `length_bound`: The upper bound for the length of the proposal in storage. Checked via
       *   `storage::read` so it is `size_of::<u32>() == 4` larger than the pure length.
       *
       * ## Complexity
       *
       * - `O(B + M + P1 + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - `P1` is the complexity of `proposal` preimage.
       * - `P2` is proposal-count (code-bounded)
       */
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
      /**
       * Disapprove a proposal, close, and remove it from the system, regardless of its current state.
       *
       * Must be called by the Root origin.
       *
       * Parameters:
       *
       * - `proposal_hash`: The hash of the proposal that should be disapproved.
       *
       * ## Complexity
       *
       * O(P) where P is the number of max proposals
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
       * ## Complexity:
       *
       * - `O(B + M + P)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` members-count (code-bounded)
       * - `P` complexity of dispatching `proposal`
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
       * `threshold` determines whether `proposal` is executed directly (`threshold < 2`) or put up
       * for voting.
       *
       * ## Complexity
       *
       * - `O(B + M + P1)` or `O(B + M + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - Branching is influenced by `threshold` where:
       * - `P1` is proposal execution complexity (`threshold < 2`)
       * - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
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
       * - `old_count`: The upper bound for the previous number of members in storage. Used for weight
       *   estimation.
       *
       * The dispatch of this call must be `SetMembersOrigin`.
       *
       * NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but the
       * weight estimations rely on it to estimate dispatchable weight.
       *
       * # WARNING:
       *
       * The `pallet-collective` can also be managed by logic outside of the pallet through the
       * implementation of the trait [`ChangeMembers`]. Any call to `set_members` must be careful
       * that the member set doesn't get out of sync with other logic managing the member set.
       *
       * ## Complexity:
       *
       * - `O(MP + N)` where:
       * - `M` old-members-count (code- and governance-bounded)
       * - `N` new-members-count (code- and governance-bounded)
       * - `P` proposals-count (code-bounded)
       */
      setMembers: AugmentedSubmittable<
        (
          newMembers: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          prime: Option<AccountId20> | null | Uint8Array | AccountId20 | string,
          oldCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Option<AccountId20>, u32]
      >;
      /**
       * Add an aye or nay vote for the sender to the given proposal.
       *
       * Requires the sender to be a member.
       *
       * Transaction fees will be waived if the member is voting on any particular proposal for the
       * first time and the call is successful. Subsequent vote changes will charge a fee.
       *
       * ## Complexity
       *
       * - `O(M)` where `M` is members-count (code- and governance-bounded)
       */
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
      /** Cancel pending request to adjust the collator candidate self bond */
      cancelCandidateBondLess: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Cancel request to change an existing delegation. */
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
      /** Increase collator candidate self bond by `more` */
      candidateBondMore: AugmentedSubmittable<
        (more: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u128]
      >;
      /**
       * DEPRECATED use delegateWithAutoCompound If caller is not a delegator and not a collator,
       * then join the set of delegators If caller is a delegator, then makes delegation to change
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
       * If caller is not a delegator and not a collator, then join the set of delegators If caller
       * is a delegator, then makes delegation to change their delegation state Sets the
       * auto-compound config for the delegation
       */
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
      /** Bond more for delegators wrt a specific collator candidate. */
      delegatorBondMore: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          more: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128]
      >;
      /** Enable/Disable marking offline feature */
      enableMarkingOffline: AugmentedSubmittable<
        (value: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [bool]
      >;
      /** Execute pending request to adjust the collator candidate self bond */
      executeCandidateBondLess: AugmentedSubmittable<
        (candidate: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Execute pending request to change an existing delegation */
      executeDelegationRequest: AugmentedSubmittable<
        (
          delegator: AccountId20 | string | Uint8Array,
          candidate: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, AccountId20]
      >;
      /** Execute leave candidates request */
      executeLeaveCandidates: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          candidateDelegationCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u32]
      >;
      /** Force join the set of collator candidates. It will skip the minimum required bond check. */
      forceJoinCandidates: AugmentedSubmittable<
        (
          account: AccountId20 | string | Uint8Array,
          bond: u128 | AnyNumber | Uint8Array,
          candidateCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128, u32]
      >;
      /** Temporarily leave the set of collator candidates without unbonding */
      goOffline: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Rejoin the set of collator candidates if previously had called `go_offline` */
      goOnline: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Hotfix to remove existing empty entries for candidates that have left. */
      hotfixRemoveDelegationRequestsExitedCandidates: AugmentedSubmittable<
        (
          candidates: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>]
      >;
      /** Join the set of collator candidates */
      joinCandidates: AugmentedSubmittable<
        (
          bond: u128 | AnyNumber | Uint8Array,
          candidateCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128, u32]
      >;
      /** Notify a collator is inactive during MaxOfflineRounds */
      notifyInactiveCollator: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** REMOVED, was schedule_leave_delegators */
      removedCall19: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** REMOVED, was execute_leave_delegators */
      removedCall20: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** REMOVED, was cancel_leave_delegators */
      removedCall21: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Request by collator candidate to decrease self bond by `less` */
      scheduleCandidateBondLess: AugmentedSubmittable<
        (less: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u128]
      >;
      /**
       * Request bond less for delegators wrt a specific collator candidate. The delegation's
       * rewards for rounds while the request is pending use the reduced bonded amount. A bond less
       * may not be performed if any other scheduled request is pending.
       */
      scheduleDelegatorBondLess: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          less: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u128]
      >;
      /**
       * Request to leave the set of candidates. If successful, the account is immediately removed
       * from the candidate pool to prevent selection as a collator.
       */
      scheduleLeaveCandidates: AugmentedSubmittable<
        (candidateCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Request to revoke an existing delegation. If successful, the delegation is scheduled to be
       * allowed to be revoked via the `execute_delegation_request` extrinsic. The delegation
       * receives no rewards for the rounds while a revoke is pending. A revoke may not be performed
       * if any other scheduled request is pending.
       */
      scheduleRevokeDelegation: AugmentedSubmittable<
        (collator: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Sets the auto-compounding reward percentage for a delegation. */
      setAutoCompound: AugmentedSubmittable<
        (
          candidate: AccountId20 | string | Uint8Array,
          value: Percent | AnyNumber | Uint8Array,
          candidateAutoCompoundingDelegationCountHint: u32 | AnyNumber | Uint8Array,
          delegationCountHint: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Percent, u32, u32]
      >;
      /**
       * Set blocks per round
       *
       * - If called with `new` less than length of current round, will transition immediately in the next block
       * - Also updates per-round inflation config
       */
      setBlocksPerRound: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Set the commission for all collators */
      setCollatorCommission: AugmentedSubmittable<
        (updated: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Perbill]
      >;
      /** Set the annual inflation rate to derive per-round inflation */
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
      /** Set the percent of inflation set aside for parachain bond */
      setInflationDistributionConfig: AugmentedSubmittable<
        (updated: Vec<Lookup61>) => SubmittableExtrinsic<ApiType>,
        [Vec<Lookup61>]
      >;
      /**
       * Deprecated: please use `set_inflation_distribution_config` instead.
       *
       * Set the account that will hold funds set aside for parachain bond
       */
      setParachainBondAccount: AugmentedSubmittable<
        (updated: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /**
       * Deprecated: please use `set_inflation_distribution_config` instead.
       *
       * Set the percent of inflation set aside for parachain bond
       */
      setParachainBondReservePercent: AugmentedSubmittable<
        (updated: Percent | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Percent]
      >;
      /**
       * Set the expectations for total staked. These expectations determine the issuance for the
       * round according to logic in `fn compute_issuance`
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
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    parachainSystem: {
      /**
       * Authorize an upgrade to a given `code_hash` for the runtime. The runtime can be supplied later.
       *
       * The `check_version` parameter sets a boolean flag for whether or not the runtime's spec
       * version and name should be verified on upgrade. Since the authorization only has a hash, it
       * cannot actually perform the verification.
       *
       * This call requires Root origin.
       */
      authorizeUpgrade: AugmentedSubmittable<
        (
          codeHash: H256 | string | Uint8Array,
          checkVersion: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, bool]
      >;
      /**
       * Provide the preimage (runtime binary) `code` for an upgrade that has been authorized.
       *
       * If the authorization required a version check, this call will ensure the spec name remains
       * unchanged and that the spec version has increased.
       *
       * Note that this function will not apply the new `code`, but only attempt to schedule the
       * upgrade with the Relay Chain.
       *
       * All origins are allowed.
       */
      enactAuthorizedUpgrade: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the current validation data.
       *
       * This should be invoked exactly once per block. It will panic at the finalization phase if
       * the call was not invoked.
       *
       * The dispatch origin for this call must be `Inherent`
       *
       * As a side effect, this function upgrades the current validation function if the appropriate
       * time has come.
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
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    parameters: {
      /**
       * Set the value of a parameter.
       *
       * The dispatch origin of this call must be `AdminOrigin` for the given `key`. Values be
       * deleted by setting them to `None`.
       */
      setParameter: AugmentedSubmittable<
        (
          keyValue:
            | MoonbaseRuntimeRuntimeParamsRuntimeParameters
            | { RuntimeConfig: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbaseRuntimeRuntimeParamsRuntimeParameters]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polkadotXcm: {
      /**
       * Claims assets trapped on this pallet because of leftover assets during XCM execution.
       *
       * - `origin`: Anyone can call this extrinsic.
       * - `assets`: The exact assets that were trapped. Use the version to specify what version was
       *   the latest when they were trapped.
       * - `beneficiary`: The location/account where the claimed assets will be deposited.
       */
      claimAssets: AugmentedSubmittable<
        (
          assets:
            | XcmVersionedAssets
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          beneficiary:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedAssets, XcmVersionedLocation]
      >;
      /**
       * Execute an XCM message from a local, signed, origin.
       *
       * An event is deposited indicating whether `msg` could be executed completely or only partially.
       *
       * No more than `max_weight` will be used in its attempted execution. If this is less than the
       * maximum amount of weight that the message could take to be executed, then no execution
       * attempt will be made.
       */
      execute: AugmentedSubmittable<
        (
          message: XcmVersionedXcm | { V2: any } | { V3: any } | { V4: any } | string | Uint8Array,
          maxWeight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedXcm, SpWeightsWeightV2Weight]
      >;
      /**
       * Set a safe XCM version (the version that XCM should be encoded with if the most recent
       * version a destination can accept is unknown).
       *
       * - `origin`: Must be an origin specified by AdminOrigin.
       * - `maybe_xcm_version`: The default XCM encoding version, or `None` to disable.
       */
      forceDefaultXcmVersion: AugmentedSubmittable<
        (
          maybeXcmVersion: Option<u32> | null | Uint8Array | u32 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [Option<u32>]
      >;
      /**
       * Ask a location to notify us regarding their XCM version and any changes to it.
       *
       * - `origin`: Must be an origin specified by AdminOrigin.
       * - `location`: The location to which we should subscribe for XCM version notifications.
       */
      forceSubscribeVersionNotify: AugmentedSubmittable<
        (
          location:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation]
      >;
      /**
       * Set or unset the global suspension state of the XCM executor.
       *
       * - `origin`: Must be an origin specified by AdminOrigin.
       * - `suspended`: `true` to suspend, `false` to resume.
       */
      forceSuspension: AugmentedSubmittable<
        (suspended: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [bool]
      >;
      /**
       * Require that a particular destination should no longer notify us regarding any XCM version changes.
       *
       * - `origin`: Must be an origin specified by AdminOrigin.
       * - `location`: The location to which we are currently subscribed for XCM version notifications
       *   which we no longer desire.
       */
      forceUnsubscribeVersionNotify: AugmentedSubmittable<
        (
          location:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation]
      >;
      /**
       * Extoll that a particular destination can be communicated with through a particular version of XCM.
       *
       * - `origin`: Must be an origin specified by AdminOrigin.
       * - `location`: The destination that is being described.
       * - `xcm_version`: The latest version of XCM that `location` supports.
       */
      forceXcmVersion: AugmentedSubmittable<
        (
          location: StagingXcmV4Location | { parents?: any; interior?: any } | string | Uint8Array,
          version: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [StagingXcmV4Location, u32]
      >;
      /**
       * Transfer some assets from the local chain to the destination chain through their local,
       * destination or remote reserve.
       *
       * `assets` must have same reserve location and may not be teleportable to `dest`.
       *
       * - `assets` have local reserve: transfer assets to sovereign account of destination chain and
       *   forward a notification XCM to `dest` to mint and deposit reserve-based assets to `beneficiary`.
       * - `assets` have destination reserve: burn local assets and forward a notification to `dest`
       *   chain to withdraw the reserve assets from this chain's sovereign account and deposit them
       *   to `beneficiary`.
       * - `assets` have remote reserve: burn local assets, forward XCM to reserve chain to move
       *   reserves from this chain's SA to `dest` chain's SA, and forward another XCM to `dest` to
       *   mint and deposit reserve-based assets to `beneficiary`.
       *
       * Fee payment on the destination side is made from the asset in the `assets` vector of index
       * `fee_asset_item`, up to enough to pay for `weight_limit` of weight. If more weight is
       * needed than `weight_limit`, then the operation will fail and the sent assets may be at risk.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `[Parent, Parachain(..)]` to
       *   send from parachain to parachain, or `[Parachain(..)]` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will
       *   generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on
       *   the `dest` (and possibly reserve) chains.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay fees.
       * - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
       */
      limitedReserveTransferAssets: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          beneficiary:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          assets:
            | XcmVersionedAssets
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array,
          weightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation, XcmVersionedLocation, XcmVersionedAssets, u32, XcmV3WeightLimit]
      >;
      /**
       * Teleport some assets from the local chain to some destination chain.
       *
       * Fee payment on the destination side is made from the asset in the `assets` vector of index
       * `fee_asset_item`, up to enough to pay for `weight_limit` of weight. If more weight is
       * needed than `weight_limit`, then the operation will fail and the sent assets may be at risk.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `[Parent, Parachain(..)]` to
       *   send from parachain to parachain, or `[Parachain(..)]` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will
       *   generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on
       *   the `dest` chain.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay fees.
       * - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
       */
      limitedTeleportAssets: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          beneficiary:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          assets:
            | XcmVersionedAssets
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array,
          weightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation, XcmVersionedLocation, XcmVersionedAssets, u32, XcmV3WeightLimit]
      >;
      /**
       * Transfer some assets from the local chain to the destination chain through their local,
       * destination or remote reserve.
       *
       * `assets` must have same reserve location and may not be teleportable to `dest`.
       *
       * - `assets` have local reserve: transfer assets to sovereign account of destination chain and
       *   forward a notification XCM to `dest` to mint and deposit reserve-based assets to `beneficiary`.
       * - `assets` have destination reserve: burn local assets and forward a notification to `dest`
       *   chain to withdraw the reserve assets from this chain's sovereign account and deposit them
       *   to `beneficiary`.
       * - `assets` have remote reserve: burn local assets, forward XCM to reserve chain to move
       *   reserves from this chain's SA to `dest` chain's SA, and forward another XCM to `dest` to
       *   mint and deposit reserve-based assets to `beneficiary`.
       *
       * **This function is deprecated: Use `limited_reserve_transfer_assets` instead.**
       *
       * Fee payment on the destination side is made from the asset in the `assets` vector of index
       * `fee_asset_item`. The weight limit for fees is not provided and thus is unlimited, with all
       * fees taken as needed from the asset.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `[Parent, Parachain(..)]` to
       *   send from parachain to parachain, or `[Parachain(..)]` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will
       *   generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on
       *   the `dest` (and possibly reserve) chains.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay fees.
       */
      reserveTransferAssets: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          beneficiary:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          assets:
            | XcmVersionedAssets
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation, XcmVersionedLocation, XcmVersionedAssets, u32]
      >;
      send: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          message: XcmVersionedXcm | { V2: any } | { V3: any } | { V4: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation, XcmVersionedXcm]
      >;
      /**
       * Teleport some assets from the local chain to some destination chain.
       *
       * **This function is deprecated: Use `limited_teleport_assets` instead.**
       *
       * Fee payment on the destination side is made from the asset in the `assets` vector of index
       * `fee_asset_item`. The weight limit for fees is not provided and thus is unlimited, with all
       * fees taken as needed from the asset.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `[Parent, Parachain(..)]` to
       *   send from parachain to parachain, or `[Parachain(..)]` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will
       *   generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on
       *   the `dest` chain.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay fees.
       */
      teleportAssets: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          beneficiary:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          assets:
            | XcmVersionedAssets
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation, XcmVersionedLocation, XcmVersionedAssets, u32]
      >;
      /**
       * Transfer some assets from the local chain to the destination chain through their local,
       * destination or remote reserve, or through teleports.
       *
       * Fee payment on the destination side is made from the asset in the `assets` vector of index
       * `fee_asset_item` (hence referred to as `fees`), up to enough to pay for `weight_limit` of
       * weight. If more weight is needed than `weight_limit`, then the operation will fail and the
       * sent assets may be at risk.
       *
       * `assets` (excluding `fees`) must have same reserve location or otherwise be teleportable to
       * `dest`, no limitations imposed on `fees`.
       *
       * - For local reserve: transfer assets to sovereign account of destination chain and forward a
       *   notification XCM to `dest` to mint and deposit reserve-based assets to `beneficiary`.
       * - For destination reserve: burn local assets and forward a notification to `dest` chain to
       *   withdraw the reserve assets from this chain's sovereign account and deposit them to `beneficiary`.
       * - For remote reserve: burn local assets, forward XCM to reserve chain to move reserves from
       *   this chain's SA to `dest` chain's SA, and forward another XCM to `dest` to mint and
       *   deposit reserve-based assets to `beneficiary`.
       * - For teleports: burn local assets and forward XCM to `dest` chain to mint/teleport assets
       *   and deposit them to `beneficiary`.
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))`
       *   to send from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
       * - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will
       *   generally be an `AccountId32` value.
       * - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on
       *   the `dest` (and possibly reserve) chains.
       * - `fee_asset_item`: The index into `assets` of the item which should be used to pay fees.
       * - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
       */
      transferAssets: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          beneficiary:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          assets:
            | XcmVersionedAssets
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          feeAssetItem: u32 | AnyNumber | Uint8Array,
          weightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation, XcmVersionedLocation, XcmVersionedAssets, u32, XcmV3WeightLimit]
      >;
      /**
       * Transfer assets from the local chain to the destination chain using explicit transfer types
       * for assets and fees.
       *
       * `assets` must have same reserve location or may be teleportable to `dest`. Caller must
       * provide the `assets_transfer_type` to be used for `assets`:
       *
       * - `TransferType::LocalReserve`: transfer assets to sovereign account of destination chain and
       *   forward a notification XCM to `dest` to mint and deposit reserve-based assets to `beneficiary`.
       * - `TransferType::DestinationReserve`: burn local assets and forward a notification to `dest`
       *   chain to withdraw the reserve assets from this chain's sovereign account and deposit them
       *   to `beneficiary`.
       * - `TransferType::RemoteReserve(reserve)`: burn local assets, forward XCM to `reserve` chain
       *   to move reserves from this chain's SA to `dest` chain's SA, and forward another XCM to
       *   `dest` to mint and deposit reserve-based assets to `beneficiary`. Typically the remote
       *   `reserve` is Asset Hub.
       * - `TransferType::Teleport`: burn local assets and forward XCM to `dest` chain to
       *   mint/teleport assets and deposit them to `beneficiary`.
       *
       * On the destination chain, as well as any intermediary hops, `BuyExecution` is used to buy
       * execution using transferred `assets` identified by `remote_fees_id`. Make sure enough of
       * the specified `remote_fees_id` asset is included in the given list of `assets`.
       * `remote_fees_id` should be enough to pay for `weight_limit`. If more weight is needed than
       * `weight_limit`, then the operation will fail and the sent assets may be at risk.
       *
       * `remote_fees_id` may use different transfer type than rest of `assets` and can be specified
       * through `fees_transfer_type`.
       *
       * The caller needs to specify what should happen to the transferred assets once they reach
       * the `dest` chain. This is done through the `custom_xcm_on_dest` parameter, which contains
       * the instructions to execute on `dest` as a final step. This is usually as simple as:
       * `Xcm(vec![DepositAsset { assets: Wild(AllCounted(assets.len())), beneficiary }])`, but
       * could be something more exotic like sending the `assets` even further.
       *
       * - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
       * - `dest`: Destination context for the assets. Will typically be `[Parent, Parachain(..)]` to
       *   send from parachain to parachain, or `[Parachain(..)]` to send from relay to parachain,
       *   or `(parents: 2, (GlobalConsensus(..), ..))` to send from parachain across a bridge to
       *   another ecosystem destination.
       * - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on
       *   the `dest` (and possibly reserve) chains.
       * - `assets_transfer_type`: The XCM `TransferType` used to transfer the `assets`.
       * - `remote_fees_id`: One of the included `assets` to be be used to pay fees.
       * - `fees_transfer_type`: The XCM `TransferType` used to transfer the `fees` assets.
       * - `custom_xcm_on_dest`: The XCM to be executed on `dest` chain as the last step of the
       *   transfer, which also determines what happens to the assets on the destination chain.
       * - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
       */
      transferAssetsUsingTypeAndThen: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          assets:
            | XcmVersionedAssets
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          assetsTransferType:
            | StagingXcmExecutorAssetTransferTransferType
            | { Teleport: any }
            | { LocalReserve: any }
            | { DestinationReserve: any }
            | { RemoteReserve: any }
            | string
            | Uint8Array,
          remoteFeesId: XcmVersionedAssetId | { V3: any } | { V4: any } | string | Uint8Array,
          feesTransferType:
            | StagingXcmExecutorAssetTransferTransferType
            | { Teleport: any }
            | { LocalReserve: any }
            | { DestinationReserve: any }
            | { RemoteReserve: any }
            | string
            | Uint8Array,
          customXcmOnDest:
            | XcmVersionedXcm
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          weightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          XcmVersionedLocation,
          XcmVersionedAssets,
          StagingXcmExecutorAssetTransferTransferType,
          XcmVersionedAssetId,
          StagingXcmExecutorAssetTransferTransferType,
          XcmVersionedXcm,
          XcmV3WeightLimit
        ]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    preimage: {
      /**
       * Ensure that the a bulk of pre-images is upgraded.
       *
       * The caller pays no fee if at least 90% of pre-images were successfully updated.
       */
      ensureUpdated: AugmentedSubmittable<
        (hashes: Vec<H256> | (H256 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>,
        [Vec<H256>]
      >;
      /**
       * Register a preimage on-chain.
       *
       * If the preimage was previously requested, no fees or deposits are taken for providing the
       * preimage. Otherwise, a deposit is taken proportional to the size of the preimage.
       */
      notePreimage: AugmentedSubmittable<
        (bytes: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Request a preimage be uploaded to the chain without paying any fees or deposits.
       *
       * If the preimage requests has already been provided on-chain, we unreserve any deposit a
       * user may have paid, and take the control of the preimage out of their hands.
       */
      requestPreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Clear an unrequested preimage from the runtime storage.
       *
       * If `len` is provided, then it will be a much cheaper operation.
       *
       * - `hash`: The hash of the preimage to be removed from the store.
       * - `len`: The length of the preimage of `hash`.
       */
      unnotePreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Clear a previously made request for a preimage.
       *
       * NOTE: THIS MUST NOT BE CALLED ON `hash` MORE TIMES THAN `request_preimage`.
       */
      unrequestPreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** Generic tx */
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
       * - `delay`: The announcement period required of the initial proxy. Will generally be zero.
       */
      addProxy: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          proxyType:
            | MoonbaseRuntimeProxyType
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
        [AccountId20, MoonbaseRuntimeProxyType, u32]
      >;
      /**
       * Publish the hash of a proxy-call that will be made in the future.
       *
       * This must be called some number of blocks before the corresponding `proxy` is attempted if
       * the delay associated with the proxy relationship is greater than zero.
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
       */
      announce: AugmentedSubmittable<
        (
          real: AccountId20 | string | Uint8Array,
          callHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, H256]
      >;
      /**
       * Spawn a fresh new account that is guaranteed to be otherwise inaccessible, and initialize
       * it with a proxy of `proxy_type` for `origin` sender.
       *
       * Requires a `Signed` origin.
       *
       * - `proxy_type`: The type of the proxy that the sender will be registered as over the new
       *   account. This will almost always be the most permissive `ProxyType` possible to allow for
       *   maximum flexibility.
       * - `index`: A disambiguation index, in case this is called multiple times in the same
       *   transaction (e.g. with `utility::batch`). Unless you're using `batch` you probably just
       *   want to use `0`.
       * - `delay`: The announcement period required of the initial proxy. Will generally be zero.
       *
       * Fails with `Duplicate` if this has already been called in this transaction, from the same
       * sender, with the same parameters.
       *
       * Fails if there are insufficient funds to pay for deposit.
       */
      createPure: AugmentedSubmittable<
        (
          proxyType:
            | MoonbaseRuntimeProxyType
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
        [MoonbaseRuntimeProxyType, u32, u16]
      >;
      /**
       * Removes a previously spawned pure proxy.
       *
       * WARNING: **All access to this account will be lost.** Any funds held in it will be inaccessible.
       *
       * Requires a `Signed` origin, and the sender account must have been created by a call to
       * `pure` with corresponding parameters.
       *
       * - `spawner`: The account that originally called `pure` to create this account.
       * - `index`: The disambiguation index originally passed to `pure`. Probably `0`.
       * - `proxy_type`: The proxy type originally passed to `pure`.
       * - `height`: The height of the chain when the call to `pure` was processed.
       * - `ext_index`: The extrinsic index in which the call to `pure` was processed.
       *
       * Fails with `NoPermission` in case the caller is not a previously created pure account whose
       * `pure` call has corresponding parameters.
       */
      killPure: AugmentedSubmittable<
        (
          spawner: AccountId20 | string | Uint8Array,
          proxyType:
            | MoonbaseRuntimeProxyType
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
        [AccountId20, MoonbaseRuntimeProxyType, u16, Compact<u32>, Compact<u32>]
      >;
      /**
       * Dispatch the given `call` from an account that the sender is authorised for through `add_proxy`.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `real`: The account that the proxy will make a call on behalf of.
       * - `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
       * - `call`: The call to be made by the `real` account.
       */
      proxy: AugmentedSubmittable<
        (
          real: AccountId20 | string | Uint8Array,
          forceProxyType:
            | Option<MoonbaseRuntimeProxyType>
            | null
            | Uint8Array
            | MoonbaseRuntimeProxyType
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
        [AccountId20, Option<MoonbaseRuntimeProxyType>, Call]
      >;
      /**
       * Dispatch the given `call` from an account that the sender is authorized for through `add_proxy`.
       *
       * Removes any corresponding announcement(s).
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `real`: The account that the proxy will make a call on behalf of.
       * - `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
       * - `call`: The call to be made by the `real` account.
       */
      proxyAnnounced: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          real: AccountId20 | string | Uint8Array,
          forceProxyType:
            | Option<MoonbaseRuntimeProxyType>
            | null
            | Uint8Array
            | MoonbaseRuntimeProxyType
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
        [AccountId20, AccountId20, Option<MoonbaseRuntimeProxyType>, Call]
      >;
      /**
       * Remove the given announcement of a delegate.
       *
       * May be called by a target (proxied) account to remove a call that one of their delegates
       * (`delegate`) has announced they want to execute. The deposit is returned.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `delegate`: The account that previously announced the call.
       * - `call_hash`: The hash of the call to be made.
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
       * May be called by a proxy account to remove a call they previously announced and return the deposit.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * Parameters:
       *
       * - `real`: The account that the proxy will make a call on behalf of.
       * - `call_hash`: The hash of the call to be made by the `real` account.
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
       * WARNING: This may be called on accounts created by `pure`, however if done, then the
       * unreserved fees will be inaccessible. **All access to this account will be lost.**
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
       */
      removeProxy: AugmentedSubmittable<
        (
          delegate: AccountId20 | string | Uint8Array,
          proxyType:
            | MoonbaseRuntimeProxyType
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
        [AccountId20, MoonbaseRuntimeProxyType, u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    randomness: {
      /** Populates `RandomnessResults` due this epoch with BABE epoch randomness */
      setBabeRandomnessResults: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    referenda: {
      /**
       * Cancel an ongoing referendum.
       *
       * - `origin`: must be the `CancelOrigin`.
       * - `index`: The index of the referendum to be cancelled.
       *
       * Emits `Cancelled`.
       */
      cancel: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Cancel an ongoing referendum and slash the deposits.
       *
       * - `origin`: must be the `KillOrigin`.
       * - `index`: The index of the referendum to be cancelled.
       *
       * Emits `Killed` and `DepositSlashed`.
       */
      kill: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Advance a referendum onto its next logical state. Only used internally.
       *
       * - `origin`: must be `Root`.
       * - `index`: the referendum to be advanced.
       */
      nudgeReferendum: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Advance a track onto its next logical state. Only used internally.
       *
       * - `origin`: must be `Root`.
       * - `track`: the track to be advanced.
       *
       * Action item for when there is now one fewer referendum in the deciding phase and the
       * `DecidingCount` is not yet updated. This means that we should either:
       *
       * - Begin deciding another referendum (and leave `DecidingCount` alone); or
       * - Decrement `DecidingCount`.
       */
      oneFewerDeciding: AugmentedSubmittable<
        (track: u16 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u16]
      >;
      /**
       * Post the Decision Deposit for a referendum.
       *
       * - `origin`: must be `Signed` and the account must have funds available for the referendum's
       *   track's Decision Deposit.
       * - `index`: The index of the submitted referendum whose Decision Deposit is yet to be posted.
       *
       * Emits `DecisionDepositPlaced`.
       */
      placeDecisionDeposit: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Refund the Decision Deposit for a closed referendum back to the depositor.
       *
       * - `origin`: must be `Signed` or `Root`.
       * - `index`: The index of a closed referendum whose Decision Deposit has not yet been refunded.
       *
       * Emits `DecisionDepositRefunded`.
       */
      refundDecisionDeposit: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Refund the Submission Deposit for a closed referendum back to the depositor.
       *
       * - `origin`: must be `Signed` or `Root`.
       * - `index`: The index of a closed referendum whose Submission Deposit has not yet been refunded.
       *
       * Emits `SubmissionDepositRefunded`.
       */
      refundSubmissionDeposit: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Set or clear metadata of a referendum.
       *
       * Parameters:
       *
       * - `origin`: Must be `Signed` by a creator of a referendum or by anyone to clear a metadata of
       *   a finished referendum.
       * - `index`: The index of a referendum to set or clear metadata for.
       * - `maybe_hash`: The hash of an on-chain stored preimage. `None` to clear a metadata.
       */
      setMetadata: AugmentedSubmittable<
        (
          index: u32 | AnyNumber | Uint8Array,
          maybeHash: Option<H256> | null | Uint8Array | H256 | string
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Option<H256>]
      >;
      /**
       * Propose a referendum on a privileged action.
       *
       * - `origin`: must be `SubmitOrigin` and the account must have `SubmissionDeposit` funds available.
       * - `proposal_origin`: The origin from which the proposal should be executed.
       * - `proposal`: The proposal.
       * - `enactment_moment`: The moment that the proposal should be enacted.
       *
       * Emits `Submitted`.
       */
      submit: AugmentedSubmittable<
        (
          proposalOrigin:
            | MoonbaseRuntimeOriginCaller
            | { system: any }
            | { Void: any }
            | { Ethereum: any }
            | { CumulusXcm: any }
            | { PolkadotXcm: any }
            | { EthereumXcm: any }
            | { TreasuryCouncilCollective: any }
            | { Origins: any }
            | { OpenTechCommitteeCollective: any }
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
          MoonbaseRuntimeOriginCaller,
          FrameSupportPreimagesBounded,
          FrameSupportScheduleDispatchTime
        ]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    rootTesting: {
      /** A dispatch that will fill the block weight up to the given ratio. */
      fillBlock: AugmentedSubmittable<
        (ratio: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Perbill]
      >;
      triggerDefensive: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    scheduler: {
      /** Cancel an anonymously scheduled task. */
      cancel: AugmentedSubmittable<
        (
          when: u32 | AnyNumber | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, u32]
      >;
      /** Cancel a named scheduled task. */
      cancelNamed: AugmentedSubmittable<
        (id: U8aFixed | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [U8aFixed]
      >;
      /** Removes the retry configuration of a task. */
      cancelRetry: AugmentedSubmittable<
        (
          task: ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array]
        ) => SubmittableExtrinsic<ApiType>,
        [ITuple<[u32, u32]>]
      >;
      /** Cancel the retry configuration of a named task. */
      cancelRetryNamed: AugmentedSubmittable<
        (id: U8aFixed | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [U8aFixed]
      >;
      /** Anonymously schedule a task. */
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
      /** Anonymously schedule a task after a delay. */
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
      /** Schedule a named task. */
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
      /** Schedule a named task after a delay. */
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
      /**
       * Set a retry configuration for a task so that, in case its scheduled run fails, it will be
       * retried after `period` blocks, for a total amount of `retries` retries or until it succeeds.
       *
       * Tasks which need to be scheduled for a retry are still subject to weight metering and
       * agenda space, same as a regular task. If a periodic task fails, it will be scheduled
       * normally while the task is retrying.
       *
       * Tasks scheduled as a result of a retry for a periodic task are unnamed, non-periodic clones
       * of the original task. Their retry configuration will be derived from the original task's
       * configuration, but will have a lower value for `remaining` than the original `total_retries`.
       */
      setRetry: AugmentedSubmittable<
        (
          task: ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          retries: u8 | AnyNumber | Uint8Array,
          period: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [ITuple<[u32, u32]>, u8, u32]
      >;
      /**
       * Set a retry configuration for a named task so that, in case its scheduled run fails, it
       * will be retried after `period` blocks, for a total amount of `retries` retries or until it succeeds.
       *
       * Tasks which need to be scheduled for a retry are still subject to weight metering and
       * agenda space, same as a regular task. If a periodic task fails, it will be scheduled
       * normally while the task is retrying.
       *
       * Tasks scheduled as a result of a retry for a periodic task are unnamed, non-periodic clones
       * of the original task. Their retry configuration will be derived from the original task's
       * configuration, but will have a lower value for `remaining` than the original `total_retries`.
       */
      setRetryNamed: AugmentedSubmittable<
        (
          id: U8aFixed | string | Uint8Array,
          retries: u8 | AnyNumber | Uint8Array,
          period: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [U8aFixed, u8, u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    sudo: {
      /**
       * Permanently removes the sudo key.
       *
       * **This cannot be un-done.**
       */
      removeKey: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /** Authenticates the current sudo key and sets the given AccountId (`new`) as the new sudo key. */
      setKey: AugmentedSubmittable<
        (updated: AccountId20 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [AccountId20]
      >;
      /** Authenticates the sudo key and dispatches a function call with `Root` origin. */
      sudo: AugmentedSubmittable<
        (call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Call]
      >;
      /**
       * Authenticates the sudo key and dispatches a function call with `Signed` origin from a given account.
       *
       * The dispatch origin for this call must be _Signed_.
       */
      sudoAs: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, Call]
      >;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root` origin. This function
       * does not check the weight of the call, and instead allows the Sudo user to specify the
       * weight of the call.
       *
       * The dispatch origin for this call must be _Signed_.
       */
      sudoUncheckedWeight: AugmentedSubmittable<
        (
          call: Call | IMethod | string | Uint8Array,
          weight: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, SpWeightsWeightV2Weight]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    system: {
      /**
       * Provide the preimage (runtime binary) `code` for an upgrade that has been authorized.
       *
       * If the authorization required a version check, this call will ensure the spec name remains
       * unchanged and that the spec version has increased.
       *
       * Depending on the runtime's `OnSetCode` configuration, this function may directly apply the
       * new `code` in the same block or attempt to schedule the upgrade.
       *
       * All origins are allowed.
       */
      applyAuthorizedUpgrade: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Authorize an upgrade to a given `code_hash` for the runtime. The runtime can be supplied later.
       *
       * This call requires Root origin.
       */
      authorizeUpgrade: AugmentedSubmittable<
        (codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Authorize an upgrade to a given `code_hash` for the runtime. The runtime can be supplied later.
       *
       * WARNING: This authorizes an upgrade that will take place without any safety checks, for
       * example that the spec name remains the same and that the version number increases. Not
       * recommended for normal use. Use `authorize_upgrade` instead.
       *
       * This call requires Root origin.
       */
      authorizeUpgradeWithoutChecks: AugmentedSubmittable<
        (codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /**
       * Kill all storage items with a key that starts with the given prefix.
       *
       * **NOTE:** We rely on the Root origin to provide us the number of subkeys under the prefix
       * we are removing to accurately calculate the weight of this function.
       */
      killPrefix: AugmentedSubmittable<
        (
          prefix: Bytes | string | Uint8Array,
          subkeys: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, u32]
      >;
      /** Kill some items from storage. */
      killStorage: AugmentedSubmittable<
        (keys: Vec<Bytes> | (Bytes | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>,
        [Vec<Bytes>]
      >;
      /**
       * Make some on-chain remark.
       *
       * Can be executed by every `origin`.
       */
      remark: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** Make some on-chain remark and emit event. */
      remarkWithEvent: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** Set the new runtime code. */
      setCode: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the new runtime code without doing any checks of the given `code`.
       *
       * Note that runtime upgrades will not run if this is called with a not-increasing spec version!
       */
      setCodeWithoutChecks: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** Set the number of pages in the WebAssembly environment's heap. */
      setHeapPages: AugmentedSubmittable<
        (pages: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u64]
      >;
      /** Set some items of storage. */
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
      /**
       * Set the current time.
       *
       * This call should be invoked exactly once per block. It will panic at the finalization
       * phase, if this call hasn't been invoked by that time.
       *
       * The timestamp should be greater than the previous one by the amount specified by
       * [`Config::MinimumPeriod`].
       *
       * The dispatch origin for this call must be _None_.
       *
       * This dispatch class is _Mandatory_ to ensure it gets executed in the block. Be aware that
       * changing the complexity of this call could result exhausting the resources in a block to
       * execute any other calls.
       *
       * ## Complexity
       *
       * - `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)
       * - 1 storage read and 1 storage mutation (codec `O(1)` because of `DidUpdate::take` in `on_finalize`)
       * - 1 event handler `on_timestamp_set`. Must be `O(1)`.
       */
      set: AugmentedSubmittable<
        (now: Compact<u64> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u64>]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    treasury: {
      /**
       * Approve a proposal.
       *
       * ## Dispatch Origin
       *
       * Must be [`Config::ApproveOrigin`].
       *
       * ## Details
       *
       * At a later time, the proposal will be allocated to the beneficiary and the original deposit
       * will be returned.
       *
       * ### Complexity
       *
       * - O(1).
       *
       * ## Events
       *
       * No events are emitted from this dispatch.
       */
      approveProposal: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /**
       * Check the status of the spend and remove it from the storage if processed.
       *
       * ## Dispatch Origin
       *
       * Must be signed.
       *
       * ## Details
       *
       * The status check is a prerequisite for retrying a failed payout. If a spend has either
       * succeeded or expired, it is removed from the storage by this function. In such instances,
       * transaction fees are refunded.
       *
       * ### Parameters
       *
       * - `index`: The spend index.
       *
       * ## Events
       *
       * Emits [`Event::PaymentFailed`] if the spend payout has failed. Emits
       * [`Event::SpendProcessed`] if the spend payout has succeed.
       */
      checkStatus: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Claim a spend.
       *
       * ## Dispatch Origin
       *
       * Must be signed.
       *
       * ## Details
       *
       * Spends must be claimed within some temporal bounds. A spend may be claimed within one
       * [`Config::PayoutPeriod`] from the `valid_from` block. In case of a payout failure, the
       * spend status must be updated with the `check_status` dispatchable before retrying with the
       * current function.
       *
       * ### Parameters
       *
       * - `index`: The spend index.
       *
       * ## Events
       *
       * Emits [`Event::Paid`] if successful.
       */
      payout: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Put forward a suggestion for spending.
       *
       * ## Dispatch Origin
       *
       * Must be signed.
       *
       * ## Details
       *
       * A deposit proportional to the value is reserved and slashed if the proposal is rejected. It
       * is returned once the proposal is awarded.
       *
       * ### Complexity
       *
       * - O(1)
       *
       * ## Events
       *
       * Emits [`Event::Proposed`] if successful.
       */
      proposeSpend: AugmentedSubmittable<
        (
          value: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Reject a proposed spend.
       *
       * ## Dispatch Origin
       *
       * Must be [`Config::RejectOrigin`].
       *
       * ## Details
       *
       * The original deposit will be slashed.
       *
       * ### Complexity
       *
       * - O(1)
       *
       * ## Events
       *
       * Emits [`Event::Rejected`] if successful.
       */
      rejectProposal: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /**
       * Force a previously approved proposal to be removed from the approval queue.
       *
       * ## Dispatch Origin
       *
       * Must be [`Config::RejectOrigin`].
       *
       * ## Details
       *
       * The original deposit will no longer be returned.
       *
       * ### Parameters
       *
       * - `proposal_id`: The index of a proposal
       *
       * ### Complexity
       *
       * - O(A) where `A` is the number of approvals
       *
       * ### Errors
       *
       * - [`Error::ProposalNotApproved`]: The `proposal_id` supplied was not found in the approval
       *   queue, i.e., the proposal has not been approved. This could also mean the proposal does
       *   not exist altogether, thus there is no way it would have been approved in the first place.
       */
      removeApproval: AugmentedSubmittable<
        (proposalId: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >;
      /**
       * Propose and approve a spend of treasury funds.
       *
       * ## Dispatch Origin
       *
       * Must be [`Config::SpendOrigin`] with the `Success` value being at least `amount` of
       * `asset_kind` in the native asset. The amount of `asset_kind` is converted for assertion
       * using the [`Config::BalanceConverter`].
       *
       * ## Details
       *
       * Create an approved spend for transferring a specific `amount` of `asset_kind` to a
       * designated beneficiary. The spend must be claimed using the `payout` dispatchable within
       * the [`Config::PayoutPeriod`].
       *
       * ### Parameters
       *
       * - `asset_kind`: An indicator of the specific asset class to be spent.
       * - `amount`: The amount to be transferred from the treasury to the `beneficiary`.
       * - `beneficiary`: The beneficiary of the spend.
       * - `valid_from`: The block number from which the spend can be claimed. It can refer to the
       *   past if the resulting spend has not yet expired according to the
       *   [`Config::PayoutPeriod`]. If `None`, the spend can be claimed immediately after approval.
       *
       * ## Events
       *
       * Emits [`Event::AssetSpendApproved`] if successful.
       */
      spend: AugmentedSubmittable<
        (
          assetKind: Null | null,
          amount: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array,
          validFrom: Option<u32> | null | Uint8Array | u32 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [Null, Compact<u128>, AccountId20, Option<u32>]
      >;
      /**
       * Propose and approve a spend of treasury funds.
       *
       * ## Dispatch Origin
       *
       * Must be [`Config::SpendOrigin`] with the `Success` value being at least `amount`.
       *
       * ### Details
       *
       * NOTE: For record-keeping purposes, the proposer is deemed to be equivalent to the beneficiary.
       *
       * ### Parameters
       *
       * - `amount`: The amount to be transferred from the treasury to the `beneficiary`.
       * - `beneficiary`: The destination account for the transfer.
       *
       * ## Events
       *
       * Emits [`Event::SpendApproved`] if successful.
       */
      spendLocal: AugmentedSubmittable<
        (
          amount: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId20 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, AccountId20]
      >;
      /**
       * Void previously approved spend.
       *
       * ## Dispatch Origin
       *
       * Must be [`Config::RejectOrigin`].
       *
       * ## Details
       *
       * A spend void is only possible if the payout has not been attempted yet.
       *
       * ### Parameters
       *
       * - `index`: The spend index.
       *
       * ## Events
       *
       * Emits [`Event::AssetSpendVoided`] if successful.
       */
      voidSpend: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    treasuryCouncilCollective: {
      /**
       * Close a vote that is either approved, disapproved or whose voting period has ended.
       *
       * May be called by any signed account in order to finish voting and close the proposal.
       *
       * If called before the end of the voting period it will only close the vote if it is has
       * enough votes to be approved or disapproved.
       *
       * If called after the end of the voting period abstentions are counted as rejections unless
       * there is a prime member set and the prime member cast an approval.
       *
       * If the close operation completes successfully with disapproval, the transaction fee will be
       * waived. Otherwise execution of the approved operation will be charged to the caller.
       *
       * - `proposal_weight_bound`: The maximum amount of weight consumed by executing the closed proposal.
       * - `length_bound`: The upper bound for the length of the proposal in storage. Checked via
       *   `storage::read` so it is `size_of::<u32>() == 4` larger than the pure length.
       *
       * ## Complexity
       *
       * - `O(B + M + P1 + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - `P1` is the complexity of `proposal` preimage.
       * - `P2` is proposal-count (code-bounded)
       */
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
      /**
       * Disapprove a proposal, close, and remove it from the system, regardless of its current state.
       *
       * Must be called by the Root origin.
       *
       * Parameters:
       *
       * - `proposal_hash`: The hash of the proposal that should be disapproved.
       *
       * ## Complexity
       *
       * O(P) where P is the number of max proposals
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
       * ## Complexity:
       *
       * - `O(B + M + P)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` members-count (code-bounded)
       * - `P` complexity of dispatching `proposal`
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
       * `threshold` determines whether `proposal` is executed directly (`threshold < 2`) or put up
       * for voting.
       *
       * ## Complexity
       *
       * - `O(B + M + P1)` or `O(B + M + P2)` where:
       * - `B` is `proposal` size in bytes (length-fee-bounded)
       * - `M` is members-count (code- and governance-bounded)
       * - Branching is influenced by `threshold` where:
       * - `P1` is proposal execution complexity (`threshold < 2`)
       * - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
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
       * - `old_count`: The upper bound for the previous number of members in storage. Used for weight
       *   estimation.
       *
       * The dispatch of this call must be `SetMembersOrigin`.
       *
       * NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but the
       * weight estimations rely on it to estimate dispatchable weight.
       *
       * # WARNING:
       *
       * The `pallet-collective` can also be managed by logic outside of the pallet through the
       * implementation of the trait [`ChangeMembers`]. Any call to `set_members` must be careful
       * that the member set doesn't get out of sync with other logic managing the member set.
       *
       * ## Complexity:
       *
       * - `O(MP + N)` where:
       * - `M` old-members-count (code- and governance-bounded)
       * - `N` new-members-count (code- and governance-bounded)
       * - `P` proposals-count (code-bounded)
       */
      setMembers: AugmentedSubmittable<
        (
          newMembers: Vec<AccountId20> | (AccountId20 | string | Uint8Array)[],
          prime: Option<AccountId20> | null | Uint8Array | AccountId20 | string,
          oldCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId20>, Option<AccountId20>, u32]
      >;
      /**
       * Add an aye or nay vote for the sender to the given proposal.
       *
       * Requires the sender to be a member.
       *
       * Transaction fees will be waived if the member is voting on any particular proposal for the
       * first time and the call is successful. Subsequent vote changes will charge a fee.
       *
       * ## Complexity
       *
       * - `O(M)` where `M` is members-count (code- and governance-bounded)
       */
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
      /**
       * Send a call through an indexed pseudonym of the sender.
       *
       * Filter from origin are passed along. The call will be dispatched with an origin which use
       * the same filter as the origin of this call.
       *
       * NOTE: If you need to ensure that any account-based filtering is not honored (i.e. because
       * you expect `proxy` to have been used prior in the call stack and you do not want the call
       * restrictions to apply to any sub-accounts), then use `as_multi_threshold_1` in the Multisig
       * pallet instead.
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
       * May be called from any origin except `None`.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       *   exceed the constant: `batched_calls_limit` (available in constant metadata).
       *
       * If origin is root then the calls are dispatched without checking origin filter. (This
       * includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * ## Complexity
       *
       * - O(C) where C is the number of calls to be batched.
       *
       * This will return `Ok` in all circumstances. To determine the success of the batch, an event
       * is deposited. If a call failed and the batch was interrupted, then the `BatchInterrupted`
       * event is deposited, along with the number of successful calls made and the error of the
       * failed call. If all were successful, then the `BatchCompleted` event is deposited.
       */
      batch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /**
       * Send a batch of dispatch calls and atomically execute them. The whole transaction will
       * rollback and fail if any of the calls failed.
       *
       * May be called from any origin except `None`.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       *   exceed the constant: `batched_calls_limit` (available in constant metadata).
       *
       * If origin is root then the calls are dispatched without checking origin filter. (This
       * includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * ## Complexity
       *
       * - O(C) where C is the number of calls to be batched.
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
       * ## Complexity
       *
       * - O(1).
       */
      dispatchAs: AugmentedSubmittable<
        (
          asOrigin:
            | MoonbaseRuntimeOriginCaller
            | { system: any }
            | { Void: any }
            | { Ethereum: any }
            | { CumulusXcm: any }
            | { PolkadotXcm: any }
            | { EthereumXcm: any }
            | { TreasuryCouncilCollective: any }
            | { Origins: any }
            | { OpenTechCommitteeCollective: any }
            | string
            | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbaseRuntimeOriginCaller, Call]
      >;
      /**
       * Send a batch of dispatch calls. Unlike `batch`, it allows errors and won't interrupt.
       *
       * May be called from any origin except `None`.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       *   exceed the constant: `batched_calls_limit` (available in constant metadata).
       *
       * If origin is root then the calls are dispatch without checking origin filter. (This
       * includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * ## Complexity
       *
       * - O(C) where C is the number of calls to be batched.
       */
      forceBatch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /**
       * Dispatch a function call with a specified weight.
       *
       * This function does not check the weight of the call, and instead allows the Root origin to
       * specify the weight of the call.
       *
       * The dispatch origin for this call must be _Root_.
       */
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
      dispatchWhitelistedCallWithPreimage: AugmentedSubmittable<
        (call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Call]
      >;
      removeWhitelistedCall: AugmentedSubmittable<
        (callHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      whitelistCall: AugmentedSubmittable<
        (callHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    xcmpQueue: {
      /**
       * Resumes all XCM executions for the XCMP queue.
       *
       * Note that this function doesn't change the status of the in/out bound channels.
       *
       * - `origin`: Must pass `ControllerOrigin`.
       */
      resumeXcmExecution: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Suspends all XCM executions for the XCMP queue, regardless of the sender's origin.
       *
       * - `origin`: Must pass `ControllerOrigin`.
       */
      suspendXcmExecution: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Overwrites the number of pages which must be in the queue after which we drop any further
       * messages from the channel.
       *
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.drop_threshold`
       */
      updateDropThreshold: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Overwrites the number of pages which the queue must be reduced to before it signals that
       * message sending may recommence after it has been suspended.
       *
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.resume_threshold`
       */
      updateResumeThreshold: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /**
       * Overwrites the number of pages which must be in the queue for the other side to be told to
       * suspend their sending.
       *
       * - `origin`: Must pass `Root`.
       * - `new`: Desired value for `QueueConfigData.suspend_value`
       */
      updateSuspendThreshold: AugmentedSubmittable<
        (updated: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    xcmTransactor: {
      /**
       * De-Register a derivative index. This prevents an account to use a derivative address
       * (represented by an index) from our of our sovereign accounts anymore
       */
      deregister: AugmentedSubmittable<
        (index: u16 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u16]
      >;
      /** Manage HRMP operations */
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
      /**
       * Register a derivative index for an account id. Dispatchable by DerivativeAddressRegistrationOrigin
       *
       * We do not store the derivative address, but only the index. We do not need to store the
       * derivative address to issue calls, only the index is enough
       *
       * For now an index is registered for all possible destinations and not per-destination. We
       * can change this in the future although it would just make things more complicated
       */
      register: AugmentedSubmittable<
        (
          who: AccountId20 | string | Uint8Array,
          index: u16 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId20, u16]
      >;
      /** Remove the fee per second of an asset on its reserve chain */
      removeFeePerSecond: AugmentedSubmittable<
        (
          assetLocation:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation]
      >;
      /** Remove the transact info of a location */
      removeTransactInfo: AugmentedSubmittable<
        (
          location:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation]
      >;
      /** Set the fee per second of an asset on its reserve chain */
      setFeePerSecond: AugmentedSubmittable<
        (
          assetLocation:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          feePerSecond: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedLocation, u128]
      >;
      /** Change the transact info of a location */
      setTransactInfo: AugmentedSubmittable<
        (
          location:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
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
          XcmVersionedLocation,
          SpWeightsWeightV2Weight,
          SpWeightsWeightV2Weight,
          Option<SpWeightsWeightV2Weight>
        ]
      >;
      /**
       * Transact the inner call through a derivative account in a destination chain, using
       * 'fee_location' to pay for the fees. This fee_location is given as a multilocation
       *
       * The caller needs to have the index registered in this pallet. The fee multiasset needs to
       * be a reserve asset for the destination transactor::multilocation.
       */
      transactThroughDerivative: AugmentedSubmittable<
        (
          dest: MoonbaseRuntimeXcmConfigTransactors | "Relay" | number | Uint8Array,
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
          MoonbaseRuntimeXcmConfigTransactors,
          u16,
          PalletXcmTransactorCurrencyPayment,
          Bytes,
          PalletXcmTransactorTransactWeights,
          bool
        ]
      >;
      /**
       * Transact the call through the a signed origin in this chain that should be converted to a
       * transaction dispatch account in the destination chain by any method implemented in the
       * destination chains runtime
       *
       * This time we are giving the currency as a currencyId instead of multilocation
       */
      transactThroughSigned: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
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
          XcmVersionedLocation,
          PalletXcmTransactorCurrencyPayment,
          Bytes,
          PalletXcmTransactorTransactWeights,
          bool
        ]
      >;
      /**
       * Transact the call through the sovereign account in a destination chain, 'fee_payer' pays for the fee
       *
       * SovereignAccountDispatcherOrigin callable only
       */
      transactThroughSovereign: AugmentedSubmittable<
        (
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          feePayer: Option<AccountId20> | null | Uint8Array | AccountId20 | string,
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
          XcmVersionedLocation,
          Option<AccountId20>,
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
    xcmWeightTrader: {
      addAsset: AugmentedSubmittable<
        (
          location: StagingXcmV4Location | { parents?: any; interior?: any } | string | Uint8Array,
          relativePrice: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [StagingXcmV4Location, u128]
      >;
      editAsset: AugmentedSubmittable<
        (
          location: StagingXcmV4Location | { parents?: any; interior?: any } | string | Uint8Array,
          relativePrice: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [StagingXcmV4Location, u128]
      >;
      pauseAssetSupport: AugmentedSubmittable<
        (
          location: StagingXcmV4Location | { parents?: any; interior?: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [StagingXcmV4Location]
      >;
      removeAsset: AugmentedSubmittable<
        (
          location: StagingXcmV4Location | { parents?: any; interior?: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [StagingXcmV4Location]
      >;
      resumeAssetSupport: AugmentedSubmittable<
        (
          location: StagingXcmV4Location | { parents?: any; interior?: any } | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [StagingXcmV4Location]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    xTokens: {
      /**
       * Transfer native currencies.
       *
       * `dest_weight_limit` is the weight for XCM execution on the dest chain, and it would be
       * charged from the transferred assets. If set below requirements, the execution may fail and
       * assets wouldn't be received.
       *
       * It's a no-op if any error on local XCM execution or message sending. Note sending assets
       * out per se doesn't guarantee they would be received. Receiving depends on if the XCM
       * message could be delivered by the network, and if the receiving chain would handle messages
       * correctly.
       */
      transfer: AugmentedSubmittable<
        (
          currencyId:
            | MoonbaseRuntimeXcmConfigCurrencyId
            | { SelfReserve: any }
            | { ForeignAsset: any }
            | { Erc20: any }
            | string
            | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbaseRuntimeXcmConfigCurrencyId, u128, XcmVersionedLocation, XcmV3WeightLimit]
      >;
      /**
       * Transfer `Asset`.
       *
       * `dest_weight_limit` is the weight for XCM execution on the dest chain, and it would be
       * charged from the transferred assets. If set below requirements, the execution may fail and
       * assets wouldn't be received.
       *
       * It's a no-op if any error on local XCM execution or message sending. Note sending assets
       * out per se doesn't guarantee they would be received. Receiving depends on if the XCM
       * message could be delivered by the network, and if the receiving chain would handle messages
       * correctly.
       */
      transferMultiasset: AugmentedSubmittable<
        (
          asset: XcmVersionedAsset | { V2: any } | { V3: any } | { V4: any } | string | Uint8Array,
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedAsset, XcmVersionedLocation, XcmV3WeightLimit]
      >;
      /**
       * Transfer several `Asset` specifying the item to be used as fee
       *
       * `dest_weight_limit` is the weight for XCM execution on the dest chain, and it would be
       * charged from the transferred assets. If set below requirements, the execution may fail and
       * assets wouldn't be received.
       *
       * `fee_item` is index of the Assets that we want to use for payment
       *
       * It's a no-op if any error on local XCM execution or message sending. Note sending assets
       * out per se doesn't guarantee they would be received. Receiving depends on if the XCM
       * message could be delivered by the network, and if the receiving chain would handle messages
       * correctly.
       */
      transferMultiassets: AugmentedSubmittable<
        (
          assets:
            | XcmVersionedAssets
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          feeItem: u32 | AnyNumber | Uint8Array,
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedAssets, u32, XcmVersionedLocation, XcmV3WeightLimit]
      >;
      /**
       * Transfer `Asset` specifying the fee and amount as separate.
       *
       * `dest_weight_limit` is the weight for XCM execution on the dest chain, and it would be
       * charged from the transferred assets. If set below requirements, the execution may fail and
       * assets wouldn't be received.
       *
       * `fee` is the Asset to be spent to pay for execution in destination chain. Both fee and
       * amount will be subtracted form the callers balance For now we only accept fee and asset
       * having the same `Location` id.
       *
       * If `fee` is not high enough to cover for the execution costs in the destination chain, then
       * the assets will be trapped in the destination chain
       *
       * It's a no-op if any error on local XCM execution or message sending. Note sending assets
       * out per se doesn't guarantee they would be received. Receiving depends on if the XCM
       * message could be delivered by the network, and if the receiving chain would handle messages
       * correctly.
       */
      transferMultiassetWithFee: AugmentedSubmittable<
        (
          asset: XcmVersionedAsset | { V2: any } | { V3: any } | { V4: any } | string | Uint8Array,
          fee: XcmVersionedAsset | { V2: any } | { V3: any } | { V4: any } | string | Uint8Array,
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [XcmVersionedAsset, XcmVersionedAsset, XcmVersionedLocation, XcmV3WeightLimit]
      >;
      /**
       * Transfer several currencies specifying the item to be used as fee
       *
       * `dest_weight_limit` is the weight for XCM execution on the dest chain, and it would be
       * charged from the transferred assets. If set below requirements, the execution may fail and
       * assets wouldn't be received.
       *
       * `fee_item` is index of the currencies tuple that we want to use for payment
       *
       * It's a no-op if any error on local XCM execution or message sending. Note sending assets
       * out per se doesn't guarantee they would be received. Receiving depends on if the XCM
       * message could be delivered by the network, and if the receiving chain would handle messages
       * correctly.
       */
      transferMulticurrencies: AugmentedSubmittable<
        (
          currencies:
            | Vec<ITuple<[MoonbaseRuntimeXcmConfigCurrencyId, u128]>>
            | [
                (
                  | MoonbaseRuntimeXcmConfigCurrencyId
                  | { SelfReserve: any }
                  | { ForeignAsset: any }
                  | { Erc20: any }
                  | string
                  | Uint8Array
                ),
                u128 | AnyNumber | Uint8Array
              ][],
          feeItem: u32 | AnyNumber | Uint8Array,
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          Vec<ITuple<[MoonbaseRuntimeXcmConfigCurrencyId, u128]>>,
          u32,
          XcmVersionedLocation,
          XcmV3WeightLimit
        ]
      >;
      /**
       * Transfer native currencies specifying the fee and amount as separate.
       *
       * `dest_weight_limit` is the weight for XCM execution on the dest chain, and it would be
       * charged from the transferred assets. If set below requirements, the execution may fail and
       * assets wouldn't be received.
       *
       * `fee` is the amount to be spent to pay for execution in destination chain. Both fee and
       * amount will be subtracted form the callers balance.
       *
       * If `fee` is not high enough to cover for the execution costs in the destination chain, then
       * the assets will be trapped in the destination chain
       *
       * It's a no-op if any error on local XCM execution or message sending. Note sending assets
       * out per se doesn't guarantee they would be received. Receiving depends on if the XCM
       * message could be delivered by the network, and if the receiving chain would handle messages
       * correctly.
       */
      transferWithFee: AugmentedSubmittable<
        (
          currencyId:
            | MoonbaseRuntimeXcmConfigCurrencyId
            | { SelfReserve: any }
            | { ForeignAsset: any }
            | { Erc20: any }
            | string
            | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array,
          fee: u128 | AnyNumber | Uint8Array,
          dest:
            | XcmVersionedLocation
            | { V2: any }
            | { V3: any }
            | { V4: any }
            | string
            | Uint8Array,
          destWeightLimit:
            | XcmV3WeightLimit
            | { Unlimited: any }
            | { Limited: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MoonbaseRuntimeXcmConfigCurrencyId, u128, u128, XcmVersionedLocation, XcmV3WeightLimit]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
  } // AugmentedSubmittables
} // declare module
