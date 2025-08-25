import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "D010102",
  title: "Block creation - suite 2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock();
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should be at block 2",
      test: async function () {
        expect(await context.viem().getBlockNumber()).toBe(2n);
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
