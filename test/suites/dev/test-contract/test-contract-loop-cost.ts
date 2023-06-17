import "@moonbeam-network/api-augment";
import {
  describeSuite,
  expect,
  TransactionTypes,
  deployCreateCompiledContract,
} from "@moonwall/cli";
import { encodeFunctionData } from "viem";
import { createEthersTxn } from "@moonwall/util";

describeSuite({
  id: "D0608",
  title: "Contract loop",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let testNumber = 0;

    const TestParameters = [
      {
        loop: 1n,
        gas: 43779n,
      },
      {
        loop: 500n,
        gas: 242393n,
      },
      {
        loop: 600n,
        gas: 282193n,
      },
    ];

    TestParameters.forEach(({ loop, gas }, index) => {
      for (const txnType of TransactionTypes) {
        testNumber++;
        it({
          id: `T${testNumber > 9 ? testNumber : "0" + testNumber}`,
          title: `should consume ${gas} for ${loop} loop for ${txnType}`,
          test: async function () {
            const { abi, contractAddress, contract } = await deployCreateCompiledContract(
              context,
              "Looper"
            );

            const { rawSigned } = await createEthersTxn(context, {
              to: contractAddress,
              data: encodeFunctionData({ abi, functionName: "incrementalLoop", args: [loop] }),
              gasLimit: 10_000_000,
              txnType,
            });

            await context.createBlock(rawSigned);

            expect(await contract.read.count()).toBe(loop);
            const block = await context.viem("public").getBlock();
            expect(block.gasUsed).toBe(gas);
          },
        });
      }
    });
  },
});
