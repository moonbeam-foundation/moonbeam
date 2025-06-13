import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import { expectEVMResult, setupLotteryWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D023112",
  title: "Randomness VRF - Starting the Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryContract: `0x${string}`;

    beforeAll(async function () {
      lotteryContract = await setupLotteryWithParticipants(context, "VRF");
    });

    it({
      id: "T01",
      title: "should be able to start",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractName: "RandomnessLotteryDemo",
          contractAddress: lotteryContract,
          functionName: "startLottery",
          value: 1n * GLMR,
          gas: 300_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");
      },
    });
  },
});
