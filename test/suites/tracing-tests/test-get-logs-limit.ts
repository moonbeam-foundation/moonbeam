import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "T01",
  title: "Test eth_getLogs RPC",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    // This variable needs to be modified if `--max-blocks-range` CLI parameter is changed
    // Using the default of 1024
    const BLOCKS_TO_CREATE = 1025;
    beforeAll(async () => {
      let blocksToCreate = BLOCKS_TO_CREATE;
      for (; blocksToCreate > 0; blocksToCreate--) {
        await context.createBlock();
      }
    });

    it({
      id: "T01",
      title: "Validate eth_getLogs block range limit",
      test: async function () {
        await expect(
          async () =>
            await customDevRpcRequest("eth_getLogs", [
              {
                fromBlock: "0x0",
                toBlock: "latest",
                topics: [],
              },
            ])
        ).rejects.toThrowError("block range is too wide (maximum 1024)");
      },
    });
  },
});
