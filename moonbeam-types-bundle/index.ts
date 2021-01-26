import {
  OverrideBundleDefinition,
  OverrideBundleType,
  DefinitionRpc,
  DefinitionRpcSub,
} from "@polkadot/types/types";

// Moonbeam specific rpc methods
const rpcDefinitions: Record<string, Record<string, DefinitionRpc | DefinitionRpcSub>> = {
  txpool: {
    content: {
      aliasSection: "txpool",
      description:
        "The detailed information regarding Ethereum transactions that are currently in the Substrate transaction pool.",
      params: [],
      type: "TxPoolResult<TransactionMap<Transaction>>",
    },
    inspect: {
      aliasSection: "txpool",
      description:
        "Summarized information of the Ethereum transactions that are currently in the Substrate transaction pool.",
      params: [],
      type: "TxPoolResult<TransactionMap<Summary>>",
    },
    status: {
      aliasSection: "txpool",
      description:
        "The number of Ethereum transaction that are currently in the Substrate transaction pool.",
      params: [],
      type: "TxPoolResult<U256>",
    },
  },
};

// #[rpc(name = "txpool_content")]
// 	fn content(&self) -> Result<TxPoolResult<TransactionMap<Transaction>>>;

// 	#[rpc(name = "txpool_inspect")]
// 	fn inspect(&self) -> Result<TxPoolResult<TransactionMap<Summary>>>;

// 	#[rpc(name = "txpool_status")]
// 	fn status(&self) -> Result<TxPoolResult<U256>>;

export const moonbeamDefinitions = {
  rpc: rpcDefinitions,
  types: [
    {
      minmax: [0, 4],
      types: {
        AccountId: "EthereumAccountId",
        Address: "AccountId",
        Balance: "u128",
        RefCount: "u8",
        LookupSource: "AccountId",
        Account: {
          nonce: "U256",
          balance: "u128",
        },
      },
    },
    {
      minmax: [5, 5],
      types: {
        AccountId: "EthereumAccountId",
        Address: "AccountId",
        Balance: "u128",
        LookupSource: "AccountId",
        Account: {
          nonce: "U256",
          balance: "u128",
        },
      },
    },
    {
      minmax: [6, undefined],
      types: {
        AccountId: "EthereumAccountId",
        Address: "AccountId",
        Balance: "u128",
        LookupSource: "AccountId",
        Account: {
          nonce: "U256",
          balance: "u128",
        },
        ExtrinsicSignature: "EthereumSignature",
        RoundIndex: "u32",
        Candidate: {
          validator: "AccountId",
          fee: "Perbill",
          nominators: "OrderedSet<Bond<AccountId, Balance>>",
          total: "Balance",
          state: "ValidatorStatus<RoundIndex>",
        },
        OrderedSet: "Vec",
      },
    },
  ],
} as OverrideBundleDefinition;

export const typesBundle = {
  spec: {
    "moonbase-alphanet": moonbeamDefinitions,
    moonbeamDefinitions,
    "moonbeam-standalone": moonbeamDefinitions,
    "node-moonbeam": moonbeamDefinitions,
  },
} as OverrideBundleType;
