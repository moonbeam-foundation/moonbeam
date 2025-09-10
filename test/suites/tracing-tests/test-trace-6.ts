import { customDevRpcRequest, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";

import {
  alith,
  ALITH_PRIVATE_KEY,
  createEthersTransaction,
  PRECOMPILE_BATCH_ADDRESS,
} from "@moonwall/util";

import { encodeFunctionData } from "viem";

describeSuite({
  id: "T07",
  title: "Trace (call list)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should correctly trace subcall",
      test: async function () {
        const { contractAddress: contractProxy, abi: abiProxy } =
          await context.deployContract!("CallForwarder");

        const { contractAddress: contractDummy, abi: abiDummy } =
          await context.deployContract!("MultiplyBy7");

        const callTx = await createEthersTransaction(context, {
          from: alith.address,
          to: contractProxy,
          gasLimit: "0x100000",
          value: "0x00",
          privateKey: ALITH_PRIVATE_KEY,
          data: encodeFunctionData({
            abi: abiProxy,
            functionName: "call",
            args: [
              contractDummy,
              encodeFunctionData({
                abi: abiDummy,
                functionName: "multiply",
                args: [42],
              }),
            ],
          }),
        });

        const data = await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
        await context.createBlock();
        const trace = await customDevRpcRequest("debug_traceTransaction", [
          data,
          { tracer: "callTracer" },
        ]);

        expect(trace.from).to.be.eq(alith.address.toLowerCase());
        expect(trace.to).to.be.eq(contractProxy.toLowerCase());
        expect(trace.calls.length).to.be.eq(1);
        expect(trace.calls[0].from).to.be.eq(contractProxy.toLowerCase());
        expect(trace.calls[0].to).to.be.eq(contractDummy.toLowerCase());
        expect(trace.calls[0].type).to.be.eq("CALL");
      },
    });

    it({
      id: "T02",
      title: "should correctly trace delegatecall subcall",
      test: async function () {
        const { contractAddress: contractProxy, abi: abiProxy } =
          await context.deployContract!("CallForwarder");

        const { contractAddress: contractDummy, abi: abiDummy } =
          await context.deployContract!("MultiplyBy7");

        const callTx = await createEthersTransaction(context, {
          from: alith.address,
          to: contractProxy,
          gasLimit: "0x100000",
          value: "0x00",
          privateKey: ALITH_PRIVATE_KEY,
          data: encodeFunctionData({
            abi: abiProxy,
            functionName: "delegateCall",
            args: [
              contractDummy,
              encodeFunctionData({
                abi: abiDummy,
                functionName: "multiply",
                args: [42],
              }),
            ],
          }),
        });

        const data = await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
        await context.createBlock();
        const trace = await customDevRpcRequest("debug_traceTransaction", [
          data,
          { tracer: "callTracer" },
        ]);

        expect(trace.from).to.be.eq(alith.address.toLowerCase());
        expect(trace.to).to.be.eq(contractProxy.toLowerCase());
        expect(trace.calls.length).to.be.eq(1);
        expect(trace.calls[0].from).to.be.eq(contractProxy.toLowerCase());
        expect(trace.calls[0].to).to.be.eq(contractDummy.toLowerCase());
        expect(trace.calls[0].type).to.be.eq("DELEGATECALL");
      },
    });

    it({
      id: "T03",
      title: "should correctly trace precompile subcall (call list)",
      timeout: 10000,
      test: async function () {
        const { contractAddress: contractProxy, abi: abiProxy } =
          await context.deployContract!("CallForwarder");

        const { contractAddress: contractDummy, abi: abiDummy } =
          await context.deployContract!("MultiplyBy7");

        const abiBatch = fetchCompiledContract("Batch").abi;

        const callTx = await createEthersTransaction(context, {
          from: alith.address,
          to: PRECOMPILE_BATCH_ADDRESS,
          gasLimit: "0x100000",
          value: "0x00",
          privateKey: ALITH_PRIVATE_KEY,
          data: encodeFunctionData({
            abi: abiBatch,
            functionName: "batchAll",
            args: [
              [contractProxy, contractProxy],
              [],
              [
                encodeFunctionData({
                  abi: abiProxy,
                  functionName: "call",
                  args: [
                    contractDummy,
                    encodeFunctionData({
                      abi: abiDummy,
                      functionName: "multiply",
                      args: [42],
                    }),
                  ],
                }),
                encodeFunctionData({
                  abi: abiProxy,
                  functionName: "delegateCall",
                  args: [
                    contractDummy,
                    encodeFunctionData({
                      abi: abiDummy,
                      functionName: "multiply",
                      args: [42],
                    }),
                  ],
                }),
              ],
              [],
            ],
          }),
        });

        const data = await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
        await context.createBlock();
        const trace = await customDevRpcRequest("debug_traceTransaction", [
          data,
          { tracer: "callTracer" },
        ]);

        expect(trace.from).to.be.eq(alith.address.toLowerCase());
        expect(trace.to).to.be.eq(PRECOMPILE_BATCH_ADDRESS);
        expect(trace.calls.length).to.be.eq(2);

        expect(trace.calls[0].from).to.be.eq(PRECOMPILE_BATCH_ADDRESS);
        expect(trace.calls[0].to).to.be.eq(contractProxy.toLowerCase());
        expect(trace.calls[0].type).to.be.eq("CALL");

        expect(trace.calls[0].calls.length).to.be.eq(1);
        expect(trace.calls[0].calls[0].from).to.be.eq(contractProxy.toLowerCase());
        expect(trace.calls[0].calls[0].to).to.be.eq(contractDummy.toLowerCase());
        expect(trace.calls[0].calls[0].type).to.be.eq("CALL");

        expect(trace.calls[1].from).to.be.eq(PRECOMPILE_BATCH_ADDRESS);
        expect(trace.calls[1].to).to.be.eq(contractProxy.toLowerCase());
        expect(trace.calls[1].type).to.be.eq("CALL");

        expect(trace.calls[1].calls.length).to.be.eq(1);
        expect(trace.calls[1].calls[0].from).to.be.eq(contractProxy.toLowerCase());
        expect(trace.calls[1].calls[0].to).to.be.eq(contractDummy.toLowerCase());
        expect(trace.calls[1].calls[0].type).to.be.eq("DELEGATECALL");
      },
    });
  },
});
