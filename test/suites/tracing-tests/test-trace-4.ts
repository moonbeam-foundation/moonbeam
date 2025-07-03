import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_PRIVATE_KEY,
  createEthersTransaction,
  PRECOMPILE_CROWDLOAN_REWARDS_ADDRESS,
  baltathar,
  alith,
} from "@moonwall/util";
import BS_TRACER from "../../helpers/tracer/blockscout_tracer.min.json" assert { type: "json" };

describeSuite({
  id: "T05",
  title: "Trace (Blockscout)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should trace correctly out of gas transaction execution",
      test: async function () {
        const { contractAddress: looperAddress } = await context.deployContract!("Looper");

        const callTx = await createEthersTransaction(context, {
          from: alith.address,
          to: looperAddress,
          data: "0x5bec9e67",
          gasLimit: "0x100000",
          value: "0x00",
          privateKey: ALITH_PRIVATE_KEY,
        });

        const data = await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
        await context.createBlock();
        const trace = await customDevRpcRequest("debug_traceTransaction", [
          data,
          { tracer: BS_TRACER.body },
        ]);
        expect(trace.length).to.be.eq(1);
        expect(trace[0].error).to.be.equal("out of gas");
      },
    });

    it({
      id: "T02",
      title: "should trace correctly precompiles",
      test: async function () {
        const callTx = await createEthersTransaction(context, {
          from: alith.address,
          to: PRECOMPILE_CROWDLOAN_REWARDS_ADDRESS,
          data: "0x4e71d92d",
          gasLimit: "0xdb3b",
          value: "0x00",
          privateKey: ALITH_PRIVATE_KEY,
        });

        const data = await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
        await context.createBlock();
        const trace = await customDevRpcRequest("debug_traceTransaction", [
          data,
          { tracer: BS_TRACER.body },
        ]);

        expect(trace.length).to.be.eq(1);
      },
    });

    it({
      id: "T03",
      title: "should trace correctly transfers (raw)",
      test: async function () {
        const callTx = await createEthersTransaction(context, {
          from: alith.address,
          to: baltathar.address,
          data: "0x",
          gasLimit: "0xdb3b",
          value: "0x10000000",
          privateKey: ALITH_PRIVATE_KEY,
        });

        const data = await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
        await context.createBlock();
        const trace = await customDevRpcRequest("debug_traceTransaction", [data]);

        expect(trace.gas).to.be.eq("0x5208"); // 21_000 gas for a transfer.
      },
    });
  },
});
