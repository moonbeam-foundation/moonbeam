import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
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

      const estimatedGas = await context.viem().estimateContractGas({
        address: lotteryContract,
        abi: fetchCompiledContract("RandomnessLotteryDemo").abi,
        functionName: "startLottery",
        value: 1n * GLMR,
      });
      log("Estimated Gas for startLottery", estimatedGas);
      expect(estimatedGas).toMatchInlineSnapshot(`218919n`);

      await context.writeContract!({
        contractAddress: lotteryContract,
        contractName: "RandomnessLotteryDemo",
        functionName: "startLottery",
        value: 1n * GLMR,
        gas: estimatedGas,
      });

      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should succeed to fulfill after the delay",
      test: async function () {
        await context.createBlock();
        await context.createBlock();

        const estimatedGas = await context.viem().estimateContractGas({
          address: "0x0000000000000000000000000000000000000809", // Randomness contract address
          abi: fetchCompiledContract("Randomness").abi,
          functionName: "fulfillRequest",
          args: [0],
        });
        log("Estimated Gas for startLottery", estimatedGas);
        expect(estimatedGas).toMatchInlineSnapshot(`285461n`);

        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "fulfillRequest",
          args: [0],
          gas: estimatedGas,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");
      },
    });
  },
});
