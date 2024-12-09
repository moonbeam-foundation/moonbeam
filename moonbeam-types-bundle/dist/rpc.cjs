var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/rpc.ts
var rpc_exports = {};
__export(rpc_exports, {
  rpcDefinitions: () => rpcDefinitions
});
module.exports = __toCommonJS(rpc_exports);
var rpcDefinitions = {
  txpool: {
    content: {
      aliasSection: "txpool",
      description: "The detailed information regarding Ethereum transactions that are currently in the Substrate transaction pool.",
      params: [],
      type: "TxPoolResultContent"
    },
    inspect: {
      aliasSection: "txpool",
      description: "Summarized information of the Ethereum transactions that are currently in the Substrate transaction pool.",
      params: [],
      type: "TxPoolResultInspect"
    },
    status: {
      aliasSection: "txpool",
      description: "The number of Ethereum transaction that are currently in the Substrate transaction pool.",
      params: [],
      type: "TxPoolResultStatus"
    }
  },
  trace: {
    filter: {
      aliasSection: "trace",
      description: "Trace Filter",
      params: [{ name: "filter", type: "FilterRequest" }],
      type: "Result<Vec<TransactionTrace>>"
    }
  },
  debug: {
    traceTransaction: {
      aliasSection: "debug",
      description: "Debug trace tx",
      params: [{ name: "transaction_hash", type: "H256" }],
      type: "Result<Vec<TransactionTrace>>"
    }
  },
  xcm: {
    injectDownwardMessage: {
      description: "Inject a downward message from the relay chain.",
      params: [{ name: "message", type: "Vec<u8>" }],
      type: "Result<()>"
    },
    injectHrmpMessage: {
      description: "Inject an HRMP message from a dedicated channel from a sibling parachain",
      params: [
        { name: "sender", type: "ParaId" },
        { name: "message", type: "Vec<u8>" }
      ],
      type: "Result<()>"
    }
  },
  moon: {
    isBlockFinalized: {
      description: "Returns whether an Ethereum block is finalized",
      params: [{ name: "blockHash", type: "Hash" }],
      type: "bool"
    },
    isTxFinalized: {
      description: "Returns whether an Ethereum transaction is finalized",
      params: [{ name: "txHash", type: "Hash" }],
      type: "bool"
    },
    getLatestSyncedBlock: {
      description: "Returns the latest synced block from frontier's backend",
      params: [],
      type: "u32"
    }
  }
};
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  rpcDefinitions
});
