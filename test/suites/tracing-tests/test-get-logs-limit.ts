import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "T20",
  title: "Test eth_getLogs RPC",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      for (let blocksToCreate = 1024; blocksToCreate > 0; blocksToCreate--) {
        await context.createBlock();
      }
    });

    it({
      id: "T01",
      title: "Validate eth_getLogs block range limit",
      test: async function () {
        const result = await customDevRpcRequest("eth_getLogs", [
          {
            fromBlock: "0x0",
            toBlock: "latest",
            topics: [],
          },
        ]);
      },
    });
  },
});
