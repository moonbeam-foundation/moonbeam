import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { Abi } from "viem";
import { verifyLatestBlockFees } from "../../../helpers/block.js";

describeSuite({
  id: "D0607",
  title: "Contract loop",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let incrementorContract: any;
    let incrementorAbi: Abi;
    let incrementorAddress: `0x${string}`;

    beforeEach(async () => {
      const {
        contract: incContract,
        contractAddress: incAddress,
        abi: incAbi,
      } = await deployCreateCompiledContract(context, "Incrementor");
      incrementorContract = incContract;
      incrementorAddress = incAddress;
      incrementorAbi = incAbi;
    });

    it({
      id: "T01",
      title: "creation  be initialized at 0",
      test: async function () {
        expect(await incrementorContract.read.count([])).toBe(0n);
      },
    });

    it({
      id: "T02",
      title: "should increment contract state",
      test: async function () {
        await context.viem("wallet").writeContract({
          abi: incrementorAbi,
          address: incrementorAddress,
          functionName: "incr",
          value: 0n,
        });
        await context.createBlock();

        expect(await incrementorContract.read.count()).toBe(1n);
      },
    });

    it({
      id: "T01",
      title: "should increment contract state (check fees)",
      test: async function () {
        await context.viem("wallet").writeContract({
          abi: incrementorAbi,
          address: incrementorAddress,
          functionName: "incr",
          value: 0n,
          maxPriorityFeePerGas: 0n,
        });

        await context.createBlock();
        await verifyLatestBlockFees(context);
      },
    });
  },
});
