import "@moonbeam-network/api-augment";
import { GLMR, alith, describeSuite, expect } from "moonwall";
import { SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D023006",
  title: "Randomness Babe - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
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
