import "@moonbeam-network/api-augment";
import { deployCreateCompiledContract, describeSuite, beforeAll, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR } from "@moonwall/util";
import { alith } from "@moonwall/util";

// TODO: expand these tests to do multiple txn types when added to viem
describeSuite({
  id: "D4007",
  title: "Proxy Call for Contract",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let contractAddress: `0x${string}`;

    beforeAll(async () => {
      const { contractAddress: deployedAddr } = await deployCreateCompiledContract(
        context,
        "ProxyForContractsDemo",
        { value: 1000n * GLMR }
      );
      contractAddress = deployedAddr;
    });

    it({
      id: `T01`,
      title: `Proxy Call to Smart Contract account should fail`,
      test: async function () {
        await expect(
          context.createBlock(
            context
              .polkadotJs()
              .tx.proxy.proxy(
                contractAddress,
                "Any",
                context.polkadotJs().tx.balances.transferAll(ALITH_ADDRESS, true)
              )
              .signAsync(alith)
          )
        ).rejects.toThrow("Invalid Transaction: Transaction call is not expected");
      },
    });
  },
});
