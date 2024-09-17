// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/events";

import type { ApiTypes, AugmentedEvent } from "@polkadot/api-base/types";
import type {
  Bytes,
  Null,
  Option,
  Result,
  U8aFixed,
  bool,
  u128,
  u16,
  u32,
  u64,
  u8,
} from "@polkadot/types-codec";
import type { ITuple } from "@polkadot/types-codec/types";
import type { AccountId20, H160, H256, Perbill, Percent } from "@polkadot/types/interfaces/runtime";
import type {
  CumulusPrimitivesCoreAggregateMessageOrigin,
  EthereumLog,
  EvmCoreErrorExitReason,
  FrameSupportDispatchDispatchInfo,
  FrameSupportDispatchPostDispatchInfo,
  FrameSupportMessagesProcessMessageError,
  FrameSupportPreimagesBounded,
  FrameSupportTokensMiscBalanceStatus,
  MoonbeamRuntimeAssetConfigAssetRegistrarMetadata,
  MoonbeamRuntimeProxyType,
  MoonbeamRuntimeXcmConfigAssetType,
  NimbusPrimitivesNimbusCryptoPublic,
  PalletConvictionVotingTally,
  PalletMultisigTimepoint,
  PalletParachainStakingDelegationRequestsCancelledScheduledRequest,
  PalletParachainStakingDelegatorAdded,
  PalletXcmTransactorHrmpOperation,
  PalletXcmTransactorRemoteTransactInfoWithMaxWeight,
  SessionKeysPrimitivesVrfVrfCryptoPublic,
  SpRuntimeDispatchError,
  SpRuntimeDispatchErrorWithPostInfo,
  SpWeightsWeightV2Weight,
  StagingXcmV4Asset,
  StagingXcmV4AssetAssets,
  StagingXcmV4Location,
  StagingXcmV4Response,
  StagingXcmV4TraitsOutcome,
  StagingXcmV4Xcm,
  XcmV3TraitsError,
  XcmVersionedAssets,
  XcmVersionedLocation,
} from "@polkadot/types/lookup";

export type __AugmentedEvent<ApiType extends ApiTypes> = AugmentedEvent<ApiType>;

