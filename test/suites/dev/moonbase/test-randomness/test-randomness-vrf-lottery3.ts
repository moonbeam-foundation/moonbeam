import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { CONTRACT_RANDOMNESS_STATUS_PENDING, GLMR } from "@moonwall/util";
import {
  expectEVMResult,
  extractRevertReason,
  setupLotteryWithParticipants,
} from "../../../../helpers";

describeSuite({
  id: "D023113",
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
      title: "should fail to fulfill before the delay",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "Randomness",
            functionName: "getRequestStatus",
            args: [0],
          })
        ).to.equal(CONTRACT_RANDOMNESS_STATUS_PENDING);

        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "fulfillRequest",
          args: [0],
          gas: 500_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Revert");

        const revertReason = await extractRevertReason(context, result!.hash);
        // Full error expected:
        // Module(ModuleError { index: 39, error: [7, 0, 0, 0],
        // message: Some("RequestCannotYetBeFulfilled") })
        expect(revertReason).to.contain("RequestCannotYetBeFulfilled");
      },
    });

    it({
      id: "T02",
      title: "should be rolling the numbers",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: lotteryContract,
            contractName: "RandomnessLotteryDemo",
            functionName: "status",
          })
        ).to.equal(1);
      },
    });
  },
});
