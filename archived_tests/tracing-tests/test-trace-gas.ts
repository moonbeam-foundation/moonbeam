import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";

describeDevMoonbeam("Trace filter - Gas Loop", (context) => {
  const testLoops: {
    count: number;
    txHash?: string;
    blockNumber?: number;
    expectedGas: number;
  }[] = [
    { count: 0, expectedGas: 0x53dd },
    { count: 100, expectedGas: 0x144ed },
    { count: 1000, expectedGas: 0x67965 },
  ];

  before("Setup: Create 4 blocks with 1 contract loop execution each", async function () {
    const { contract, rawTx } = await createContract(context, "Looper");
    await context.createBlock(rawTx);

    // For each loop, create a block with the contract execution.
    // 1 block is create for each so it is easier to select the execution using trace_filter
    // by specifying the fromBlock and toBlock
    for (let i = 0; i < testLoops.length; i++) {
      const loop = testLoops[i];
      const { result } = await context.createBlock(
        createContractExecution(
          context,
          {
            contract,
            contractCall: contract.methods.incrementalLoop(loop.count),
          },
          { gas: 3_000_000 }
        )
      );
      loop.txHash = result.hash;
      loop.blockNumber = i + 2;
    }
  });

  it("should return 21630 gasUsed for 0 loop", async function () {
    const { rawTx } = await createContract(context, "Looper");
    await context.createBlock(rawTx);

    const trace = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: context.web3.utils.numberToHex(testLoops[0].blockNumber),
        toBlock: context.web3.utils.numberToHex(testLoops[0].blockNumber),
      },
    ]);
    expect(trace.result.length).to.equal(1);
    expect(trace.result[0].error).to.not.exist;
    expect(trace.result[0].result.gasUsed).to.equal(
      context.web3.utils.numberToHex(testLoops[0].expectedGas)
    );
  });

  it("should return 245542 gasUsed for 100 loop", async function () {
    const { rawTx } = await createContract(context, "Looper");
    await context.createBlock(rawTx);

    const trace = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: context.web3.utils.numberToHex(testLoops[1].blockNumber),
        toBlock: context.web3.utils.numberToHex(testLoops[1].blockNumber),
      },
    ]);
    expect(trace.result.length).to.equal(1);
    expect(trace.result[0].error).to.not.exist;
    expect(trace.result[0].result.gasUsed).to.equal(
      context.web3.utils.numberToHex(testLoops[1].expectedGas)
    );
  });

  it("should return 2068654 gasUsed for 1000 loop", async function () {
    this.timeout(12000);
    const { rawTx } = await createContract(context, "Looper", { gas: 3_000_000 });
    await context.createBlock(rawTx);

    const trace = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: context.web3.utils.numberToHex(testLoops[2].blockNumber),
        toBlock: context.web3.utils.numberToHex(testLoops[2].blockNumber),
      },
    ]);
    expect(trace.result.length).to.equal(1);
    expect(trace.result[0].error).to.not.exist;
    expect(trace.result[0].result.gasUsed).to.equal(
      context.web3.utils.numberToHex(testLoops[2].expectedGas)
    );
  });
});
