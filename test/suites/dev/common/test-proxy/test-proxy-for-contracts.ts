import "@moonbeam-network/api-augment";
import { deployCreateCompiledContract, describeSuite, beforeAll, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR } from "@moonwall/util";
import { alith } from "@moonwall/util";

describeSuite({
  id: "D013003",
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
      const ethEvent = (await context.polkadotJs().query.system.events()).find(({ event }) =>
        context.polkadotJs().events.ethereum.Executed.is(event)
      );
      expect((ethEvent.toHuman() as any).event["data"]["exitReason"]["Succeed"]).equals("Returned");

      contractAddress = deployedAddr;
    });

    // See ProxyForContractsDemo.sol for more explanation
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

    it({
      id: `T02`,
      title: `Adding a proxy to an existing contract should succeed`,
      test: async function () {
        const { result } = await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(contractAddress, "Any", 0)
        );
        expect(result?.successful).to.be.true;
      },
    });
  },
});
