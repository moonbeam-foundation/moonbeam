import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { gasLimit } from "../config";

describeSuite({
  id: "D010404",
  title: "Block genesis",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      expect(await context.viem().getBlockNumber()).toBe(0n);
    });

    it({
      id: "T01",
      title: "should contain block details",
      test: async function () {
        expect(await context.viem().getBlockNumber()).to.equal(0n);
        const block = await context.viem().getBlock({ blockNumber: 0n });
        expect(block).to.include({
          author: "0x0000000000000000000000000000000000000000",
          difficulty: 0n,
          extraData: "0x",
          gasLimit: gasLimit(context),
          gasUsed: 0n,
          logsBloom: `0x${"0".repeat(512)}`,
          number: 0n,
          receiptsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
          sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
          totalDifficulty: 0n,
          transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
        });

        expect(block.transactions).to.be.a("array").empty;
        expect(block.uncles).to.be.a("array").empty;
        expect(block.nonce).to.be.eq("0x0000000000000000");
        expect(block.hash).to.be.a("string").lengthOf(66);
        expect(block.parentHash).to.be.a("string").lengthOf(66);
        expect(block.timestamp).to.be.a("bigint");
      },
    });

    it({
      id: "T02",
      title: "should be accessible by hash",
      test: async function () {
        const block = await context.viem().getBlock({ blockNumber: 0n });
        const blockByHash = await context.viem().getBlock({ blockHash: block.hash! });
        expect(blockByHash).to.include({
          author: "0x0000000000000000000000000000000000000000",
          difficulty: 0n,
          extraData: "0x",
          gasLimit: gasLimit(context),
          gasUsed: 0n,
          logsBloom: `0x${"0".repeat(512)}`,
          number: 0n,
          receiptsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
          sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
          totalDifficulty: 0n,
          transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
        });
      },
    });
  },]
});
