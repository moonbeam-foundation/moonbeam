import "@moonbeam-network/api-augment/moonbase";
import { beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  BALTATHAR_PRIVATE_KEY,
  CONTRACT_RANDOMNESS_STATUS_PENDING,
  GLMR,
  createViemTransaction,
} from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";
import {
  fakeBabeResultTransaction,
  setupLotteryWithParticipants,
} from "../../../../helpers/randomness.js";

describeSuite({
  id: "D013103",
  title: "Randomness Babe - Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryAddress: `0x${string}`;

    beforeEach(async function () {
      lotteryAddress = await setupLotteryWithParticipants(context, "BABE");

      await context.writeContract!({
        contractName: "RandomnessLotteryDemo",
        contractAddress: lotteryAddress,
        functionName: "startLottery",
        gas: 500_000n,
        value: 1n * GLMR,
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
        ).toBe(CONTRACT_RANDOMNESS_STATUS_PENDING);

        const rawTxn = await createViemTransaction(context as any, {
          to: lotteryAddress,
          data: encodeFunctionData({
            abi: fetchCompiledContract("Randomness").abi,
            functionName: "fulfillRequest",
            args: [0],
          }),
          gas: 500_000n,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Revert");

        expect(
          await context.readContract!({
            contractName: "RandomnessLotteryDemo",
            contractAddress: lotteryAddress,
            functionName: "status",
          })
        ).to.equal(1);
      },
    });

    it({
      id: "T02",
      title: "should succeed to fulfill after the delay",
      test: async function () {
        await context.createBlock();

        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "fulfillRequest",
          args: [0],
          gas: 500_000n,
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        const { result } = await context.createBlock([
          // Faking relay epoch + 2 in randomness storage
          fakeBabeResultTransaction(context),
          rawTxn,
        ]);

        expectEVMResult(result![1].events, "Succeed");
      },
    });
  },
});
