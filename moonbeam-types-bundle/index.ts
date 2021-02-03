import {
  OverrideBundleDefinition,
  OverrideBundleType,
  DefinitionRpc,
  DefinitionRpcSub,
} from "@polkadot/types/types";

// Moonbeam specific rpc methods
export const rpcDefinitions: Record<string, Record<string, DefinitionRpc | DefinitionRpcSub>> = {
  txpool: {
    content: {
      aliasSection: "txpool",
      description:
        "The detailed information regarding Ethereum transactions that are currently in the " +
        "Substrate transaction pool.",
      params: [],
      type: "TxPoolResultContent",
    },
    inspect: {
      aliasSection: "txpool",
      description:
        "Summarized information of the Ethereum transactions that are currently in the Substrate" +
        " transaction pool.",
      params: [],
      type: "TxPoolResultInspect",
    },
    status: {
      aliasSection: "txpool",
      description:
        "The number of Ethereum transaction that are currently in the Substrate transaction pool.",
      params: [],
      type: "TxPoolResultStatus",
    },
  },
};

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
          nominators: "Vec<Bond>",
          total: "Balance",
          state: "ValidatorStatus",
        },
        Bond: {
          owner: "AccountId",
          amount: "Balance",
        },
        ValidatorStatus: {
          _enum: ["Active", "Idle", "Leaving(RoundIndex)"],
        },
        TxPoolResultContent: {
          pending: "HashMap<H160, HashMap<U256, PoolTransaction>>",
          queued: "HashMap<H160, HashMap<U256, PoolTransaction>>",
        },
        TxPoolResultInspect: {
          pending: "HashMap<H160, HashMap<U256, Summary>>",
          queued: "HashMap<H160, HashMap<U256, Summary>>",
        },
        TxPoolResultStatus: {
          pending: "U256",
          queued: "U256",
        },
        Summary: "Bytes",
        PoolTransaction: {
          hash: "H256",
          nonce: "U256",
          block_hash: "Option<H256>",
          block_number: "Option<U256>",
          from: "H160",
          to: "Option<H160>",
          value: "U256",
          gas_price: "U256",
          gas: "U256",
          input: "Bytes",
        },
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
