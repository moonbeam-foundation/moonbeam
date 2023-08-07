import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "D0610",
  title: "Block Contract - Block variables",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let blockContract: any;

    beforeAll(async function () {
      const { contract } = await deployCreateCompiledContract(context, "BlockVariables", {
        gas: 1000000n,
      });
      blockContract = contract;
    });

    it({
      id: "T01",
      title: "should store the valid block number at creation",
      test: async function () {
        expect(await blockContract.read.initialnumber()).toBe(1n);
      },
    });

    it({
      id: "T02",
      title: "should return parent block number + 1 when accessed by RPC call",
      test: async function () {
        const block = await context.viem("public").getBlock();
        expect(await blockContract.read.getNumber()).toBe(1n);
        expect(await blockContract.read.getNumber()).toBe(block.number);
      },
    });

    it({
      id: "T03",
      title: "should store the valid chain id at creation",
      test: async function () {
        expect(await blockContract.read.initialchainid()).toBe(1281n);
      },
    });
  },
});
