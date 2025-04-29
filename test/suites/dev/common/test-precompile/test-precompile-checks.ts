import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D015001",
  title: "Precompiles - Validate PrecompileChecks",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let multiplyBy7Contract;

    beforeAll(async () => {
      // Test setup
      multiplyBy7Contract = await context.deployContract!("MultiplyBy7");

      const ethEvent = (await context.polkadotJs().query.system.events()).find(({ event }) =>
        context.polkadotJs().events.ethereum.Executed.is(event)
      );
      expect((ethEvent.toHuman() as any).event["data"]["exitReason"]["Succeed"]).equals(
        "Returned"
      );
    });

    it({
      id: "T01",
      title: `Validate "CallableByContract" by calling precompile from smart-contract constructor`,
      test: async function () {
        const result = await context.deployContract!("CallBatchPrecompileFromConstructor", {
          gas: 5_000_000n,
          rawTxOnly: true,
          args: [
            multiplyBy7Contract.contractAddress,
            [
              encodeFunctionData({
                abi: multiplyBy7Contract.abi,
                functionName: "multiply",
                args: [5],
              }),
            ],
          ],
        });

        const ethEvent = (await context.polkadotJs().query.system.events()).find(({ event }) =>
          context.polkadotJs().events.ethereum.Executed.is(event)
        );
        expect((ethEvent.toHuman() as any).event["data"]["exitReason"]["Revert"]).equals(
          "Reverted"
        );
      },
    });
  },
});
