export default {
  types: {},
  rpc: {
    moon: {
      isBlockFinalized: {
        description: "Checks if an Ethereum block is finalized",
        params: [{ name: "blockHash", type: "Hash" }],
        type: "bool",
      },
      isTxFinalized: {
        description: "Checks if an Ethereum transaction is finalized",
        params: [{ name: "transactionHash", type: "Hash" }],
        type: "bool",
      },
    },
  },
};
