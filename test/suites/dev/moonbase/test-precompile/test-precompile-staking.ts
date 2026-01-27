import "@moonbeam-network/api-augment";
import { ALITH_ADDRESS, describeSuite, expect } from "moonwall";

describeSuite({
  id: "D022758",
  title: "Precompiles - Staking - Genesis",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should include collator from the specs",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "isSelectedCandidate",
            args: [ALITH_ADDRESS],
          })
        ).toBe(true);
      },
    });

    it({
      id: "T02",
      title: "should have one collator",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "candidateCount",
          })
        ).toBe(1n);
      },
    });
  },
});
