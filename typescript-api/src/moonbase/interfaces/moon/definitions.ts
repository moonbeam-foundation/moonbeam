// TODO: update default export to make use of all definitions in moonbeam-types-bundle
// import { moonbeamDefinitions } from "moonbeam-types-bundle";

// TODO: Import this from moonbeam-types-bundle
export default {
  types: {},
  rpc: {
    isBlockFinalized: {
      description: "Returns whether an Ethereum block is finalized",
      params: [{ name: "blockHash", type: "Hash" }],
      type: "bool",
    },
    isTxFinalized: {
      description: "Returns whether an Ethereum transaction is finalized",
      params: [{ name: "txHash", type: "Hash" }],
      type: "bool",
    },
    getLatestSyncedBlock: {
      description: "Returns the latest synced block from Frontier's backend",
      params: [],
      type: "u32",
    },
  },
};
