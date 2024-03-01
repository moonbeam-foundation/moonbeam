import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import { expectEVMResult, setupLotteryWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D013114",
  title: "Randomness VRF - Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryContract: `0x${string}`;

    beforeAll(async function () {
      lotteryContract = await setupLotteryWithParticipants(context, "VRF");
      await context.writeContract!({
        contractAddress: lotteryContract,
        contractName: "RandomnessLotteryDemo",
        functionName: "startLottery",
        value: 1n * GLMR,
        gas: 300_000n,
      });
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should succeed to fulfill after the delay",
      test: async function () {
        await context.createBlock();

        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "fulfillRequest",
          args: [0],
          gas: 500_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");
      },
    });
  },
});
