import { customDevRpcRequest, describeSuite, expect, TransactionTypes } from "@moonwall/cli";
import {
  alith,
  ALITH_PRIVATE_KEY,
  createEthersTransaction,
  PRECOMPILE_CROWDLOAN_REWARDS_ADDRESS,
} from "@moonwall/util";
import BS_TRACER_V2 from "../../helpers/tracer/blockscout_tracer_v2.min.json";

describeSuite({
  id: "T04",
  title: "Trace (Blockscout v2) - AllEthTxTypes",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
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
            { tracer: BS_TRACER_V2.body },
          ]);
          expect(trace.length).to.be.eq(1);
          expect(trace[0].error).to.be.equal("out of gas");
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 2}`,
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
            { tracer: BS_TRACER_V2.body },
          ]);

          expect(trace.length).to.be.eq(1);
        },
      });
    }
  },
});
