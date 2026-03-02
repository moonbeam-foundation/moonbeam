import { beforeAll, describeSuite, expect } from "moonwall";

describeSuite({
  id: "D020509",
  title: "Block Contract - Block variables",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let blockContract: `0x${string}`;

    beforeAll(async function () {
      const { contractAddress } = await context.deployContract!("BlockVariables", {
        gas: 1000000n,
      });
      blockContract = contractAddress;
    });

    it({
      id: "T01",
      title: "should store the valid block number at creation",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "BlockVariables",
            contractAddress: blockContract,
            functionName: "initialnumber",
          })
        ).toBe(1n);
      },
    });

    it({
      id: "T02",
      title: "should return parent block number + 1 when accessed by RPC call",
      test: async function () {
        const header = await context.polkadotJs().rpc.chain.getHeader();
        expect(
          await context.readContract!({
            contractName: "BlockVariables",
            contractAddress: blockContract,
            functionName: "getNumber",
          })
        ).toBe(1n);
        expect(
          await context.readContract!({
            contractName: "BlockVariables",
            contractAddress: blockContract,
            functionName: "getNumber",
          })
        ).toBe(header.number.toBigInt());
      },
    });

    it({
      id: "T03",
      title: "should store the valid chain id at creation",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "BlockVariables",
            contractAddress: blockContract,
            functionName: "initialchainid",
          })
        ).toBe(1281n);
      },
    });
  },
});
