import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D013117",
  title: "Randomness VRF - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be successful",
      test: async function () {
        const rawTxn = context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, 2],
          gas: 100_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        expect(result!.successful).to.be.true;

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(1);
      },
    });
  },
});
