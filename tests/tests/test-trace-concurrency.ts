import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";

describeDevMoonbeam("Trace filter - Concurrency", (context) => {
  before("Setup: Create 50 blocks with 1 contract loop execution each", async function () {
    const { contract, rawTx } = await createContract(context.web3, "FiniteLoopContract");
    await context.createBlock({ transactions: [rawTx] });

    for (let i = 0; i < 50; i++) {
      await context.createBlock({
        transactions: [
          await createContractExecution(context.web3, {
            contract,
            contractCall: contract.methods.incr(2000),
          }),
        ],
      });
    }
  });

  // This test is based on the time needed for trace_filter to perform those actions.
  // It will start a slow query (taking 1s) and will try to execute a fast one after to see if it
  // goes through or wait for the first one to finish
  it("should allow concurrent execution", async function () {
    const queryRange = async (range, index) => {
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

describeDevMoonbeam("Trace debug - Concurrency", (context) => {
  it("should allow 4 concurrent execution", async function () {
    this.timeout(20000);
    const { contract, rawTx } = await createContract(context.web3, "TestContract");
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    // A single request.
    let start = Date.now();
    // await customWeb3Request(
    //     context.web3, "debug_traceTransaction", [txResults[0].result]
    // );
    let end = Date.now();
    // // 300 ms
    // console.log(end - start);
    let i;
    let promises = [];
    for (i = 0; i < 4; i++) {
      promises.push(
        customWeb3Request(context.web3, "debug_traceTransaction", [txResults[0].result])
      );
    }
    start = Date.now();
    await Promise.all(promises);
    end = Date.now() - start;
    console.log(end);
  });
});

describeDevMoonbeam("Trace debug - Concurrency", (context) => {
  it("should allow long concurrent execution", async function () {
    this.timeout(30000);

    const { contract, rawTx } = await createContract(context.web3, "FiniteLoopContract");
    await context.createBlock({ transactions: [rawTx] });

    const txResults = [];
    for (let i = 0; i < 10; i++) {
      txResults.push(
        (
          await context.createBlock({
            transactions: [
              await createContractExecution(context.web3, {
                contract,
                contractCall: contract.methods.incr(1000),
              }),
            ],
          })
        ).txResults[0]
      );
    }
    // A single request.
    let start = Date.now();
    // await customWeb3Request(
    //     context.web3, "debug_traceTransaction", [txResults[0].result]
    // );
    let end = Date.now();
    // // 300 ms
    // console.log(end - start);
    let i;
    let promises = [];
    for (i = 0; i < 2; i++) {
      promises.push(
        customWeb3Request(context.web3, "debug_traceTransaction", [txResults[i].result])
      );
    }
    start = Date.now();
    await Promise.all(promises);
    end = Date.now() - start;
    console.log(end);
  });
});
