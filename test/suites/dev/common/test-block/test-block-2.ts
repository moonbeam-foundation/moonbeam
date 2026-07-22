import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "moonwall";
import { waitFor } from "../../../../helpers";

describeSuite({
  id: "D010102",
  title: "Block creation - suite 2",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await context.createBlock();
      await context.createBlock();
      // The eth RPC layer can lag behind freshly sealed blocks. Force fresh
      // reads while waiting for it to catch up before running assertions.
      const isReady = await waitFor(
        async () => (await context.viem().getBlockNumber({ cacheTime: 0 })) >= 2n
      );
      if (!isReady) {
        throw new Error("Timed out waiting for RPC layer to catch up to block 2");
      }
    });

    it({
      id: "T01",
      title: "should be at block 2",
      test: async function () {
        expect(await context.viem().getBlockNumber({ cacheTime: 0 })).toBe(2n);
      },
    });

    it({
      id: "T02",
      title: "should include previous block hash as parent",
      test: async function () {
        const block = await context.viem().getBlock({ blockTag: "latest" });
        const previousBlock = await context.viem().getBlock({ blockNumber: 1n });
        expect(block.hash).to.not.equal(previousBlock.hash);
        expect(block.parentHash).to.equal(previousBlock.hash);
      },
    });
  },
});
