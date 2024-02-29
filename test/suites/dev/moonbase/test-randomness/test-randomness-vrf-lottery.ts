import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import { setupLotteryWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D013111",
  title: "Randomness VRF - Preparing Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryContract: `0x${string}`;

    beforeAll(async function () {
      lotteryContract = await setupLotteryWithParticipants(context, "VRF");
    });

    it({
      id: "T01",
      title: "should have a jackpot of 3 tokens",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "RandomnessLotteryDemo",
            contractAddress: lotteryContract,
            functionName: "jackpot",
            args: [],
          })
        ).to.equal(3n * GLMR);
      },
    });

    it({
      id: "T02",
      title: "should be open for registrations",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "RandomnessLotteryDemo",
            contractAddress: lotteryContract,
            functionName: "status",
            args: [],
          })
        ).to.equal(0);
      },
    });
  },
});
