import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  GLMR,
  alith,
} from "@moonwall/util";
import type { Option } from "@polkadot/types";
import type { PalletRandomnessRandomnessResult } from "@polkadot/types/lookup";
import { jumpBlocks, SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D023108",
  title: "Randomness Result - Fulfilling one of multiple random numbers",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should leave 1 randomness result",
      test: async function () {
        const delayBlocks = 2;

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, delayBlocks],
          gas: 120_000n,
          privateKey: ALITH_PRIVATE_KEY,
        });

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [BALTATHAR_ADDRESS, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, delayBlocks],

          gas: 120_000n,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        await context.createBlock([]);

        await jumpBlocks(context, delayBlocks);

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "fulfillRequest",
          args: [
            0, // request id
          ],
          gas: 200_000n,
        });

        await context.createBlock([], { signer: alith, allowFailures: false });

        const randomessResults = await context
          .polkadotJs()
          .query.randomness.randomnessResults.entries();
        expect(randomessResults).to.be.length(1);
        const randomessResult = randomessResults[0][1] as Option<PalletRandomnessRandomnessResult>;
        expect(randomessResult.unwrap().requestCount.toNumber()).to.equal(1);
        expect(randomessResult.unwrap().randomness.isSome).to.be.true;
      },
    });
  },
});
