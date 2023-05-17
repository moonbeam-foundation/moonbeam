import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { alith, ALITH_ADDRESS, baltathar, GLMR, MIN_GAS_PRICE } from "@moonwall/util";
import { expectTypeOf } from "vitest";
import { PrivateKeyAccount } from "viem";
import { privateKeyToAccount, generatePrivateKey } from "viem/accounts";
import { TransactionTypes, createRawTransfer } from "../../../helpers/viem.js";

describeSuite({
  id: "D0402",
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
        expect( await context.viemClient("public").getBlockNumber()).toBe(2n)
      },
    });

    it({
      id: "T02",
      title: "should include previous block hash as parent",
      test: async function () {
        const block = await context.viemClient("public").getBlock({blockTag: "latest"})
        const  previousBlock =  await context.viemClient("public").getBlock({blockNumber: 1n})
        expect(block.hash).to.not.equal(previousBlock.hash);
        expect(block.parentHash).to.equal(previousBlock.hash);
      },
    });
  },
});
