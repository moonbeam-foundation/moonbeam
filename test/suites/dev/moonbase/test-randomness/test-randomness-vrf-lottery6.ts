import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import type { TransactionReceipt } from "viem";
import { setupLotteryWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D023116",
  title: "Randomness VRF - Static fulfilling Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryContract: `0x${string}`;
    let fulFillReceipt: TransactionReceipt;
    let lotteryContractStatus: any;

    beforeAll(async function () {
      lotteryContract = await setupLotteryWithParticipants(context, "VRF");

      await context.writeContract!({
        contractAddress: lotteryContract,
        contractName: "RandomnessLotteryDemo",
        functionName: "startLottery",
        value: 4n * GLMR,
        gas: 300_000n,
      });

      await context.createBlock();

      lotteryContractStatus = (await context.readContract!({
        contractAddress: lotteryContract,
        contractName: "RandomnessLotteryDemo",
        functionName: "status",
      })) as any;
      expect(lotteryContractStatus).to.equal(1);

      await context.createBlock();
      await context.createBlock();

      const { contractAddress: staticSubcallAddr } = await context.deployContract!(
        "StaticSubcall",
        { gas: 5_000_000n }
      );

      const rawTxn = await context.writeContract!({
        contractAddress: staticSubcallAddr,
        contractName: "StaticSubcall",
        functionName: "staticFulfill",
        args: [0],
        gas: 500_000n,
        rawTxOnly: true,
      });

      const { result } = await context.createBlock(rawTxn);

      fulFillReceipt = await context
        .viem()
        .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
    });
    it({
      id: "T01",
      title: "lottery contract status did not change",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: lotteryContract,
            contractName: "RandomnessLotteryDemo",
            functionName: "status",
          })
        ).to.equal(lotteryContractStatus);
      },
    });

    it({
      id: "T02",
      title: "should have no event",
      test: async function () {
        expect(fulFillReceipt.logs.length).to.equal(0);
      },
    });
  },
});
