import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";
const debug = require("debug")("test:debug-module");

describeDevMoonbeam("Debug module - Concurrency", (context) => {
  const CONCURRENT_EXECUTION = 4;
  const CONTRACT_LOOP = 70;
  let txResults = [];

  before("Setup: Create 9 blocks with 1 contract loop execution each", async function () {
    const { contract, rawTx } = await createContract(context.web3, "FiniteLoopContract");
    await context.createBlock({ transactions: [rawTx] });

    for (let i = 0; i < CONCURRENT_EXECUTION + 1; i++) {
      const response = await context.createBlock({
        transactions: [
          await createContractExecution(context.web3, {
            contract,
            contractCall: contract.methods.incr(CONTRACT_LOOP),
          }),
        ],
      });
      txResults.push(response.txResults[0]);
    }
  });

  it("should allow optimized concurrent execution", async function () {
    const webs = await Promise.all(
      [...Array(CONCURRENT_EXECUTION).keys()].map(() => context.createWeb3())
    );
    const startWitness = Date.now();
    await customWeb3Request(webs[0], "debug_traceTransaction", [txResults[0].result]);
    const witnesstime = Date.now() - startWitness;

    const startConcurrent = Date.now();
    await Promise.all(
      [...Array(CONCURRENT_EXECUTION).keys()].map((i) =>
        customWeb3Request(webs[i], "debug_traceTransaction", [txResults[i + 1].result]).then((r) =>
          // We verify that the result contains enough data.
          expect(r.result.stepLogs).to.have.length.greaterThan(CONTRACT_LOOP * 100)
        )
      )
    );
    const timeConcurrent = Date.now() - startConcurrent;

    debug(`Witness time: ${witnesstime}`);
    debug(`Concurrent (${CONCURRENT_EXECUTION}) time: ${timeConcurrent}`);

    // We consider it concurrent if the 4 requests take less than 2 times a single request
    expect(witnesstime * 2).to.be.at.least(timeConcurrent);
  });
});
