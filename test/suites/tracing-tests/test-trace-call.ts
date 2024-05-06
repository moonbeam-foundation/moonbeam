import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { ALITH_PRIVATE_KEY, alith, createEthersTransaction } from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { createContracts, nestedSingle } from "../../helpers";

describeSuite({
  id: "T16",
  title: "Test 'debug_traceCall'",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should trace nested contract calls",
      test: async function () {
        const contracts = await createContracts(context);
        const callParams = {
          to: contracts.callerAddr,
          data: encodeFunctionData({
            abi: contracts.abiCaller,
            functionName: "someAction",
            args: [contracts.calleeAddr, 6],
          })
        };
        const traceTx = await customDevRpcRequest("debug_traceCall", [callParams, "latest"]);
        const logs: any[] = [];
        for (const log of traceTx.structLogs) {
          if (logs.length == 1) {
            logs.push(log);
          }
          if (log.op == "RETURN") {
            logs.push(log);
          }
        }
        expect(logs).to.be.lengthOf(2);
        expect(logs[0].depth).to.be.equal(2);
        expect(logs[1].depth).to.be.equal(1);
      },
    });
  },
});
