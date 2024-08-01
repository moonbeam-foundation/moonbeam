// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/calls";

import type { ApiTypes, AugmentedCall, DecoratedCallBase } from "@polkadot/api-base/types";
import type {
  Bytes,
  Null,
  Option,
  Result,
  U256,
  Vec,
  bool,
  u256,
  u32,
  u64,
} from "@polkadot/types-codec";
import type { AnyNumber, ITuple } from "@polkadot/types-codec/types";
import type { CheckInherentsResult, InherentData } from "@polkadot/types/interfaces/blockbuilder";
import type { BlockHash } from "@polkadot/types/interfaces/chain";
import type { CollationInfo } from "@polkadot/types/interfaces/cumulus";
import type {
  BlockV2,
  EthReceiptV3,
  EthTransactionStatus,
  TransactionV2,
} from "@polkadot/types/interfaces/eth";
import type { EvmAccount, EvmCallInfoV2, EvmCreateInfoV2 } from "@polkadot/types/interfaces/evm";
import type { Extrinsic } from "@polkadot/types/interfaces/extrinsics";
import type { GenesisBuildErr } from "@polkadot/types/interfaces/genesisBuilder";
import type { OpaqueMetadata } from "@polkadot/types/interfaces/metadata";
import type { FeeDetails, RuntimeDispatchInfo } from "@polkadot/types/interfaces/payment";
import type {
  AccountId,
  Balance,
  Block,
  ExtrinsicInclusionMode,
  H160,
  H256,
  Header,
  Index,
  KeyTypeId,
  Permill,
  Weight,
} from "@polkadot/types/interfaces/runtime";
import type { RuntimeVersion } from "@polkadot/types/interfaces/state";
import type { ApplyExtrinsicResult, DispatchError } from "@polkadot/types/interfaces/system";
import type { TransactionSource, TransactionValidity } from "@polkadot/types/interfaces/txqueue";
import type { IExtrinsic, Observable } from "@polkadot/types/types";

export type __AugmentedCall<ApiType extends ApiTypes> = AugmentedCall<ApiType>;
export type __DecoratedCallBase<ApiType extends ApiTypes> = DecoratedCallBase<ApiType>;

