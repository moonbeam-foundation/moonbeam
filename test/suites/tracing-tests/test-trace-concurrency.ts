import "@moonbeam-network/api-augment";
import {
  describeSuite,
  expect,
  beforeAll,
  deployCreateCompiledContract,
  customDevRpcRequest,
} from "@moonwall/cli";
import { createEthersTransaction } from "@moonwall/util";
import { type Abi, encodeFunctionData } from "viem";

describeSuite({
  id: "T10",
  title: "Trace filter - Concurrency",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let looperAddress: `0x${string}`;
    let looperABI: Abi;

    beforeAll(async () => {
      const { contractAddress, abi } = await deployCreateCompiledContract(context, "Looper");
      looperAddress = contractAddress;
      looperABI = abi;

      await context.createBlock();

      for (let i = 0; i < 50; i++) {
        const rawSigned = await createEthersTransaction(context, {
          to: looperAddress,
          data: encodeFunctionData({
            abi: looperABI,
            functionName: "incrementalLoop",
            args: [2000],
          }),
          gasLimit: 200_000,
        });

        await context.createBlock(rawSigned);
      }
    }, 180000);

    // This test is based on the time needed for trace_filter to perform those actions.
    // It will start a slow query (taking 1s) and will try to execute a fast one after to see if it
    // goes through or wait for the first one to finish
    it({
      id: "T01",
      title: "should allow concurrent execution",
      modifier: "skip",
      test: async function () {
        const queryRange = async (range: number, index: number) => {
          const start = Date.now();
          await customDevRpcRequest("trace_filter", [
            {
              fromBlock: context.web3().utils.numberToHex(1),
              toBlock: context.web3().utils.numberToHex(range),
            },
          ]);
          const end = Date.now();
          console.log(`[${index}] 1-${range} Took: ${end - start} ms`);
        };

        // We start the slow query (around 1000ms), without waiting for it
        const initialQueryPromise = queryRange(40, 1);
        const startTime = Date.now();
        await queryRange(1, 2);
        const endTime = Date.now();
        // Less than 500ms is large enough (it should take at max 50ms)
        expect(endTime - startTime).to.be.lessThan(1000);

        // Wait for the initial query to finish to avoid pending queries
        await initialQueryPromise;
      },
    });
  },
});
