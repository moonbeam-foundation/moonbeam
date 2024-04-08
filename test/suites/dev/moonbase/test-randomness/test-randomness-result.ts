import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  DOROTHY_PRIVATE_KEY,
  GLMR,
  alith,
} from "@moonwall/util";
import { Option } from "@polkadot/types";
import { PalletRandomnessRandomnessResult } from "@polkadot/types/lookup";
import { SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D013107",
  title: "Randomness Result - Requesting 4 random numbers for the same target block",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should only have 1 randomness result with 4 requests",
      test: async function () {
        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [
            alith.address, // refund address
            1n * GLMR, // fee
            120_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
            3, // future blocks
          ],
          gas: 120_000n,
          privateKey: ALITH_PRIVATE_KEY,
        });

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 120_000n, SIMPLE_SALT, 1, 3],
          gas: 120_000n,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 120_000n, SIMPLE_SALT, 1, 3],
          gas: 120_000n,
          privateKey: CHARLETH_PRIVATE_KEY,
        });

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 120_000n, SIMPLE_SALT, 1, 3],
          gas: 120_000n,
          privateKey: DOROTHY_PRIVATE_KEY,
        });

        await context.createBlock([], { signer: alith, allowFailures: false });

        const randomessResults = await context
          .polkadotJs()
          .query.randomness.randomnessResults.entries();
        expect(randomessResults).to.be.length(1);
        const randomessResult = randomessResults[0][1] as Option<PalletRandomnessRandomnessResult>;

        log(randomessResult.toHuman());
        expect(randomessResult.unwrap().requestCount.toNumber()).to.equal(4);
        expect(randomessResult.unwrap().randomness.isNone).to.be.true;
      },
    });
  },
});
