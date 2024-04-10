import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D013122",
  title: "Randomness VRF - Fulfilling one of the 2 random requests at same block/delay",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      await context.writePrecompile!({
        precompileName: "Randomness",
        functionName: "requestLocalVRFRandomWords",
        args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, 3],
        gas: 120_000n,
      });
      await context.createBlock();
      await context.writePrecompile!({
        precompileName: "Randomness",
        functionName: "requestLocalVRFRandomWords",
        args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, 2],
        gas: 120_000n,
      });
      await context.createBlock();
      await context.createBlock();
      await context.createBlock();

      await context.writePrecompile!({
        precompileName: "Randomness",
        functionName: "fulfillRequest",
        args: [0],
        gas: 200_000n,
      });
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should keep the 2nd request",
      test: async function () {
        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(1);
      },
    });

    it({
      id: "T02",
      title: "should keep the randomness results",
      test: async function () {
        const randomnessResults = await context
          .polkadotJs()
          .query.randomness.randomnessResults.entries();
        expect(randomnessResults.length).to.equal(1);
      },
    });
  },
});
