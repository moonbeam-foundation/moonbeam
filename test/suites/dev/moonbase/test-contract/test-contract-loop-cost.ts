import "@moonbeam-network/api-augment";
import { TransactionTypes, createEthersTransaction, describeSuite, expect } from "moonwall";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D020507",
  title: "Contract loop",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let testNumber = 0;

    const TestParameters = [
      {
        loop: 1n,
        gas: 43_774n,
      },
      {
        loop: 500n,
        gas: 241_390n,
      },
      {
        loop: 600n,
        gas: 280_990n,
      },
    ];

    TestParameters.forEach(({ loop, gas }) => {
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

            const { result } = await context.createBlock(rawSigned);

            expect(
              await context.readContract!({
                contractName: "Looper",
                contractAddress,
                functionName: "count",
              })
            ).toBe(loop);
            const receipt = await context
              .viem()
              .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
            expect(receipt.gasUsed).toBe(gas);
          },
        });
      }
    });
  },
});
