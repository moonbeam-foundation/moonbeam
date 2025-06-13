import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D023106",
  title: "Randomness Babe - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should succeed for 100 random words",
      test: async function () {
        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestRelayBabeEpochRandomWords",
          args: [
            alith.address, // refund address
            1n * GLMR, // fee
            120_000n, // gas limit
            SIMPLE_SALT,
            100, // number of random words
          ],
          gas: 120_000n,
        });
        await context.createBlock([], { signer: alith, allowFailures: false });
        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(1);
      },
    });
  },
});
