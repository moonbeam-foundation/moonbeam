import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "moonwall";

describeSuite({
  id: "D010102",
  title: "Block creation - suite 2",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await context.createBlock();
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should be at block 2",
      test: async function () {
        // viem caches the block number for `cacheTime` (defaults to the 4s
        // polling interval), so a plain `getBlockNumber()` can return a stale
        // height right after the blocks were sealed. Force a fresh read and
        // poll briefly to absorb any eth-layer import lag.
        let blockNumber = 0n;
        for (let i = 0; i < 20; i++) {
          blockNumber = await context.viem().getBlockNumber({ cacheTime: 0 });
          if (blockNumber === 2n) {
            break;
          }
          await new Promise((resolve) => setTimeout(resolve, 50));
        }
        expect(blockNumber).toBe(2n);
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
