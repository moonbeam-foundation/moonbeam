import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS, GLMR, alith } from "@moonwall/util";
import { SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D013104",
  title: "Randomness Babe - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be successful",
      test: async function () {
        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestRelayBabeEpochRandomWords",
          gas: 120_000n,
          args: [
            alith.address, // refund address
            1n * GLMR, // fee
            120_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
          ],
        });
        await context.createBlock([], { signer: alith, allowFailures: false });

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(1);
        log(
          `Randomness returned for ${randomnessRequests[0][1]
            .unwrap()
            .request.numWords.toNumber()} words`
        );
      },
    });

    it({
      id: "T02",
      title: "should be marked as pending before the end of the delay",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "Randomness",
            functionName: "getRequestStatus",
            args: [1337],
          })
        ).to.equal(CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS);
      },
    });
  },
});
