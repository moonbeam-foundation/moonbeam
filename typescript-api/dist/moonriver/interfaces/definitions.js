// src/moonriver/interfaces/moon/definitions.ts
var definitions_default = {
  types: {},
  rpc: {
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
      description: "Returns the latest synced block from Frontier's backend",
      params: [],
      type: "u32"
    }
  }
};
export {
  definitions_default as moon
};
