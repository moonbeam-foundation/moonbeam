import "@moonbeam-network/api-augment";
import { describeSuite, customDevRpcRequest, beforeAll, expect } from "@moonwall/cli";
import { createEthersTransaction } from "@moonwall/util";
import { type Abi, encodeFunctionData } from "viem";
import { numberToHex } from "@polkadot/util";

describeSuite({
  id: "T20",
  title: "Trace filter - Gas Loop",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const testLoops: {
      count: number;
      txHash?: string;
      blockNumber?: number;
      expectedGas: string;
    }[] = [
      { count: 0, expectedGas: "0x53da" },
      { count: 100, expectedGas: "0x14422" },
      { count: 1000, expectedGas: "0x67192" },
    ];

    let looperAddress: `0x${string}`;
    let looperAbi: Abi;
    beforeAll(async () => {
      const { contractAddress, abi } = await context.deployContract!("Looper");
      looperAddress = contractAddress;
      looperAbi = abi;

      // For each loop, create a block with the contract execution.
      // 1 block is create for each so it is easier to select the execution using trace_filter
      // by specifying the fromBlock and toBlock
      for (let i = 0; i < testLoops.length; i++) {
        const loop = testLoops[i];
        const { result } = await context.createBlock(
          createEthersTransaction(context, {
            to: looperAddress,
            data: encodeFunctionData({
              abi: looperAbi,
              functionName: "incrementalLoop",
              args: [loop.count],
            }),
            gasLimit: 3_000_000,
          })
        );
        loop.txHash = result?.hash;
        loop.blockNumber = i + 2;
      }
    });

    it({
      id: "T01",
      title: "should return 21466 gasUsed for 0 loop",
      test: async function () {
        const trace = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: numberToHex(testLoops[0].blockNumber),
            toBlock: numberToHex(testLoops[0].blockNumber),
          },
        ]);
        expect(trace[0].result).to.not.be.undefined;
        expect(trace[0].result.error).to.not.exist;
        expect(trace[0].result.gasUsed).to.equal(testLoops[0].expectedGas);
      },
    });

    it({
      id: "T02",
      title: "should return 82978 gasUsed for 100 loop",
      test: async function () {
        const trace = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: numberToHex(testLoops[1].blockNumber),
            toBlock: numberToHex(testLoops[1].blockNumber),
          },
        ]);

        expect(trace[0].result).to.not.be.undefined;
        expect(trace[0].result.error).to.not.exist;
        expect(trace[0].result.gasUsed).to.equal(testLoops[1].expectedGas);
      },
    });

    it({
      id: "T03",
      title: "should return 422290 gasUsed for 1000 loop",
      test: async function () {
        const trace = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: numberToHex(testLoops[2].blockNumber),
            toBlock: numberToHex(testLoops[2].blockNumber),
          },
        ]);

        expect(trace[0].result).to.not.be.undefined;
        expect(trace[0].result.error).to.not.exist;
        expect(trace[0].result.gasUsed).to.equal(testLoops[2].expectedGas);
      },
    });
  },
});
