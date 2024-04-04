import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, ETHAN_ADDRESS } from "@moonwall/util";

describeSuite({
  id: "D012886",
  title: "Precompiles - Staking - AwardedPoints",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async function () {
      await context.createBlock();
    });
    it({
      id: "T01",
      title: "should get awarded points for ALITH",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "awardedPoints",
            args: [1, ALITH_ADDRESS],
          })
        ).toBe(20);
      },
    });

    it({
      id: "T02",
      title: "should get no awarded points for ETHAN",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "awardedPoints",
            args: [1, ETHAN_ADDRESS],
          })
        ).toBe(0);
      },
    });
  },
});
