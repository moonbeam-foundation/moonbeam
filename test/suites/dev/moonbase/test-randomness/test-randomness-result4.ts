import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_PRIVATE_KEY, GLMR, alith } from "@moonwall/util";
import { Option } from "@polkadot/types";
import { PalletRandomnessRandomnessResult } from "@polkadot/types/lookup";
import { jumpBlocks, SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D013110",
  title: "Randomness Result - Passing targetted block",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should fill the randomness value",
      test: async function () {
        const delayBlocks = 2;

        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, delayBlocks],
          gas: 100_000n,
          privateKey: ALITH_PRIVATE_KEY,
        });
        await context.createBlock();
        await jumpBlocks(context, delayBlocks);

        const randomessResults = await context
          .polkadotJs()
          .query.randomness.randomnessResults.entries();
        expect(randomessResults).to.be.length(1);
        const randomessResult = randomessResults[0][1] as Option<PalletRandomnessRandomnessResult>;
        expect(randomessResult.unwrap().randomness.isSome).to.be.true;
        expect(randomessResult.unwrap().randomness.unwrap().toHex()).to.equal(
          "0xb1ffdd4a26e0f2a2fd1e0862a1c9be422c66dddd68257306ed55dc7bd9dce647"
        );
      },
    });
  },
});
