import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";

describeDevMoonbeam("Trace filter - Concurrency", (context) => {
  before("Setup: Create 50 blocks with 1 contract loop execution each", async function () {
    this.timeout(180000);
    const { contract, rawTx } = await createContract(context, "Looper");
    await context.createBlock(rawTx);

    for (let i = 0; i < 50; i++) {
      await context.createBlock(
        createContractExecution(context, {
          contract,
          contractCall: contract.methods.incrementalLoop(2000),
        })
      );
    }
  });

  // This test is based on the time needed for trace_filter to perform those actions.
  // It will start a slow query (taking 1s) and will try to execute a fast one after to see if it
  // goes through or wait for the first one to finish
  it.skip("should allow concurrent execution", async function () {
    const queryRange = async (range: number, index: number) => {
      const start = Date.now();
      await customWeb3Request(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(1),
          toBlock: context.web3.utils.numberToHex(range),
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
  });
});
