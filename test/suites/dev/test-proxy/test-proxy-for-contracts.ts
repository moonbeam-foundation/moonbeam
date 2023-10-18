import "@moonbeam-network/api-augment";
import { deployCreateCompiledContract, describeSuite, expect, beforeAll } from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import { ALITH_ADDRESS, alith } from "@moonwall/util";

// TODO: expand these tests to do multiple txn types when added to viem
describeSuite({
  id: "D4007",
  title: "Proxy Call for Contract",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let contractAddress: `0x${string}`;

    beforeAll(async () => {
      const { contractAddress } = await deployCreateCompiledContract(context, "ProxyForContract");
      // Fund the contract account with some GLMR
      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(contractAddress, 1000n * GLMR),
        {
          allowFailures: false,
        }
      );
    });

    it({
      id: `T01`,
      title: `Proxy Call to Smart Contract account should not be able to execute`,
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              contractAddress,
              "Any",
              context.polkadotJs().tx.balances.transfer(contractAddress, 10n * GLMR)
            )
            .signAsync(alith.address)
        );
        console.log(ALITH_ADDRESS);
      },
    });
  },
});
