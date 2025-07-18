import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { ALITH_PRIVATE_KEY, alith, createEthersTransaction } from "@moonwall/util";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "T08",
  title: "Raw trace limits",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should not trace call that would produce too big responses",
      timeout: 50000,
      test: async function () {
        const { contractAddress: traceFilterContract, abi: abiTraceFilter } =
          await context.deployContract!("TraceFilter", {
            args: [false],
          });

        const callTx = await createEthersTransaction(context, {
          from: alith.address,
          to: traceFilterContract,
          gasLimit: "0x800000",
          value: "0x00",
          privateKey: ALITH_PRIVATE_KEY,
          data: encodeFunctionData({
            abi: abiTraceFilter,
            functionName: "heavy_steps",
            args: [
              100, // number of storage modified
              1000, // numbers of simple steps (that will have 100 storage items in trace)
            ],
          }),
        });

        const data = await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
        await context.createBlock();

        expect(
          async () => await customDevRpcRequest("debug_traceTransaction", [data]),
          "Trace should be reverted but it worked instead"
        ).rejects.toThrowError(
          "replayed transaction generated too much data. try disabling memory or storage?"
        );
      },
    });
  },
});
