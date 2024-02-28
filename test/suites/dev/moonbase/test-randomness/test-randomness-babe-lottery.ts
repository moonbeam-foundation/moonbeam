import "@moonbeam-network/api-augment/moonbase";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import { expectEVMResult, setupLotteryWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D013101",
  title: "Randomness Babe - Preparing Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryAddress: `0x${string}`;

    beforeEach(async function () {
      lotteryAddress = await setupLotteryWithParticipants(context, "BABE");
    });

    it({
      id: "T01",
      title: "should have a jackpot of 3 tokens",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "RandomnessLotteryDemo",
            contractAddress: lotteryAddress,
            functionName: "jackpot",
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
            contractAddress: lotteryAddress,
            functionName: "status",
          })
        ).toBe(0);
      },
    });

    it({
      id: "T03",
      title: "should be able to start",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractName: "RandomnessLotteryDemo",
          contractAddress: lotteryAddress,
          functionName: "startLottery",
          gas: 500_000n,
          rawTxOnly: true,
          value: 1n * GLMR,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");
      },
    });
  },
});