declare module "@polkadot/api-base/types/calls" {
  interface AugmentedCalls<ApiType extends ApiTypes> {
    /** 0xbc9d89904f5b923f/1 */
    accountNonceApi: {
      /** The API to query account nonce (aka transaction index) */
      accountNonce: AugmentedCall<
        ApiType,
        (accountId: AccountId | string | Uint8Array) => Observable<Index>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0x40fe3ad401f8959a/6 */
    blockBuilder: {
      /** Apply the given extrinsic. */
      applyExtrinsic: AugmentedCall<
        ApiType,
        (
          extrinsic: Extrinsic | IExtrinsic | string | Uint8Array
        ) => Observable<ApplyExtrinsicResult>
      >;
      /** Check that the inherents are valid. */
      checkInherents: AugmentedCall<
        ApiType,
        (
          block: Block | { header?: any; extrinsics?: any } | string | Uint8Array,
          data: InherentData | { data?: any } | string | Uint8Array
        ) => Observable<CheckInherentsResult>
      >;
      /** Finish the current block. */
      finalizeBlock: AugmentedCall<ApiType, () => Observable<Header>>;
      /** Generate inherent extrinsics. */
      inherentExtrinsics: AugmentedCall<
        ApiType,
        (
          inherent: InherentData | { data?: any } | string | Uint8Array
        ) => Observable<Vec<Extrinsic>>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0xea93e3f16f3d6962/2 */
    collectCollationInfo: {
      /** Collect information about a collation. */
      collectCollationInfo: AugmentedCall<
        ApiType,
        (
          header:
            | Header
            | {
                parentHash?: any;
                number?: any;
                stateRoot?: any;
                extrinsicsRoot?: any;
                digest?: any;
              }
            | string
            | Uint8Array
        ) => Observable<CollationInfo>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0xe65b00e46cedd0aa/2 */
    convertTransactionRuntimeApi: {
      /** Converts an Ethereum-style transaction to Extrinsic */
      convertTransaction: AugmentedCall<
        ApiType,
        (
          transaction:
            | TransactionV2
            | { Legacy: any }
            | { EIP2930: any }
            | { EIP1559: any }
            | string
            | Uint8Array
        ) => Observable<Extrinsic>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0xdf6acb689907609b/5 */
    core: {
      /** Execute the given block. */
      executeBlock: AugmentedCall<
        ApiType,
        (
          block: Block | { header?: any; extrinsics?: any } | string | Uint8Array
        ) => Observable<Null>
      >;
      /** Initialize a block with the given header. */
      initializeBlock: AugmentedCall<
        ApiType,
        (
          header:
            | Header
            | {
                parentHash?: any;
                number?: any;
                stateRoot?: any;
                extrinsicsRoot?: any;
                digest?: any;
              }
            | string
            | Uint8Array
        ) => Observable<ExtrinsicInclusionMode>
      >;
      /** Returns the version of the runtime. */
      version: AugmentedCall<ApiType, () => Observable<RuntimeVersion>>;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0x582211f65bb14b89/5 */
    ethereumRuntimeRPCApi: {
      /** Returns pallet_evm::Accounts by address. */
      accountBasic: AugmentedCall<
        ApiType,
        (address: H160 | string | Uint8Array) => Observable<EvmAccount>
      >;
      /** For a given account address, returns pallet_evm::AccountCodes. */
      accountCodeAt: AugmentedCall<
        ApiType,
        (address: H160 | string | Uint8Array) => Observable<Bytes>
      >;
      /** Returns the converted FindAuthor::find_author authority id. */
      author: AugmentedCall<ApiType, () => Observable<H160>>;
      /** Returns a frame_ethereum::call response. If `estimate` is true, */
      call: AugmentedCall<
        ApiType,
        (
          from: H160 | string | Uint8Array,
          to: H160 | string | Uint8Array,
          data: Bytes | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: U256 | AnyNumber | Uint8Array,
          maxFeePerGas: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          maxPriorityFeePerGas: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          nonce: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          estimate: bool | boolean | Uint8Array,
          accessList:
            | Option<Vec<ITuple<[H160, Vec<H256>]>>>
            | null
            | Uint8Array
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [H160 | string | Uint8Array, Vec<H256> | (H256 | string | Uint8Array)[]][]
        ) => Observable<Result<EvmCallInfoV2, DispatchError>>
      >;
      /** Returns runtime defined pallet_evm::ChainId. */
      chainId: AugmentedCall<ApiType, () => Observable<u64>>;
      /** Returns a frame_ethereum::call response. If `estimate` is true, */
      create: AugmentedCall<
        ApiType,
        (
          from: H160 | string | Uint8Array,
          data: Bytes | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: U256 | AnyNumber | Uint8Array,
          maxFeePerGas: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          maxPriorityFeePerGas: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          nonce: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          estimate: bool | boolean | Uint8Array,
          accessList:
            | Option<Vec<ITuple<[H160, Vec<H256>]>>>
            | null
            | Uint8Array
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [H160 | string | Uint8Array, Vec<H256> | (H256 | string | Uint8Array)[]][]
        ) => Observable<Result<EvmCreateInfoV2, DispatchError>>
      >;
      /** Return all the current data for a block in a single runtime call. */
      currentAll: AugmentedCall<
        ApiType,
        () => Observable<
          ITuple<[Option<BlockV2>, Option<Vec<EthReceiptV3>>, Option<Vec<EthTransactionStatus>>]>
        >
      >;
      /** Return the current block. */
      currentBlock: AugmentedCall<ApiType, () => Observable<BlockV2>>;
      /** Return the current receipt. */
      currentReceipts: AugmentedCall<ApiType, () => Observable<Option<Vec<EthReceiptV3>>>>;
      /** Return the current transaction status. */
      currentTransactionStatuses: AugmentedCall<
        ApiType,
        () => Observable<Option<Vec<EthTransactionStatus>>>
      >;
      /** Return the elasticity multiplier. */
      elasticity: AugmentedCall<ApiType, () => Observable<Option<Permill>>>;
      /** Receives a `Vec<OpaqueExtrinsic>` and filters all the ethereum transactions. */
      extrinsicFilter: AugmentedCall<
        ApiType,
        (
          xts: Vec<Extrinsic> | (Extrinsic | IExtrinsic | string | Uint8Array)[]
        ) => Observable<Vec<TransactionV2>>
      >;
      /** Returns FixedGasPrice::min_gas_price */
      gasPrice: AugmentedCall<ApiType, () => Observable<u256>>;
      /** For a given account address and index, returns pallet_evm::AccountStorages. */
      storageAt: AugmentedCall<
        ApiType,
        (
          address: H160 | string | Uint8Array,
          index: u256 | AnyNumber | Uint8Array
        ) => Observable<H256>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0xfbc577b9d747efd6/1 */
    genesisBuilder: {
      /** Build `RuntimeGenesisConfig` from a JSON blob not using any defaults and store it in the storage. */
      buildConfig: AugmentedCall<
        ApiType,
        (json: Bytes | string | Uint8Array) => Observable<Result<ITuple<[]>, GenesisBuildErr>>
      >;
      /** Creates the default `RuntimeGenesisConfig` and returns it as a JSON blob. */
      createDefaultConfig: AugmentedCall<ApiType, () => Observable<Bytes>>;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0x37e397fc7c91f5e4/2 */
    metadata: {
      /** Returns the metadata of a runtime */
      metadata: AugmentedCall<ApiType, () => Observable<OpaqueMetadata>>;
      /** Returns the metadata at a given version. */
      metadataAtVersion: AugmentedCall<
        ApiType,
        (version: u32 | AnyNumber | Uint8Array) => Observable<Option<OpaqueMetadata>>
      >;
      /** Returns the supported metadata versions. */
      metadataVersions: AugmentedCall<ApiType, () => Observable<Vec<u32>>>;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0x2aa62120049dd2d2/1 */
    nimbusApi: {
      /** The runtime api used to predict whether a Nimbus author will be eligible in the given slot */
      canAuthor: AugmentedCall<
        ApiType,
        (
          author: AccountId | string | Uint8Array,
          relayParent: u32 | AnyNumber | Uint8Array,
          parentHeader:
            | Header
            | {
                parentHash?: any;
                number?: any;
                stateRoot?: any;
                extrinsicsRoot?: any;
                digest?: any;
              }
            | string
            | Uint8Array
        ) => Observable<bool>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0xf78b278be53f454c/2 */
    offchainWorkerApi: {
      /** Starts the off-chain task for given block header. */
      offchainWorker: AugmentedCall<
        ApiType,
        (
          header:
            | Header
            | {
                parentHash?: any;
                number?: any;
                stateRoot?: any;
                extrinsicsRoot?: any;
                digest?: any;
              }
            | string
            | Uint8Array
        ) => Observable<Null>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0xab3c0572291feb8b/1 */
    sessionKeys: {
      /** Decode the given public session keys. */
      decodeSessionKeys: AugmentedCall<
        ApiType,
        (
          encoded: Bytes | string | Uint8Array
        ) => Observable<Option<Vec<ITuple<[Bytes, KeyTypeId]>>>>
      >;
      /** Generate a set of session keys with optionally using the given seed. */
      generateSessionKeys: AugmentedCall<
        ApiType,
        (seed: Option<Bytes> | null | Uint8Array | Bytes | string) => Observable<Bytes>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0xd2bc9897eed08f15/3 */
    taggedTransactionQueue: {
      /** Validate the transaction. */
      validateTransaction: AugmentedCall<
        ApiType,
        (
          source: TransactionSource | "InBlock" | "Local" | "External" | number | Uint8Array,
          tx: Extrinsic | IExtrinsic | string | Uint8Array,
          blockHash: BlockHash | string | Uint8Array
        ) => Observable<TransactionValidity>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
    /** 0x37c8bb1350a9a2a8/4 */
    transactionPaymentApi: {
      /** The transaction fee details */
      queryFeeDetails: AugmentedCall<
        ApiType,
        (
          uxt: Extrinsic | IExtrinsic | string | Uint8Array,
          len: u32 | AnyNumber | Uint8Array
        ) => Observable<FeeDetails>
      >;
      /** The transaction info */
      queryInfo: AugmentedCall<
        ApiType,
        (
          uxt: Extrinsic | IExtrinsic | string | Uint8Array,
          len: u32 | AnyNumber | Uint8Array
        ) => Observable<RuntimeDispatchInfo>
      >;
      /** Query the output of the current LengthToFee given some input */
      queryLengthToFee: AugmentedCall<
        ApiType,
        (length: u32 | AnyNumber | Uint8Array) => Observable<Balance>
      >;
      /** Query the output of the current WeightToFee given some input */
      queryWeightToFee: AugmentedCall<
        ApiType,
        (
          weight: Weight | { refTime?: any; proofSize?: any } | string | Uint8Array
        ) => Observable<Balance>
      >;
      /** Generic call */
      [key: string]: DecoratedCallBase<ApiType>;
    };
  } // AugmentedCalls
} // declare module
