import "@moonbeam-network/api-augment";
import {
  describeSuite,
  expect,
  TransactionTypes,
  deployCreateCompiledContract,
} from "@moonwall/cli";
import { encodeFunctionData } from "viem";
import { createEthersTransaction } from "@moonwall/util";

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
            const { abi, contractAddress } = await context.deployContract!("Looper");

            const rawSigned = await createEthersTransaction(context, {
              to: contractAddress,
              data: encodeFunctionData({ abi, functionName: "incrementalLoop", args: [loop] }),
              gasLimit: 10_000_000,
              txnType,
            });

            await context.createBlock(rawSigned);

            expect(
              await context.readContract!({
                contractName: "Looper",
                contractAddress,
                functionName: "count",
              })
            ).toBe(loop);
            const block = await context.viem().getBlock();
            expect(block.gasUsed).toBe(gas);
          },
        });
      }
    });
  },
});
