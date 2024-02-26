import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  GLMR,
  alith,
} from "@moonwall/util";
import { jumpBlocks, SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D013109",
  title: "Randomness Result - Fulfilling all of random numbers",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should empty randomness results",
      test: async function () {
        const delayBlocks = 2;

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, delayBlocks],
          gas: 100_000n,
          privateKey: ALITH_PRIVATE_KEY,
        });

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [BALTATHAR_ADDRESS, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, delayBlocks],
          gas: 100_000n,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        await context.createBlock([], { signer: alith, allowFailures: false });

        await jumpBlocks(context, delayBlocks);

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "fulfillRequest",
          args: [0],
          gas: 200_000n,
        });

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "fulfillRequest",
          args: [1],
          gas: 200_000n,
        });

        await context.createBlock([]);

        const randomessResults = await context
          .polkadotJs()
          .query.randomness.randomnessResults.entries();
        expect(randomessResults).to.be.length(0);
      },
    });
  },
});