declare module "@polkadot/api-base/types/events" {
  interface AugmentedEvents<ApiType extends ApiTypes> {
    assetManager: {
      /** Removed all information related to an assetId and destroyed asset */
      ForeignAssetDestroyed: AugmentedEvent<
        ApiType,
        [assetId: u128, assetType: MoonbeamRuntimeXcmConfigAssetType],
        { assetId: u128; assetType: MoonbeamRuntimeXcmConfigAssetType }
      >;
      /** New asset with the asset manager is registered */
      ForeignAssetRegistered: AugmentedEvent<
        ApiType,
        [
          assetId: u128,
          asset: MoonbeamRuntimeXcmConfigAssetType,
          metadata: MoonbeamRuntimeAssetConfigAssetRegistrarMetadata
        ],
        {
          assetId: u128;
          asset: MoonbeamRuntimeXcmConfigAssetType;
          metadata: MoonbeamRuntimeAssetConfigAssetRegistrarMetadata;
        }
      >;
      /** Removed all information related to an assetId */
      ForeignAssetRemoved: AugmentedEvent<
        ApiType,
        [assetId: u128, assetType: MoonbeamRuntimeXcmConfigAssetType],
        { assetId: u128; assetType: MoonbeamRuntimeXcmConfigAssetType }
      >;
      /** Changed the xcm type mapping for a given asset id */
      ForeignAssetXcmLocationChanged: AugmentedEvent<
        ApiType,
        [assetId: u128, newAssetType: MoonbeamRuntimeXcmConfigAssetType],
        { assetId: u128; newAssetType: MoonbeamRuntimeXcmConfigAssetType }
      >;
      /** Removed all information related to an assetId and destroyed asset */
      LocalAssetDestroyed: AugmentedEvent<ApiType, [assetId: u128], { assetId: u128 }>;
      /** Supported asset type for fee payment removed */
      SupportedAssetRemoved: AugmentedEvent<
        ApiType,
        [assetType: MoonbeamRuntimeXcmConfigAssetType],
        { assetType: MoonbeamRuntimeXcmConfigAssetType }
      >;
      /** Changed the amount of units we are charging per execution second for a given asset */
      UnitsPerSecondChanged: AugmentedEvent<
        ApiType,
        [assetType: MoonbeamRuntimeXcmConfigAssetType, unitsPerSecond: u128],
        { assetType: MoonbeamRuntimeXcmConfigAssetType; unitsPerSecond: u128 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    assets: {
      /** Accounts were destroyed for given asset. */
      AccountsDestroyed: AugmentedEvent<
        ApiType,
        [assetId: u128, accountsDestroyed: u32, accountsRemaining: u32],
        { assetId: u128; accountsDestroyed: u32; accountsRemaining: u32 }
      >;
      /** An approval for account `delegate` was cancelled by `owner`. */
      ApprovalCancelled: AugmentedEvent<
        ApiType,
        [assetId: u128, owner: AccountId20, delegate: AccountId20],
        { assetId: u128; owner: AccountId20; delegate: AccountId20 }
      >;
      /** Approvals were destroyed for given asset. */
      ApprovalsDestroyed: AugmentedEvent<
        ApiType,
        [assetId: u128, approvalsDestroyed: u32, approvalsRemaining: u32],
        { assetId: u128; approvalsDestroyed: u32; approvalsRemaining: u32 }
      >;
      /** (Additional) funds have been approved for transfer to a destination account. */
      ApprovedTransfer: AugmentedEvent<
        ApiType,
        [assetId: u128, source: AccountId20, delegate: AccountId20, amount: u128],
        { assetId: u128; source: AccountId20; delegate: AccountId20; amount: u128 }
      >;
      /** Some asset `asset_id` was frozen. */
      AssetFrozen: AugmentedEvent<ApiType, [assetId: u128], { assetId: u128 }>;
      /** The min_balance of an asset has been updated by the asset owner. */
      AssetMinBalanceChanged: AugmentedEvent<
        ApiType,
        [assetId: u128, newMinBalance: u128],
        { assetId: u128; newMinBalance: u128 }
      >;
      /** An asset has had its attributes changed by the `Force` origin. */
      AssetStatusChanged: AugmentedEvent<ApiType, [assetId: u128], { assetId: u128 }>;
      /** Some asset `asset_id` was thawed. */
      AssetThawed: AugmentedEvent<ApiType, [assetId: u128], { assetId: u128 }>;
      /** Some account `who` was blocked. */
      Blocked: AugmentedEvent<
        ApiType,
        [assetId: u128, who: AccountId20],
        { assetId: u128; who: AccountId20 }
      >;
      /** Some assets were destroyed. */
      Burned: AugmentedEvent<
        ApiType,
        [assetId: u128, owner: AccountId20, balance: u128],
        { assetId: u128; owner: AccountId20; balance: u128 }
      >;
      /** Some asset class was created. */
      Created: AugmentedEvent<
        ApiType,
        [assetId: u128, creator: AccountId20, owner: AccountId20],
        { assetId: u128; creator: AccountId20; owner: AccountId20 }
      >;
      /** An asset class was destroyed. */
      Destroyed: AugmentedEvent<ApiType, [assetId: u128], { assetId: u128 }>;
      /** An asset class is in the process of being destroyed. */
      DestructionStarted: AugmentedEvent<ApiType, [assetId: u128], { assetId: u128 }>;
      /** Some asset class was force-created. */
      ForceCreated: AugmentedEvent<
        ApiType,
        [assetId: u128, owner: AccountId20],
        { assetId: u128; owner: AccountId20 }
      >;
      /** Some account `who` was frozen. */
      Frozen: AugmentedEvent<
        ApiType,
        [assetId: u128, who: AccountId20],
        { assetId: u128; who: AccountId20 }
      >;
      /** Some assets were issued. */
      Issued: AugmentedEvent<
        ApiType,
        [assetId: u128, owner: AccountId20, amount: u128],
        { assetId: u128; owner: AccountId20; amount: u128 }
      >;
      /** Metadata has been cleared for an asset. */
      MetadataCleared: AugmentedEvent<ApiType, [assetId: u128], { assetId: u128 }>;
      /** New metadata has been set for an asset. */
      MetadataSet: AugmentedEvent<
        ApiType,
        [assetId: u128, name: Bytes, symbol_: Bytes, decimals: u8, isFrozen: bool],
        { assetId: u128; name: Bytes; symbol: Bytes; decimals: u8; isFrozen: bool }
      >;
      /** The owner changed. */
      OwnerChanged: AugmentedEvent<
        ApiType,
        [assetId: u128, owner: AccountId20],
        { assetId: u128; owner: AccountId20 }
      >;
      /** The management team changed. */
      TeamChanged: AugmentedEvent<
        ApiType,
        [assetId: u128, issuer: AccountId20, admin: AccountId20, freezer: AccountId20],
        { assetId: u128; issuer: AccountId20; admin: AccountId20; freezer: AccountId20 }
      >;
      /** Some account `who` was thawed. */
      Thawed: AugmentedEvent<
        ApiType,
        [assetId: u128, who: AccountId20],
        { assetId: u128; who: AccountId20 }
      >;
      /** Some account `who` was created with a deposit from `depositor`. */
      Touched: AugmentedEvent<
        ApiType,
        [assetId: u128, who: AccountId20, depositor: AccountId20],
        { assetId: u128; who: AccountId20; depositor: AccountId20 }
      >;
      /** Some assets were transferred. */
      Transferred: AugmentedEvent<
        ApiType,
        [assetId: u128, from: AccountId20, to: AccountId20, amount: u128],
        { assetId: u128; from: AccountId20; to: AccountId20; amount: u128 }
      >;
      /** An `amount` was transferred in its entirety from `owner` to `destination` by the approved `delegate`. */
      TransferredApproved: AugmentedEvent<
        ApiType,
        [
          assetId: u128,
          owner: AccountId20,
          delegate: AccountId20,
          destination: AccountId20,
          amount: u128
        ],
        {
          assetId: u128;
          owner: AccountId20;
          delegate: AccountId20;
          destination: AccountId20;
          amount: u128;
        }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    authorFilter: {
      /** The amount of eligible authors for the filter to select has been changed. */
      EligibleUpdated: AugmentedEvent<ApiType, [u32]>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    authorMapping: {
      /** A NimbusId has been registered and mapped to an AccountId. */
      KeysRegistered: AugmentedEvent<
        ApiType,
        [
          nimbusId: NimbusPrimitivesNimbusCryptoPublic,
          accountId: AccountId20,
          keys_: SessionKeysPrimitivesVrfVrfCryptoPublic
        ],
        {
          nimbusId: NimbusPrimitivesNimbusCryptoPublic;
          accountId: AccountId20;
          keys_: SessionKeysPrimitivesVrfVrfCryptoPublic;
        }
      >;
      /** An NimbusId has been de-registered, and its AccountId mapping removed. */
      KeysRemoved: AugmentedEvent<
        ApiType,
        [
          nimbusId: NimbusPrimitivesNimbusCryptoPublic,
          accountId: AccountId20,
          keys_: SessionKeysPrimitivesVrfVrfCryptoPublic
        ],
        {
          nimbusId: NimbusPrimitivesNimbusCryptoPublic;
          accountId: AccountId20;
          keys_: SessionKeysPrimitivesVrfVrfCryptoPublic;
        }
      >;
      /** An NimbusId has been registered, replacing a previous registration and its mapping. */
      KeysRotated: AugmentedEvent<
        ApiType,
        [
          newNimbusId: NimbusPrimitivesNimbusCryptoPublic,
          accountId: AccountId20,
          newKeys: SessionKeysPrimitivesVrfVrfCryptoPublic
        ],
        {
          newNimbusId: NimbusPrimitivesNimbusCryptoPublic;
          accountId: AccountId20;
          newKeys: SessionKeysPrimitivesVrfVrfCryptoPublic;
        }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    balances: {
      /** A balance was set by root. */
      BalanceSet: AugmentedEvent<
        ApiType,
        [who: AccountId20, free: u128],
        { who: AccountId20; free: u128 }
      >;
      /** Some amount was burned from an account. */
      Burned: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Some amount was deposited (e.g. for transaction fees). */
      Deposit: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /**
       * An account was removed whose balance was non-zero but below ExistentialDeposit, resulting
       * in an outright loss.
       */
      DustLost: AugmentedEvent<
        ApiType,
        [account: AccountId20, amount: u128],
        { account: AccountId20; amount: u128 }
      >;
      /** An account was created with some free balance. */
      Endowed: AugmentedEvent<
        ApiType,
        [account: AccountId20, freeBalance: u128],
        { account: AccountId20; freeBalance: u128 }
      >;
      /** Some balance was frozen. */
      Frozen: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Total issuance was increased by `amount`, creating a credit to be balanced. */
      Issued: AugmentedEvent<ApiType, [amount: u128], { amount: u128 }>;
      /** Some balance was locked. */
      Locked: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Some amount was minted into an account. */
      Minted: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Total issuance was decreased by `amount`, creating a debt to be balanced. */
      Rescinded: AugmentedEvent<ApiType, [amount: u128], { amount: u128 }>;
      /** Some balance was reserved (moved from free to reserved). */
      Reserved: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /**
       * Some balance was moved from the reserve of the first account to the second account. Final
       * argument indicates the destination balance type.
       */
      ReserveRepatriated: AugmentedEvent<
        ApiType,
        [
          from: AccountId20,
          to: AccountId20,
          amount: u128,
          destinationStatus: FrameSupportTokensMiscBalanceStatus
        ],
        {
          from: AccountId20;
          to: AccountId20;
          amount: u128;
          destinationStatus: FrameSupportTokensMiscBalanceStatus;
        }
      >;
      /** Some amount was restored into an account. */
      Restored: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Some amount was removed from the account (e.g. for misbehavior). */
      Slashed: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Some amount was suspended from an account (it can be restored later). */
      Suspended: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Some balance was thawed. */
      Thawed: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** The `TotalIssuance` was forcefully changed. */
      TotalIssuanceForced: AugmentedEvent<
        ApiType,
        [old: u128, new_: u128],
        { old: u128; new_: u128 }
      >;
      /** Transfer succeeded. */
      Transfer: AugmentedEvent<
        ApiType,
        [from: AccountId20, to: AccountId20, amount: u128],
        { from: AccountId20; to: AccountId20; amount: u128 }
      >;
      /** Some balance was unlocked. */
      Unlocked: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Some balance was unreserved (moved from reserved to free). */
      Unreserved: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** An account was upgraded. */
      Upgraded: AugmentedEvent<ApiType, [who: AccountId20], { who: AccountId20 }>;
      /** Some amount was withdrawn from the account (e.g. for transaction fees). */
      Withdraw: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    convictionVoting: {
      /** An account has delegated their vote to another account. [who, target] */
      Delegated: AugmentedEvent<ApiType, [AccountId20, AccountId20]>;
      /** An [account] has cancelled a previous delegation operation. */
      Undelegated: AugmentedEvent<ApiType, [AccountId20]>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    crowdloanRewards: {
      /** When initializing the reward vec an already initialized account was found */
      InitializedAccountWithNotEnoughContribution: AugmentedEvent<
        ApiType,
        [U8aFixed, Option<AccountId20>, u128]
      >;
      /** When initializing the reward vec an already initialized account was found */
      InitializedAlreadyInitializedAccount: AugmentedEvent<
        ApiType,
        [U8aFixed, Option<AccountId20>, u128]
      >;
      /** The initial payment of InitializationPayment % was paid */
      InitialPaymentMade: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Someone has proven they made a contribution and associated a native identity with it. Data
       * is the relay account, native account and the total amount of _rewards_ that will be paid
       */
      NativeIdentityAssociated: AugmentedEvent<ApiType, [U8aFixed, AccountId20, u128]>;
      /** A contributor has updated the reward address. */
      RewardAddressUpdated: AugmentedEvent<ApiType, [AccountId20, AccountId20]>;
      /**
       * A contributor has claimed some rewards. Data is the account getting paid and the amount of
       * rewards paid.
       */
      RewardsPaid: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    cumulusXcm: {
      /** Downward message executed with the given outcome. [ id, outcome ] */
      ExecutedDownward: AugmentedEvent<ApiType, [U8aFixed, StagingXcmV4TraitsOutcome]>;
      /** Downward message is invalid XCM. [ id ] */
      InvalidFormat: AugmentedEvent<ApiType, [U8aFixed]>;
      /** Downward message is unsupported version of XCM. [ id ] */
      UnsupportedVersion: AugmentedEvent<ApiType, [U8aFixed]>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    dmpQueue: {
      /** Some debris was cleaned up. */
      CleanedSome: AugmentedEvent<ApiType, [keysRemoved: u32], { keysRemoved: u32 }>;
      /** The cleanup of remaining pallet storage completed. */
      Completed: AugmentedEvent<ApiType, [error: bool], { error: bool }>;
      /** The export of pages completed. */
      CompletedExport: AugmentedEvent<ApiType, []>;
      /** The export of overweight messages completed. */
      CompletedOverweightExport: AugmentedEvent<ApiType, []>;
      /** The export of a page completed. */
      Exported: AugmentedEvent<ApiType, [page: u32], { page: u32 }>;
      /** The export of an overweight message completed. */
      ExportedOverweight: AugmentedEvent<ApiType, [index: u64], { index: u64 }>;
      /**
       * The export of a page failed.
       *
       * This should never be emitted.
       */
      ExportFailed: AugmentedEvent<ApiType, [page: u32], { page: u32 }>;
      /**
       * The export of an overweight message failed.
       *
       * This should never be emitted.
       */
      ExportOverweightFailed: AugmentedEvent<ApiType, [index: u64], { index: u64 }>;
      /** The cleanup of remaining pallet storage started. */
      StartedCleanup: AugmentedEvent<ApiType, []>;
      /** The export of pages started. */
      StartedExport: AugmentedEvent<ApiType, []>;
      /** The export of overweight messages started. */
      StartedOverweightExport: AugmentedEvent<ApiType, []>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    ethereum: {
      /** An ethereum transaction was successfully executed. */
      Executed: AugmentedEvent<
        ApiType,
        [
          from: H160,
          to: H160,
          transactionHash: H256,
          exitReason: EvmCoreErrorExitReason,
          extraData: Bytes
        ],
        {
          from: H160;
          to: H160;
          transactionHash: H256;
          exitReason: EvmCoreErrorExitReason;
          extraData: Bytes;
        }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    ethereumXcm: {
      /** Ethereum transaction executed from XCM */
      ExecutedFromXcm: AugmentedEvent<
        ApiType,
        [xcmMsgHash: H256, ethTxHash: H256],
        { xcmMsgHash: H256; ethTxHash: H256 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    evm: {
      /** A contract has been created at given address. */
      Created: AugmentedEvent<ApiType, [address: H160], { address: H160 }>;
      /** A contract was attempted to be created, but the execution failed. */
      CreatedFailed: AugmentedEvent<ApiType, [address: H160], { address: H160 }>;
      /** A contract has been executed successfully with states applied. */
      Executed: AugmentedEvent<ApiType, [address: H160], { address: H160 }>;
      /** A contract has been executed with errors. States are reverted with only gas fees applied. */
      ExecutedFailed: AugmentedEvent<ApiType, [address: H160], { address: H160 }>;
      /** Ethereum events from contracts. */
      Log: AugmentedEvent<ApiType, [log: EthereumLog], { log: EthereumLog }>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    identity: {
      /** A username authority was added. */
      AuthorityAdded: AugmentedEvent<ApiType, [authority: AccountId20], { authority: AccountId20 }>;
      /** A username authority was removed. */
      AuthorityRemoved: AugmentedEvent<
        ApiType,
        [authority: AccountId20],
        { authority: AccountId20 }
      >;
      /**
       * A dangling username (as in, a username corresponding to an account that has removed its
       * identity) has been removed.
       */
      DanglingUsernameRemoved: AugmentedEvent<
        ApiType,
        [who: AccountId20, username: Bytes],
        { who: AccountId20; username: Bytes }
      >;
      /** A name was cleared, and the given balance returned. */
      IdentityCleared: AugmentedEvent<
        ApiType,
        [who: AccountId20, deposit: u128],
        { who: AccountId20; deposit: u128 }
      >;
      /** A name was removed and the given balance slashed. */
      IdentityKilled: AugmentedEvent<
        ApiType,
        [who: AccountId20, deposit: u128],
        { who: AccountId20; deposit: u128 }
      >;
      /** A name was set or reset (which will remove all judgements). */
      IdentitySet: AugmentedEvent<ApiType, [who: AccountId20], { who: AccountId20 }>;
      /** A judgement was given by a registrar. */
      JudgementGiven: AugmentedEvent<
        ApiType,
        [target: AccountId20, registrarIndex: u32],
        { target: AccountId20; registrarIndex: u32 }
      >;
      /** A judgement was asked from a registrar. */
      JudgementRequested: AugmentedEvent<
        ApiType,
        [who: AccountId20, registrarIndex: u32],
        { who: AccountId20; registrarIndex: u32 }
      >;
      /** A judgement request was retracted. */
      JudgementUnrequested: AugmentedEvent<
        ApiType,
        [who: AccountId20, registrarIndex: u32],
        { who: AccountId20; registrarIndex: u32 }
      >;
      /** A queued username passed its expiration without being claimed and was removed. */
      PreapprovalExpired: AugmentedEvent<ApiType, [whose: AccountId20], { whose: AccountId20 }>;
      /** A username was set as a primary and can be looked up from `who`. */
      PrimaryUsernameSet: AugmentedEvent<
        ApiType,
        [who: AccountId20, username: Bytes],
        { who: AccountId20; username: Bytes }
      >;
      /** A registrar was added. */
      RegistrarAdded: AugmentedEvent<ApiType, [registrarIndex: u32], { registrarIndex: u32 }>;
      /** A sub-identity was added to an identity and the deposit paid. */
      SubIdentityAdded: AugmentedEvent<
        ApiType,
        [sub: AccountId20, main: AccountId20, deposit: u128],
        { sub: AccountId20; main: AccountId20; deposit: u128 }
      >;
      /** A sub-identity was removed from an identity and the deposit freed. */
      SubIdentityRemoved: AugmentedEvent<
        ApiType,
        [sub: AccountId20, main: AccountId20, deposit: u128],
        { sub: AccountId20; main: AccountId20; deposit: u128 }
      >;
      /**
       * A sub-identity was cleared, and the given deposit repatriated from the main identity
       * account to the sub-identity account.
       */
      SubIdentityRevoked: AugmentedEvent<
        ApiType,
        [sub: AccountId20, main: AccountId20, deposit: u128],
        { sub: AccountId20; main: AccountId20; deposit: u128 }
      >;
      /** A username was queued, but `who` must accept it prior to `expiration`. */
      UsernameQueued: AugmentedEvent<
        ApiType,
        [who: AccountId20, username: Bytes, expiration: u32],
        { who: AccountId20; username: Bytes; expiration: u32 }
      >;
      /** A username was set for `who`. */
      UsernameSet: AugmentedEvent<
        ApiType,
        [who: AccountId20, username: Bytes],
        { who: AccountId20; username: Bytes }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    maintenanceMode: {
      /** The chain was put into Maintenance Mode */
      EnteredMaintenanceMode: AugmentedEvent<ApiType, []>;
      /** The call to resume on_idle XCM execution failed with inner error */
      FailedToResumeIdleXcmExecution: AugmentedEvent<
        ApiType,
        [error: SpRuntimeDispatchError],
        { error: SpRuntimeDispatchError }
      >;
      /** The call to suspend on_idle XCM execution failed with inner error */
      FailedToSuspendIdleXcmExecution: AugmentedEvent<
        ApiType,
        [error: SpRuntimeDispatchError],
        { error: SpRuntimeDispatchError }
      >;
      /** The chain returned to its normal operating state */
      NormalOperationResumed: AugmentedEvent<ApiType, []>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    messageQueue: {
      /** Message placed in overweight queue. */
      OverweightEnqueued: AugmentedEvent<
        ApiType,
        [
          id: U8aFixed,
          origin: CumulusPrimitivesCoreAggregateMessageOrigin,
          pageIndex: u32,
          messageIndex: u32
        ],
        {
          id: U8aFixed;
          origin: CumulusPrimitivesCoreAggregateMessageOrigin;
          pageIndex: u32;
          messageIndex: u32;
        }
      >;
      /** This page was reaped. */
      PageReaped: AugmentedEvent<
        ApiType,
        [origin: CumulusPrimitivesCoreAggregateMessageOrigin, index: u32],
        { origin: CumulusPrimitivesCoreAggregateMessageOrigin; index: u32 }
      >;
      /** Message is processed. */
      Processed: AugmentedEvent<
        ApiType,
        [
          id: H256,
          origin: CumulusPrimitivesCoreAggregateMessageOrigin,
          weightUsed: SpWeightsWeightV2Weight,
          success: bool
        ],
        {
          id: H256;
          origin: CumulusPrimitivesCoreAggregateMessageOrigin;
          weightUsed: SpWeightsWeightV2Weight;
          success: bool;
        }
      >;
      /** Message discarded due to an error in the `MessageProcessor` (usually a format error). */
      ProcessingFailed: AugmentedEvent<
        ApiType,
        [
          id: H256,
          origin: CumulusPrimitivesCoreAggregateMessageOrigin,
          error: FrameSupportMessagesProcessMessageError
        ],
        {
          id: H256;
          origin: CumulusPrimitivesCoreAggregateMessageOrigin;
          error: FrameSupportMessagesProcessMessageError;
        }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    migrations: {
      /** XCM execution resume failed with inner error */
      FailedToResumeIdleXcmExecution: AugmentedEvent<
        ApiType,
        [error: SpRuntimeDispatchError],
        { error: SpRuntimeDispatchError }
      >;
      /** XCM execution suspension failed with inner error */
      FailedToSuspendIdleXcmExecution: AugmentedEvent<
        ApiType,
        [error: SpRuntimeDispatchError],
        { error: SpRuntimeDispatchError }
      >;
      /** Migration completed */
      MigrationCompleted: AugmentedEvent<
        ApiType,
        [migrationName: Bytes, consumedWeight: SpWeightsWeightV2Weight],
        { migrationName: Bytes; consumedWeight: SpWeightsWeightV2Weight }
      >;
      /** Migration started */
      MigrationStarted: AugmentedEvent<ApiType, [migrationName: Bytes], { migrationName: Bytes }>;
      /** Runtime upgrade completed */
      RuntimeUpgradeCompleted: AugmentedEvent<
        ApiType,
        [weight: SpWeightsWeightV2Weight],
        { weight: SpWeightsWeightV2Weight }
      >;
      /** Runtime upgrade started */
      RuntimeUpgradeStarted: AugmentedEvent<ApiType, []>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    moonbeamOrbiters: {
      /** An orbiter join a collator pool */
      OrbiterJoinCollatorPool: AugmentedEvent<
        ApiType,
        [collator: AccountId20, orbiter: AccountId20],
        { collator: AccountId20; orbiter: AccountId20 }
      >;
      /** An orbiter leave a collator pool */
      OrbiterLeaveCollatorPool: AugmentedEvent<
        ApiType,
        [collator: AccountId20, orbiter: AccountId20],
        { collator: AccountId20; orbiter: AccountId20 }
      >;
      /** An orbiter has registered */
      OrbiterRegistered: AugmentedEvent<
        ApiType,
        [account: AccountId20, deposit: u128],
        { account: AccountId20; deposit: u128 }
      >;
      /** Paid the orbiter account the balance as liquid rewards. */
      OrbiterRewarded: AugmentedEvent<
        ApiType,
        [account: AccountId20, rewards: u128],
        { account: AccountId20; rewards: u128 }
      >;
      OrbiterRotation: AugmentedEvent<
        ApiType,
        [collator: AccountId20, oldOrbiter: Option<AccountId20>, newOrbiter: Option<AccountId20>],
        { collator: AccountId20; oldOrbiter: Option<AccountId20>; newOrbiter: Option<AccountId20> }
      >;
      /** An orbiter has unregistered */
      OrbiterUnregistered: AugmentedEvent<
        ApiType,
        [account: AccountId20],
        { account: AccountId20 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    multisig: {
      /** A multisig operation has been approved by someone. */
      MultisigApproval: AugmentedEvent<
        ApiType,
        [
          approving: AccountId20,
          timepoint: PalletMultisigTimepoint,
          multisig: AccountId20,
          callHash: U8aFixed
        ],
        {
          approving: AccountId20;
          timepoint: PalletMultisigTimepoint;
          multisig: AccountId20;
          callHash: U8aFixed;
        }
      >;
      /** A multisig operation has been cancelled. */
      MultisigCancelled: AugmentedEvent<
        ApiType,
        [
          cancelling: AccountId20,
          timepoint: PalletMultisigTimepoint,
          multisig: AccountId20,
          callHash: U8aFixed
        ],
        {
          cancelling: AccountId20;
          timepoint: PalletMultisigTimepoint;
          multisig: AccountId20;
          callHash: U8aFixed;
        }
      >;
      /** A multisig operation has been executed. */
      MultisigExecuted: AugmentedEvent<
        ApiType,
        [
          approving: AccountId20,
          timepoint: PalletMultisigTimepoint,
          multisig: AccountId20,
          callHash: U8aFixed,
          result: Result<Null, SpRuntimeDispatchError>
        ],
        {
          approving: AccountId20;
          timepoint: PalletMultisigTimepoint;
          multisig: AccountId20;
          callHash: U8aFixed;
          result: Result<Null, SpRuntimeDispatchError>;
        }
      >;
      /** A new multisig operation has begun. */
      NewMultisig: AugmentedEvent<
        ApiType,
        [approving: AccountId20, multisig: AccountId20, callHash: U8aFixed],
        { approving: AccountId20; multisig: AccountId20; callHash: U8aFixed }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    openTechCommitteeCollective: {
      /** A motion was approved by the required threshold. */
      Approved: AugmentedEvent<ApiType, [proposalHash: H256], { proposalHash: H256 }>;
      /** A proposal was closed because its threshold was reached or after its duration was up. */
      Closed: AugmentedEvent<
        ApiType,
        [proposalHash: H256, yes: u32, no: u32],
        { proposalHash: H256; yes: u32; no: u32 }
      >;
      /** A motion was not approved by the required threshold. */
      Disapproved: AugmentedEvent<ApiType, [proposalHash: H256], { proposalHash: H256 }>;
      /** A motion was executed; result will be `Ok` if it returned without error. */
      Executed: AugmentedEvent<
        ApiType,
        [proposalHash: H256, result: Result<Null, SpRuntimeDispatchError>],
        { proposalHash: H256; result: Result<Null, SpRuntimeDispatchError> }
      >;
      /** A single member did some action; result will be `Ok` if it returned without error. */
      MemberExecuted: AugmentedEvent<
        ApiType,
        [proposalHash: H256, result: Result<Null, SpRuntimeDispatchError>],
        { proposalHash: H256; result: Result<Null, SpRuntimeDispatchError> }
      >;
      /** A motion (given hash) has been proposed (by given account) with a threshold (given `MemberCount`). */
      Proposed: AugmentedEvent<
        ApiType,
        [account: AccountId20, proposalIndex: u32, proposalHash: H256, threshold: u32],
        { account: AccountId20; proposalIndex: u32; proposalHash: H256; threshold: u32 }
      >;
      /**
       * A motion (given hash) has been voted on by given account, leaving a tally (yes votes and no
       * votes given respectively as `MemberCount`).
       */
      Voted: AugmentedEvent<
        ApiType,
        [account: AccountId20, proposalHash: H256, voted: bool, yes: u32, no: u32],
        { account: AccountId20; proposalHash: H256; voted: bool; yes: u32; no: u32 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    parachainStaking: {
      /** Auto-compounding reward percent was set for a delegation. */
      AutoCompoundSet: AugmentedEvent<
        ApiType,
        [candidate: AccountId20, delegator: AccountId20, value: Percent],
        { candidate: AccountId20; delegator: AccountId20; value: Percent }
      >;
      /** Set blocks per round */
      BlocksPerRoundSet: AugmentedEvent<
        ApiType,
        [
          currentRound: u32,
          firstBlock: u32,
          old: u32,
          new_: u32,
          newPerRoundInflationMin: Perbill,
          newPerRoundInflationIdeal: Perbill,
          newPerRoundInflationMax: Perbill
        ],
        {
          currentRound: u32;
          firstBlock: u32;
          old: u32;
          new_: u32;
          newPerRoundInflationMin: Perbill;
          newPerRoundInflationIdeal: Perbill;
          newPerRoundInflationMax: Perbill;
        }
      >;
      /** Cancelled request to decrease candidate's bond. */
      CancelledCandidateBondLess: AugmentedEvent<
        ApiType,
        [candidate: AccountId20, amount: u128, executeRound: u32],
        { candidate: AccountId20; amount: u128; executeRound: u32 }
      >;
      /** Cancelled request to leave the set of candidates. */
      CancelledCandidateExit: AugmentedEvent<
        ApiType,
        [candidate: AccountId20],
        { candidate: AccountId20 }
      >;
      /** Cancelled request to change an existing delegation. */
      CancelledDelegationRequest: AugmentedEvent<
        ApiType,
        [
          delegator: AccountId20,
          cancelledRequest: PalletParachainStakingDelegationRequestsCancelledScheduledRequest,
          collator: AccountId20
        ],
        {
          delegator: AccountId20;
          cancelledRequest: PalletParachainStakingDelegationRequestsCancelledScheduledRequest;
          collator: AccountId20;
        }
      >;
      /** Candidate rejoins the set of collator candidates. */
      CandidateBackOnline: AugmentedEvent<
        ApiType,
        [candidate: AccountId20],
        { candidate: AccountId20 }
      >;
      /** Candidate has decreased a self bond. */
      CandidateBondedLess: AugmentedEvent<
        ApiType,
        [candidate: AccountId20, amount: u128, newBond: u128],
        { candidate: AccountId20; amount: u128; newBond: u128 }
      >;
      /** Candidate has increased a self bond. */
      CandidateBondedMore: AugmentedEvent<
        ApiType,
        [candidate: AccountId20, amount: u128, newTotalBond: u128],
        { candidate: AccountId20; amount: u128; newTotalBond: u128 }
      >;
      /** Candidate requested to decrease a self bond. */
      CandidateBondLessRequested: AugmentedEvent<
        ApiType,
        [candidate: AccountId20, amountToDecrease: u128, executeRound: u32],
        { candidate: AccountId20; amountToDecrease: u128; executeRound: u32 }
      >;
      /** Candidate has left the set of candidates. */
      CandidateLeft: AugmentedEvent<
        ApiType,
        [exCandidate: AccountId20, unlockedAmount: u128, newTotalAmtLocked: u128],
        { exCandidate: AccountId20; unlockedAmount: u128; newTotalAmtLocked: u128 }
      >;
      /** Candidate has requested to leave the set of candidates. */
      CandidateScheduledExit: AugmentedEvent<
        ApiType,
        [exitAllowedRound: u32, candidate: AccountId20, scheduledExit: u32],
        { exitAllowedRound: u32; candidate: AccountId20; scheduledExit: u32 }
      >;
      /** Candidate temporarily leave the set of collator candidates without unbonding. */
      CandidateWentOffline: AugmentedEvent<
        ApiType,
        [candidate: AccountId20],
        { candidate: AccountId20 }
      >;
      /** Candidate selected for collators. Total Exposed Amount includes all delegations. */
      CollatorChosen: AugmentedEvent<
        ApiType,
        [round: u32, collatorAccount: AccountId20, totalExposedAmount: u128],
        { round: u32; collatorAccount: AccountId20; totalExposedAmount: u128 }
      >;
      /** Set collator commission to this value. */
      CollatorCommissionSet: AugmentedEvent<
        ApiType,
        [old: Perbill, new_: Perbill],
        { old: Perbill; new_: Perbill }
      >;
      /** Compounded a portion of rewards towards the delegation. */
      Compounded: AugmentedEvent<
        ApiType,
        [candidate: AccountId20, delegator: AccountId20, amount: u128],
        { candidate: AccountId20; delegator: AccountId20; amount: u128 }
      >;
      /** New delegation (increase of the existing one). */
      Delegation: AugmentedEvent<
        ApiType,
        [
          delegator: AccountId20,
          lockedAmount: u128,
          candidate: AccountId20,
          delegatorPosition: PalletParachainStakingDelegatorAdded,
          autoCompound: Percent
        ],
        {
          delegator: AccountId20;
          lockedAmount: u128;
          candidate: AccountId20;
          delegatorPosition: PalletParachainStakingDelegatorAdded;
          autoCompound: Percent;
        }
      >;
      DelegationDecreased: AugmentedEvent<
        ApiType,
        [delegator: AccountId20, candidate: AccountId20, amount: u128, inTop: bool],
        { delegator: AccountId20; candidate: AccountId20; amount: u128; inTop: bool }
      >;
      /** Delegator requested to decrease a bond for the collator candidate. */
      DelegationDecreaseScheduled: AugmentedEvent<
        ApiType,
        [delegator: AccountId20, candidate: AccountId20, amountToDecrease: u128, executeRound: u32],
        {
          delegator: AccountId20;
          candidate: AccountId20;
          amountToDecrease: u128;
          executeRound: u32;
        }
      >;
      DelegationIncreased: AugmentedEvent<
        ApiType,
        [delegator: AccountId20, candidate: AccountId20, amount: u128, inTop: bool],
        { delegator: AccountId20; candidate: AccountId20; amount: u128; inTop: bool }
      >;
      /** Delegation kicked. */
      DelegationKicked: AugmentedEvent<
        ApiType,
        [delegator: AccountId20, candidate: AccountId20, unstakedAmount: u128],
        { delegator: AccountId20; candidate: AccountId20; unstakedAmount: u128 }
      >;
      /** Delegator requested to revoke delegation. */
      DelegationRevocationScheduled: AugmentedEvent<
        ApiType,
        [round: u32, delegator: AccountId20, candidate: AccountId20, scheduledExit: u32],
        { round: u32; delegator: AccountId20; candidate: AccountId20; scheduledExit: u32 }
      >;
      /** Delegation revoked. */
      DelegationRevoked: AugmentedEvent<
        ApiType,
        [delegator: AccountId20, candidate: AccountId20, unstakedAmount: u128],
        { delegator: AccountId20; candidate: AccountId20; unstakedAmount: u128 }
      >;
      /** Cancelled a pending request to exit the set of delegators. */
      DelegatorExitCancelled: AugmentedEvent<
        ApiType,
        [delegator: AccountId20],
        { delegator: AccountId20 }
      >;
      /** Delegator requested to leave the set of delegators. */
      DelegatorExitScheduled: AugmentedEvent<
        ApiType,
        [round: u32, delegator: AccountId20, scheduledExit: u32],
        { round: u32; delegator: AccountId20; scheduledExit: u32 }
      >;
      /** Delegator has left the set of delegators. */
      DelegatorLeft: AugmentedEvent<
        ApiType,
        [delegator: AccountId20, unstakedAmount: u128],
        { delegator: AccountId20; unstakedAmount: u128 }
      >;
      /** Delegation from candidate state has been remove. */
      DelegatorLeftCandidate: AugmentedEvent<
        ApiType,
        [
          delegator: AccountId20,
          candidate: AccountId20,
          unstakedAmount: u128,
          totalCandidateStaked: u128
        ],
        {
          delegator: AccountId20;
          candidate: AccountId20;
          unstakedAmount: u128;
          totalCandidateStaked: u128;
        }
      >;
      /** Annual inflation input (first 3) was used to derive new per-round inflation (last 3) */
      InflationSet: AugmentedEvent<
        ApiType,
        [
          annualMin: Perbill,
          annualIdeal: Perbill,
          annualMax: Perbill,
          roundMin: Perbill,
          roundIdeal: Perbill,
          roundMax: Perbill
        ],
        {
          annualMin: Perbill;
          annualIdeal: Perbill;
          annualMax: Perbill;
          roundMin: Perbill;
          roundIdeal: Perbill;
          roundMax: Perbill;
        }
      >;
      /** Account joined the set of collator candidates. */
      JoinedCollatorCandidates: AugmentedEvent<
        ApiType,
        [account: AccountId20, amountLocked: u128, newTotalAmtLocked: u128],
        { account: AccountId20; amountLocked: u128; newTotalAmtLocked: u128 }
      >;
      /** Started new round. */
      NewRound: AugmentedEvent<
        ApiType,
        [startingBlock: u32, round: u32, selectedCollatorsNumber: u32, totalBalance: u128],
        { startingBlock: u32; round: u32; selectedCollatorsNumber: u32; totalBalance: u128 }
      >;
      /** Account (re)set for parachain bond treasury. */
      ParachainBondAccountSet: AugmentedEvent<
        ApiType,
        [old: AccountId20, new_: AccountId20],
        { old: AccountId20; new_: AccountId20 }
      >;
      /** Percent of inflation reserved for parachain bond (re)set. */
      ParachainBondReservePercentSet: AugmentedEvent<
        ApiType,
        [old: Percent, new_: Percent],
        { old: Percent; new_: Percent }
      >;
      /** Transferred to account which holds funds reserved for parachain bond. */
      ReservedForParachainBond: AugmentedEvent<
        ApiType,
        [account: AccountId20, value: u128],
        { account: AccountId20; value: u128 }
      >;
      /** Paid the account (delegator or collator) the balance as liquid rewards. */
      Rewarded: AugmentedEvent<
        ApiType,
        [account: AccountId20, rewards: u128],
        { account: AccountId20; rewards: u128 }
      >;
      /** Staking expectations set. */
      StakeExpectationsSet: AugmentedEvent<
        ApiType,
        [expectMin: u128, expectIdeal: u128, expectMax: u128],
        { expectMin: u128; expectIdeal: u128; expectMax: u128 }
      >;
      /** Set total selected candidates to this value. */
      TotalSelectedSet: AugmentedEvent<ApiType, [old: u32, new_: u32], { old: u32; new_: u32 }>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    parachainSystem: {
      /** Downward messages were processed using the given weight. */
      DownwardMessagesProcessed: AugmentedEvent<
        ApiType,
        [weightUsed: SpWeightsWeightV2Weight, dmqHead: H256],
        { weightUsed: SpWeightsWeightV2Weight; dmqHead: H256 }
      >;
      /** Some downward messages have been received and will be processed. */
      DownwardMessagesReceived: AugmentedEvent<ApiType, [count: u32], { count: u32 }>;
      /** An upward message was sent to the relay chain. */
      UpwardMessageSent: AugmentedEvent<
        ApiType,
        [messageHash: Option<U8aFixed>],
        { messageHash: Option<U8aFixed> }
      >;
      /** The validation function was applied as of the contained relay chain block number. */
      ValidationFunctionApplied: AugmentedEvent<
        ApiType,
        [relayChainBlockNum: u32],
        { relayChainBlockNum: u32 }
      >;
      /** The relay-chain aborted the upgrade process. */
      ValidationFunctionDiscarded: AugmentedEvent<ApiType, []>;
      /** The validation function has been scheduled to apply. */
      ValidationFunctionStored: AugmentedEvent<ApiType, []>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    polkadotXcm: {
      /** Some assets have been claimed from an asset trap */
      AssetsClaimed: AugmentedEvent<
        ApiType,
        [hash_: H256, origin: StagingXcmV4Location, assets: XcmVersionedAssets],
        { hash_: H256; origin: StagingXcmV4Location; assets: XcmVersionedAssets }
      >;
      /** Some assets have been placed in an asset trap. */
      AssetsTrapped: AugmentedEvent<
        ApiType,
        [hash_: H256, origin: StagingXcmV4Location, assets: XcmVersionedAssets],
        { hash_: H256; origin: StagingXcmV4Location; assets: XcmVersionedAssets }
      >;
      /** Execution of an XCM message was attempted. */
      Attempted: AugmentedEvent<
        ApiType,
        [outcome: StagingXcmV4TraitsOutcome],
        { outcome: StagingXcmV4TraitsOutcome }
      >;
      /** Fees were paid from a location for an operation (often for using `SendXcm`). */
      FeesPaid: AugmentedEvent<
        ApiType,
        [paying: StagingXcmV4Location, fees: StagingXcmV4AssetAssets],
        { paying: StagingXcmV4Location; fees: StagingXcmV4AssetAssets }
      >;
      /**
       * Expected query response has been received but the querier location of the response does not
       * match the expected. The query remains registered for a later, valid, response to be
       * received and acted upon.
       */
      InvalidQuerier: AugmentedEvent<
        ApiType,
        [
          origin: StagingXcmV4Location,
          queryId: u64,
          expectedQuerier: StagingXcmV4Location,
          maybeActualQuerier: Option<StagingXcmV4Location>
        ],
        {
          origin: StagingXcmV4Location;
          queryId: u64;
          expectedQuerier: StagingXcmV4Location;
          maybeActualQuerier: Option<StagingXcmV4Location>;
        }
      >;
      /**
       * Expected query response has been received but the expected querier location placed in
       * storage by this runtime previously cannot be decoded. The query remains registered.
       *
       * This is unexpected (since a location placed in storage in a previously executing runtime
       * should be readable prior to query timeout) and dangerous since the possibly valid response
       * will be dropped. Manual governance intervention is probably going to be needed.
       */
      InvalidQuerierVersion: AugmentedEvent<
        ApiType,
        [origin: StagingXcmV4Location, queryId: u64],
        { origin: StagingXcmV4Location; queryId: u64 }
      >;
      /**
       * Expected query response has been received but the origin location of the response does not
       * match that expected. The query remains registered for a later, valid, response to be
       * received and acted upon.
       */
      InvalidResponder: AugmentedEvent<
        ApiType,
        [
          origin: StagingXcmV4Location,
          queryId: u64,
          expectedLocation: Option<StagingXcmV4Location>
        ],
        {
          origin: StagingXcmV4Location;
          queryId: u64;
          expectedLocation: Option<StagingXcmV4Location>;
        }
      >;
      /**
       * Expected query response has been received but the expected origin location placed in
       * storage by this runtime previously cannot be decoded. The query remains registered.
       *
       * This is unexpected (since a location placed in storage in a previously executing runtime
       * should be readable prior to query timeout) and dangerous since the possibly valid response
       * will be dropped. Manual governance intervention is probably going to be needed.
       */
      InvalidResponderVersion: AugmentedEvent<
        ApiType,
        [origin: StagingXcmV4Location, queryId: u64],
        { origin: StagingXcmV4Location; queryId: u64 }
      >;
      /**
       * Query response has been received and query is removed. The registered notification has been
       * dispatched and executed successfully.
       */
      Notified: AugmentedEvent<
        ApiType,
        [queryId: u64, palletIndex: u8, callIndex: u8],
        { queryId: u64; palletIndex: u8; callIndex: u8 }
      >;
      /**
       * Query response has been received and query is removed. The dispatch was unable to be
       * decoded into a `Call`; this might be due to dispatch function having a signature which is
       * not `(origin, QueryId, Response)`.
       */
      NotifyDecodeFailed: AugmentedEvent<
        ApiType,
        [queryId: u64, palletIndex: u8, callIndex: u8],
        { queryId: u64; palletIndex: u8; callIndex: u8 }
      >;
      /**
       * Query response has been received and query is removed. There was a general error with
       * dispatching the notification call.
       */
      NotifyDispatchError: AugmentedEvent<
        ApiType,
        [queryId: u64, palletIndex: u8, callIndex: u8],
        { queryId: u64; palletIndex: u8; callIndex: u8 }
      >;
      /**
       * Query response has been received and query is removed. The registered notification could
       * not be dispatched because the dispatch weight is greater than the maximum weight originally
       * budgeted by this runtime for the query result.
       */
      NotifyOverweight: AugmentedEvent<
        ApiType,
        [
          queryId: u64,
          palletIndex: u8,
          callIndex: u8,
          actualWeight: SpWeightsWeightV2Weight,
          maxBudgetedWeight: SpWeightsWeightV2Weight
        ],
        {
          queryId: u64;
          palletIndex: u8;
          callIndex: u8;
          actualWeight: SpWeightsWeightV2Weight;
          maxBudgetedWeight: SpWeightsWeightV2Weight;
        }
      >;
      /**
       * A given location which had a version change subscription was dropped owing to an error
       * migrating the location to our new XCM format.
       */
      NotifyTargetMigrationFail: AugmentedEvent<
        ApiType,
        [location: XcmVersionedLocation, queryId: u64],
        { location: XcmVersionedLocation; queryId: u64 }
      >;
      /**
       * A given location which had a version change subscription was dropped owing to an error
       * sending the notification to it.
       */
      NotifyTargetSendFail: AugmentedEvent<
        ApiType,
        [location: StagingXcmV4Location, queryId: u64, error: XcmV3TraitsError],
        { location: StagingXcmV4Location; queryId: u64; error: XcmV3TraitsError }
      >;
      /**
       * Query response has been received and is ready for taking with `take_response`. There is no
       * registered notification call.
       */
      ResponseReady: AugmentedEvent<
        ApiType,
        [queryId: u64, response: StagingXcmV4Response],
        { queryId: u64; response: StagingXcmV4Response }
      >;
      /** Received query response has been read and removed. */
      ResponseTaken: AugmentedEvent<ApiType, [queryId: u64], { queryId: u64 }>;
      /** A XCM message was sent. */
      Sent: AugmentedEvent<
        ApiType,
        [
          origin: StagingXcmV4Location,
          destination: StagingXcmV4Location,
          message: StagingXcmV4Xcm,
          messageId: U8aFixed
        ],
        {
          origin: StagingXcmV4Location;
          destination: StagingXcmV4Location;
          message: StagingXcmV4Xcm;
          messageId: U8aFixed;
        }
      >;
      /**
       * The supported version of a location has been changed. This might be through an automatic
       * notification or a manual intervention.
       */
      SupportedVersionChanged: AugmentedEvent<
        ApiType,
        [location: StagingXcmV4Location, version: u32],
        { location: StagingXcmV4Location; version: u32 }
      >;
      /**
       * Query response received which does not match a registered query. This may be because a
       * matching query was never registered, it may be because it is a duplicate response, or
       * because the query timed out.
       */
      UnexpectedResponse: AugmentedEvent<
        ApiType,
        [origin: StagingXcmV4Location, queryId: u64],
        { origin: StagingXcmV4Location; queryId: u64 }
      >;
      /**
       * An XCM version change notification message has been attempted to be sent.
       *
       * The cost of sending it (borne by the chain) is included.
       */
      VersionChangeNotified: AugmentedEvent<
        ApiType,
        [
          destination: StagingXcmV4Location,
          result: u32,
          cost: StagingXcmV4AssetAssets,
          messageId: U8aFixed
        ],
        {
          destination: StagingXcmV4Location;
          result: u32;
          cost: StagingXcmV4AssetAssets;
          messageId: U8aFixed;
        }
      >;
      /** A XCM version migration finished. */
      VersionMigrationFinished: AugmentedEvent<ApiType, [version: u32], { version: u32 }>;
      /** We have requested that a remote chain send us XCM version change notifications. */
      VersionNotifyRequested: AugmentedEvent<
        ApiType,
        [destination: StagingXcmV4Location, cost: StagingXcmV4AssetAssets, messageId: U8aFixed],
        { destination: StagingXcmV4Location; cost: StagingXcmV4AssetAssets; messageId: U8aFixed }
      >;
      /**
       * A remote has requested XCM version change notification from us and we have honored it. A
       * version information message is sent to them and its cost is included.
       */
      VersionNotifyStarted: AugmentedEvent<
        ApiType,
        [destination: StagingXcmV4Location, cost: StagingXcmV4AssetAssets, messageId: U8aFixed],
        { destination: StagingXcmV4Location; cost: StagingXcmV4AssetAssets; messageId: U8aFixed }
      >;
      /** We have requested that a remote chain stops sending us XCM version change notifications. */
      VersionNotifyUnrequested: AugmentedEvent<
        ApiType,
        [destination: StagingXcmV4Location, cost: StagingXcmV4AssetAssets, messageId: U8aFixed],
        { destination: StagingXcmV4Location; cost: StagingXcmV4AssetAssets; messageId: U8aFixed }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    preimage: {
      /** A preimage has ben cleared. */
      Cleared: AugmentedEvent<ApiType, [hash_: H256], { hash_: H256 }>;
      /** A preimage has been noted. */
      Noted: AugmentedEvent<ApiType, [hash_: H256], { hash_: H256 }>;
      /** A preimage has been requested. */
      Requested: AugmentedEvent<ApiType, [hash_: H256], { hash_: H256 }>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    proxy: {
      /** An announcement was placed to make a call in the future. */
      Announced: AugmentedEvent<
        ApiType,
        [real: AccountId20, proxy: AccountId20, callHash: H256],
        { real: AccountId20; proxy: AccountId20; callHash: H256 }
      >;
      /** A proxy was added. */
      ProxyAdded: AugmentedEvent<
        ApiType,
        [
          delegator: AccountId20,
          delegatee: AccountId20,
          proxyType: MoonbeamRuntimeProxyType,
          delay: u32
        ],
        {
          delegator: AccountId20;
          delegatee: AccountId20;
          proxyType: MoonbeamRuntimeProxyType;
          delay: u32;
        }
      >;
      /** A proxy was executed correctly, with the given. */
      ProxyExecuted: AugmentedEvent<
        ApiType,
        [result: Result<Null, SpRuntimeDispatchError>],
        { result: Result<Null, SpRuntimeDispatchError> }
      >;
      /** A proxy was removed. */
      ProxyRemoved: AugmentedEvent<
        ApiType,
        [
          delegator: AccountId20,
          delegatee: AccountId20,
          proxyType: MoonbeamRuntimeProxyType,
          delay: u32
        ],
        {
          delegator: AccountId20;
          delegatee: AccountId20;
          proxyType: MoonbeamRuntimeProxyType;
          delay: u32;
        }
      >;
      /** A pure account has been created by new proxy with given disambiguation index and proxy type. */
      PureCreated: AugmentedEvent<
        ApiType,
        [
          pure: AccountId20,
          who: AccountId20,
          proxyType: MoonbeamRuntimeProxyType,
          disambiguationIndex: u16
        ],
        {
          pure: AccountId20;
          who: AccountId20;
          proxyType: MoonbeamRuntimeProxyType;
          disambiguationIndex: u16;
        }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    randomness: {
      RandomnessRequestedBabeEpoch: AugmentedEvent<
        ApiType,
        [
          id: u64,
          refundAddress: H160,
          contractAddress: H160,
          fee: u128,
          gasLimit: u64,
          numWords: u8,
          salt: H256,
          earliestEpoch: u64
        ],
        {
          id: u64;
          refundAddress: H160;
          contractAddress: H160;
          fee: u128;
          gasLimit: u64;
          numWords: u8;
          salt: H256;
          earliestEpoch: u64;
        }
      >;
      RandomnessRequestedLocal: AugmentedEvent<
        ApiType,
        [
          id: u64,
          refundAddress: H160,
          contractAddress: H160,
          fee: u128,
          gasLimit: u64,
          numWords: u8,
          salt: H256,
          earliestBlock: u32
        ],
        {
          id: u64;
          refundAddress: H160;
          contractAddress: H160;
          fee: u128;
          gasLimit: u64;
          numWords: u8;
          salt: H256;
          earliestBlock: u32;
        }
      >;
      RequestExpirationExecuted: AugmentedEvent<ApiType, [id: u64], { id: u64 }>;
      RequestFeeIncreased: AugmentedEvent<
        ApiType,
        [id: u64, newFee: u128],
        { id: u64; newFee: u128 }
      >;
      RequestFulfilled: AugmentedEvent<ApiType, [id: u64], { id: u64 }>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    referenda: {
      /** A referendum has been approved and its proposal has been scheduled. */
      Approved: AugmentedEvent<ApiType, [index: u32], { index: u32 }>;
      /** A referendum has been cancelled. */
      Cancelled: AugmentedEvent<
        ApiType,
        [index: u32, tally: PalletConvictionVotingTally],
        { index: u32; tally: PalletConvictionVotingTally }
      >;
      ConfirmAborted: AugmentedEvent<ApiType, [index: u32], { index: u32 }>;
      /** A referendum has ended its confirmation phase and is ready for approval. */
      Confirmed: AugmentedEvent<
        ApiType,
        [index: u32, tally: PalletConvictionVotingTally],
        { index: u32; tally: PalletConvictionVotingTally }
      >;
      ConfirmStarted: AugmentedEvent<ApiType, [index: u32], { index: u32 }>;
      /** The decision deposit has been placed. */
      DecisionDepositPlaced: AugmentedEvent<
        ApiType,
        [index: u32, who: AccountId20, amount: u128],
        { index: u32; who: AccountId20; amount: u128 }
      >;
      /** The decision deposit has been refunded. */
      DecisionDepositRefunded: AugmentedEvent<
        ApiType,
        [index: u32, who: AccountId20, amount: u128],
        { index: u32; who: AccountId20; amount: u128 }
      >;
      /** A referendum has moved into the deciding phase. */
      DecisionStarted: AugmentedEvent<
        ApiType,
        [
          index: u32,
          track: u16,
          proposal: FrameSupportPreimagesBounded,
          tally: PalletConvictionVotingTally
        ],
        {
          index: u32;
          track: u16;
          proposal: FrameSupportPreimagesBounded;
          tally: PalletConvictionVotingTally;
        }
      >;
      /** A deposit has been slashed. */
      DepositSlashed: AugmentedEvent<
        ApiType,
        [who: AccountId20, amount: u128],
        { who: AccountId20; amount: u128 }
      >;
      /** A referendum has been killed. */
      Killed: AugmentedEvent<
        ApiType,
        [index: u32, tally: PalletConvictionVotingTally],
        { index: u32; tally: PalletConvictionVotingTally }
      >;
      /** Metadata for a referendum has been cleared. */
      MetadataCleared: AugmentedEvent<
        ApiType,
        [index: u32, hash_: H256],
        { index: u32; hash_: H256 }
      >;
      /** Metadata for a referendum has been set. */
      MetadataSet: AugmentedEvent<ApiType, [index: u32, hash_: H256], { index: u32; hash_: H256 }>;
      /** A proposal has been rejected by referendum. */
      Rejected: AugmentedEvent<
        ApiType,
        [index: u32, tally: PalletConvictionVotingTally],
        { index: u32; tally: PalletConvictionVotingTally }
      >;
      /** The submission deposit has been refunded. */
      SubmissionDepositRefunded: AugmentedEvent<
        ApiType,
        [index: u32, who: AccountId20, amount: u128],
        { index: u32; who: AccountId20; amount: u128 }
      >;
      /** A referendum has been submitted. */
      Submitted: AugmentedEvent<
        ApiType,
        [index: u32, track: u16, proposal: FrameSupportPreimagesBounded],
        { index: u32; track: u16; proposal: FrameSupportPreimagesBounded }
      >;
      /** A referendum has been timed out without being decided. */
      TimedOut: AugmentedEvent<
        ApiType,
        [index: u32, tally: PalletConvictionVotingTally],
        { index: u32; tally: PalletConvictionVotingTally }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    rootTesting: {
      /** Event dispatched when the trigger_defensive extrinsic is called. */
      DefensiveTestCall: AugmentedEvent<ApiType, []>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    scheduler: {
      /** The call for the provided hash was not found so the task has been aborted. */
      CallUnavailable: AugmentedEvent<
        ApiType,
        [task: ITuple<[u32, u32]>, id: Option<U8aFixed>],
        { task: ITuple<[u32, u32]>; id: Option<U8aFixed> }
      >;
      /** Canceled some task. */
      Canceled: AugmentedEvent<ApiType, [when: u32, index: u32], { when: u32; index: u32 }>;
      /** Dispatched some task. */
      Dispatched: AugmentedEvent<
        ApiType,
        [
          task: ITuple<[u32, u32]>,
          id: Option<U8aFixed>,
          result: Result<Null, SpRuntimeDispatchError>
        ],
        {
          task: ITuple<[u32, u32]>;
          id: Option<U8aFixed>;
          result: Result<Null, SpRuntimeDispatchError>;
        }
      >;
      /** The given task was unable to be renewed since the agenda is full at that block. */
      PeriodicFailed: AugmentedEvent<
        ApiType,
        [task: ITuple<[u32, u32]>, id: Option<U8aFixed>],
        { task: ITuple<[u32, u32]>; id: Option<U8aFixed> }
      >;
      /** The given task can never be executed since it is overweight. */
      PermanentlyOverweight: AugmentedEvent<
        ApiType,
        [task: ITuple<[u32, u32]>, id: Option<U8aFixed>],
        { task: ITuple<[u32, u32]>; id: Option<U8aFixed> }
      >;
      /** Cancel a retry configuration for some task. */
      RetryCancelled: AugmentedEvent<
        ApiType,
        [task: ITuple<[u32, u32]>, id: Option<U8aFixed>],
        { task: ITuple<[u32, u32]>; id: Option<U8aFixed> }
      >;
      /**
       * The given task was unable to be retried since the agenda is full at that block or there was
       * not enough weight to reschedule it.
       */
      RetryFailed: AugmentedEvent<
        ApiType,
        [task: ITuple<[u32, u32]>, id: Option<U8aFixed>],
        { task: ITuple<[u32, u32]>; id: Option<U8aFixed> }
      >;
      /** Set a retry configuration for some task. */
      RetrySet: AugmentedEvent<
        ApiType,
        [task: ITuple<[u32, u32]>, id: Option<U8aFixed>, period: u32, retries: u8],
        { task: ITuple<[u32, u32]>; id: Option<U8aFixed>; period: u32; retries: u8 }
      >;
      /** Scheduled some task. */
      Scheduled: AugmentedEvent<ApiType, [when: u32, index: u32], { when: u32; index: u32 }>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    system: {
      /** `:code` was updated. */
      CodeUpdated: AugmentedEvent<ApiType, []>;
      /** An extrinsic failed. */
      ExtrinsicFailed: AugmentedEvent<
        ApiType,
        [dispatchError: SpRuntimeDispatchError, dispatchInfo: FrameSupportDispatchDispatchInfo],
        { dispatchError: SpRuntimeDispatchError; dispatchInfo: FrameSupportDispatchDispatchInfo }
      >;
      /** An extrinsic completed successfully. */
      ExtrinsicSuccess: AugmentedEvent<
        ApiType,
        [dispatchInfo: FrameSupportDispatchDispatchInfo],
        { dispatchInfo: FrameSupportDispatchDispatchInfo }
      >;
      /** An account was reaped. */
      KilledAccount: AugmentedEvent<ApiType, [account: AccountId20], { account: AccountId20 }>;
      /** A new account was created. */
      NewAccount: AugmentedEvent<ApiType, [account: AccountId20], { account: AccountId20 }>;
      /** On on-chain remark happened. */
      Remarked: AugmentedEvent<
        ApiType,
        [sender: AccountId20, hash_: H256],
        { sender: AccountId20; hash_: H256 }
      >;
      /** An upgrade was authorized. */
      UpgradeAuthorized: AugmentedEvent<
        ApiType,
        [codeHash: H256, checkVersion: bool],
        { codeHash: H256; checkVersion: bool }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    transactionPayment: {
      /**
       * A transaction fee `actual_fee`, of which `tip` was added to the minimum inclusion fee, has
       * been paid by `who`.
       */
      TransactionFeePaid: AugmentedEvent<
        ApiType,
        [who: AccountId20, actualFee: u128, tip: u128],
        { who: AccountId20; actualFee: u128; tip: u128 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    treasury: {
      /** A new asset spend proposal has been approved. */
      AssetSpendApproved: AugmentedEvent<
        ApiType,
        [
          index: u32,
          assetKind: Null,
          amount: u128,
          beneficiary: AccountId20,
          validFrom: u32,
          expireAt: u32
        ],
        {
          index: u32;
          assetKind: Null;
          amount: u128;
          beneficiary: AccountId20;
          validFrom: u32;
          expireAt: u32;
        }
      >;
      /** An approved spend was voided. */
      AssetSpendVoided: AugmentedEvent<ApiType, [index: u32], { index: u32 }>;
      /** Some funds have been allocated. */
      Awarded: AugmentedEvent<
        ApiType,
        [proposalIndex: u32, award: u128, account: AccountId20],
        { proposalIndex: u32; award: u128; account: AccountId20 }
      >;
      /** Some of our funds have been burnt. */
      Burnt: AugmentedEvent<ApiType, [burntFunds: u128], { burntFunds: u128 }>;
      /** Some funds have been deposited. */
      Deposit: AugmentedEvent<ApiType, [value: u128], { value: u128 }>;
      /** A payment happened. */
      Paid: AugmentedEvent<ApiType, [index: u32, paymentId: Null], { index: u32; paymentId: Null }>;
      /** A payment failed and can be retried. */
      PaymentFailed: AugmentedEvent<
        ApiType,
        [index: u32, paymentId: Null],
        { index: u32; paymentId: Null }
      >;
      /** New proposal. */
      Proposed: AugmentedEvent<ApiType, [proposalIndex: u32], { proposalIndex: u32 }>;
      /** A proposal was rejected; funds were slashed. */
      Rejected: AugmentedEvent<
        ApiType,
        [proposalIndex: u32, slashed: u128],
        { proposalIndex: u32; slashed: u128 }
      >;
      /** Spending has finished; this is the amount that rolls over until next spend. */
      Rollover: AugmentedEvent<ApiType, [rolloverBalance: u128], { rolloverBalance: u128 }>;
      /** A new spend proposal has been approved. */
      SpendApproved: AugmentedEvent<
        ApiType,
        [proposalIndex: u32, amount: u128, beneficiary: AccountId20],
        { proposalIndex: u32; amount: u128; beneficiary: AccountId20 }
      >;
      /** We have ended a spend period and will now allocate funds. */
      Spending: AugmentedEvent<ApiType, [budgetRemaining: u128], { budgetRemaining: u128 }>;
      /**
       * A spend was processed and removed from the storage. It might have been successfully paid or
       * it may have expired.
       */
      SpendProcessed: AugmentedEvent<ApiType, [index: u32], { index: u32 }>;
      /** The inactive funds of the pallet have been updated. */
      UpdatedInactive: AugmentedEvent<
        ApiType,
        [reactivated: u128, deactivated: u128],
        { reactivated: u128; deactivated: u128 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    treasuryCouncilCollective: {
      /** A motion was approved by the required threshold. */
      Approved: AugmentedEvent<ApiType, [proposalHash: H256], { proposalHash: H256 }>;
      /** A proposal was closed because its threshold was reached or after its duration was up. */
      Closed: AugmentedEvent<
        ApiType,
        [proposalHash: H256, yes: u32, no: u32],
        { proposalHash: H256; yes: u32; no: u32 }
      >;
      /** A motion was not approved by the required threshold. */
      Disapproved: AugmentedEvent<ApiType, [proposalHash: H256], { proposalHash: H256 }>;
      /** A motion was executed; result will be `Ok` if it returned without error. */
      Executed: AugmentedEvent<
        ApiType,
        [proposalHash: H256, result: Result<Null, SpRuntimeDispatchError>],
        { proposalHash: H256; result: Result<Null, SpRuntimeDispatchError> }
      >;
      /** A single member did some action; result will be `Ok` if it returned without error. */
      MemberExecuted: AugmentedEvent<
        ApiType,
        [proposalHash: H256, result: Result<Null, SpRuntimeDispatchError>],
        { proposalHash: H256; result: Result<Null, SpRuntimeDispatchError> }
      >;
      /** A motion (given hash) has been proposed (by given account) with a threshold (given `MemberCount`). */
      Proposed: AugmentedEvent<
        ApiType,
        [account: AccountId20, proposalIndex: u32, proposalHash: H256, threshold: u32],
        { account: AccountId20; proposalIndex: u32; proposalHash: H256; threshold: u32 }
      >;
      /**
       * A motion (given hash) has been voted on by given account, leaving a tally (yes votes and no
       * votes given respectively as `MemberCount`).
       */
      Voted: AugmentedEvent<
        ApiType,
        [account: AccountId20, proposalHash: H256, voted: bool, yes: u32, no: u32],
        { account: AccountId20; proposalHash: H256; voted: bool; yes: u32; no: u32 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    utility: {
      /** Batch of dispatches completed fully with no error. */
      BatchCompleted: AugmentedEvent<ApiType, []>;
      /** Batch of dispatches completed but has errors. */
      BatchCompletedWithErrors: AugmentedEvent<ApiType, []>;
      /**
       * Batch of dispatches did not complete fully. Index of first failing dispatch given, as well
       * as the error.
       */
      BatchInterrupted: AugmentedEvent<
        ApiType,
        [index: u32, error: SpRuntimeDispatchError],
        { index: u32; error: SpRuntimeDispatchError }
      >;
      /** A call was dispatched. */
      DispatchedAs: AugmentedEvent<
        ApiType,
        [result: Result<Null, SpRuntimeDispatchError>],
        { result: Result<Null, SpRuntimeDispatchError> }
      >;
      /** A single item within a Batch of dispatches has completed with no error. */
      ItemCompleted: AugmentedEvent<ApiType, []>;
      /** A single item within a Batch of dispatches has completed with error. */
      ItemFailed: AugmentedEvent<
        ApiType,
        [error: SpRuntimeDispatchError],
        { error: SpRuntimeDispatchError }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    whitelist: {
      CallWhitelisted: AugmentedEvent<ApiType, [callHash: H256], { callHash: H256 }>;
      WhitelistedCallDispatched: AugmentedEvent<
        ApiType,
        [
          callHash: H256,
          result: Result<FrameSupportDispatchPostDispatchInfo, SpRuntimeDispatchErrorWithPostInfo>
        ],
        {
          callHash: H256;
          result: Result<FrameSupportDispatchPostDispatchInfo, SpRuntimeDispatchErrorWithPostInfo>;
        }
      >;
      WhitelistedCallRemoved: AugmentedEvent<ApiType, [callHash: H256], { callHash: H256 }>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    xcmpQueue: {
      /** An HRMP message was sent to a sibling parachain. */
      XcmpMessageSent: AugmentedEvent<ApiType, [messageHash: U8aFixed], { messageHash: U8aFixed }>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    xcmTransactor: {
      DeRegisteredDerivative: AugmentedEvent<ApiType, [index: u16], { index: u16 }>;
      /** Set dest fee per second */
      DestFeePerSecondChanged: AugmentedEvent<
        ApiType,
        [location: StagingXcmV4Location, feePerSecond: u128],
        { location: StagingXcmV4Location; feePerSecond: u128 }
      >;
      /** Remove dest fee per second */
      DestFeePerSecondRemoved: AugmentedEvent<
        ApiType,
        [location: StagingXcmV4Location],
        { location: StagingXcmV4Location }
      >;
      /** HRMP manage action succesfully sent */
      HrmpManagementSent: AugmentedEvent<
        ApiType,
        [action: PalletXcmTransactorHrmpOperation],
        { action: PalletXcmTransactorHrmpOperation }
      >;
      /** Registered a derivative index for an account id. */
      RegisteredDerivative: AugmentedEvent<
        ApiType,
        [accountId: AccountId20, index: u16],
        { accountId: AccountId20; index: u16 }
      >;
      /** Transacted the inner call through a derivative account in a destination chain. */
      TransactedDerivative: AugmentedEvent<
        ApiType,
        [accountId: AccountId20, dest: StagingXcmV4Location, call: Bytes, index: u16],
        { accountId: AccountId20; dest: StagingXcmV4Location; call: Bytes; index: u16 }
      >;
      /** Transacted the call through a signed account in a destination chain. */
      TransactedSigned: AugmentedEvent<
        ApiType,
        [feePayer: AccountId20, dest: StagingXcmV4Location, call: Bytes],
        { feePayer: AccountId20; dest: StagingXcmV4Location; call: Bytes }
      >;
      /** Transacted the call through the sovereign account in a destination chain. */
      TransactedSovereign: AugmentedEvent<
        ApiType,
        [feePayer: Option<AccountId20>, dest: StagingXcmV4Location, call: Bytes],
        { feePayer: Option<AccountId20>; dest: StagingXcmV4Location; call: Bytes }
      >;
      /** Transact failed */
      TransactFailed: AugmentedEvent<
        ApiType,
        [error: XcmV3TraitsError],
        { error: XcmV3TraitsError }
      >;
      /** Changed the transact info of a location */
      TransactInfoChanged: AugmentedEvent<
        ApiType,
        [
          location: StagingXcmV4Location,
          remoteInfo: PalletXcmTransactorRemoteTransactInfoWithMaxWeight
        ],
        {
          location: StagingXcmV4Location;
          remoteInfo: PalletXcmTransactorRemoteTransactInfoWithMaxWeight;
        }
      >;
      /** Removed the transact info of a location */
      TransactInfoRemoved: AugmentedEvent<
        ApiType,
        [location: StagingXcmV4Location],
        { location: StagingXcmV4Location }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    xTokens: {
      /** Transferred `Asset` with fee. */
      TransferredAssets: AugmentedEvent<
        ApiType,
        [
          sender: AccountId20,
          assets: StagingXcmV4AssetAssets,
          fee: StagingXcmV4Asset,
          dest: StagingXcmV4Location
        ],
        {
          sender: AccountId20;
          assets: StagingXcmV4AssetAssets;
          fee: StagingXcmV4Asset;
          dest: StagingXcmV4Location;
        }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
  } // AugmentedEvents
} // declare module
